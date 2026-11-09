use std::cell::RefCell;
use std::collections::HashMap;

use ctdesign::{Rgba, SYMBOL};
use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::Size;

use crate::styles::{ControlState, ImageStyle, Paints, ShapeStyle, StateValue, Theme, resolve};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Symbol {
    MagnifyingGlass,
    XMark,
    SpeakerWave,
    Plus,
    Minus,
    Ellipsis,
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
    House,
    Tv,
    Film,
    Star,
    Clock,
    Bookmark,
    ArrowDown,
    Checkmark,
    Captions,
    Expand,
    GoBackward,
    GoForward,
    StarFill,
    Folder,
    Document,
    ChevronDoubleRight,
    Accessibility,
    Display,
    Moon,
    Bell,
    Bed,
    Person,
    Briefcase,
    Printer,
    Phone,
    SquareOnSquare,
    Wifi,
    Bluetooth,
    Network,
    Lock,
    Bolt,
    Battery,
    EllipsisCircle,
}

impl Symbol {
    fn path(self) -> &'static str {
        match self {
            Self::MagnifyingGlass => {
                "M11 6.8a4.2 4.2 0 1 1-8.4 0a4.2 4.2 0 1 1 8.4 0M10 10l3.2 3.2"
            }
            Self::XMark => "M4 4l8 8M12 4l-8 8",
            Self::Folder => {
                "M2.5 4.5a1 1 0 0 1 1-1h3l1.5 1.5h4.5a1 1 0 0 1 1 1v5.5a1 1 0 0 1-1 1h-9a1 1 0 0 1-1-1z"
            }
            Self::ChevronDoubleRight => "M4 4l4 4-4 4M8.5 4l4 4-4 4",
            Self::Document => "M4 2.5h5l3 3v8H4zM9 2.5v3h3",
            Self::Accessibility => {
                "M8 14a6 6 0 1 0 0-12a6 6 0 0 0 0 12M8 4.8h0M5.2 6.6 8 7.1l2.8-.5M8 7.1v2.3M6.6 11.8 8 9.4l1.4 2.4"
            }
            Self::Display => "M2.5 3.5h11v7.5h-11zM6 13.5h4M8 11v2.5",
            Self::Bed => {
                "M2 12.5V5M2 9.5h12v3M14 9.5V8a1.5 1.5 0 0 0-1.5-1.5H7v3M4.5 8.5a1 1 0 1 0 0-2a1 1 0 0 0 0 2"
            }
            Self::Person => "M8 7.5a2.5 2.5 0 1 0 0-5a2.5 2.5 0 0 0 0 5M3 13.5a5 5 0 0 1 10 0",
            Self::Briefcase => {
                "M2.5 5.5h11v7.5h-11zM6 5.5V4a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v1.5M2.5 9h11"
            }
            Self::Printer => "M4.5 6V2.5h7V6M4.5 11.5h-2v-5.5h11v5.5h-2M4.5 9.5h7v4h-7z",
            Self::Phone => {
                "M4 2.5h2l1 3-1.5 1a7 7 0 0 0 4 4l1-1.5 3 1v2a1 1 0 0 1-1 1A10.5 10.5 0 0 1 3 3.5a1 1 0 0 1 1-1"
            }
            Self::SquareOnSquare => "M5.5 5.5h7v7h-7zM3.5 10.5v-7h7",
            Self::Wifi => {
                "M2 6.5a9 9 0 0 1 12 0M4.3 9a5.5 5.5 0 0 1 7.4 0M6.5 11.4a2.2 2.2 0 0 1 3 0M8 13.5h0"
            }
            Self::Bluetooth => "M4.5 5l7 6-3.5 3V2l3.5 3-7 6",
            Self::Network => {
                "M8 14a6 6 0 1 0 0-12a6 6 0 0 0 0 12M2 8h12M8 2c-3.4 3.4-3.4 8.6 0 12M8 2c3.4 3.4 3.4 8.6 0 12"
            }
            Self::Lock => "M4 7.5h8v6H4zM5.5 7.5v-2a2.5 2.5 0 0 1 5 0v2",
            Self::Bolt => "M9 2 4 9h4l-1 5 5-7H8z",
            Self::Battery => "M2 5h10.5v6H2zM14 7v2",
            Self::EllipsisCircle => "M8 14a6 6 0 1 0 0-12a6 6 0 0 0 0 12M5.4 8h0M8 8h0M10.6 8h0",
            Self::Bell => {
                "M8 2.5A3.5 3.5 0 0 0 4.5 6v2.6L3 11.5h10l-1.5-2.9V6A3.5 3.5 0 0 0 8 2.5M6.5 13.5a1.5 1.5 0 0 0 3 0"
            }
            Self::Moon => "M13 9.6A5.5 5.5 0 1 1 6.4 3a4.4 4.4 0 0 0 6.6 6.6z",
            Self::Plus => "M8 3.5v9M3.5 8h9",
            Self::Minus => "M3.5 8h9",
            Self::Ellipsis => "M3.5 8h0M8 8h0M12.5 8h0",
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
            Self::House => "M2.5 7.5 8 3l5.5 4.5V13h-11z",
            Self::Tv => "M2.5 4h11v7.5h-11zM6 13.5h4",
            Self::Film => "M2.5 3.5h11v9h-11zM2.5 6.5h11M5.5 3.5v3M10.5 3.5v3",
            Self::Star | Self::StarFill => {
                "M8 2.5l1.7 3.4 3.8.6-2.7 2.7.6 3.8L8 11.2 4.6 13l.6-3.8-2.7-2.7 3.8-.6z"
            }
            Self::Clock => "M8 4.5V8l2.5 1.5M8 14a6 6 0 1 0 0-12a6 6 0 0 0 0 12",
            Self::Bookmark => "M4 2.5h8v11L8 10.5l-4 3z",
            Self::ArrowDown => "M8 2v8M4.5 7 8 10.5 11.5 7M3 13.5h10",
            Self::Checkmark => "M3.5 8.5l3 3 6-7",
            Self::Captions => "M2.5 4h11v8h-11zM5 9.5h2.5M9 9.5h2M5 7h6",
            Self::Expand => "M3 6.5V3h3.5M13 6.5V3H9.5M3 9.5V13h3.5M13 9.5V13H9.5",
            Self::GoBackward => "M3.5 8a4.5 4.5 0 1 0 1.5-3.4M3.5 2.5v2.8h2.8",
            Self::GoForward => "M12.5 8a4.5 4.5 0 1 1-1.5-3.4M12.5 2.5v2.8H9.7",
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
                    r#"<svg viewBox="0 0 {grid} {grid}"><path d="{path}" fill="{fill}" stroke="currentColor" stroke-width="{stroke}" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
                    grid = SYMBOL.grid,
                    path = symbol.path(),
                    fill = if symbol == Symbol::StarFill {
                        "currentColor"
                    } else {
                        "none"
                    },
                    stroke = SYMBOL.stroke,
                ))
            })
            .clone()
    })
}

#[derive(Clone, PartialEq)]
pub struct Image {
    symbol: Symbol,
    style: Theme,
}

impl Image {
    pub fn system(symbol: Symbol) -> Self {
        Self {
            symbol,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(ImageStyle) -> ImageStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn size(self, size: f32) -> Self {
        self.style(move |s| ImageStyle { size, ..s })
    }

    pub fn color(self, color: Rgba) -> Self {
        self.style(move |s| ImageStyle {
            foreground: StateValue::new(ShapeStyle::Color(color)),
            ..s
        })
    }
}

impl From<Image> for Element {
    fn from(image: Image) -> Self {
        let style: ImageStyle = resolve(&image.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let color = paints.color(style.foreground.resolve(ControlState::default()));
        symbol(image.symbol, style.size, color).into_element()
    }
}

pub(crate) fn symbol(symbol: Symbol, size: f32, color: Rgba) -> SvgViewer {
    SvgViewer::new(source(symbol))
        .width(Size::px(size))
        .height(Size::px(size))
        .color(crate::theme::color(color))
        .show_loader(false)
}
