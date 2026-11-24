#[cfg(feature = "a11y")]
use std::collections::HashMap;
use std::ops::BitOr;

use accesskit::{Node, Role};
#[cfg(feature = "a11y")]
use accesskit::{NodeId, TreeUpdate};
use freya_core::prelude::AccessibilityExt;

// Freya rewrites a node's label from its text when the tree is built, so what a view asked for travels in the
// node's class name until `resolve_marks` applies it to the finished tree.
const MARKS: &str = "bk\u{1f}";
const SEPARATOR: char = '\u{1f}';

#[derive(Default)]
struct Marks {
    label: Option<String>,
    children: Option<AccessibilityChildBehavior>,
    // Whether the field holds text, and its placeholder.
    field: Option<(bool, String)>,
}

impl Marks {
    fn of(node: &Node) -> Self {
        let mut marks = Self::default();
        let Some(encoded) = node.class_name().and_then(|name| name.strip_prefix(MARKS)) else {
            return marks;
        };
        for part in encoded.split(SEPARATOR) {
            let (tag, text) = part.split_at(part.chars().next().map_or(0, char::len_utf8));
            match tag {
                "L" => marks.label = Some(text.to_owned()),
                "C" => marks.children = AccessibilityChildBehavior::parse(text),
                "F" => {
                    marks.field = text
                        .split_once(':')
                        .map(|(filled, placeholder)| (filled == "1", placeholder.to_owned()))
                }
                _ => {}
            }
        }
        marks
    }

    fn store(&self, node: &mut Node) {
        let mut parts = Vec::new();
        if let Some(label) = &self.label {
            parts.push(format!("L{label}"));
        }
        if let Some(children) = self.children {
            parts.push(format!("C{children:?}"));
        }
        if let Some((filled, placeholder)) = &self.field {
            parts.push(format!("F{}:{placeholder}", u8::from(*filled)));
        }
        if parts.is_empty() {
            node.clear_class_name();
        } else {
            node.set_class_name(format!("{MARKS}{}", parts.join(&SEPARATOR.to_string())));
        }
    }

    fn overlay(&mut self, other: Self) {
        self.label = other.label.or(self.label.take());
        self.children = other.children.or(self.children);
        self.field = other.field.or(self.field.take());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessibilityChildBehavior {
    Ignore,
    Combine,
    Contain,
}

impl AccessibilityChildBehavior {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "Ignore" => Some(Self::Ignore),
            "Combine" => Some(Self::Combine),
            "Contain" => Some(Self::Contain),
            _ => None,
        }
    }
}

fn mark(node: &mut Node, change: impl FnOnce(&mut Marks)) {
    let mut marks = Marks::of(node);
    change(&mut marks);
    marks.store(node);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AccessibilityTraits(u8);

impl AccessibilityTraits {
    pub const IS_BUTTON: Self = Self(1);
    pub const IS_HEADER: Self = Self(1 << 1);
    pub const IS_LINK: Self = Self(1 << 2);
    pub const IS_IMAGE: Self = Self(1 << 3);
    pub const IS_STATIC_TEXT: Self = Self(1 << 4);
    pub const IS_SELECTED: Self = Self(1 << 5);

    fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for AccessibilityTraits {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// SwiftUI's accessibility modifiers on any view that exposes Freya's accessibility data.
pub trait AccessibilityModifiers: AccessibilityExt {
    /// Wins over the text Freya takes from the view's children.
    fn accessibility_label(self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.a11y_builder(|node| {
            node.set_label(label.clone());
            mark(node, |marks| marks.label = Some(label));
        })
    }

    /// The hint reaches VoiceOver as AXHelp on macOS.
    fn accessibility_hint(self, hint: impl Into<String>) -> Self {
        self.a11y_builder(|node| node.set_description(hint))
    }

    fn accessibility_value(self, value: impl Into<String>) -> Self {
        self.a11y_builder(|node| node.set_value(value))
    }

    /// A hidden view and everything below it leave the accessibility tree.
    fn accessibility_hidden(self, hidden: bool) -> Self {
        self.a11y_builder(|node| {
            if hidden {
                node.set_hidden();
            } else {
                node.clear_hidden();
            }
        })
    }

    fn accessibility_identifier(self, identifier: impl Into<String>) -> Self {
        self.a11y_builder(|node| node.set_author_id(identifier))
    }

    /// Ignore and Combine fold the children into this view.
    fn accessibility_element(self, children: AccessibilityChildBehavior) -> Self {
        self.a11y_builder(|node| mark(node, |marks| marks.children = Some(children)))
    }

    /// When several role traits are given the last of button, header, link, image and static text wins.
    fn accessibility_add_traits(self, traits: AccessibilityTraits) -> Self {
        self.a11y_builder(|node| add_traits(node, traits))
    }
}

impl<T: AccessibilityExt> AccessibilityModifiers for T {}

fn add_traits(node: &mut Node, traits: AccessibilityTraits) {
    for (flag, role) in [
        (AccessibilityTraits::IS_BUTTON, Role::Button),
        (AccessibilityTraits::IS_HEADER, Role::Heading),
        (AccessibilityTraits::IS_LINK, Role::Link),
        (AccessibilityTraits::IS_IMAGE, Role::Image),
        (AccessibilityTraits::IS_STATIC_TEXT, Role::Label),
    ] {
        if traits.contains(flag) {
            node.set_role(role);
        }
    }
    if traits.contains(AccessibilityTraits::IS_SELECTED) {
        node.set_selected(true);
    }
}

// What a view's own modifiers say wins over what the control that wraps it set.
pub(crate) fn merge(control: &mut Node, modifiers: &Node) {
    if modifiers.role() != Role::Unknown {
        control.set_role(modifiers.role());
    }
    if let Some(label) = modifiers.label() {
        control.set_label(label);
    }
    if let Some(description) = modifiers.description() {
        control.set_description(description);
    }
    if let Some(value) = modifiers.value() {
        control.set_value(value);
    }
    if let Some(identifier) = modifiers.author_id() {
        control.set_author_id(identifier);
    }
    let mut marks = Marks::of(control);
    marks.overlay(Marks::of(modifiers));
    marks.store(control);
    if modifiers.is_hidden() {
        control.set_hidden();
    }
    if let Some(selected) = modifiers.is_selected() {
        control.set_selected(selected);
    }
}

pub(crate) fn mark_field(node: &mut Node, placeholder: &str, filled: bool) {
    mark(node, |marks| {
        marks.field = Some((filled, placeholder.to_owned()))
    });
}

#[cfg(feature = "a11y")]
fn is_text_input(node: &Node) -> bool {
    matches!(
        node.role(),
        Role::TextInput | Role::MultilineTextInput | Role::SearchInput
    )
}

#[cfg(feature = "a11y")]
pub(crate) fn has_marks(update: &TreeUpdate) -> bool {
    update.nodes.iter().any(|(_, node)| {
        node.class_name()
            .is_some_and(|name| name.starts_with(MARKS))
    })
}

#[cfg(feature = "a11y")]
fn first_text_input(
    update: &TreeUpdate,
    positions: &HashMap<NodeId, usize>,
    id: NodeId,
) -> Option<usize> {
    let position = *positions.get(&id)?;
    update.nodes[position]
        .1
        .children()
        .iter()
        .find_map(|child| {
            let child_position = *positions.get(child)?;
            if is_text_input(&update.nodes[child_position].1) {
                Some(child_position)
            } else {
                first_text_input(update, positions, *child)
            }
        })
}

// Freya puts the typed text, or the placeholder when empty, in the input's label and draws it with inner nodes.
#[cfg(feature = "a11y")]
fn name_the_input(node: &mut Node, name: String, (filled, placeholder): (bool, String)) {
    let typed = node.label().map(str::to_owned).unwrap_or_default();
    node.set_label(name);
    node.set_value(if filled { typed } else { String::new() });
    if !placeholder.is_empty() {
        node.set_placeholder(placeholder);
    }
    node.set_children(Vec::new());
}

// Works on a full tree. Returns whether a view left a mark, so the caller keeps sending full trees.
#[cfg(feature = "a11y")]
pub(crate) fn resolve_marks(update: &mut TreeUpdate) -> bool {
    if !has_marks(update) {
        return false;
    }
    for (_, node) in &mut update.nodes {
        if let Some(label) = Marks::of(node).label {
            if node.role() == Role::Label {
                node.set_value(label.clone());
            }
            node.set_label(label);
        }
    }
    let positions: HashMap<NodeId, usize> = update
        .nodes
        .iter()
        .enumerate()
        .map(|(position, (id, _))| (*id, position))
        .collect();
    let mut pending: Vec<NodeId> = update.tree.iter().map(|tree| tree.root).collect();
    let mut folded: HashMap<NodeId, NodeId> = HashMap::new();
    while let Some(id) = pending.pop() {
        let Some(&position) = positions.get(&id) else {
            continue;
        };
        let marks = Marks::of(&update.nodes[position].1);
        update.nodes[position].1.clear_class_name();
        if let Some(field) = marks.field
            && let Some(input) = first_text_input(update, &positions, id)
        {
            fold(update, &positions, &mut folded, input);
            let name = marks.label.clone().unwrap_or_else(|| field.1.clone());
            name_the_input(&mut update.nodes[input].1, name, field);
            update.nodes[position].1.clear_label();
        }
        let node = &update.nodes[position].1;
        let Some(behavior) = marks.children else {
            pending.extend(node.children());
            continue;
        };
        if matches!(node.role(), Role::Unknown | Role::GenericContainer) {
            update.nodes[position].1.set_role(Role::Group);
        }
        if behavior == AccessibilityChildBehavior::Contain {
            pending.extend(update.nodes[position].1.children());
            continue;
        }
        let texts = fold(update, &positions, &mut folded, position);
        let node = &mut update.nodes[position].1;
        if behavior == AccessibilityChildBehavior::Combine && marks.label.is_none() {
            node.set_label(texts.join(", "));
        }
        node.set_children(Vec::new());
    }
    update.nodes.retain(|(id, _)| !folded.contains_key(id));
    if let Some(owner) = folded.get(&update.focus) {
        update.focus = *owner;
    }
    true
}

#[cfg(feature = "a11y")]
fn fold(
    update: &TreeUpdate,
    positions: &HashMap<NodeId, usize>,
    folded: &mut HashMap<NodeId, NodeId>,
    owner: usize,
) -> Vec<String> {
    let owner_id = update.nodes[owner].0;
    let mut descendants = Vec::new();
    let mut texts = Vec::new();
    collect_descendants(update, positions, owner_id, &mut descendants, &mut texts);
    folded.extend(
        descendants
            .into_iter()
            .map(|descendant| (descendant, owner_id)),
    );
    texts
}

#[cfg(feature = "a11y")]
fn collect_descendants(
    update: &TreeUpdate,
    positions: &HashMap<NodeId, usize>,
    id: NodeId,
    descendants: &mut Vec<NodeId>,
    texts: &mut Vec<String>,
) {
    let Some(&position) = positions.get(&id) else {
        return;
    };
    for child in update.nodes[position].1.children() {
        descendants.push(*child);
        let Some(&child_position) = positions.get(child) else {
            continue;
        };
        let node = &update.nodes[child_position].1;
        if node.is_hidden() {
            continue;
        }
        if let Some(text) = node
            .label()
            .or_else(|| node.value())
            .filter(|t| !t.is_empty())
        {
            texts.push(text.to_owned());
        }
        collect_descendants(update, positions, *child, descendants, texts);
    }
}
