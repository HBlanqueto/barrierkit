use freya_core::prelude::*;
use torin::prelude::Size;

use crate::elements::forward_key;
use crate::styles::Axis;

#[derive(PartialEq)]
pub struct ViewThatFits {
    axis: Axis,
    candidates: Vec<(f32, Element)>,
    key: DiffKey,
}

impl ViewThatFits {
    pub fn new(axis: Axis) -> Self {
        Self {
            axis,
            candidates: Vec::new(),
            key: DiffKey::None,
        }
    }

    /// Adds a view shown when the space offered along the axis is at least `extent`; the last one also shows when none fits.
    pub fn candidate(mut self, extent: f32, view: impl IntoElement) -> Self {
        self.candidates.push((extent, view.into_element()));
        self
    }
}

forward_key!(ViewThatFits);

// Torin cannot measure a view before it picks it, so each candidate states its extent and the container's measured size chooses (ADR 0110).
impl Component for ViewThatFits {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let mut offered = use_state(|| 0.0f32);
        let space = *offered.read();
        let chosen = self
            .candidates
            .iter()
            .find(|(extent, _)| *extent <= space)
            .or(self.candidates.last())
            .map(|(_, view)| view.clone());
        let horizontal = self.axis == Axis::Horizontal;
        rect()
            .maybe(horizontal, |container| container.width(Size::fill()))
            .maybe(!horizontal, |container| container.height(Size::fill()))
            .on_sized(move |event: Event<SizedEventData>| {
                let area = event.area;
                offered.set_if_modified(if horizontal {
                    area.width()
                } else {
                    area.height()
                });
            })
            .maybe_child(chosen)
    }
}
