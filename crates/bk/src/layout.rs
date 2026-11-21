use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use ctdesign::SPACE_2;
use freya_core::data::{AccessibilityData, LayoutData};
use freya_core::element::{ElementExt, LayoutContext, PostMeasureContext};
use freya_core::elements::rect::RectElement;
use freya_core::node_id::NodeId;
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use torin::prelude::{
    Alignment, Content, Direction, Length, Position, PostMeasure, Size, Size2D, Torin,
};

use crate::elements::forward_container;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HorizontalAlignment {
    Leading,
    #[default]
    Center,
    Trailing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VerticalAlignment {
    Top,
    #[default]
    Center,
    Bottom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContentAlignment {
    pub horizontal: HorizontalAlignment,
    pub vertical: VerticalAlignment,
}

impl ContentAlignment {
    const fn new(horizontal: HorizontalAlignment, vertical: VerticalAlignment) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    pub const TOP_LEADING: Self = Self::new(HorizontalAlignment::Leading, VerticalAlignment::Top);
    pub const TOP: Self = Self::new(HorizontalAlignment::Center, VerticalAlignment::Top);
    pub const TOP_TRAILING: Self = Self::new(HorizontalAlignment::Trailing, VerticalAlignment::Top);
    pub const LEADING: Self = Self::new(HorizontalAlignment::Leading, VerticalAlignment::Center);
    pub const CENTER: Self = Self::new(HorizontalAlignment::Center, VerticalAlignment::Center);
    pub const TRAILING: Self = Self::new(HorizontalAlignment::Trailing, VerticalAlignment::Center);
    pub const BOTTOM_LEADING: Self =
        Self::new(HorizontalAlignment::Leading, VerticalAlignment::Bottom);
    pub const BOTTOM: Self = Self::new(HorizontalAlignment::Center, VerticalAlignment::Bottom);
    pub const BOTTOM_TRAILING: Self =
        Self::new(HorizontalAlignment::Trailing, VerticalAlignment::Bottom);
}

impl From<HorizontalAlignment> for Alignment {
    fn from(alignment: HorizontalAlignment) -> Self {
        match alignment {
            HorizontalAlignment::Leading => Alignment::Start,
            HorizontalAlignment::Center => Alignment::Center,
            HorizontalAlignment::Trailing => Alignment::End,
        }
    }
}

impl From<VerticalAlignment> for Alignment {
    fn from(alignment: VerticalAlignment) -> Self {
        match alignment {
            VerticalAlignment::Top => Alignment::Start,
            VerticalAlignment::Center => Alignment::Center,
            VerticalAlignment::Bottom => Alignment::End,
        }
    }
}

fn share_of_free_space(alignment: impl Into<Alignment>) -> f32 {
    match alignment.into() {
        Alignment::Center => 0.5,
        Alignment::End => 1.0,
        _ => 0.0,
    }
}

fn layout_diff<T: PartialEq + 'static>(element: &T, other: &Rc<dyn ElementExt>) -> DiffModifies {
    match (other.as_ref() as &dyn Any).downcast_ref::<T>() {
        Some(other) if other == element => DiffModifies::empty(),
        Some(_) => DiffModifies::LAYOUT,
        None => DiffModifies::all(),
    }
}

fn container_accessibility() -> AccessibilityData {
    let mut data = AccessibilityData::default();
    data.builder.set_role(accesskit::Role::GenericContainer);
    data
}

fn custom(element: impl ElementExt, elements: Vec<Element>) -> Element {
    Element::Element {
        key: DiffKey::None,
        element: Rc::new(element),
        elements,
    }
}

pub struct Frame(Rect);

impl Frame {
    pub fn new(content: impl IntoElement) -> Self {
        Self(
            rect()
                .main_align(Alignment::Center)
                .cross_align(Alignment::Center)
                .child(content),
        )
    }

    pub fn alignment(self, alignment: ContentAlignment) -> Self {
        Self(
            self.0
                .main_align(alignment.vertical)
                .cross_align(alignment.horizontal),
        )
    }
}

forward_container!(Frame);

#[derive(Clone, Copy, PartialEq)]
pub struct Spacer {
    min_length: f32,
    axis: Option<Direction>,
}

impl Spacer {
    pub fn new() -> Self {
        Self {
            min_length: SPACE_2,
            axis: None,
        }
    }

    pub fn min_length(self, min_length: f32) -> Self {
        Self { min_length, ..self }
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Spacer> for Element {
    fn from(spacer: Spacer) -> Self {
        custom(spacer, Vec::new())
    }
}

impl ElementExt for Spacer {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        !layout_diff(self, other).is_empty()
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        layout_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        let along = |direction| self.axis.is_none_or(|axis| axis == direction);
        let (grows_wide, grows_tall) = (along(Direction::Horizontal), along(Direction::Vertical));
        let size = |grows: bool| if grows { Size::flex(1.0) } else { Size::Inner };
        let minimum = |grows: bool| {
            if grows {
                Size::px(self.min_length)
            } else {
                Size::Inner
            }
        };
        Cow::Owned(
            torin::node::Node {
                width: size(grows_wide),
                height: size(grows_tall),
                minimum_width: minimum(grows_wide),
                minimum_height: minimum(grows_tall),
                ..Default::default()
            }
            .into(),
        )
    }
}

// A stack with a Spacer shares its space with flex and fills its axis, and the Spacer takes that axis (ADR 0089).
pub(crate) fn resolve_spacers(mut element: Element) -> Element {
    let Element::Element {
        element: node,
        elements,
        ..
    } = &mut element
    else {
        return element;
    };
    let Some(stack) = (node.as_ref() as &dyn Any).downcast_ref::<RectElement>() else {
        return element;
    };
    let axis = stack.layout.direction;
    let mut flex_stack = stack.clone();
    let mut has_spacer = false;
    for child in elements.iter_mut() {
        let Element::Element { element: inner, .. } = child else {
            continue;
        };
        if let Some(spacer) = (inner.as_ref() as &dyn Any).downcast_ref::<Spacer>() {
            *inner = Rc::new(Spacer {
                axis: Some(axis),
                ..*spacer
            });
            has_spacer = true;
        }
    }
    if has_spacer {
        if flex_stack.layout.content == Content::Normal {
            flex_stack.layout.content = Content::Flex;
        }
        let along = match axis {
            Direction::Horizontal => &mut flex_stack.layout.width,
            Direction::Vertical => &mut flex_stack.layout.height,
        };
        if *along == Size::Inner {
            *along = Size::Fill;
        }
        *node = Rc::new(flex_stack);
    }
    element
}

const WHOLE_ROW: usize = usize::MAX;

#[derive(Clone, Copy, PartialEq)]
struct CellSpec {
    columns: usize,
    alignment: Option<HorizontalAlignment>,
}

#[derive(Clone, PartialEq)]
struct RowSpec {
    alignment: Option<VerticalAlignment>,
    cells: Vec<CellSpec>,
}

struct Slot {
    row: usize,
    column: usize,
    columns: usize,
    alignment: Option<HorizontalAlignment>,
}

pub struct GridCell {
    content: Element,
    spec: CellSpec,
}

impl GridCell {
    fn new(content: Element) -> Self {
        Self {
            content,
            spec: CellSpec {
                columns: 1,
                alignment: None,
            },
        }
    }

    pub fn grid_cell_columns(mut self, columns: usize) -> Self {
        self.spec.columns = columns.max(1);
        self
    }

    pub fn grid_column_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.spec.alignment = Some(alignment);
        self
    }
}

impl From<GridCell> for Element {
    fn from(cell: GridCell) -> Self {
        custom(CellElement(cell.spec), vec![cell.content])
    }
}

pub trait GridModifiers: IntoElement + Sized {
    fn grid_cell_columns(self, columns: usize) -> GridCell {
        GridCell::new(self.into_element()).grid_cell_columns(columns)
    }

    fn grid_column_alignment(self, alignment: HorizontalAlignment) -> GridCell {
        GridCell::new(self.into_element()).grid_column_alignment(alignment)
    }
}

impl<T: IntoElement> GridModifiers for T {}

#[derive(PartialEq)]
struct CellElement(CellSpec);

impl ElementExt for CellElement {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        !layout_diff(self, other).is_empty()
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        layout_diff(self, other)
    }

    fn accessibility(&'_ self) -> Cow<'_, AccessibilityData> {
        Cow::Owned(container_accessibility())
    }
}

pub struct GridRow {
    alignment: Option<VerticalAlignment>,
    cells: Vec<Element>,
}

impl GridRow {
    pub fn new() -> Self {
        Self {
            alignment: None,
            cells: Vec::new(),
        }
    }

    pub fn alignment(self, alignment: VerticalAlignment) -> Self {
        Self {
            alignment: Some(alignment),
            ..self
        }
    }
}

impl Default for GridRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ChildrenExt for GridRow {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.cells
    }
}

pub struct Grid {
    element: GridElement,
    key: DiffKey,
    cells: Vec<Element>,
}

impl Grid {
    pub fn new() -> Self {
        let layout = torin::node::Node {
            direction: Direction::Horizontal,
            spacing: Length::new(SPACE_2),
            ..Default::default()
        };
        Self {
            element: GridElement {
                layout: layout.into(),
                accessibility: container_accessibility(),
                alignment: ContentAlignment::CENTER,
                vertical_spacing: SPACE_2,
                rows: Vec::new(),
            },
            key: DiffKey::None,
            cells: Vec::new(),
        }
    }

    pub fn alignment(mut self, alignment: ContentAlignment) -> Self {
        self.element.alignment = alignment;
        self
    }

    pub fn horizontal_spacing(mut self, spacing: f32) -> Self {
        self.element.layout.spacing = Length::new(spacing);
        self
    }

    pub fn vertical_spacing(mut self, spacing: f32) -> Self {
        self.element.vertical_spacing = spacing;
        self
    }

    pub fn row(mut self, row: GridRow) -> Self {
        let cells = row
            .cells
            .into_iter()
            .map(|cell| self.take_spec(cell))
            .collect();
        self.element.rows.push(RowSpec {
            alignment: row.alignment,
            cells,
        });
        self
    }

    pub fn child(mut self, content: impl IntoElement) -> Self {
        self.cells.push(content.into_element());
        self.element.rows.push(RowSpec {
            alignment: None,
            cells: vec![CellSpec {
                columns: WHOLE_ROW,
                alignment: None,
            }],
        });
        self
    }

    fn take_spec(&mut self, cell: Element) -> CellSpec {
        let spec = match &cell {
            Element::Element { element, .. } => (element.as_ref() as &dyn Any)
                .downcast_ref::<CellElement>()
                .map(|wrapper| wrapper.0),
            Element::Component { .. } => None,
        };
        self.cells.push(cell);
        spec.unwrap_or(CellSpec {
            columns: 1,
            alignment: None,
        })
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

// A column is a vertical stack so torin sizes it before post_measure; spanning cells float above the stacks (ADR 0089).
impl From<Grid> for Element {
    fn from(grid: Grid) -> Self {
        let Grid {
            element,
            key,
            cells,
        } = grid;
        let mut stacks = vec![Vec::new(); element.column_count()];
        let mut floating = Vec::new();
        for (slot, cell) in element.slots().iter().zip(cells) {
            if slot.columns == 1 {
                stacks[slot.column].push(cell);
            } else {
                floating.push(custom(FloatingElement, vec![cell]));
            }
        }
        let spacing = element.vertical_spacing;
        let columns = stacks
            .into_iter()
            .map(|cells| custom(ColumnElement::new(spacing), cells));
        Element::Element {
            key,
            elements: columns.chain(floating).collect(),
            element: Rc::new(element),
        }
    }
}

impl KeyExt for Grid {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl LayoutExt for Grid {
    fn layout_mut(&mut self) -> &mut LayoutData {
        &mut self.element.layout
    }
}

impl ContainerExt for Grid {}

impl AccessibilityExt for Grid {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.element.accessibility
    }
}

#[derive(PartialEq)]
struct FloatingElement;

impl ElementExt for FloatingElement {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        !layout_diff(self, other).is_empty()
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        layout_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Owned(
            torin::node::Node {
                position: Position::new_absolute(),
                ..Default::default()
            }
            .into(),
        )
    }

    fn accessibility(&'_ self) -> Cow<'_, AccessibilityData> {
        Cow::Owned(container_accessibility())
    }
}

struct ColumnCells(RefCell<Vec<NodeId>>);

struct ColumnElement {
    spacing: f32,
    cells: Rc<ColumnCells>,
}

impl ColumnElement {
    fn new(spacing: f32) -> Self {
        Self {
            spacing,
            cells: Rc::new(ColumnCells(RefCell::default())),
        }
    }
}

impl PartialEq for ColumnElement {
    fn eq(&self, other: &Self) -> bool {
        self.spacing == other.spacing
    }
}

// The column hands its cells to the Grid through the node data that torin keeps (torin custom_measurer.rs).
impl ElementExt for ColumnElement {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        !layout_diff(self, other).is_empty()
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        layout_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Owned(
            torin::node::Node {
                spacing: Length::new(self.spacing),
                ..Default::default()
            }
            .into(),
        )
    }

    fn accessibility(&'_ self) -> Cow<'_, AccessibilityData> {
        Cow::Owned(container_accessibility())
    }

    fn should_hook_measurement(&self) -> bool {
        true
    }

    fn measure(&self, _context: LayoutContext) -> Option<(Size2D, Rc<dyn Any>)> {
        Some((Size2D::zero(), self.cells.clone()))
    }

    fn needs_post_measure(&self) -> bool {
        true
    }

    fn post_measure(&self, context: PostMeasureContext) -> PostMeasure<NodeId> {
        self.cells.0.replace(context.children.to_vec());
        PostMeasure::default()
    }
}

fn cells_of(layout: &Torin<NodeId>, column: &NodeId) -> Vec<NodeId> {
    layout
        .get(column)
        .and_then(|node| node.data.clone())
        .and_then(|data| data.downcast::<ColumnCells>().ok())
        .map(|cells| cells.0.borrow().clone())
        .unwrap_or_default()
}

#[derive(PartialEq)]
struct GridElement {
    layout: LayoutData,
    accessibility: AccessibilityData,
    alignment: ContentAlignment,
    vertical_spacing: f32,
    rows: Vec<RowSpec>,
}

struct Placement {
    origins: Vec<(f32, f32)>,
    size: Size2D,
}

impl GridElement {
    fn horizontal_spacing(&self) -> f32 {
        self.layout.spacing.get()
    }

    fn column_count(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.cells.iter().all(|cell| cell.columns != WHOLE_ROW))
            .map(|row| row.cells.iter().map(|cell| cell.columns).sum())
            .max()
            .unwrap_or(1)
            .max(1)
    }

    fn slots(&self) -> Vec<Slot> {
        let columns = self.column_count();
        let mut slots = Vec::new();
        for (row, spec) in self.rows.iter().enumerate() {
            let mut column = 0;
            for cell in &spec.cells {
                let spanned = cell.columns.min(columns - column).max(1);
                slots.push(Slot {
                    row,
                    column,
                    columns: spanned,
                    alignment: cell.alignment,
                });
                column += spanned;
            }
        }
        slots
    }

    fn place(&self, slots: &[Slot], sizes: &[Size2D]) -> Placement {
        let (spacing_x, spacing_y) = (self.horizontal_spacing(), self.vertical_spacing);
        let mut widths = vec![0.0_f32; self.column_count()];
        let mut heights = vec![0.0_f32; self.rows.len()];
        let mut column_alignment = vec![None; widths.len()];
        for (slot, size) in slots.iter().zip(sizes) {
            heights[slot.row] = heights[slot.row].max(size.height);
            if slot.columns == 1 {
                widths[slot.column] = widths[slot.column].max(size.width);
                column_alignment[slot.column] = column_alignment[slot.column].or(slot.alignment);
            }
        }
        let span_width = |widths: &[f32], slot: &Slot| {
            widths[slot.column..slot.column + slot.columns]
                .iter()
                .sum::<f32>()
                + spacing_x * (slot.columns - 1) as f32
        };
        for (slot, size) in slots.iter().zip(sizes) {
            if slot.columns > 1 {
                let extra = (size.width - span_width(&widths, slot)).max(0.0) / slot.columns as f32;
                for width in &mut widths[slot.column..slot.column + slot.columns] {
                    *width += extra;
                }
            }
        }

        let starts = |lengths: &[f32], spacing: f32| {
            lengths
                .iter()
                .scan(0.0, |start, length| {
                    let here = *start;
                    *start += length + spacing;
                    Some(here)
                })
                .collect::<Vec<_>>()
        };
        let (column_x, row_y) = (starts(&widths, spacing_x), starts(&heights, spacing_y));
        let origins = slots
            .iter()
            .zip(sizes)
            .map(|(slot, size)| {
                let horizontal = if slot.columns == 1 {
                    column_alignment[slot.column].unwrap_or(self.alignment.horizontal)
                } else {
                    self.alignment.horizontal
                };
                let vertical = self.rows[slot.row]
                    .alignment
                    .unwrap_or(self.alignment.vertical);
                (
                    column_x[slot.column]
                        + (span_width(&widths, slot) - size.width)
                            * share_of_free_space(horizontal),
                    row_y[slot.row]
                        + (heights[slot.row] - size.height) * share_of_free_space(vertical),
                )
            })
            .collect();
        let total = |lengths: &[f32], spacing: f32| {
            lengths.iter().sum::<f32>() + spacing * lengths.len().saturating_sub(1) as f32
        };
        Placement {
            origins,
            size: Size2D::new(total(&widths, spacing_x), total(&heights, spacing_y)),
        }
    }
}

// Like Freya's paragraph with inline children (freya-core 0.5.3 elements/paragraph.rs), post_measure moves each cell to its slot (ADR 0089).
impl ElementExt for GridElement {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        !layout_diff(self, other).is_empty()
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        layout_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn accessibility(&'_ self) -> Cow<'_, AccessibilityData> {
        Cow::Borrowed(&self.accessibility)
    }

    fn needs_post_measure(&self) -> bool {
        true
    }

    fn post_measure(&self, context: PostMeasureContext) -> PostMeasure<NodeId> {
        let slots = self.slots();
        let (columns, floating) = context
            .children
            .split_at(self.column_count().min(context.children.len()));
        let stacks = columns
            .iter()
            .map(|column| cells_of(context.layout, column))
            .collect::<Vec<_>>();
        let (mut taken, mut floating) = (vec![0; stacks.len()], floating.iter());
        let nodes = slots
            .iter()
            .map(|slot| {
                if slot.columns == 1 {
                    let next = taken.get_mut(slot.column)?;
                    *next += 1;
                    stacks[slot.column].get(*next - 1).copied()
                } else {
                    floating.next().copied()
                }
            })
            .collect::<Vec<_>>();
        let areas = nodes
            .iter()
            .map(|node| node.and_then(|node| context.layout.get(&node).map(|found| found.area)))
            .collect::<Vec<_>>();
        let sizes = areas
            .iter()
            .map(|area| area.map_or_else(Size2D::zero, |area| area.size))
            .collect::<Vec<_>>();
        let placement = self.place(&slots, &sizes);
        let origin = context.node_layout.inner_area.origin;
        let offsets = nodes
            .iter()
            .zip(&areas)
            .zip(&placement.origins)
            .filter_map(|((node, area), (x, y))| {
                let (node, area) = (node.as_ref()?, area.as_ref()?);
                Some((
                    *node,
                    Length::new(origin.x + x - area.origin.x),
                    Length::new(origin.y + y - area.origin.y),
                ))
            })
            .collect();
        PostMeasure {
            content_size: Some(Size2D::new(
                placement.size.width + self.layout.padding.horizontal(),
                placement.size.height + self.layout.padding.vertical(),
            )),
            offsets,
            hidden_children: Vec::new(),
        }
    }
}
