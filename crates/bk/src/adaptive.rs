use ctdesign::BREAKPOINT;
use freya_core::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeClass {
    Compact,
    Medium,
    Wide,
}

impl SizeClass {
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
pub struct SizeClassReader {
    width: State<f32>,
}

pub fn use_size_class() -> SizeClassReader {
    SizeClassReader {
        width: use_state(|| f32::MAX),
    }
}

impl SizeClassReader {
    pub fn current(&self) -> SizeClass {
        SizeClass::for_width(*self.width.read())
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
            SizeClass::for_width(BREAKPOINT.medium - 1.0),
            SizeClass::Compact
        );
        assert_eq!(SizeClass::for_width(BREAKPOINT.medium), SizeClass::Medium);
        assert_eq!(
            SizeClass::for_width(BREAKPOINT.wide - 1.0),
            SizeClass::Medium
        );
        assert_eq!(SizeClass::for_width(BREAKPOINT.wide), SizeClass::Wide);
    }
}
