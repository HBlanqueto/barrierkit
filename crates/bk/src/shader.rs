use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use freya_engine::prelude::{Data, ImageFilter, RuntimeEffect, Shader, image_filters};
use skia_safe::runtime_effect::RuntimeShaderBuilder;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Slot {
    pub(crate) name: String,
    pub(crate) offset: usize,
    pub(crate) floats: usize,
}

#[derive(Debug, PartialEq)]
pub(crate) enum BindError {
    Unknown(String),
    Missing(String),
    Size {
        name: String,
        expected: usize,
        got: usize,
    },
}

impl std::fmt::Display for BindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(name) => write!(f, "the shader has no uniform `{name}`"),
            Self::Missing(name) => write!(f, "the uniform `{name}` was not bound"),
            Self::Size {
                name,
                expected,
                got,
            } => write!(f, "the uniform `{name}` takes {expected} floats, not {got}"),
        }
    }
}

// The uniform block of `size` bytes with every slot written from its argument. A uniform nobody binds is an error: Skia would
// read zeros and the effect would draw wrong without a sign.
pub(crate) fn pack(
    slots: &[Slot],
    size: usize,
    arguments: &[(&str, &[f32])],
) -> Result<Vec<u8>, BindError> {
    let mut bytes = vec![0u8; size];
    let mut bound = vec![false; slots.len()];
    for (name, values) in arguments {
        let index = slots
            .iter()
            .position(|slot| slot.name == *name)
            .ok_or_else(|| BindError::Unknown((*name).to_string()))?;
        let slot = &slots[index];
        if values.len() != slot.floats {
            return Err(BindError::Size {
                name: slot.name.clone(),
                expected: slot.floats,
                got: values.len(),
            });
        }
        for (i, value) in values.iter().enumerate() {
            let at = slot.offset + i * 4;
            bytes[at..at + 4].copy_from_slice(&value.to_ne_bytes());
        }
        bound[index] = true;
    }
    match bound.iter().position(|bound| !bound) {
        Some(index) => Err(BindError::Missing(slots[index].name.clone())),
        None => Ok(bytes),
    }
}

pub(crate) struct Program {
    effect: RuntimeEffect,
    slots: Vec<Slot>,
}

type Compiled = HashMap<(usize, usize), Option<Rc<Program>>>;

thread_local! {
    static PROGRAMS: RefCell<Compiled> = RefCell::new(HashMap::new());
}

impl Program {
    // The program of `source`, compiled the first time; `None` after a compile error, which is logged once.
    pub(crate) fn get(source: &'static str) -> Option<Rc<Self>> {
        PROGRAMS.with(|programs| {
            programs
                .borrow_mut()
                .entry((source.as_ptr() as usize, source.len()))
                .or_insert_with(|| {
                    RuntimeEffect::make_for_shader(source, None)
                        .inspect_err(
                            |error| tracing::warn!(%error, "BarrierKit: a shader did not compile"),
                        )
                        .ok()
                        .map(|effect| {
                            let slots = effect
                                .uniforms()
                                .iter()
                                .map(|uniform| Slot {
                                    name: uniform.name().to_string(),
                                    offset: uniform.offset(),
                                    floats: uniform.size_in_bytes() / 4,
                                })
                                .collect();
                            Rc::new(Self { effect, slots })
                        })
                })
                .clone()
        })
    }

    pub(crate) fn has_uniform(&self, name: &str) -> bool {
        self.slots.iter().any(|slot| slot.name == name)
    }

    pub(crate) fn shader(&self, arguments: &[(&str, &[f32])]) -> Option<Shader> {
        let bytes = pack(&self.slots, self.effect.uniform_size(), arguments)
            .inspect_err(|error| tracing::warn!(%error, "BarrierKit: a shader was not bound"))
            .ok()?;
        self.effect.make_shader(Data::new_copy(&bytes), &[], None)
    }

    pub(crate) fn filter(&self, arguments: &[(&str, &[f32])], child: &str) -> Option<ImageFilter> {
        pack(&self.slots, self.effect.uniform_size(), arguments)
            .inspect_err(|error| tracing::warn!(%error, "BarrierKit: a shader was not bound"))
            .ok()?;
        let mut builder = RuntimeShaderBuilder::new(self.effect.clone());
        for (name, values) in arguments {
            builder.set_uniform_float(name, values).ok()?;
        }
        image_filters::runtime_shader(&builder, child, None)
    }
}
