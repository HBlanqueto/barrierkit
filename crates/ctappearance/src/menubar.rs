use ctdesign::{
    MENUBAR_HEIGHT, MENUBAR_HEIGHT_WIDE, MENUBAR_OVER_DARK_GLASS, MENUBAR_OVER_DARK_INK,
    MENUBAR_OVER_DARK_SELECTION, MENUBAR_OVER_LIGHT_GLASS, MENUBAR_OVER_LIGHT_INK,
    MENUBAR_OVER_LIGHT_SELECTION, Rgba,
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

pub fn colors_over(wallpaper: Rgba) -> MenubarColors {
    if wallpaper.contrast_ratio(MENUBAR_OVER_LIGHT_INK)
        >= wallpaper.contrast_ratio(MENUBAR_OVER_DARK_INK)
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_style_falls_back_to_floating_and_keeps_the_rest() {
        let config: MenubarConfig =
            toml::from_str("style = \"glass\"\nbar_height = \"wide\"\nshow_vpn = false\n").unwrap();
        assert_eq!(config.style, MenubarStyle::Floating);
        assert_eq!(config.height(), MENUBAR_HEIGHT_WIDE);
        assert!(!config.show_vpn);
    }

    #[test]
    fn a_pale_wallpaper_gets_dark_ink_and_a_dark_one_light_ink() {
        assert_eq!(
            colors_over(Rgba::rgb8(230, 225, 210)).ink,
            MENUBAR_OVER_LIGHT_INK
        );
        assert_eq!(
            colors_over(Rgba::rgb8(20, 30, 60)).ink,
            MENUBAR_OVER_DARK_INK
        );
    }
}
