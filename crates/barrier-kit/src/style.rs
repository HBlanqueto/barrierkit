use std::borrow::Cow;

use freya_core::prelude::*;
use vow_design::{Font, Rgba, WINDOW_CORNER};

use crate::theme::color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub label: Rgba,
    pub secondary_label: Rgba,
    pub tertiary_label: Rgba,
    pub separator: Rgba,
    pub control_border: Rgba,
    pub window_background: Rgba,
    pub content_background: Rgba,
    pub control_background: Rgba,
    pub control_hover: Rgba,
    pub accent: Rgba,
    pub accent_inactive: Rgba,
    pub on_accent: Rgba,
    pub destructive: Rgba,
    pub raised_background: Rgba,
    pub sunken_background: Rgba,
    pub scrim: Rgba,
    pub selected_control: Rgba,
    pub card_border: Rgba,
}

impl Palette {
    pub fn current() -> Self {
        let palette = try_consume_context::<State<Palette>>()
            .map_or_else(Self::system, |state| *state.read());
        if crate::platform::window_activated() {
            palette
        } else {
            Self {
                accent: palette.accent_inactive,
                on_accent: vow_appearance::backdrop::ink_on(palette.accent_inactive),
                ..palette
            }
        }
    }

    pub fn system() -> Self {
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        Self {
            label: base.ink_strong,
            secondary_label: base.ink_muted,
            tertiary_label: base.ink_faint,
            separator: base.line,
            control_border: base.line_strong,
            window_background: resolved.surfaces.window,
            content_background: resolved.surfaces.surface,
            control_background: resolved.surfaces.control,
            control_hover: resolved.surfaces.hover,
            accent: resolved.accent,
            accent_inactive: base.accent_inactive,
            on_accent: resolved.on_accent,
            destructive: base.danger,
            raised_background: resolved.surfaces.raised,
            sunken_background: resolved.surfaces.sunken,
            scrim: base.scrim,
            selected_control: base.selected_control,
            card_border: base.card_border,
        }
    }
}

pub fn text(content: impl Into<Cow<'static, str>>, face: Font, ink: Rgba) -> Label {
    label()
        .text(content)
        .font_family(face.family)
        .font_size(face.size as f32)
        .font_weight(face.weight as i32)
        .line_height(face.line_height as f32 / face.size as f32)
        .max_lines(1)
        .color(color(ink))
}

// Freya's smoothing s runs the curve to (1 + s) R, Apple's 1.5287 R (malaysia ADR 0059).
pub fn continuous_corner(radius: f32) -> CornerRadius {
    CornerRadius::new_all(radius).with_smoothing(WINDOW_CORNER.expansion - 1.0)
}
