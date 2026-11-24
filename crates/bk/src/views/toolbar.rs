use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarItemPlacement {
    Automatic,
    Principal,
    Navigation,
    PrimaryAction,
    Status,
}

#[derive(Clone, PartialEq)]
pub struct ToolbarItem {
    placement: ToolbarItemPlacement,
    view: Element,
}

impl ToolbarItem {
    pub fn new(placement: ToolbarItemPlacement, view: impl IntoElement) -> Self {
        Self {
            placement,
            view: view.into_element(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ToolbarSpacer {
    flexible: bool,
}

impl ToolbarSpacer {
    pub fn flexible() -> Self {
        Self { flexible: true }
    }

    pub fn fixed() -> Self {
        Self { flexible: false }
    }
}

#[derive(Clone, PartialEq)]
pub enum ToolbarContent {
    Item(ToolbarItem),
    Spacer(ToolbarSpacer),
}

impl From<ToolbarItem> for ToolbarContent {
    fn from(item: ToolbarItem) -> Self {
        Self::Item(item)
    }
}

impl From<ToolbarSpacer> for ToolbarContent {
    fn from(spacer: ToolbarSpacer) -> Self {
        Self::Spacer(spacer)
    }
}

#[derive(Default)]
pub(crate) struct Sections {
    pub navigation: Vec<Element>,
    pub center: Vec<Element>,
    pub trailing: Vec<Element>,
}

impl Sections {
    fn push(&mut self, section: Section, view: Element) {
        match section {
            Section::Navigation => self.navigation.push(view),
            Section::Center => self.center.push(view),
            Section::Trailing => self.trailing.push(view),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Section {
    Navigation,
    Center,
    Trailing,
}

fn section(placement: ToolbarItemPlacement, automatic: Section) -> Section {
    match placement {
        ToolbarItemPlacement::Navigation => Section::Navigation,
        ToolbarItemPlacement::Principal | ToolbarItemPlacement::Status => Section::Center,
        ToolbarItemPlacement::PrimaryAction => Section::Trailing,
        ToolbarItemPlacement::Automatic => automatic,
    }
}

pub(crate) fn lay_out(content: &[ToolbarContent], gap: f32) -> Sections {
    let flexible = |content: &ToolbarContent| matches!(content, ToolbarContent::Spacer(spacer) if spacer.flexible);
    let mut automatic = if content.iter().any(flexible) {
        Section::Navigation
    } else {
        Section::Trailing
    };
    let mut sections = Sections::default();
    for content in content {
        match content {
            ToolbarContent::Item(item) => {
                sections.push(section(item.placement, automatic), item.view.clone())
            }
            ToolbarContent::Spacer(spacer) if spacer.flexible => automatic = Section::Trailing,
            ToolbarContent::Spacer(_) => {
                sections.push(automatic, rect().width(Size::px(gap)).into_element())
            }
        }
    }
    sections
}

pub(crate) fn row(mut views: Vec<Element>, gap: f32) -> Option<Element> {
    match views.len() {
        0 => None,
        1 => views.pop(),
        _ => Some(
            rect()
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(gap)
                .children(views)
                .into_element(),
        ),
    }
}
