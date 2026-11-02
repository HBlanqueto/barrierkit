//! Layouts that change shape with the width of the window.

use freya_core::prelude::*;
use vow_design::BREAKPOINT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Breakpoint {
    Compact,
    Medium,
    Wide,
}

impl Breakpoint {
    pub fn for_width(width: f32) -> Self {
        if width < BREAKPOINT.medium {
            Self::Compact
        } else if width < BREAKPOINT.wide {
            Self::Medium
        } else {
            Self::Wide
        }
    }
}

#[derive(Clone, Copy)]
pub struct Adaptive {
    width: State<f32>,
}

pub fn use_breakpoint() -> Adaptive {
    Adaptive {
        width: use_state(|| f32::MAX),
    }
}

impl Adaptive {
    pub fn current(&self) -> Breakpoint {
        Breakpoint::for_width(*self.width.read())
    }

    pub fn attach(&self, element: Rect) -> Rect {
        let mut width = self.width;
        element
            .on_sized(move |event: Event<SizedEventData>| width.set_if_modified(event.area.width()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_map_to_their_breakpoint_at_the_token_edges() {
        assert_eq!(
            Breakpoint::for_width(BREAKPOINT.medium - 1.0),
            Breakpoint::Compact
        );
        assert_eq!(Breakpoint::for_width(BREAKPOINT.medium), Breakpoint::Medium);
        assert_eq!(
            Breakpoint::for_width(BREAKPOINT.wide - 1.0),
            Breakpoint::Medium
        );
        assert_eq!(Breakpoint::for_width(BREAKPOINT.wide), Breakpoint::Wide);
    }
}
