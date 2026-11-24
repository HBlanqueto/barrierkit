use ctdesign::{
    MENUBAR_CLASSIC_ALPHA, MENUBAR_DARK_INK_ABOVE, MENUBAR_HEIGHT, MENUBAR_HEIGHT_WIDE,
    MENUBAR_OVER_DARK_GLASS, MENUBAR_OVER_DARK_INK, MENUBAR_OVER_DARK_SELECTION,
    MENUBAR_OVER_LIGHT_GLASS, MENUBAR_OVER_LIGHT_INK, MENUBAR_OVER_LIGHT_SELECTION, Rgba,
};
use serde::{Deserialize, Serialize};

use crate::config::lenient;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MenubarStyle {
    Solid,
    Translucent,
    #[default]
    Floating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MenubarHeight {
    #[default]
    Slim,
    Wide,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct MenubarConfig {
    pub app_menus: bool,
    pub show_wifi: bool,
    pub show_bluetooth: bool,
    pub show_battery: bool,
    pub show_battery_percentage: bool,
    pub show_vpn: bool,
    pub show_printer: bool,
    #[serde(deserialize_with = "lenient")]
    pub style: MenubarStyle,
    #[serde(deserialize_with = "lenient")]
    pub bar_height: MenubarHeight,
}

impl MenubarConfig {
    pub fn height(&self) -> i32 {
        match self.bar_height {
            MenubarHeight::Slim => MENUBAR_HEIGHT,
            MenubarHeight::Wide => MENUBAR_HEIGHT_WIDE,
        }
    }
}

impl Default for MenubarConfig {
    fn default() -> Self {
        Self {
            app_menus: true,
            show_wifi: true,
            show_bluetooth: true,
            show_battery: true,
            show_battery_percentage: true,
            show_vpn: true,
            show_printer: true,
            style: MenubarStyle::default(),
            bar_height: MenubarHeight::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenubarColors {
    pub ink: Rgba,
    pub glass: Rgba,
    pub selection: Rgba,
}

pub fn classic_bar(theme_bar: Rgba, reduce_transparency: bool) -> Rgba {
    Rgba {
        a: if reduce_transparency {
            1.0
        } else {
            MENUBAR_CLASSIC_ALPHA
        },
        ..theme_bar
    }
}

pub fn colors_over(wallpaper: Rgba) -> MenubarColors {
    let veiled = wallpaper.darken(MENUBAR_OVER_LIGHT_GLASS.a);
    if veiled.relative_luminance() > MENUBAR_DARK_INK_ABOVE {
        MenubarColors {
            ink: MENUBAR_OVER_LIGHT_INK,
            glass: MENUBAR_OVER_LIGHT_GLASS,
            selection: MENUBAR_OVER_LIGHT_SELECTION,
        }
    } else {
        MenubarColors {
            ink: MENUBAR_OVER_DARK_INK,
            glass: MENUBAR_OVER_DARK_GLASS,
            selection: MENUBAR_OVER_DARK_SELECTION,
        }
    }
}
