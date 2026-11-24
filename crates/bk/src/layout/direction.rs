use std::any::Any;
use std::rc::Rc;

use freya_core::current_context::CurrentContext;
use freya_core::elements::label::LabelElement;
use freya_core::elements::rect::RectElement;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};

use crate::environment::EnvironmentScope;
use crate::layout::layers::LayoutElement;
use crate::layout::{GridElement, resolve_spacers};
use crate::styles::LayoutDirection;

pub(crate) fn ambient() -> LayoutDirection {
    CurrentContext::try_with(|_| try_consume_context::<State<LayoutDirection>>())
        .flatten()
        .map_or_else(LayoutDirection::default, |direction| *direction.read())
}

pub(crate) fn mirrored_x(x: f32, width: f32, container_width: f32) -> f32 {
    container_width - x - width
}

pub(crate) fn resolve_mirrored(element: Element) -> Element {
    let element = resolve_spacers(element);
    match ambient() {
        LayoutDirection::LeftToRight => element,
        LayoutDirection::RightToLeft => mirrored(element),
    }
}

pub(crate) fn flipped(direction: LayoutDirection) -> LayoutDirection {
    match direction {
        LayoutDirection::LeftToRight => LayoutDirection::RightToLeft,
        LayoutDirection::RightToLeft => LayoutDirection::LeftToRight,
    }
}

fn swapped(alignment: Alignment) -> Alignment {
    match alignment {
        Alignment::Start => Alignment::End,
        Alignment::End => Alignment::Start,
        other => other,
    }
}

// A row runs the other way and a column puts its children on the other edge; padding and positions are physical and stay. A label that hugs its text keeps its alignment: Skia would draw it shifted out of its box.
fn mirrored(mut element: Element) -> Element {
    let Element::Element {
        element: node,
        elements,
        ..
    } = &mut element
    else {
        return element;
    };
    let any = node.as_ref() as &dyn Any;
    if let Some(stack) = any.downcast_ref::<RectElement>() {
        let mut stack = stack.clone();
        match stack.layout.direction {
            Direction::Horizontal => {
                elements.reverse();
                stack.layout.main_alignment = swapped(stack.layout.main_alignment.clone());
            }
            Direction::Vertical => {
                stack.layout.cross_alignment = swapped(stack.layout.cross_alignment.clone());
            }
        }
        *node = Rc::new(stack);
    } else if let Some(layout) = any.downcast_ref::<LayoutElement>() {
        *node = Rc::new(layout.flipped());
    } else if let Some(grid) = any.downcast_ref::<GridElement>() {
        *node = Rc::new(grid.flipped());
    } else if let Some(label) = any
        .downcast_ref::<LabelElement>()
        .filter(|label| label.layout.width != Size::Inner)
    {
        let mut label = label.clone();
        label.text_style_data.text_align = match label.text_style_data.text_align {
            None | Some(TextAlign::Start) => Some(TextAlign::Right),
            Some(TextAlign::End) => Some(TextAlign::Start),
            other => other,
        };
        *node = Rc::new(label);
    }
    element
}

fn mirrored_tree(element: Element) -> Element {
    let mut element = mirrored(element);
    if let Element::Element { elements, .. } = &mut element {
        *elements = std::mem::take(elements)
            .into_iter()
            .map(mirrored_tree)
            .collect();
    }
    element
}

#[derive(PartialEq)]
pub(crate) struct LayoutDirectionScope {
    direction: LayoutDirection,
    built_under: LayoutDirection,
    child: Element,
}

impl LayoutDirectionScope {
    pub(crate) fn new(direction: LayoutDirection, child: impl IntoElement) -> Self {
        Self {
            direction,
            built_under: ambient(),
            child: child.into_element(),
        }
    }
}

// The child was built under the direction around the call, so it flips once if that differs; components below read the value themselves.
impl Component for LayoutDirectionScope {
    fn render(&self) -> impl IntoElement {
        let direction = self.direction;
        let child = if self.built_under == direction {
            self.child.clone()
        } else {
            mirrored_tree(self.child.clone())
        };
        EnvironmentScope::new(direction, child)
    }
}
