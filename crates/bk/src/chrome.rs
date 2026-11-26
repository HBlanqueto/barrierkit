use freya_core::prelude::*;
use torin::prelude::{Position, Size};

use crate::platform::{FrameLines, WindowControls};
use crate::styles::{FullscreenBarStyle, Theme, resolve, use_provide_theme};

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
    // Where the kit draws the frame, it also rounds the window; Malaysia rounds its own.
    let root = rect().expanded().child(app);
    let root = if lines.is_some() {
        let resolved = crate::theme::resolved();
        let background =
            crate::theme::surface_background(&resolved, crate::theme::SurfaceKind::Toplevel);
        root.background(crate::theme::color(background))
            .corner_radius(ctdesign::CORNER_WINDOW.fitted)
            .overflow(Overflow::Clip)
    } else {
        root
    };
    root.maybe_child(lines)
}

// Where no compositor frames the window, the kit draws the frame's two lines above the content.
fn frame_lines() -> Option<Rect> {
    let status = *try_consume_root_context::<WindowControls>()?.status.read();
    let drawn = try_consume_root_context::<FrameLines>().is_some_and(|lines| lines.0);
    if !drawn || status.maximized || status.fullscreen {
        return None;
    }
    let palette = crate::theme::resolved().palette();
    let radius = ctdesign::CORNER_WINDOW.fitted;
    let pixel = 1.0 / *Platform::get().current_window().scale_factor.read() as f32;
    let line = |fill| {
        Border::new()
            .fill(crate::theme::color(fill))
            .width(pixel)
            .alignment(BorderAlignment::Inner)
    };
    Some(
        rect()
            .position(Position::new_absolute().top(0.0).left(0.0))
            // `expanded` would take the height the content left over, which is none.
            .width(Size::percent(100.0))
            .height(Size::percent(100.0))
            .layer(Layer::OverlayLevel(16))
            .interactive(Interactive::No)
            .corner_radius(radius)
            .border(line(palette.outer_border))
            .padding(pixel)
            .child(
                rect()
                    .expanded()
                    .corner_radius((radius - pixel).max(0.0))
                    .border(line(
                        palette.inner_line.unwrap_or(ctdesign::WINDOW_INNER_LINE),
                    )),
            ),
    )
}
