use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::lenient;

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
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppNotifications {
    pub allow: bool,
    pub desktop: bool,
    pub center: bool,
    pub lock_screen: bool,
    #[serde(deserialize_with = "lenient")]
    pub style: AlertStyle,
    pub critical: bool,
    pub time_sensitive: bool,
    pub sound: bool,
    pub badge: bool,
    #[serde(deserialize_with = "lenient")]
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
            lock_screen: true,
            style: AlertStyle::Temporary,
            critical: false,
            time_sensitive: true,
            sound: true,
            badge: true,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Focus {
    DoNotDisturb,
    Sleep,
    Personal,
    Work,
}

impl Focus {
    pub const ALL: [Self; 4] = [Self::DoNotDisturb, Self::Sleep, Self::Personal, Self::Work];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FocusSettings {
    pub allowed_apps: Vec<String>,
    pub time_sensitive: bool,
}

impl Default for FocusSettings {
    fn default() -> Self {
        Self {
            allowed_apps: Vec::new(),
            time_sensitive: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Widget {
    Calendar,
    Batteries,
}

impl Widget {
    pub const ALL: [Self; 2] = [Self::Calendar, Self::Batteries];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationsConfig {
    #[serde(deserialize_with = "lenient")]
    pub show_previews: ShowPreviews,
    pub when_locked: bool,
    pub apps: BTreeMap<String, AppNotifications>,
    #[serde(deserialize_with = "lenient")]
    pub focus: BTreeMap<Focus, FocusSettings>,
    pub widgets: Vec<Widget>,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            show_previews: ShowPreviews::default(),
            when_locked: true,
            apps: BTreeMap::new(),
            focus: BTreeMap::new(),
            widgets: Widget::ALL.to_vec(),
        }
    }
}

impl NotificationsConfig {
    pub fn app(&self, app_id: &str) -> AppNotifications {
        self.apps.get(app_id).cloned().unwrap_or_default()
    }

    pub fn app_mut(&mut self, app_id: &str) -> &mut AppNotifications {
        self.apps.entry(app_id.to_string()).or_default()
    }

    pub fn focus(&self, focus: Focus) -> FocusSettings {
        self.focus.get(&focus).cloned().unwrap_or_default()
    }

    pub fn focus_mut(&mut self, focus: Focus) -> &mut FocusSettings {
        self.focus.entry(focus).or_default()
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
    fn focus_modes_and_widgets_round_trip_through_toml() {
        let mut config = NotificationsConfig::default();
        config
            .focus_mut(Focus::Work)
            .allowed_apps
            .push("org.gnome.Calendar".into());
        config.widgets = vec![Widget::Batteries];
        let text = toml::to_string(&config).unwrap();
        let back: NotificationsConfig = toml::from_str(&text).unwrap();
        assert_eq!(back, config);
        assert!(back.focus(Focus::Sleep).time_sensitive);
    }

    #[test]
    fn an_unknown_value_resets_only_its_field() {
        let config: NotificationsConfig = toml::from_str(
            "show_previews = \"when_unlocked\"\nwhen_locked = false\n\
             [apps.\"org.example.App\"]\nlock_screen = false\nstyle = \"banner\"\n\
             [focus.gaming]\ntime_sensitive = false\n",
        )
        .unwrap();
        assert_eq!(config.show_previews, ShowPreviews::Always);
        assert!(!config.when_locked);
        assert!(!config.app("org.example.App").lock_screen);
        assert!(config.focus.is_empty());
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
