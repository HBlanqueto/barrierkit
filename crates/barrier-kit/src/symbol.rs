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
