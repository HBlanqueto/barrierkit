use freya_core::prelude::*;
use torin::prelude::{Position, Size};

use crate::{
    platform::WindowControls,
    style::{concentric, continuous_corner},
    styles::{FullscreenBarStyle, Theme, resolve, use_provide_theme},
};

fn use_focus_bar_reveal() {
    let controls = try_consume_root_context::<WindowControls>();
    let fallback = use_state(|| false);
    let shown = controls
        .as_ref()
        .map_or(fallback, |controls| controls.focus_bar);
    let style: FullscreenBarStyle = resolve(&Theme::tengen());
    let progress = crate::animation::use_switch_progress(shown, style.animation);
    if let Some(controls) = controls {
        let mut state = controls.focus_bar_progress;
        state.set_if_modified(progress);
    }
}

pub(crate) fn window_root(theme: Theme, app: impl IntoElement) -> Rect {
    use_provide_theme(theme);
    use_focus_bar_reveal();
    if try_consume_root_context::<WindowControls>().is_some_and(|controls| controls.hud) {
        crate::theme::use_appearance(|config| {
            config.color_scheme = ctappearance::ColorScheme::Dark
        });
    }
    let lines = frame_lines();
    rect()
        .expanded()
        .child(app)
        .maybe(lines.is_some(), |root| {
            let resolved = crate::theme::resolved();
            let background =
                crate::theme::surface_background(&resolved, crate::theme::SurfaceKind::Toplevel);
            root.background(crate::theme::color(background))
                .corner_radius(continuous_corner(ctdesign::CORNER_WINDOW.fitted))
                .overflow(Overflow::Clip)
        })
        .maybe_child(lines)
}

fn frame_lines() -> Option<Rect> {
    let controls =
        try_consume_root_context::<WindowControls>().filter(WindowControls::draws_frame)?;
    let lines = crate::theme::resolved().scheme.pick([
        ctdesign::FRAME_LINES_LIGHT,
        ctdesign::FRAME_LINES_DARK,
        ctdesign::FRAME_LINES_DARK_TINT,
    ]);
    let (outer, inner) = if controls.status.read().activated {
        (lines.outer, lines.inner)
    } else {
        (lines.outer_inactive, lines.inner_inactive)
    };
    let corner = continuous_corner(ctdesign::CORNER_WINDOW.fitted);
    let scale = *Platform::get().current_window().scale_factor.read() as f32;
    let width = (ctdesign::FRAME_LINE_WIDTH * scale).round().max(1.0) / scale;
    let line = |fill| {
        Border::new()
            .fill(crate::theme::color(fill))
            .width(width)
            .alignment(BorderAlignment::Inner)
    };
    Some(
        rect()
            .position(Position::new_absolute().top(0.0).left(0.0))
            .width(Size::percent(100.0))
            .height(Size::percent(100.0))
            .layer(Layer::OverlayLevel(16))
            .interactive(Interactive::No)
            .corner_radius(corner)
            .border(line(outer))
            .padding(width)
            .child(
                rect()
                    .expanded()
                    .corner_radius(concentric(corner, width))
                    .border(line(inner)),
            ),
    )
}
