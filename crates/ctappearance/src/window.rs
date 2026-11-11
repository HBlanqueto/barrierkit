use std::path::Path;

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

// window.domain also holds the compositor's keys; only this one is rewritten.
pub fn write_double_click_titlebar(path: &Path, action: DoubleClickAction) -> std::io::Result<()> {
    let mut doc = match std::fs::read_to_string(path) {
        Ok(text) => text
            .parse::<toml_edit::DocumentMut>()
            .map_err(std::io::Error::other)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => toml_edit::DocumentMut::new(),
        Err(error) => return Err(error),
    };
    let value = toml_edit::ser::to_document(&WindowBehavior {
        double_click_titlebar: action,
        ..WindowBehavior::default()
    })
    .map_err(std::io::Error::other)?;
    doc["double_click_titlebar"] = value["double_click_titlebar"].clone();
    crate::config::replace_file(path, &doc.to_string())
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
    fn writing_the_double_click_action_keeps_the_compositor_keys() {
        let dir = std::env::temp_dir().join(format!("ctappearance-window-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("window.domain");
        std::fs::write(
            &path,
            "resize_border = 12\ndouble_click_titlebar = \"zoom\"\n",
        )
        .unwrap();
        write_double_click_titlebar(&path, DoubleClickAction::Minimize).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("resize_border = 12"));
        let behavior: WindowBehavior = toml::from_str(&text).unwrap();
        assert_eq!(behavior.double_click_titlebar, DoubleClickAction::Minimize);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_titlebar_double_click_only_fills_never_enters_focus_mode() {
        for action in [DoubleClickAction::Zoom, DoubleClickAction::Fill] {
            assert_eq!(double_click_action(action, true), TitlebarAction::Maximize);
            assert_eq!(double_click_action(action, false), TitlebarAction::None);
        }
    }
}
