use freya_core::prelude::*;

use crate::platform::WindowControls;
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
    rect().expanded().child(app)
}
