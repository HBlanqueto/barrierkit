use std::collections::HashSet;

use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};
use vow_design::{BODY, DISCLOSURE, HAIRLINE, SPACE_1};

use crate::interaction::{track_modifiers, use_held_modifiers};
use crate::scroll::ScrollView;
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;
use crate::views::collection::select;

#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode {
    pub title: String,
    pub icon: Option<Symbol>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            children: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: Symbol) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }
}

pub fn visible_rows(
    nodes: &[TreeNode],
    expanded: &HashSet<Vec<usize>>,
) -> Vec<(Vec<usize>, usize)> {
    fn walk(
        nodes: &[TreeNode],
        prefix: &[usize],
        expanded: &HashSet<Vec<usize>>,
        rows: &mut Vec<(Vec<usize>, usize)>,
    ) {
        for (index, node) in nodes.iter().enumerate() {
            let mut path = prefix.to_vec();
            path.push(index);
            rows.push((path.clone(), prefix.len()));
            if expanded.contains(&path) {
                walk(&node.children, &path, expanded, rows);
            }
        }
    }
    let mut rows = Vec::new();
    walk(nodes, &[], expanded, &mut rows);
    rows
}

pub fn node_at<'a>(nodes: &'a [TreeNode], path: &[usize]) -> Option<&'a TreeNode> {
    let (first, rest) = path.split_first()?;
    let node = nodes.get(*first)?;
    if rest.is_empty() {
        Some(node)
    } else {
        node_at(&node.children, rest)
    }
}

pub(crate) fn row(
    node: &TreeNode,
    selected: bool,
    lead: Option<Element>,
    trail: Option<Element>,
) -> Rect {
    let palette = Palette::current();
    let ink = if selected {
        palette.on_accent
    } else {
        palette.label
    };
    rect()
        .width(Size::fill())
        .height(Size::px(DISCLOSURE.row_height))
        .corner_radius(DISCLOSURE.row_radius)
        .maybe(selected, |el| el.background(color(palette.accent)))
        .direction(Direction::Horizontal)
        .cross_align(Alignment::Center)
        .spacing(SPACE_1)
        .content(torin::prelude::Content::flex())
        .a11y_role(AccessibilityRole::TreeItem)
        .a11y_alt(node.title.clone())
        .a11y_builder(move |a11y| a11y.set_selected(selected))
        .maybe_child(lead)
        .maybe_child(node.icon.map(|icon| {
            symbol(
                icon,
                DISCLOSURE.icon,
                if selected { ink } else { palette.accent },
            )
        }))
        .child(
            rect()
                .width(Size::flex(1.0))
                .child(text(node.title.clone(), BODY, ink).max_lines(1)),
        )
        .maybe_child(trail)
}

#[derive(PartialEq)]
pub struct OutlineView {
    label: &'static str,
    nodes: Vec<TreeNode>,
    on_select: Option<Callback<Vec<Vec<usize>>, ()>>,
}

impl OutlineView {
    pub fn new(label: &'static str, nodes: Vec<TreeNode>) -> Self {
        Self {
            label,
            nodes,
            on_select: None,
        }
    }

    pub fn on_select(mut self, callback: impl Into<Callback<Vec<Vec<usize>>, ()>>) -> Self {
        self.on_select = Some(callback.into());
        self
    }
}

impl Component for OutlineView {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut expanded = use_state(HashSet::<Vec<usize>>::new);
        let mut selected = use_state(Vec::<Vec<usize>>::new);
        let held = use_held_modifiers();
        let rows = visible_rows(&self.nodes, &expanded.read());
        let paths: Vec<Vec<usize>> = rows.iter().map(|(path, _)| path.clone()).collect();
        let items = rows
            .into_iter()
            .enumerate()
            .filter_map(|(row_index, (path, depth))| {
                let node = node_at(&self.nodes, &path)?;
                let open = expanded.read().contains(&path);
                let chosen = selected.read().contains(&path);
                let ink = if chosen {
                    palette.on_accent
                } else {
                    palette.secondary_label
                };
                let toggle_path = path.clone();
                let chevron = rect()
                    .width(Size::px(DISCLOSURE.indent))
                    .center()
                    .maybe(!node.children.is_empty(), |el| {
                        el.a11y_role(AccessibilityRole::Button)
                            .on_press(move |event: Event<PressEventData>| {
                                event.stop_propagation();
                                let mut paths = expanded.write();
                                if !paths.remove(&toggle_path) {
                                    paths.insert(toggle_path.clone());
                                }
                            })
                            .child(symbol(
                                if open {
                                    Symbol::ChevronDown
                                } else {
                                    Symbol::ChevronRight
                                },
                                DISCLOSURE.chevron,
                                ink,
                            ))
                    })
                    .into_element();
                let on_select = self.on_select.clone();
                let paths = paths.clone();
                Some(
                    rect()
                        .width(Size::fill())
                        .padding(Gaps::new(0.0, 0.0, 0.0, DISCLOSURE.indent * depth as f32))
                        .child(
                            row(node, chosen, Some(chevron), None)
                                .a11y_builder(move |a11y| a11y.set_expanded(open))
                                .on_press(move |_| {
                                    let current: Vec<usize> = selected
                                        .peek()
                                        .iter()
                                        .filter_map(|path| paths.iter().position(|row| row == path))
                                        .collect();
                                    let next: Vec<Vec<usize>> =
                                        select(&current, row_index, *held.peek())
                                            .into_iter()
                                            .map(|index| paths[index].clone())
                                            .collect();
                                    if let Some(on_select) = &on_select {
                                        on_select.call(next.clone());
                                    }
                                    selected.set(next);
                                }),
                        )
                        .into_element(),
                )
            });
        track_modifiers(rect(), held)
            .a11y_role(AccessibilityRole::Tree)
            .a11y_alt(self.label)
            .expanded()
            .child(ScrollView::new().expanded().children(items))
    }
}

#[derive(PartialEq)]
pub struct ColumnBrowser {
    label: &'static str,
    nodes: Vec<TreeNode>,
    on_select: Option<Callback<Vec<usize>, ()>>,
}

impl ColumnBrowser {
    pub fn new(label: &'static str, nodes: Vec<TreeNode>) -> Self {
        Self {
            label,
            nodes,
            on_select: None,
        }
    }

    pub fn on_select(mut self, callback: impl Into<Callback<Vec<usize>, ()>>) -> Self {
        self.on_select = Some(callback.into());
        self
    }
}

impl Component for ColumnBrowser {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut path = use_state(Vec::<usize>::new);
        let current = path.read().clone();
        let mut columns: Vec<(Vec<usize>, &[TreeNode])> = vec![(Vec::new(), &self.nodes)];
        for depth in 0..current.len() {
            let prefix = current[..=depth].to_vec();
            match node_at(&self.nodes, &prefix) {
                Some(node) if !node.children.is_empty() => columns.push((prefix, &node.children)),
                _ => break,
            }
        }
        let views = columns.into_iter().map(|(prefix, nodes)| {
            let rows = nodes.iter().enumerate().map(|(index, node)| {
                let mut target = prefix.clone();
                target.push(index);
                let chosen =
                    current.get(prefix.len()) == Some(&index) && current.starts_with(&prefix);
                let trail = (!node.children.is_empty()).then(|| {
                    symbol(
                        Symbol::ChevronRight,
                        DISCLOSURE.chevron,
                        if chosen {
                            palette.on_accent
                        } else {
                            palette.tertiary_label
                        },
                    )
                    .into_element()
                });
                let on_select = self.on_select.clone();
                row(node, chosen, None, trail)
                    .padding(Gaps::new(0.0, SPACE_1, 0.0, SPACE_1))
                    .on_press(move |_| {
                        path.set(target.clone());
                        if let Some(on_select) = &on_select {
                            on_select.call(target.clone());
                        }
                    })
                    .into_element()
            });
            rect()
                .width(Size::px(DISCLOSURE.column_width))
                .height(Size::fill())
                .direction(Direction::Horizontal)
                .child(
                    ScrollView::new()
                        .width(Size::flex(1.0))
                        .height(Size::fill())
                        .children(rows),
                )
                .child(
                    rect()
                        .width(Size::px(HAIRLINE))
                        .height(Size::fill())
                        .background(color(palette.separator)),
                )
                .content(torin::prelude::Content::flex())
                .into_element()
        });
        rect()
            .a11y_role(AccessibilityRole::Group)
            .a11y_alt(self.label)
            .expanded()
            .child(
                ScrollView::new()
                    .direction(Direction::Horizontal)
                    .expanded()
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .height(Size::fill())
                            .children(views),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Vec<TreeNode> {
        vec![
            TreeNode::new("a")
                .child(TreeNode::new("a1"))
                .child(TreeNode::new("a2")),
            TreeNode::new("b"),
        ]
    }

    #[test]
    fn collapsed_nodes_hide_their_children_and_expanded_ones_indent_them() {
        let nodes = tree();
        let closed = visible_rows(&nodes, &HashSet::new());
        assert_eq!(closed, vec![(vec![0], 0), (vec![1], 0)]);
        let open = visible_rows(&nodes, &HashSet::from([vec![0]]));
        assert_eq!(
            open,
            vec![(vec![0], 0), (vec![0, 0], 1), (vec![0, 1], 1), (vec![1], 0)]
        );
    }

    #[test]
    fn a_path_finds_its_node_and_a_wrong_path_finds_none() {
        let nodes = tree();
        assert_eq!(
            node_at(&nodes, &[0, 1]).map(|n| n.title.as_str()),
            Some("a2")
        );
        assert!(node_at(&nodes, &[1, 0]).is_none());
        assert!(node_at(&nodes, &[]).is_none());
    }
}
