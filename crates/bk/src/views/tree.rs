use std::borrow::Cow;
use std::collections::HashSet;
use std::rc::Rc;

use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use crate::elements::{Overrides, forward_overrides};
use crate::interaction::{track_modifiers, use_held_modifiers, use_interaction};
use crate::scroll::ScrollView;
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    ControlState, LayoutDirection, OutlineViewStyle, Paints, StateValue, Theme, resolve,
};
use crate::symbol::{Symbol, stroked_symbol, symbol};
use crate::views::collection::select;
use crate::views::table::row_divider;

#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode {
    pub title: Cow<'static, str>,
    pub icon: Option<Symbol>,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(title: impl Into<Cow<'static, str>>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            expanded: false,
            children: Vec::new(),
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
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

pub fn open_paths(nodes: &[TreeNode]) -> HashSet<Vec<usize>> {
    fn walk(nodes: &[TreeNode], prefix: &[usize], open: &mut HashSet<Vec<usize>>) {
        for (index, node) in nodes.iter().enumerate() {
            let mut path = prefix.to_vec();
            path.push(index);
            if node.expanded {
                open.insert(path.clone());
            }
            walk(&node.children, &path, open);
        }
    }
    let mut open = HashSet::new();
    walk(nodes, &[], &mut open);
    open
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
    style: &OutlineViewStyle,
    paints: &Paints,
    state: ControlState,
    node: &TreeNode,
    selected: bool,
    lead: Option<Element>,
    trail: Option<Element>,
) -> Rect {
    let (background, label, icon) = if selected {
        (
            &style.selected_row_background,
            &style.selected_label,
            &style.selected_icon,
        )
    } else {
        (&style.row_background, &style.label, &style.icon)
    };
    rect()
        .width(Size::fill())
        .height(Size::px(style.row_height))
        .corner_radius(style.row_shape.corner_radius(
            style.row_height,
            style.row_height,
            LayoutDirection::LeftToRight,
        ))
        .painted(paints, background)
        .direction(Direction::Horizontal)
        .cross_align(Alignment::Center)
        .content(torin::prelude::Content::flex())
        .a11y_role(AccessibilityRole::TreeItem)
        .a11y_alt(node.title.clone())
        .a11y_builder(move |a11y| a11y.set_selected(selected))
        .maybe_child(lead)
        .maybe_child(node.icon.map(|glyph| {
            rect()
                .margin(Gaps::new(0.0, style.spacing, 0.0, 0.0))
                .child(symbol(
                    glyph,
                    style.icon_size,
                    paints.color(icon.resolve(state)),
                ))
        }))
        .child(
            rect().width(Size::flex(1.0)).child(
                text(
                    node.title.clone(),
                    style.font,
                    paints.color(label.resolve(state)),
                )
                .max_lines(1),
            ),
        )
        .maybe_child(trail)
}

#[derive(PartialEq)]
pub struct OutlineView {
    label: Cow<'static, str>,
    nodes: Vec<TreeNode>,
    selection: Option<State<Vec<Vec<usize>>>>,
    scroll_bar: bool,
    style: Theme,
    overrides: Overrides,
}

impl OutlineView {
    pub fn new(label: impl Into<Cow<'static, str>>, nodes: Vec<TreeNode>) -> Self {
        Self {
            label: label.into(),
            nodes,
            selection: None,
            scroll_bar: true,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(OutlineViewStyle) -> OutlineViewStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn selection(mut self, selection: State<Vec<Vec<usize>>>) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn scroll_bar(mut self, shown: bool) -> Self {
        self.scroll_bar = shown;
        self
    }
}

forward_overrides!(OutlineView);

impl Component for OutlineView {
    fn render(&self) -> impl IntoElement {
        let style: OutlineViewStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = ControlState {
            focused: interaction.focused(),
            ..ControlState::default()
        };
        let paints = Paints::current(state);
        let mut expanded = use_state(|| open_paths(&self.nodes));
        let own = use_state(Vec::new);
        let mut selected = self.selection.unwrap_or(own);
        let held = use_held_modifiers();
        let rows = visible_rows(&self.nodes, &expanded.read());
        let paths: Rc<Vec<Vec<usize>>> =
            Rc::new(rows.iter().map(|(path, _)| path.clone()).collect());
        let items = rows
            .into_iter()
            .enumerate()
            .filter_map(|(row_index, (path, depth))| {
                let node = node_at(&self.nodes, &path)?;
                let open = expanded.read().contains(&path);
                let chosen = selected.read().contains(&path);
                let ink = paints.color(
                    if chosen {
                        &style.selected_chevron
                    } else {
                        &style.disclosure
                    }
                    .resolve(state),
                );
                let toggle_path = path.clone();
                let level = style.indent * depth as f32;
                let (shift_x, shift_y) = if open {
                    style.open_chevron_offset
                } else {
                    style.chevron_offset
                };
                let chevron = rect()
                    .width(Size::px(style.lead_width + level))
                    .padding(Gaps::new(0.0, 0.0, 0.0, level))
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
                            .child(rect().offset_x(shift_x).offset_y(shift_y).child(
                                stroked_symbol(
                                    if open {
                                        Symbol::ChevronDown
                                    } else {
                                        Symbol::ChevronRight
                                    },
                                    style.triangle_size.0,
                                    style.triangle_size.1,
                                    ink,
                                ),
                            ))
                    })
                    .into_element();
                let paths = paths.clone();
                let divided = !chosen
                    && path[0] + 1 < self.nodes.len()
                    && paths
                        .get(row_index + 1)
                        .is_some_and(|next| !selected.read().contains(next));
                let title_start = level
                    + style.lead_width
                    + node.icon.map_or(0.0, |_| style.icon_size + style.spacing);
                Some(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(style.row_height))
                        .padding(Gaps::new(0.0, style.inset, 0.0, style.inset))
                        .child(
                            row(&style, &paints, state, node, chosen, Some(chevron), None)
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
                                    selected.set(next);
                                }),
                        )
                        .maybe_child(style.row_separator.as_ref().filter(|_| divided).map(
                            |paint| {
                                row_divider(
                                    paints.fill(&StateValue::new(paint.clone())),
                                    title_start,
                                    style.separator_margin,
                                )
                            },
                        ))
                        .into_element(),
                )
            });
        let root = track_modifiers(interaction.attach(rect(), true), held)
            .a11y_role(AccessibilityRole::Tree)
            .a11y_alt(&*self.label)
            .expanded()
            .child(
                ScrollView::new()
                    .scroll_bar(self.scroll_bar)
                    .expanded()
                    .child(
                        rect()
                            .width(Size::fill())
                            .padding(Gaps::new(style.inset, 0.0, 0.0, 0.0))
                            .children(items),
                    ),
            );
        self.overrides.apply(root)
    }
}

#[derive(PartialEq)]
pub struct Browser {
    label: Cow<'static, str>,
    nodes: Vec<TreeNode>,
    selection: Option<State<Vec<usize>>>,
    style: Theme,
    overrides: Overrides,
}

impl Browser {
    pub fn new(label: impl Into<Cow<'static, str>>, nodes: Vec<TreeNode>) -> Self {
        Self {
            label: label.into(),
            nodes,
            selection: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(OutlineViewStyle) -> OutlineViewStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn selection(mut self, selection: State<Vec<usize>>) -> Self {
        self.selection = Some(selection);
        self
    }
}

forward_overrides!(Browser);

impl Component for Browser {
    fn render(&self) -> impl IntoElement {
        let style: OutlineViewStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = ControlState {
            focused: interaction.focused(),
            ..ControlState::default()
        };
        let paints = Paints::current(state);
        let own = use_state(Vec::<usize>::new);
        let mut path = self.selection.unwrap_or(own);
        let current = path.read().clone();
        let mut columns: Vec<(Vec<usize>, &[TreeNode])> = vec![(Vec::new(), &self.nodes)];
        for depth in 0..current.len() {
            let prefix = current[..=depth].to_vec();
            match node_at(&self.nodes, &prefix) {
                Some(node) if !node.children.is_empty() => columns.push((prefix, &node.children)),
                _ => break,
            }
        }
        let (separator, separator_width) = &style.column_separator;
        let separator_width = *separator_width;
        let views = columns.into_iter().map(|(prefix, nodes)| {
            let rows = nodes.iter().enumerate().map(|(index, node)| {
                let mut target = prefix.clone();
                target.push(index);
                let chosen =
                    current.get(prefix.len()) == Some(&index) && current.starts_with(&prefix);
                let trail = (!node.children.is_empty()).then(|| {
                    symbol(
                        Symbol::ChevronRight,
                        style.chevron_size,
                        paints.color(
                            if chosen {
                                &style.selected_chevron
                            } else {
                                &style.column_chevron
                            }
                            .resolve(state),
                        ),
                    )
                    .into_element()
                });
                row(&style, &paints, state, node, chosen, None, trail)
                    .padding(Gaps::new(
                        0.0,
                        style.column_padding_x,
                        0.0,
                        style.column_padding_x,
                    ))
                    .on_press(move |_| path.set(target.clone()))
                    .into_element()
            });
            rect()
                .width(Size::px(style.column_width))
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
                        .width(Size::px(separator_width))
                        .height(Size::fill())
                        .background(paints.fill(&StateValue::new(separator.clone()))),
                )
                .content(torin::prelude::Content::flex())
                .into_element()
        });
        let root = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Group)
            .a11y_alt(&*self.label)
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
            );
        self.overrides.apply(root)
    }
}
