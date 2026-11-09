use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct MenubarConfig {
    pub app_menus: bool,
    pub show_wifi: bool,
    pub show_bluetooth: bool,
    pub show_battery: bool,
    pub show_battery_percentage: bool,
    pub show_vpn: bool,
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
        }
    }
}
