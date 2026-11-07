use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GreenButtonAction {
    #[default]
    #[serde(alias = "fullscreen")]
    Focus,
    Zoom,
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoubleClickAction {
    Fill,
    Minimize,
    #[default]
    Zoom,
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct WindowBehavior {
    pub double_click_titlebar: DoubleClickAction,
    pub green_button: GreenButtonAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlebarAction {
    Focus,
    Maximize,
    Minimize,
    None,
}

pub fn read_window_behavior() -> WindowBehavior {
    let path = crate::config_dir().join("window.domain");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return WindowBehavior::default();
    };
    toml::from_str(&text).unwrap_or_else(|error| {
        tracing::warn!(path = %path.display(), %error, "invalid window.domain");
        WindowBehavior::default()
    })
}

pub fn green_action(action: GreenButtonAction, fixed_width: bool) -> TitlebarAction {
    match action {
        GreenButtonAction::Focus if fixed_width => TitlebarAction::Maximize,
        GreenButtonAction::Focus => TitlebarAction::Focus,
        GreenButtonAction::Zoom => TitlebarAction::Maximize,
        GreenButtonAction::None => TitlebarAction::None,
    }
}

pub fn double_click_action(action: DoubleClickAction, zoomable: bool) -> TitlebarAction {
    match action {
        DoubleClickAction::Fill | DoubleClickAction::Zoom if zoomable => TitlebarAction::Maximize,
        DoubleClickAction::Fill | DoubleClickAction::Zoom => TitlebarAction::None,
        DoubleClickAction::Minimize => TitlebarAction::Minimize,
        DoubleClickAction::None => TitlebarAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_window_domain_enters_focus_mode_on_green_and_zooms_on_double_click() {
        let behavior: WindowBehavior = toml::from_str("").unwrap();
        assert_eq!(
            green_action(behavior.green_button, false),
            TitlebarAction::Focus
        );
        assert_eq!(behavior.double_click_titlebar, DoubleClickAction::Zoom);
    }

    #[test]
    fn fullscreen_is_another_name_for_focus_mode() {
        let behavior: WindowBehavior = toml::from_str("green_button = \"fullscreen\"").unwrap();
        assert_eq!(behavior.green_button, GreenButtonAction::Focus);
    }

    #[test]
    fn a_fixed_width_window_maximizes_instead_of_entering_focus_mode() {
        assert_eq!(
            green_action(GreenButtonAction::Focus, true),
            TitlebarAction::Maximize
        );
    }

    #[test]
    fn a_titlebar_double_click_only_fills_never_enters_focus_mode() {
        for action in [DoubleClickAction::Zoom, DoubleClickAction::Fill] {
            assert_eq!(double_click_action(action, true), TitlebarAction::Maximize);
            assert_eq!(double_click_action(action, false), TitlebarAction::None);
        }
    }
}
