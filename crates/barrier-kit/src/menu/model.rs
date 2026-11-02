use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchSource {
    Menu,
    Help,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchLocation {
    pub title_index: usize,
    pub indices: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchableEntry {
    pub path: String,
    pub item: MenuItem,
    pub location: Option<SearchLocation>,
    pub source: SearchSource,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchQuery {
    pub text: String,
    pub selected: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckState {
    #[default]
    None,
    Checkbox(bool),
    Radio(bool),
}

pub type SubmenuSource = Rc<dyn Fn(&str) -> Option<Vec<MenuItem>>>;

pub type ItemsSource = Rc<dyn Fn() -> Vec<MenuItem>>;

pub type SearchableSource = Rc<dyn Fn() -> Vec<SearchableEntry>>;

pub type RevealRoots = Rc<dyn Fn() -> Vec<Vec<MenuItem>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuNavigation {
    Prev,
    Next,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuIcon {
    Svg(Arc<str>),
    Image(Arc<[u8]>),
}

impl MenuIcon {
    pub const MAX_IMAGE_BYTES: usize = 64 * 1024;

    pub fn svg(s: impl Into<Arc<str>>) -> Self {
        Self::Svg(s.into())
    }

    pub fn image(bytes: impl Into<Arc<[u8]>>) -> Option<Self> {
        let bytes = bytes.into();
        if bytes.len() > Self::MAX_IMAGE_BYTES {
            tracing::debug!(
                bytes = bytes.len(),
                "menu: icon-data is too large; dropping the icon"
            );
            return None;
        }
        Some(Self::Image(bytes))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuItem {
    Action {
        id: String,
        label: String,
        shortcut: Option<String>,
        enabled: bool,
        check: CheckState,
        icon: Option<MenuIcon>,
    },
    Submenu {
        id: Option<String>,
        label: String,
        enabled: bool,
        items: Vec<MenuItem>,
        icon: Option<MenuIcon>,
    },
    Header {
        label: String,
        count: Option<usize>,
    },
    Separator,
    SearchField {
        placeholder: String,
    },
    SearchResult {
        path: String,
        item: Box<MenuItem>,
        source: SearchSource,
        location: Option<SearchLocation>,
    },
}

impl MenuItem {
    pub(super) fn enabled(&self) -> bool {
        match self {
            MenuItem::Action { enabled, .. } => *enabled,
            MenuItem::Submenu { enabled, .. } => *enabled,
            MenuItem::SearchResult { item, .. } => item.enabled(),
            MenuItem::Header { .. } | MenuItem::Separator | MenuItem::SearchField { .. } => false,
        }
    }

    pub(super) fn selectable(&self) -> bool {
        matches!(
            self,
            MenuItem::Action { .. } | MenuItem::Submenu { .. } | MenuItem::SearchResult { .. }
        ) && self.enabled()
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            MenuItem::Action { label, .. }
            | MenuItem::Submenu { label, .. }
            | MenuItem::Header { label, .. } => Some(label),
            MenuItem::SearchResult { item, .. } => item.label(),
            MenuItem::Separator | MenuItem::SearchField { .. } => None,
        }
    }

    pub(super) fn shortcut(&self) -> Option<String> {
        match self {
            MenuItem::Action { shortcut, .. } => shortcut.as_deref().map(format_shortcut),
            MenuItem::SearchResult { item, .. } => item.shortcut(),
            MenuItem::Submenu { .. }
            | MenuItem::Header { .. }
            | MenuItem::Separator
            | MenuItem::SearchField { .. } => None,
        }
    }

    pub(crate) fn height(&self, m: MenuPalette) -> f32 {
        match self {
            MenuItem::Separator => m.separator_height + m.separator_margin_y * 2.0,
            MenuItem::SearchField { .. } => m.search_height + m.search_margin_y * 2.0,
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

pub(super) fn empty_hint() -> MenuItem {
    MenuItem::Action {
        id: String::new(),
        label: "Sin elementos".to_string(),
        shortcut: None,
        enabled: false,
        check: CheckState::None,
        icon: None,
    }
}

pub(super) fn with_empty_hint(items: Vec<MenuItem>) -> Vec<MenuItem> {
    if items.is_empty() {
        vec![empty_hint()]
    } else {
        items
    }
}
