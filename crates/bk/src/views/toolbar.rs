use freya_core::prelude::*;
use std::any::TypeId;
use torin::prelude::{Alignment, Direction, Size};

use crate::compact::IconButton;
use crate::controls::ToolGroup;

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
    bare_button: bool,
}

impl ToolbarItem {
    pub fn new<V: IntoElement + 'static>(placement: ToolbarItemPlacement, view: V) -> Self {
        Self {
            placement,
            view: view.into_element(),
            bare_button: TypeId::of::<V>() == TypeId::of::<IconButton>(),
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

// Adjacent bare buttons share one glass, as SwiftUI groups adjacent toolbar items; a spacer or another view ends the group.
#[derive(Default)]
struct Pieces {
    views: Vec<Element>,
    group: Vec<Element>,
}

impl Pieces {
    fn push(&mut self, view: Element, bare_button: bool) {
        if bare_button {
            self.group.push(view);
        } else {
            self.close();
            self.views.push(view);
        }
    }

    fn close(&mut self) {
        if !self.group.is_empty() {
            let buttons = std::mem::take(&mut self.group);
            self.views
                .push(ToolGroup::new().children(buttons).into_element());
        }
    }

    fn finish(mut self) -> Vec<Element> {
        self.close();
        self.views
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
    let mut pieces: [Pieces; 3] = Default::default();
    let at = |section: Section| match section {
        Section::Navigation => 0,
        Section::Center => 1,
        Section::Trailing => 2,
    };
    for content in content {
        match content {
            ToolbarContent::Item(item) => pieces[at(section(item.placement, automatic))]
                .push(item.view.clone(), item.bare_button),
            ToolbarContent::Spacer(spacer) if spacer.flexible => {
                pieces[at(automatic)].close();
                automatic = Section::Trailing;
            }
            ToolbarContent::Spacer(_) => {
                pieces[at(automatic)].push(rect().width(Size::px(gap)).into_element(), false)
            }
        }
    }
    let [navigation, center, trailing] = pieces.map(Pieces::finish);
    Sections {
        navigation,
        center,
        trailing,
    }
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
