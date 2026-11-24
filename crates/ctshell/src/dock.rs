use ctappearance::lenient;
use serde::{Deserialize, Serialize};

use crate::design::{
    DOCK_LARGESIZE, DOCK_MAGNIFY_MIN, DOCK_TILESIZE, DOCK_TILESIZE_MAX, DOCK_TILESIZE_MIN,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DockOrientation {
    Left,
    #[default]
    Bottom,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MinimizeEffect {
    #[default]
    Genie,
    Scale,
    Suck,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct DockConfig {
    pub autohide: bool,
    #[serde(deserialize_with = "lenient")]
    pub orientation: DockOrientation,
    pub magnification: bool,
    pub tilesize: i32,
    pub largesize: i32,
    #[serde(deserialize_with = "lenient")]
    pub mineffect: MinimizeEffect,
    pub show_process_indicators: bool,
    pub static_only: bool,
    pub minimize_to_application: bool,
    pub show_recents: bool,
    pub show_downloads: bool,
    pub launchanim: bool,
    pub autohide_delay: f32,
    pub autohide_time_modifier: f32,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            autohide: false,
            orientation: DockOrientation::Bottom,
            magnification: false,
            tilesize: DOCK_TILESIZE,
            largesize: DOCK_LARGESIZE,
            mineffect: MinimizeEffect::Genie,
            show_process_indicators: true,
            static_only: false,
            minimize_to_application: false,
            show_recents: true,
            show_downloads: true,
            launchanim: true,
            autohide_delay: 0.0,
            autohide_time_modifier: 1.0,
        }
    }
}

impl DockConfig {
    pub fn tilesize(&self) -> f32 {
        self.tilesize.clamp(DOCK_TILESIZE_MIN, DOCK_TILESIZE_MAX) as f32
    }

    pub fn largesize(&self) -> f32 {
        if self.magnification {
            (self.largesize.clamp(DOCK_TILESIZE_MIN, DOCK_TILESIZE_MAX) as f32)
                .max(self.tilesize() * DOCK_MAGNIFY_MIN)
        } else {
            self.tilesize()
        }
    }
}
