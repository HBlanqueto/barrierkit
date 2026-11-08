use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct MenubarConfig {
    pub app_menus: bool,
}

impl Default for MenubarConfig {
    fn default() -> Self {
        Self { app_menus: true }
    }
}
