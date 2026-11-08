use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShowPreviews {
    #[default]
    Always,
    WhenUnlocked,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlertStyle {
    #[default]
    Temporary,
    Persistent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Grouping {
    #[default]
    Automatic,
    ByApp,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppNotifications {
    pub allow: bool,
    pub desktop: bool,
    pub center: bool,
    pub style: AlertStyle,
    pub critical: bool,
    pub time_sensitive: bool,
    pub sound: bool,
    pub grouping: Grouping,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted_until: Option<i64>,
}

impl Default for AppNotifications {
    fn default() -> Self {
        Self {
            allow: true,
            desktop: true,
            center: true,
            style: AlertStyle::Temporary,
            critical: false,
            time_sensitive: true,
            sound: true,
            grouping: Grouping::Automatic,
            muted_until: None,
        }
    }
}

impl AppNotifications {
    pub fn muted(&self, now: i64) -> bool {
        self.muted_until.is_some_and(|until| until > now)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationsConfig {
    pub show_previews: ShowPreviews,
    pub apps: BTreeMap<String, AppNotifications>,
}

impl NotificationsConfig {
    pub fn app(&self, app_id: &str) -> AppNotifications {
        self.apps.get(app_id).cloned().unwrap_or_default()
    }

    pub fn app_mut(&mut self, app_id: &str) -> &mut AppNotifications {
        self.apps.entry(app_id.to_string()).or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_app_without_an_entry_gets_temporary_banners_with_sound() {
        let config: NotificationsConfig = toml::from_str("show_previews = \"never\"\n").unwrap();
        let app = config.app("org.example.App");
        assert_eq!(config.show_previews, ShowPreviews::Never);
        assert!(app.allow && app.desktop && app.center && app.sound);
        assert_eq!(app.style, AlertStyle::Temporary);
        assert!(!app.critical);
    }

    #[test]
    fn a_mute_lasts_until_its_time() {
        let app = AppNotifications {
            muted_until: Some(100),
            ..AppNotifications::default()
        };
        assert!(app.muted(99));
        assert!(!app.muted(100));
    }
}
