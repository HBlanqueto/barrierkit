use super::*;
use crate::i18n::text;
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckState {
    #[default]
    None,
    Checkbox(bool),
    Radio(bool),
    Mixed,
    Minimized,
}

pub type ItemsSource = Rc<dyn Fn() -> Vec<MenuItem>>;

#[derive(Clone, Debug, PartialEq)]
pub enum MenuIcon {
    Svg(Arc<str>),
    Swatch(Rgba),
}

impl MenuIcon {
    pub fn svg(s: impl Into<Arc<str>>) -> Self {
        Self::Svg(s.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuItem {
    Action {
        id: String,
        label: Cow<'static, str>,
        shortcut: Option<Cow<'static, str>>,
        enabled: bool,
        check: CheckState,
        icon: Option<MenuIcon>,
        badge: Option<Cow<'static, str>>,
    },
    Submenu {
        id: Option<String>,
        label: Cow<'static, str>,
        enabled: bool,
        items: Vec<MenuItem>,
        icon: Option<MenuIcon>,
    },
    Header {
        label: Cow<'static, str>,
    },
    Separator,
    Palette {
        items: Vec<MenuItem>,
    },
}

impl MenuItem {
    pub fn action(id: impl Into<String>, label: impl Into<Cow<'static, str>>) -> Self {
        Self::Action {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            enabled: true,
            check: CheckState::None,
            icon: None,
            badge: None,
        }
    }

    pub fn palette(items: Vec<MenuItem>) -> Self {
        Self::Palette { items }
    }

    pub fn with_badge(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        if let MenuItem::Action { badge, .. } = &mut self {
            *badge = Some(text.into());
        }
        self
    }

    pub fn placeholder(label: impl Into<Cow<'static, str>>) -> Self {
        Self::action("", label).with_enabled(false)
    }

    pub fn header(label: impl Into<Cow<'static, str>>) -> Self {
        Self::Header {
            label: label.into(),
        }
    }

    pub fn submenu(label: impl Into<Cow<'static, str>>, items: Vec<MenuItem>) -> Self {
        Self::Submenu {
            id: None,
            label: label.into(),
            enabled: true,
            items,
            icon: None,
        }
    }

    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        if let Self::Submenu { id, .. } = &mut self {
            *id = Some(value.into());
        }
        self
    }

    pub fn with_shortcut(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        if let Self::Action { shortcut, .. } = &mut self {
            *shortcut = Some(text.into());
        }
        self
    }

    pub fn with_enabled(mut self, value: bool) -> Self {
        if let Self::Action { enabled, .. } | Self::Submenu { enabled, .. } = &mut self {
            *enabled = value;
        }
        self
    }

    pub fn with_check(mut self, state: CheckState) -> Self {
        if let Self::Action { check, .. } = &mut self {
            *check = state;
        }
        self
    }

    pub fn with_icon(mut self, value: impl Into<Option<MenuIcon>>) -> Self {
        if let Self::Action { icon, .. } | Self::Submenu { icon, .. } = &mut self {
            *icon = value.into();
        }
        self
    }

    pub(super) fn enabled(&self) -> bool {
        match self {
            MenuItem::Action { enabled, .. } => *enabled,
            MenuItem::Submenu { enabled, .. } => *enabled,
            MenuItem::Palette { items } => items.iter().any(MenuItem::enabled),
            MenuItem::Header { .. } | MenuItem::Separator => false,
        }
    }

    pub(super) fn selectable(&self) -> bool {
        matches!(
            self,
            MenuItem::Action { .. } | MenuItem::Submenu { .. } | MenuItem::Palette { .. }
        ) && self.enabled()
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            MenuItem::Action { label, .. }
            | MenuItem::Submenu { label, .. }
            | MenuItem::Header { label, .. } => Some(label),
            MenuItem::Separator | MenuItem::Palette { .. } => None,
        }
    }

    pub(super) fn shortcut(&self) -> Option<Cow<'static, str>> {
        match self {
            MenuItem::Action { shortcut, .. } => {
                shortcut.as_deref().map(|s| format_shortcut(s).into())
            }
            MenuItem::Submenu { .. }
            | MenuItem::Header { .. }
            | MenuItem::Separator
            | MenuItem::Palette { .. } => None,
        }
    }

    pub(crate) fn height(&self, m: MenuPalette) -> f32 {
        match self {
            MenuItem::Separator => m.separator_height + m.separator_margin_y * 2.0,
            MenuItem::Header { .. } => m.header_height,
            _ => m.item_height,
        }
    }
}

pub fn normalize_separators(items: &mut Vec<MenuItem>) {
    for item in items.iter_mut() {
        if let MenuItem::Submenu { items, .. } = item {
            normalize_separators(items);
        }
    }
    let mut after_separator = true;
    items.retain(|item| {
        if matches!(item, MenuItem::Separator) {
            let keep = !after_separator;
            after_separator = true;
            keep
        } else {
            after_separator = false;
            true
        }
    });
    while matches!(items.last(), Some(MenuItem::Separator)) {
        items.pop();
    }
}

pub(super) fn with_empty_hint(items: &[MenuItem]) -> Cow<'_, [MenuItem]> {
    if items.is_empty() {
        Cow::Owned(vec![MenuItem::placeholder(text!("no-items"))])
    } else {
        Cow::Borrowed(items)
    }
}
