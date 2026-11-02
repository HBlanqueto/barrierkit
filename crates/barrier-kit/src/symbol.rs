//! Named line icons drawn on the Tengen symbol grid.

use std::cell::RefCell;
use std::collections::HashMap;

use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::Size;
use vow_design::{Rgba, SYMBOL};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Symbol {
    MagnifyingGlass,
    XMark,
    SpeakerWave,
    Plus,
    Minus,
    ChevronUpDown,
    ChevronRight,
    ChevronLeft,
    ChevronUp,
    ChevronDown,
    CircleHalf,
    Play,
    Pause,
    Backward,
    Forward,
    ListBullet,
    SquareGrid,
    SlidersVertical,
    SidebarLeft,
}

impl Symbol {
    fn path(self) -> &'static str {
        match self {
            Self::MagnifyingGlass => {
                "M11 6.8a4.2 4.2 0 1 1-8.4 0a4.2 4.2 0 1 1 8.4 0M10 10l3.2 3.2"
            }
            Self::XMark => "M4 4l8 8M12 4l-8 8",
            Self::Plus => "M8 3.5v9M3.5 8h9",
            Self::Minus => "M3.5 8h9",
            Self::ChevronUpDown => "M4 6l4-4 4 4M4 10l4 4 4-4",
            Self::ChevronRight => "M6 3l5 5-5 5",
            Self::ChevronLeft => "M10 3L5 8l5 5",
            Self::ChevronUp => "M3 10l5-5 5 5",
            Self::ChevronDown => "M3 6l5 5 5-5",
            Self::CircleHalf => {
                "M8 2.5a5.5 5.5 0 1 1 0 11a5.5 5.5 0 1 1 0-11M8 2.5v11M6 3v10M4 4.6v6.8"
            }
            Self::Play => "M5 3.5v9l7.5-4.5z",
            Self::Pause => "M5.5 3.5v9M10.5 3.5v9",
            Self::Backward => "M4 3.5v9M12 3.5 6 8l6 4.5z",
            Self::Forward => "M12 3.5v9M4 3.5 10 8l-6 4.5z",
            Self::ListBullet => "M6 4h8M6 8h8M6 12h8M2.5 4h.01M2.5 8h.01M2.5 12h.01",
            Self::SquareGrid => {
                "M2.5 2.5h4.5v4.5H2.5zM9 2.5h4.5v4.5H9zM2.5 9h4.5v4.5H2.5zM9 9h4.5v4.5H9z"
            }
            Self::SlidersVertical => "M4 2v12M8 2v12M12 2v12M2.5 10h3M6.5 5h3M10.5 9h3",
            Self::SidebarLeft => "M2.5 3.5h11v9h-11zM6.5 3.5v9",
            Self::SpeakerWave => {
                "M3 6h2l3-3v10l-3-3H3zM10.5 5.5a3.5 3.5 0 0 1 0 5M12.5 3.5a6.5 6.5 0 0 1 0 9"
            }
        }
    }
}

thread_local! {
    static CACHE: RefCell<HashMap<Symbol, Bytes>> = RefCell::new(HashMap::new());
}

fn source(symbol: Symbol) -> Bytes {
    CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(symbol)
            .or_insert_with(|| {
                Bytes::from(format!(
                    r#"<svg viewBox="0 0 {grid} {grid}"><path d="{path}" fill="none" stroke="currentColor" stroke-width="{stroke}" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
                    grid = SYMBOL.grid,
                    path = symbol.path(),
                    stroke = SYMBOL.stroke,
                ))
            })
            .clone()
    })
}

pub fn symbol(symbol: Symbol, size: f32, color: Rgba) -> SvgViewer {
    SvgViewer::new(source(symbol))
        .width(Size::px(size))
        .height(Size::px(size))
        .color(crate::theme::color(color))
        .show_loader(false)
}
