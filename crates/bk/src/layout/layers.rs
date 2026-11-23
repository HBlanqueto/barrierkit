use std::any::Any;
use std::borrow::Cow;
use std::rc::Rc;

use freya_core::data::{AccessibilityData, LayoutData};
use freya_core::element::{ElementExt, PostMeasureContext};
use freya_core::node_id::NodeId;
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use torin::prelude::{Length, Position, PostMeasure, Size, Size2D};

use crate::layout::direction::{ambient, flipped, mirrored_x};
use crate::layout::{
    ContentAlignment, Placement, container_accessibility, layout_diff, share_of_free_space,
};
use crate::styles::LayoutDirection;

pub trait Layout: PartialEq + 'static {
    /// Returns the container's size and the origin of each subview in it, given the space offered and each subview's size.
    fn place_subviews(&self, proposal: Size2D, subviews: &[Size2D]) -> Placement;

    /// The subview torin measures in the flow, which sizes the container for its parent until the layout corrects it.
    fn sizing_subview(&self) -> usize {
        0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZStackLayout {
    pub alignment: ContentAlignment,
    base: Option<usize>,
}

impl ZStackLayout {
    pub fn new(alignment: ContentAlignment) -> Self {
        Self {
            alignment,
            base: None,
        }
    }

    fn sized_by(alignment: ContentAlignment, base: usize) -> Self {
        Self {
            alignment,
            base: Some(base),
        }
    }
}

impl Layout for ZStackLayout {
    fn place_subviews(&self, _proposal: Size2D, subviews: &[Size2D]) -> Placement {
        let extent = |of: fn(&Size2D) -> f32| match self.base {
            Some(base) => subviews.get(base).map_or(0.0, of),
            None => subviews.iter().map(of).fold(0.0, f32::max),
        };
        let size = Size2D::new(extent(|size| size.width), extent(|size| size.height));
        let origins = subviews
            .iter()
            .map(|subview| {
                (
                    (size.width - subview.width) * share_of_free_space(self.alignment.horizontal),
                    (size.height - subview.height) * share_of_free_space(self.alignment.vertical),
                )
            })
            .collect();
        Placement { size, origins }
    }

    fn sizing_subview(&self) -> usize {
        self.base.unwrap_or(0)
    }
}

trait Arrangement {
    fn place(&self, proposal: Size2D, subviews: &[Size2D]) -> Placement;
    fn sizing(&self) -> usize;
    fn same_as(&self, other: &dyn Arrangement) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<L: Layout> Arrangement for L {
    fn place(&self, proposal: Size2D, subviews: &[Size2D]) -> Placement {
        self.place_subviews(proposal, subviews)
    }

    fn sizing(&self) -> usize {
        self.sizing_subview()
    }

    fn same_as(&self, other: &dyn Arrangement) -> bool {
        other.as_any().downcast_ref::<L>() == Some(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub(crate) struct LayoutElement {
    layout: LayoutData,
    accessibility: AccessibilityData,
    arrangement: Rc<dyn Arrangement>,
    direction: LayoutDirection,
}

impl PartialEq for LayoutElement {
    fn eq(&self, other: &Self) -> bool {
        self.layout == other.layout
            && self.accessibility == other.accessibility
            && self.direction == other.direction
            && self.arrangement.same_as(other.arrangement.as_ref())
    }
}

impl LayoutElement {
    pub(crate) fn flipped(&self) -> Self {
        Self {
            direction: flipped(self.direction),
            ..self.clone()
        }
    }
}

// Torin measures the subviews with the space the container has; `post_measure` then moves them where the layout says (ADR 0110).
impl ElementExt for LayoutElement {
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
        let areas = context
            .children
            .iter()
            .map(|child| context.layout.get(child).map(|found| found.area))
            .collect::<Vec<_>>();
        let sizes = areas
            .iter()
            .map(|area| area.map_or_else(Size2D::zero, |area| area.size))
            .collect::<Vec<_>>();
        let inner = context.node_layout.inner_area;
        let mut placement = self.arrangement.place(inner.size.cast_unit(), &sizes);
        let sized_by_content = self.layout.width == Size::Inner;
        let width = if sized_by_content {
            placement.size.width
        } else {
            inner.width()
        };
        if self.direction == LayoutDirection::RightToLeft {
            placement.origins = placement
                .origins
                .iter()
                .zip(&sizes)
                .map(|(&(x, y), size)| (mirrored_x(x, size.width, width), y))
                .collect();
        }
        let offsets = context
            .children
            .iter()
            .zip(&areas)
            .zip(&placement.origins)
            .filter_map(|((child, area), (x, y))| {
                let area = area.as_ref()?;
                Some((
                    *child,
                    Length::new(inner.origin.x + x - area.origin.x),
                    Length::new(inner.origin.y + y - area.origin.y),
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

pub struct LayoutContainer<L: Layout> {
    layout: LayoutData,
    accessibility: AccessibilityData,
    arrangement: L,
    key: DiffKey,
    subviews: Vec<Element>,
}

impl<L: Layout> LayoutContainer<L> {
    pub fn with_layout(arrangement: L) -> Self {
        Self {
            layout: LayoutData::default(),
            accessibility: container_accessibility(),
            arrangement,
            key: DiffKey::None,
            subviews: Vec::new(),
        }
    }
}

// Only the sizing subview stays in the flow, so a parent that centers the container sees the size it will have; the others are absolute.
impl<L: Layout> From<LayoutContainer<L>> for Element {
    fn from(container: LayoutContainer<L>) -> Self {
        let LayoutContainer {
            layout,
            accessibility,
            arrangement,
            key,
            subviews,
        } = container;
        let sizing = arrangement.sizing_subview();
        let element = LayoutElement {
            layout,
            accessibility,
            arrangement: Rc::new(arrangement),
            direction: ambient(),
        };
        let elements = subviews
            .into_iter()
            .enumerate()
            .map(|(index, subview)| {
                if index == sizing {
                    subview
                } else {
                    rect()
                        .position(Position::new_absolute())
                        .child(subview)
                        .into_element()
                }
            })
            .collect();
        Element::Element {
            key,
            element: Rc::new(element),
            elements,
        }
    }
}

impl<L: Layout> KeyExt for LayoutContainer<L> {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl<L: Layout> LayoutExt for LayoutContainer<L> {
    fn layout_mut(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl<L: Layout> ChildrenExt for LayoutContainer<L> {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.subviews
    }
}

impl<L: Layout> ContainerExt for LayoutContainer<L> {}

impl<L: Layout> AccessibilityExt for LayoutContainer<L> {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

pub type ZStack = LayoutContainer<ZStackLayout>;

impl ZStack {
    pub fn new() -> Self {
        Self::with_layout(ZStackLayout::default())
    }

    pub fn alignment(mut self, alignment: ContentAlignment) -> Self {
        self.arrangement.alignment = alignment;
        self
    }
}

impl Default for ZStack {
    fn default() -> Self {
        Self::new()
    }
}

pub trait OverlayModifiers: IntoElement + Sized {
    /// Draws `content` over the view, placed by `alignment` in the view's frame; the view alone sizes the result.
    fn overlay(self, alignment: ContentAlignment, content: impl IntoElement) -> ZStack {
        ZStack::with_layout(ZStackLayout::sized_by(alignment, 0))
            .child(self)
            .child(content)
    }

    /// Draws `content` behind the view, placed by `alignment` in the view's frame; the view alone sizes the result.
    fn background_view(self, alignment: ContentAlignment, content: impl IntoElement) -> ZStack {
        ZStack::with_layout(ZStackLayout::sized_by(alignment, 1))
            .child(content)
            .child(self)
    }
}

impl<T: IntoElement> OverlayModifiers for T {}
