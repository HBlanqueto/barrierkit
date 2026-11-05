use freya_core::prelude::*;
use torin::prelude::{Position, Size};
use vow_design::{KNOB_SHADOW, SLIDER, SLIDER_FAT, SLIDER_KEY_STEP, SWITCH, SYMBOL};

use crate::backdrop::Tint;
use crate::interaction::use_interaction;
use crate::style::Palette;
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LevelStyle {
    #[default]
    Regular,
    Fat(Symbol),
}

#[derive(PartialEq)]
pub struct LevelSlider {
    label: &'static str,
    value: f32,
    style: LevelStyle,
    disabled: bool,
    on_change: Option<Callback<f32, ()>>,
    tint: Tint,
    key: DiffKey,
}

impl LevelSlider {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            value: 0.0,
            style: LevelStyle::default(),
            disabled: false,
            on_change: None,
            tint: Tint::Accent,
            key: DiffKey::None,
        }
    }

    pub fn tint(mut self, tint: Tint) -> Self {
        self.tint = tint;
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self
    }

    pub fn style(mut self, style: LevelStyle) -> Self {
        self.style = style;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<f32, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for LevelSlider {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for LevelSlider {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let value = crate::controlled::use_controlled(&self.value);
        let mut width = use_state(|| 0.0f32);
        let mut left = use_state(|| 0.0f64);
        let enabled = !self.disabled;
        let style = self.style;
        let fraction = *value.read();
        let measured = *width.read();

        let (height, run, offset) = match style {
            LevelStyle::Regular => (SLIDER.height, measured - SLIDER.knob, SLIDER.knob / 2.0),
            LevelStyle::Fat(_) => (SLIDER_FAT.height, measured, 0.0),
        };
        let set = {
            let mut value = value;
            let on_change = self.on_change.clone();
            move |new: f32| {
                let new = new.clamp(0.0, 1.0);
                if new != *value.read() {
                    value.set(new);
                    if let Some(cb) = &on_change {
                        cb.call(new);
                    }
                }
            }
        };
        let from_x = move |x: f64| ((x as f32 - offset) / run.max(1.0)).clamp(0.0, 1.0);
        let mut set_down = set.clone();
        let mut set_move = set.clone();
        let mut set_key = set;

        let mut root = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(fraction)))
            .width(Size::fill())
            .height(Size::px(height))
            .on_sized(move |event: Event<SizedEventData>| {
                width.set_if_modified(event.data().area.size.width);
            })
            .maybe(enabled, |el| {
                el.on_mouse_down(move |event: Event<MouseEventData>| {
                    left.set(event.global_location.x - event.element_location.x);
                    set_down(from_x(event.element_location.x));
                })
                .on_global_pointer_move(move |event: Event<PointerEventData>| {
                    if interaction.held() {
                        set_move(from_x(event.global_location().x - *left.peek()));
                    }
                })
                .on_key_down(move |event: Event<KeyboardEventData>| {
                    let step = match event.key {
                        Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowDown) => -SLIDER_KEY_STEP,
                        Key::Named(NamedKey::ArrowRight | NamedKey::ArrowUp) => SLIDER_KEY_STEP,
                        _ => return,
                    };
                    set_key(fraction + step);
                })
            });

        root = match style {
            LevelStyle::Regular => {
                let track_top = (height - SLIDER.track) / 2.0;
                let knob_x = run.max(0.0) * fraction;
                // freya 0.4.3 dims each element on its own (render_pipeline.rs): a dimmed knob would show the fill.
                root.child(
                    bar(
                        offset,
                        track_top,
                        run.max(0.0),
                        SLIDER.track,
                        color(palette.control_background),
                    )
                    .maybe(!enabled, |el| el.opacity(SWITCH.disabled_opacity))
                    .child(bar(
                        -offset,
                        0.0,
                        knob_x + offset,
                        SLIDER.track,
                        self.tint.fill(&palette),
                    )),
                )
                .child(
                    knob(knob_x, (height - SLIDER.knob) / 2.0, SLIDER.knob).layer(KNOB_ABOVE_FILL),
                )
            }
            LevelStyle::Fat(lead) => {
                let fill = (measured * fraction).max(SLIDER_FAT.fill_min);
                root.maybe(!enabled, |el| el.opacity(SWITCH.disabled_opacity))
                    .corner_radius(height / 2.0)
                    .background(color(palette.control_background))
                    .child(bar(0.0, 0.0, fill, height, self.tint.fill(&palette)))
                    .child(
                        rect()
                            .position(
                                Position::new_absolute()
                                    .left(SLIDER_FAT.lead_icon_x)
                                    .top((height - SYMBOL.grid) / 2.0),
                            )
                            .child(symbol(lead, SYMBOL.grid, palette.on_accent)),
                    )
            }
        };
        interaction.focus_ring(root, height / 2.0)
    }
}

pub(crate) fn knob_shadow() -> Shadow {
    (
        0.0,
        KNOB_SHADOW.y_offset as f32,
        KNOB_SHADOW.blur as f32,
        0.0,
        color(KNOB_SHADOW.color),
    )
        .into()
}

// freya 0.4.3 paints by depth (data.rs `LayerState`); a sibling knob must outrank the fill, two levels down.
pub(crate) const KNOB_ABOVE_FILL: i16 = 2;

pub(crate) fn knob(left: f32, top: f32, size: f32) -> Rect {
    rect()
        .position(Position::new_absolute().left(left).top(top))
        .width(Size::px(size))
        .height(Size::px(size))
        .corner_radius(size / 2.0)
        .background(color(SWITCH.knob_color))
        .shadow(knob_shadow())
}

pub(crate) fn bar(left: f32, top: f32, width: f32, height: f32, fill: impl Into<Fill>) -> Rect {
    rect()
        .position(Position::new_absolute().left(left).top(top))
        .width(Size::px(width))
        .height(Size::px(height))
        .corner_radius(height / 2.0)
        .background(fill)
}
