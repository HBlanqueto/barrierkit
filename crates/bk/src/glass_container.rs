use ctdesign::Rgba;
use freya_core::prelude::*;
use torin::prelude::{Area, Direction, Position, Size};

use crate::elements::{Overrides, forward_overrides};
use crate::glass::{material, transparency_reduced};
use crate::platform::backdrop::{Declared, GlassLook};
use crate::styles::{
    ControlState, GlassContainerStyle, Paints, ShapeStyle, StateValue, Theme, resolve,
};

/// SwiftUI's `Glass`: the variant of the native glass; the kit's fallback ignores `CLEAR`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    pub(crate) clear: bool,
    pub(crate) tint: Option<Rgba>,
}

impl Glass {
    pub const REGULAR: Self = Self {
        clear: false,
        tint: None,
    };
    pub const CLEAR: Self = Self {
        clear: true,
        tint: None,
    };

    // SwiftUI's prominent glass is tinted with the accent.
    pub(crate) fn prominent(self) -> Self {
        Self {
            tint: Some(crate::style::Palette::current().accent),
            ..self
        }
    }

    pub(crate) fn look(self) -> GlassLook {
        GlassLook {
            clear: self.clear,
            spacing: 0.0,
            tint: self.tint,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

type Measured = Vec<(&'static str, Frame)>;

#[derive(PartialEq)]
pub struct GlassContainer {
    spacing: f32,
    gap: f32,
    items: Vec<(&'static str, f32, bool, Element)>,
    style: Theme,
    overrides: Overrides,
}

impl GlassContainer {
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            gap: spacing,
            items: Vec::new(),
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(GlassContainerStyle) -> GlassContainerStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn item(self, id: &'static str, radius: f32, content: impl IntoElement) -> Self {
        self.shaped_item(id, radius, false, content)
    }

    // A circle is the largest one centered in the view, as in SwiftUI.
    pub(crate) fn shaped_item(
        mut self,
        id: &'static str,
        radius: f32,
        circle: bool,
        content: impl IntoElement,
    ) -> Self {
        self.items
            .push((id, radius, circle, content.into_element()));
        self
    }
}

forward_overrides!(GlassContainer);

impl Component for GlassContainer {
    fn render(&self) -> impl IntoElement {
        let style: GlassContainerStyle = resolve(&self.style);
        let mut origin = use_state(Area::default);
        let measured = use_state(Measured::new);
        let items = self.items.iter().map(|(id, radius, circle, content)| {
            GlassItem {
                id,
                radius: *radius,
                circle: *circle,
                measured,
                content: content.clone(),
            }
            .into_element()
        });
        let shown_origin = *origin.read();
        let paints = Paints::current(ControlState::default());
        let ink =
            |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(ControlState::default()));
        let native = crate::glass::native_glass();
        let fill = crate::theme::color(material(
            ink(&style.tint),
            ink(&style.solid),
            transparency_reduced(),
        ));
        let look = GlassLook {
            clear: style.clear,
            spacing: self.spacing,
            tint: None,
        };
        let backdrop: Vec<Element> = measured
            .read()
            .iter()
            .map(|(_, frame)| {
                let at = (
                    frame.x - shown_origin.origin.x,
                    frame.y - shown_origin.origin.y,
                );
                if native {
                    Declared::glass(frame.radius, (frame.width, frame.height), at, look)
                } else {
                    rect()
                        .position(Position::new_absolute().left(at.0).top(at.1))
                        .width(Size::px(frame.width))
                        .height(Size::px(frame.height))
                        .corner_radius(frame.radius)
                        .background(fill)
                        .into_element()
                }
            })
            .collect();
        let root = rect()
            .direction(Direction::Horizontal)
            .spacing(self.gap)
            .on_sized(move |event: Event<SizedEventData>| origin.set_if_modified(event.area))
            .children(backdrop)
            .children(items);
        self.overrides.apply(root)
    }
}

fn glass_frame(area: Area, radius: f32, circle: bool) -> Frame {
    let side = area.width().min(area.height());
    let (width, height, radius) = if circle {
        (side, side, side / 2.0)
    } else {
        (area.width(), area.height(), radius.min(area.height() / 2.0))
    };
    Frame {
        x: area.origin.x + (area.width() - width) / 2.0,
        y: area.origin.y + (area.height() - height) / 2.0,
        width,
        height,
        radius,
    }
}

#[derive(PartialEq)]
struct GlassItem {
    id: &'static str,
    radius: f32,
    circle: bool,
    measured: State<Measured>,
    content: Element,
}

impl Component for GlassItem {
    fn render(&self) -> impl IntoElement {
        let id = self.id;
        let (radius, circle) = (self.radius, self.circle);
        let mut measured = self.measured;
        use_drop(move || measured.write().retain(|(shown, _)| *shown != id));
        rect()
            .on_sized(move |event: Event<SizedEventData>| {
                let frame = glass_frame(event.area, radius, circle);
                let mut list = measured.peek().clone();
                match list.iter_mut().find(|(shown, _)| *shown == id) {
                    Some((_, current)) if *current == frame => return,
                    Some((_, current)) => *current = frame,
                    None => list.push((id, frame)),
                }
                measured.set(list);
            })
            .child(self.content.clone())
    }
}
