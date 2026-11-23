//! P2 of `docs/research/shader-needs.md`: each SkSL source is compiled once per thread and its uniforms are bound by name, in the
//! order and with the sizes that Skia reads from the source, so a shader no longer packs a flat array in a hand-kept order.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use freya_engine::prelude::{Data, RuntimeEffect, Shader};

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

    #[cfg(test)]
    pub(crate) fn slots(&self) -> &[Slot] {
        &self.slots
    }

    pub(crate) fn shader(&self, arguments: &[(&str, &[f32])]) -> Option<Shader> {
        let bytes = pack(&self.slots, self.effect.uniform_size(), arguments)
            .inspect_err(|error| tracing::warn!(%error, "BarrierKit: a shader was not bound"))
            .ok()?;
        self.effect.make_shader(Data::new_copy(&bytes), &[], None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "uniform float4 box; uniform float2 ramp; uniform half4 tint; half4 main(float2 p) { return tint * half(box.x + ramp.y); }";

    fn slot(name: &str, offset: usize, floats: usize) -> Slot {
        Slot {
            name: name.to_string(),
            offset,
            floats,
        }
    }

    #[test]
    fn arguments_land_at_the_offsets_of_their_uniforms_whatever_their_order() {
        let slots = [slot("a", 0, 2), slot("b", 8, 1)];
        let bytes = pack(&slots, 12, &[("b", &[3.0]), ("a", &[1.0, 2.0])]).unwrap();
        let floats: Vec<f32> = bytes
            .chunks(4)
            .map(|c| f32::from_ne_bytes(c.try_into().unwrap()))
            .collect();
        assert_eq!(floats, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn an_unknown_missing_or_misshapen_argument_is_an_error() {
        let slots = [slot("a", 0, 2)];
        assert_eq!(
            pack(&slots, 8, &[("c", &[1.0])]),
            Err(BindError::Unknown("c".into()))
        );
        assert_eq!(pack(&slots, 8, &[]), Err(BindError::Missing("a".into())));
        assert_eq!(
            pack(&slots, 8, &[("a", &[1.0])]),
            Err(BindError::Size {
                name: "a".into(),
                expected: 2,
                got: 1
            })
        );
    }

    #[test]
    fn skia_reports_the_uniforms_of_a_source_in_declaration_order_and_the_program_is_compiled_once()
    {
        let program = Program::get(SOURCE).unwrap();
        let slots: Vec<(&str, usize, usize)> = program
            .slots()
            .iter()
            .map(|s| (s.name.as_str(), s.offset, s.floats))
            .collect();
        assert_eq!(slots, [("box", 0, 4), ("ramp", 16, 2), ("tint", 24, 4)]);
        assert!(Rc::ptr_eq(&program, &Program::get(SOURCE).unwrap()));
        let args: [(&str, &[f32]); 3] =
            [("box", &[0.0; 4]), ("ramp", &[0.0; 2]), ("tint", &[0.0; 4])];
        assert!(program.shader(&args).is_some());
        assert!(program.shader(&args[..2]).is_none());
    }
}
