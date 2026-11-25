use std::borrow::Cow;

use crate::menu::MenuItem;

pub const STATUS_ITEM_ACTIVATE: &str = "status-item.activate";

#[derive(Clone, Debug, PartialEq)]
pub struct StatusItem {
    pub id: String,
    pub title: Cow<'static, str>,
    pub icon_name: String,
    pub items: Vec<MenuItem>,
}

impl StatusItem {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<Cow<'static, str>>,
        icon_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            icon_name: icon_name.into(),
            items: Vec::new(),
        }
    }

    pub fn items(mut self, items: Vec<MenuItem>) -> Self {
        self.items = items;
        self
    }
}
