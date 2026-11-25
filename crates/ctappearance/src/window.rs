use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GreenButtonAction {
    #[default]
    #[serde(alias = "fullscreen")]
    Focus,
    Zoom,
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DoubleClickAction {
    Fill,
    Minimize,
    #[default]
    Zoom,
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
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
