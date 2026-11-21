use std::cell::RefCell;
use std::collections::HashMap;

pub use ctappearance::icon_style::IconStyle;
use ctdesign::{Rgba, SYMBOL};
use freya_components::image_viewer::ImageViewer;
use freya_components::svg_viewer::SvgViewer;
use freya_core::elements::image::{ImageHandle, SamplingMode};
use freya_core::prelude::*;
use freya_engine::prelude::{
    AlphaType, BlendMode, ColorType, FontMgr, ImageInfo, Paint, SkColor, SkData, SkImage, SkRect,
    color_filters, raster_from_data, raster_n32_premul, svg,
};
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
    TriangleLeft,
    TriangleRight,
    Dot,
    CircleHalf,
    Play,
    Pause,
    Shuffle,
    ChevronLeftForwardSlashChevronRight,
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
    MenubarDockRectangle,
    Photo,
    Flower,
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
    Vpn,
    Lock,
    Bolt,
    Battery,
    Keyboard,
    QuestionMark,
    EllipsisCircle,
    Square,
    Circle,
    CapsuleOnRectangle,
    CapsuleOnRectangleFill,
    ICloud,
    ArrowDownCircle,
    MusicNote,
    InternalDrive,
    ExternalDrive,
    Trash,
    Tag,
    SquareAndArrowUp,
    SquareAndArrowDown,
    ArrowUpArrowDown,
    Line3HorizontalDecrease,
    InfoCircle,
    PlusCircle,
    MinusCircle,
    XMarkCircle,
    CheckmarkCircle,
    ExclamationMarkCircle,
    QuestionMarkCircle,
    ExclamationMarkTriangle,
    NoSign,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowUpForward,
    Heart,
    Key,
    Mic,
    Camera,
    Envelope,
    Calendar,
    Flag,
    PaperPlane,
    Scissors,
    XMarkCircleFill,
    PlusCircleFill,
    MinusCircleFill,
    CheckmarkCircleFill,
    ExclamationMarkCircleFill,
    InfoCircleFill,
    QuestionMarkCircleFill,
    ExclamationMarkTriangleFill,
    GearShape,
    ArrowClockwise,
    ArrowCounterclockwise,
    TextAlignLeft,
    TextAlignCenter,
    TextAlignRight,
    TextJustify,
    MacWindow,
    SidebarRight,
    DocOnDoc,
    ArrowUturnBackward,
    ArrowUturnForward,
    DesktopComputer,
    PersonCropCircle,
    Eject,
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
                "M8 14a6 6 0 1 0 0-12a6 6 0 0 0 0 12M8.5 5.1a.5 .5 0 1 1-1 0a.5 .5 0 1 1 1 0M5.4 6.4 8 7.1l2.6-.7M8 7.1v2.6M6.4 12 8 9.7l1.6 2.3"
            }
            Self::Display => "M2.5 3.5h11v7.5h-11zM6 13.5h4M8 11v2.5",
            Self::MenubarDockRectangle => {
                "M3.1 4.9h9.8a1.6 1.6 0 0 1 1.6 1.6v5.4a1.6 1.6 0 0 1-1.6 1.6H3.1a1.6 1.6 0 0 1-1.6-1.6V6.5a1.6 1.6 0 0 1 1.6-1.6zM4.6 11.4h6.8"
            }
            Self::Photo => "M2.5 3.5h11v9h-11zM2.5 11l3.5-3.5 3 3 1.5-1.5 3 3M10.5 6.5h0",
            Self::Flower => {
                "M8 1.6a1.7 2.5 0 1 0 0 5a1.7 2.5 0 1 0 0 -5M12.53 3.47a1.7 2.5 45 1 0 -3.54 3.54a1.7 2.5 45 1 0 3.54 -3.54M14.4 8a1.7 2.5 90 1 0 -5 0a1.7 2.5 90 1 0 5 0M12.53 12.53a1.7 2.5 135 1 0 -3.54 -3.54a1.7 2.5 135 1 0 3.54 3.54M8 14.4a1.7 2.5 180 1 0 0 -5a1.7 2.5 180 1 0 0 5M3.47 12.53a1.7 2.5 225 1 0 3.54 -3.54a1.7 2.5 225 1 0 -3.54 3.54M1.6 8a1.7 2.5 270 1 0 5 0a1.7 2.5 270 1 0 -5 0M3.47 3.47a1.7 2.5 315 1 0 3.54 3.54a1.7 2.5 315 1 0 -3.54 -3.54M9.6 8a1.6 1.6 0 1 1-3.2 0a1.6 1.6 0 1 1 3.2 0"
            }
            Self::Bed => {
                "M2 12.5V5M2 9.5h12v3M14 9.5V8a1.5 1.5 0 0 0-1.5-1.5H7v3M4.5 8.5a1 1 0 1 0 0-2a1 1 0 0 0 0 2"
            }
            Self::Person => "M8 7.5a2.5 2.5 0 1 0 0-5a2.5 2.5 0 0 0 0 5M3 13.5a5 5 0 0 1 10 0",
            Self::Briefcase => {
                "M2.5 5.5h11v7.5h-11zM6 5.5V4a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v1.5M2.5 9h11"
            }
            Self::Keyboard => "M2 4.5h12v7H2zM4.5 7h0M7 7h0M9 7h0M11.5 7h0M5.5 9.5h5",
            Self::QuestionMark => {
                "M5.3 6.65a2.45 2.45 0 1 1 3.9 1.7c-.6.4-1.2.8-1.2 1.6V10.65M8 13.45h0"
            }
            Self::Printer => "M4.5 6V2.5h7V6M4.5 11.5h-2v-5.5h11v5.5h-2M4.5 9.5h7v4h-7z",
            Self::Phone => {
                "M4 2.5h2l1 3-1.5 1a7 7 0 0 0 4 4l1-1.5 3 1v2a1 1 0 0 1-1 1A10.5 10.5 0 0 1 3 3.5a1 1 0 0 1 1-1"
            }
            Self::SquareOnSquare => "M5.5 5.5h7v7h-7zM3.5 10.5v-7h7",
            Self::Wifi => {
                "M8 13.03 6.37 11.35A3.2 3.2 0 0 1 9.63 11.35zM3.98 8.37A7 7 0 0 1 12.02 8.37L11.67 8.86A6.4 6.4 0 0 0 4.33 8.86zM1.34 5.34A11 11 0 0 1 14.66 5.34L14.3 5.82A10.4 10.4 0 0 0 1.7 5.82z"
            }
            Self::Bluetooth => "M4.3 4.8l6.2 5.6-3.2 3.3V1.3l3.2 3.3-6.2 5.6",
            Self::Network => {
                "M8 14a6 6 0 1 0 0-12a6 6 0 0 0 0 12M10.54 2.56a2.6 6 25 0 0-5.07 10.88a2.6 6 25 0 0 5.07-10.88M2.06 8.87 11.15 13.11M2.56 5.46 13.44 10.54M4.85 2.89 13.94 7.13"
            }
            Self::Vpn => {
                "M8 10.6a4.6 4.6 0 1 0 0-9.2a4.6 4.6 0 0 0 0 9.2M9.94 1.83a2 4.6 25 0 0-3.89 8.34a2 4.6 25 0 0 3.89-8.34M3.42 6.4 10.64 9.77M5.36 2.23 12.58 5.6M8 10.6v2.1M2.5 13.5h4.6M8.9 13.5h4.6M8.9 13.5a.9 .9 0 1 1-1.8 0a.9 .9 0 1 1 1.8 0"
            }
            Self::Lock => "M4 7.5h8v6H4zM5.5 7.5v-2a2.5 2.5 0 0 1 5 0v2",
            Self::Bolt => "M9.9 1 3.6 8.4h3.8l-1.5 6.6 6.4-7.7H8.5z",
            Self::Battery => {
                "M3 5.5h8.4a1.5 1.5 0 0 1 1.5 1.5v2a1.5 1.5 0 0 1-1.5 1.5H3A1.5 1.5 0 0 1 1.5 9V7A1.5 1.5 0 0 1 3 5.5zM14.9 7.3v1.4"
            }
            Self::EllipsisCircle => "M8 14a6 6 0 1 0 0-12a6 6 0 0 0 0 12M5.4 8h0M8 8h0M10.6 8h0",
            Self::Bell => {
                "M8 2.5A3.5 3.5 0 0 0 4.5 6v2.6L3 11.5h10l-1.5-2.9V6A3.5 3.5 0 0 0 8 2.5M6.5 13.5a1.5 1.5 0 0 0 3 0"
            }
            Self::Moon => "M13 9.6A5.5 5.5 0 1 1 6.4 3a4.4 4.4 0 0 0 6.6 6.6z",
            Self::Plus => "M8 3.5v9M3.5 8h9",
            Self::Minus => "M3.5 8h9",
            Self::Square => "M3.5 3.5h9v9h-9z",
            Self::Circle => "M8 12.5a4.5 4.5 0 1 0 0-9a4.5 4.5 0 0 0 0 9",
            Self::CapsuleOnRectangle => {
                "M3 3h6.5A1.5 1.5 0 0 1 11 4.5v4A1.5 1.5 0 0 1 9.5 10H3a1.5 1.5 0 0 1-1.5-1.5v-4A1.5 1.5 0 0 1 3 3zM7.5 6.5h4a3 3 0 0 1 0 6h-4a3 3 0 0 1 0-6z"
            }
            // The rectangle stops short of the capsule: the gap keeps them apart once filled.
            Self::CapsuleOnRectangleFill => {
                "M3 3h6.5A1.5 1.5 0 0 1 10.9 4.3H7.5A5.2 5.2 0 0 0 2.3 9.5H1.5v-5A1.5 1.5 0 0 1 3 3zM7.5 6.5h4a3 3 0 0 1 0 6h-4a3 3 0 0 1 0-6z"
            }
            Self::Ellipsis => "M3.5 8h0M8 8h0M12.5 8h0",
            Self::ChevronUpDown => "M5.05 7.47l3.2-3.6 3.2 3.6M5.05 10.03l3.2 3.6 3.2-3.6",
            Self::ChevronRight => "M5.5 3l5 5-5 5",
            Self::ChevronLeft => "M10.5 3L5.5 8l5 5",
            Self::ChevronUp => "M3 10.5l5-5 5 5",
            Self::ChevronDown => "M3 5.5l5 5 5-5",
            Self::CircleHalf => {
                "M8 1.5a6.5 6.5 0 1 1 0 13a6.5 6.5 0 1 1 0-13zM8 2.4a5.6 5.6 0 1 0 0 11.2a5.6 5.6 0 1 0 0-11.2zM8 2.4a5.6 5.6 0 0 0 0 11.2zM8 5a1.9 3 0 1 0 0 6a1.9 3 0 1 0 0-6z"
            }
            Self::Play => "M5 3.5v9l7.5-4.5z",
            Self::TriangleLeft => "M11 4.5v7L5 8z",
            Self::TriangleRight => "M5 4.5v7l6-3.5z",
            Self::Dot => "M8 4.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 1 0 0-7z",
            Self::Pause => "M5.5 3.5v9M10.5 3.5v9",
            Self::ChevronLeftForwardSlashChevronRight => {
                "M6 5.5 3.8 8 6 10.5M10 5.5 12.2 8 10 10.5"
            }
            Self::Shuffle => {
                "M2.5 5h2.6c2.8 0 3.9 6 6.8 6h1.6M11.8 9.2l1.7 1.8-1.7 1.8M2.5 11h2.6c1.2 0 2-1.1 2.7-2.3M9.4 6.2C10 5.5 10.8 5 11.9 5h1.6M11.8 3.2 13.5 5l-1.7 1.8"
            }
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
                "M8 2.825L9.274 6.746H13.397L10.062 9.17 11.336 13.091 8 10.668 4.664 13.091 5.938 9.17 2.603 6.746H6.726z"
            }
            Self::Clock => "M8 4.5V8l2.5 1.5M8 14a6 6 0 1 0 0-12a6 6 0 0 0 0 12",
            Self::Bookmark => "M4 2.5h8v11L8 10.5l-4 3z",
            Self::ArrowDown => "M8 2v8M4.5 7 8 10.5 11.5 7M3 13.5h10",
            Self::Checkmark => "M4.1 8.6l2.7 3.4 4.9-7.2",
            Self::Captions => "M2.5 4h11v8h-11zM5 9.5h2.5M9 9.5h2M5 7h6",
            Self::Expand => "M3 6.5V3h3.5M13 6.5V3H9.5M3 9.5V13h3.5M13 9.5V13H9.5",
            Self::GoBackward => "M3.5 8a4.5 4.5 0 1 0 1.5-3.4M3.5 2.5v2.8h2.8",
            Self::GoForward => "M12.5 8a4.5 4.5 0 1 1-1.5-3.4M12.5 2.5v2.8H9.7",
            Self::ICloud => {
                "M3.62 12.97A3.27 3.27 0 0 1 1.82 6.97A5.39 5.39 0 0 1 11.14 5.41A3.85 3.85 0 0 1 12.13 12.97L3.62 12.97"
            }
            Self::ArrowDownCircle => {
                "M13.64 8.13A5.77 5.77 0 0 1 2.1 8.13A5.77 5.77 0 0 1 13.64 8.13M7.87 5.71L7.87 10.67M5.91 8.71L7.87 10.67L9.83 8.71"
            }
            Self::MusicNote => {
                "M8.44 11.95A1.95 1.95 0 0 1 4.54 11.95A1.95 1.95 0 0 1 8.44 11.95M7.94 2.44L7.94 11.95M7.94 2.44C8.74 5.14 10.97 4.94 10.97 7.94C10.07 6.14 8.94 6.34 7.94 5.94Z"
            }
            Self::InternalDrive => {
                "M2.76 4.48A1.6 1.6 0 0 1 4.27 3.42L11.55 3.42A1.6 1.6 0 0 1 13.06 4.48L14.63 8.89A1.6 1.6 0 0 1 14.73 9.43L14.73 11.06A1.91 1.91 0 0 1 12.82 12.97L3.01 12.97A1.91 1.91 0 0 1 1.1 11.06L1.1 9.43A1.6 1.6 0 0 1 1.19 8.89ZM1.1 9.15L14.73 9.15M12.13 11.06h0"
            }
            Self::ExternalDrive => {
                "M2.76 4.48A1.6 1.6 0 0 1 4.27 3.42L11.55 3.42A1.6 1.6 0 0 1 13.06 4.48L14.63 8.89A1.6 1.6 0 0 1 14.73 9.43L14.73 11.06A1.91 1.91 0 0 1 12.82 12.97L3.01 12.97A1.91 1.91 0 0 1 1.1 11.06L1.1 9.43A1.6 1.6 0 0 1 1.19 8.89ZM1.1 9.15L14.73 9.15M12.13 11.06h0"
            }
            Self::Trash => {
                "M2.58 4.56L13.14 4.56M6.07 4.56L6.07 1.77L9.65 1.77L9.65 4.56M3.64 4.56L3.64 12.66A1.8 1.8 0 0 0 5.44 14.46L10.28 14.46A1.8 1.8 0 0 0 12.08 12.66L12.08 4.56M6.38 6.96L6.38 12.06M9.34 6.96L9.34 12.06"
            }
            Self::Tag => "M1.19 1.49L8.73 1.49L14.31 9.01L6.77 14.57L1.19 8.03ZM4.47 4.76h0",
            Self::SquareAndArrowUp => {
                "M6.19 6.01L4.76 6.01A1.8 1.8 0 0 0 2.96 7.81L2.96 13.34A1.8 1.8 0 0 0 4.76 15.14L10.95 15.14A1.8 1.8 0 0 0 12.75 13.34L12.75 7.81A1.8 1.8 0 0 0 10.95 6.01L9.52 6.01M7.86 10.97L7.86 2.1M10.8 5.04L7.86 2.1L4.92 5.04"
            }
            Self::SquareAndArrowDown => {
                "M6.19 6.3L4.76 6.3A1.8 1.8 0 0 0 2.96 8.1L2.96 13.02A1.8 1.8 0 0 0 4.76 14.82L10.95 14.82A1.8 1.8 0 0 0 12.75 13.02L12.75 8.1A1.8 1.8 0 0 0 10.95 6.3L9.52 6.3M7.86 2.65L7.86 10.2M4.92 7.26L7.86 10.2L10.8 7.26"
            }
            Self::ArrowUpArrowDown => {
                "M4.6 13.66L4.6 2.62M8.04 6.06L4.6 2.62L1.17 6.06M11.21 2.62L11.21 13.66M7.77 10.23L11.21 13.66L14.64 10.23"
            }
            Self::Line3HorizontalDecrease => {
                "M2.1 4.98L13.69 4.98M3.72 8.16L12.07 8.16M5.35 11.33L10.45 11.33"
            }
            Self::InfoCircle => {
                "M13.64 8.13A5.77 5.77 0 0 1 2.1 8.13A5.77 5.77 0 0 1 13.64 8.13M8.47 5.71A0.6 0.6 0 0 1 7.27 5.71A0.6 0.6 0 0 1 8.47 5.71M7.87 7.44L7.87 11.3"
            }
            Self::PlusCircle => {
                "M13.64 8.13A5.77 5.77 0 0 1 2.1 8.13A5.77 5.77 0 0 1 13.64 8.13M4.98 8.13L10.75 8.13M7.87 5.25L7.87 11.02"
            }
            Self::MinusCircle => {
                "M13.64 8.13A5.77 5.77 0 0 1 2.1 8.13A5.77 5.77 0 0 1 13.64 8.13M4.98 8.13L10.75 8.13"
            }
            Self::XMarkCircle => {
                "M13.64 8.13A5.77 5.77 0 0 1 2.1 8.13A5.77 5.77 0 0 1 13.64 8.13M5.68 5.94L10.06 10.32M5.68 10.32L10.06 5.94"
            }
            Self::CheckmarkCircle => {
                "M13.64 8.13A5.77 5.77 0 0 1 2.1 8.13A5.77 5.77 0 0 1 13.64 8.13M5.56 8.13L7.29 9.86L10.29 6.4"
            }
            Self::ExclamationMarkCircle => {
                "M13.64 8.13A5.77 5.77 0 0 1 2.1 8.13A5.77 5.77 0 0 1 13.64 8.13M7.87 5.13L7.87 8.82M8.32 10.84A0.45 0.45 0 0 1 7.42 10.84A0.45 0.45 0 0 1 8.32 10.84"
            }
            Self::QuestionMarkCircle => {
                "M13.64 8.13A5.77 5.77 0 0 1 2.1 8.13A5.77 5.77 0 0 1 13.64 8.13M6.25 7C6.25 4.58 9.73 4.58 9.65 7C9.57 8.62 7.87 8.78 7.87 10.07M7.87 11.52h0"
            }
            Self::ExclamationMarkTriangle => {
                "M6.48 3.45A1.5 1.5 0 0 1 9.08 3.45L13.59 11.22A1.5 1.5 0 0 1 12.29 13.47L3.27 13.47A1.5 1.5 0 0 1 1.97 11.22ZM7.78 6.24L7.78 9.31M8.18 11.14A0.4 0.4 0 0 1 7.38 11.14A0.4 0.4 0 0 1 8.18 11.14"
            }
            Self::NoSign => {
                "M13.64 8.13A5.77 5.77 0 0 1 2.1 8.13A5.77 5.77 0 0 1 13.64 8.13M3.79 4.05L11.95 12.21"
            }
            Self::ArrowLeft => "M12.97 8.11L2.7 8.11M6.96 3.85L2.7 8.11L6.96 12.37",
            Self::ArrowRight => "M2.7 8.11L12.97 8.11M8.71 12.37L12.97 8.11L8.71 3.85",
            Self::ArrowUp => "M7.84 13.47L7.84 2.86M12.02 7.04L7.84 2.86L3.65 7.04",
            Self::ArrowUpForward => "M3.7 12.3L12.07 3.96M10.23 3.96L12.07 3.96L12.07 10.46",
            Self::Heart => {
                "M7.94 13.22C5.19 11.24 2.43 8.69 2.43 5.57C2.43 2.74 5.46 1.89 7.94 5C10.42 1.89 13.46 2.74 13.46 5.57C13.46 8.69 10.7 11.24 7.94 13.22Z"
            }
            Self::Key => {
                "M11.58 4.32A3.69 3.69 0 0 1 4.19 4.32A3.69 3.69 0 0 1 11.58 4.32M8.79 4.32A0.9 0.9 0 0 1 6.99 4.32A0.9 0.9 0 0 1 8.79 4.32M7.89 8.01L7.89 15.5M7.89 14.3L11.21 14.3M7.89 11.7L11.21 11.7"
            }
            Self::Mic => {
                "M5.87 3.96A1.96 1.96 0 0 1 7.83 2L7.83 2A1.96 1.96 0 0 1 9.79 3.96L9.79 6.76A1.96 1.96 0 0 1 7.83 8.71L7.83 8.71A1.96 1.96 0 0 1 5.87 6.76ZM3.91 7.8A3.92 3.92 0 0 0 11.75 7.8M7.83 12.37L7.83 14.21M5.87 14.21L9.79 14.21"
            }
            Self::Camera => {
                "M0.74 6.81A1.76 1.76 0 0 1 2.5 5.05L3.77 5.05A0.8 0.8 0 0 0 4.48 4.62L5.3 3.05A0.8 0.8 0 0 1 6.01 2.62L9.55 2.62A0.8 0.8 0 0 1 10.26 3.05L11.08 4.62A0.8 0.8 0 0 0 11.79 5.05L13.06 5.05A1.76 1.76 0 0 1 14.82 6.81L14.82 11.64A2 2 0 0 1 12.82 13.64L2.74 13.64A2 2 0 0 1 0.74 11.64ZM10.64 9.23A2.86 2.86 0 0 1 4.92 9.23A2.86 2.86 0 0 1 10.64 9.23"
            }
            Self::Envelope => {
                "M1.02 5.33A2 2 0 0 1 3.02 3.33L12.79 3.33A2 2 0 0 1 14.79 5.33L14.79 11.01A2 2 0 0 1 12.79 13.01L3.02 13.01A2 2 0 0 1 1.02 11.01ZM1.72 4.13L7.91 8.9L14.09 4.13M1.82 12.21L6.18 7.93M13.99 12.21L9.63 7.93"
            }
            Self::Calendar => {
                "M2.1 6.59A2.2 2.2 0 0 1 4.3 4.39L11.32 4.39A2.2 2.2 0 0 1 13.52 6.59L13.52 11.23A2.2 2.2 0 0 1 11.32 13.43L4.3 13.43A2.2 2.2 0 0 1 2.1 11.23ZM2.1 7.49L13.52 7.49M2.1 6.19L13.52 6.19M4.95 2.89L4.95 5.89M10.66 2.89L10.66 5.89M4.95 9.21h0M4.95 11.03h0M7.81 9.21h0M7.81 11.03h0M10.66 9.21h0M10.66 11.03h0"
            }
            Self::Flag => {
                "M3.12 2.76L3.12 13.66M3.12 2.76C7.16 1.13 9.18 7.12 13.22 5.49L13.22 8.76C9.18 10.39 7.16 4.94 3.12 7.67"
            }
            Self::PaperPlane => {
                "M2.49 9A1 1 0 0 1 2.44 7.17L11.32 3.03A1 1 0 0 1 12.69 4.26L9.61 13.36A1 1 0 0 1 7.78 13.5L6.32 10.68A0.5 0.5 0 0 0 6.07 10.44ZM6.24 10.51L13.44 2.05"
            }
            Self::Scissors => {
                "M3.58 4.11A1.75 1.75 0 0 1 0.08 4.11A1.75 1.75 0 0 1 3.58 4.11M3.33 4.11A1.5 1.5 0 0 1 0.33 4.11A1.5 1.5 0 0 1 3.33 4.11M3.58 11.66A1.75 1.75 0 0 1 0.08 11.66A1.75 1.75 0 0 1 3.58 11.66M3.33 11.66A1.5 1.5 0 0 1 0.33 11.66A1.5 1.5 0 0 1 3.33 11.66M3.42 4.86L15.29 11.28L6.17 7.58ZM3.42 10.91L15.29 4.49L6.17 8.2Z"
            }
            Self::XMarkCircleFill => {
                "M14.14 8.13A6.27 6.27 0 0 1 1.6 8.13A6.27 6.27 0 0 1 14.14 8.13M10.71 6.21L8.79 8.13L10.71 10.05L9.79 10.97L7.87 9.05L5.95 10.97L5.03 10.05L6.95 8.13L5.03 6.21L5.95 5.29L7.87 7.21L9.79 5.29Z"
            }
            Self::PlusCircleFill => {
                "M14.14 8.13A6.27 6.27 0 0 1 1.6 8.13A6.27 6.27 0 0 1 14.14 8.13M8.52 5L8.52 7.48L11 7.48L11 8.78L8.52 8.78L8.52 11.27L7.22 11.27L7.22 8.78L4.73 8.78L4.73 7.48L7.22 7.48L7.22 5Z"
            }
            Self::MinusCircleFill => {
                "M14.14 8.13A6.27 6.27 0 0 1 1.6 8.13A6.27 6.27 0 0 1 14.14 8.13M4.73 7.48L11 7.48L11 8.78L4.73 8.78Z"
            }
            Self::CheckmarkCircleFill => {
                "M14.14 8.13A6.27 6.27 0 0 1 1.6 8.13A6.27 6.27 0 0 1 14.14 8.13M4.9 8.59L7.28 10.96L10.99 6.68L11.14 6.37L11.12 6.04L10.93 5.76L10.63 5.61L10.29 5.64L10.01 5.82L7.21 9.06L5.82 7.67L5.53 7.5L5.19 7.5L4.9 7.67L4.73 7.96L4.73 8.3Z"
            }
            Self::ExclamationMarkCircleFill => {
                "M14.14 8.13A6.27 6.27 0 0 1 1.6 8.13A6.27 6.27 0 0 1 14.14 8.13M7.22 5L7.22 8.88L7.31 9.21L7.54 9.45L7.87 9.53L8.19 9.45L8.43 9.21L8.52 8.88L8.52 5L8.43 4.67L8.19 4.43L7.87 4.35L7.54 4.43L7.31 4.67ZM8.52 11.08A0.65 0.65 0 0 1 7.22 11.08A0.65 0.65 0 0 1 8.52 11.08"
            }
            Self::InfoCircleFill => {
                "M14.14 8.13A6.27 6.27 0 0 1 1.6 8.13A6.27 6.27 0 0 1 14.14 8.13M7.22 7.5L7.22 11.39L7.31 11.72L7.54 11.95L7.87 12.04L8.19 11.95L8.43 11.72L8.52 11.39L8.52 7.5L8.43 7.18L8.19 6.94L7.87 6.85L7.54 6.94L7.31 7.18ZM8.52 5.62A0.65 0.65 0 0 1 7.22 5.62A0.65 0.65 0 0 1 8.52 5.62"
            }
            Self::QuestionMarkCircleFill => {
                "M14.14 8.13A6.27 6.27 0 0 1 1.6 8.13A6.27 6.27 0 0 1 14.14 8.13M6.76 6.97L6.81 6.52L6.91 6.22L7.06 5.99L7.25 5.82L7.47 5.69L7.72 5.61L7.99 5.58L8.25 5.61L8.5 5.68L8.71 5.81L8.89 5.98L9.03 6.21L9.12 6.5L9.15 6.88L9.12 7.12L9.06 7.34L8.97 7.52L8.85 7.7L8.71 7.87L8.54 8.05L8.36 8.23L8.15 8.42L7.94 8.63L7.73 8.86L7.53 9.14L7.37 9.46L7.26 9.83L7.22 10.18L7.28 10.51L7.49 10.77L7.81 10.88L8.14 10.83L8.4 10.61L8.52 10.3L8.54 10.08L8.58 9.95L8.64 9.82L8.74 9.68L8.88 9.52L9.05 9.36L9.25 9.17L9.46 8.97L9.69 8.74L9.91 8.47L10.11 8.16L10.28 7.79L10.39 7.39L10.45 6.92L10.41 6.26L10.22 5.66L9.91 5.16L9.5 4.77L9.03 4.49L8.52 4.33L7.99 4.27L7.46 4.33L6.95 4.49L6.47 4.76L6.06 5.15L5.73 5.65L5.53 6.24L5.47 6.83L5.52 7.17L5.73 7.43L6.05 7.55L6.38 7.5L6.64 7.28ZM8.52 11.82A0.65 0.65 0 0 1 7.22 11.82A0.65 0.65 0 0 1 8.52 11.82"
            }
            Self::ExclamationMarkTriangleFill => {
                "M6.06 3.26A2 2 0 0 1 9.51 3.26L14.03 10.96A2 2 0 0 1 12.31 13.97L3.26 13.97A2 2 0 0 1 1.53 10.96ZM7.13 5.92L7.13 9.33L7.22 9.65L7.46 9.89L7.78 9.98L8.11 9.89L8.34 9.65L8.43 9.33L8.43 5.92L8.34 5.59L8.11 5.35L7.78 5.27L7.46 5.35L7.22 5.59ZM8.43 11.38A0.65 0.65 0 0 1 7.13 11.38A0.65 0.65 0 0 1 8.43 11.38"
            }
            Self::GearShape => {
                "M6.46 3.42A0.6 0.6 0 0 0 6.82 3L6.94 2.46A0.6 0.6 0 0 1 7.52 1.99L8.21 1.99A0.6 0.6 0 0 1 8.8 2.46L8.91 3A0.6 0.6 0 0 0 9.27 3.42L10.21 3.81A0.6 0.6 0 0 0 10.76 3.76L11.22 3.46A0.6 0.6 0 0 1 11.97 3.55L12.45 4.03A0.6 0.6 0 0 1 12.54 4.78L12.24 5.24A0.6 0.6 0 0 0 12.19 5.79L12.58 6.73A0.6 0.6 0 0 0 13 7.09L13.54 7.2A0.6 0.6 0 0 1 14.01 7.79L14.01 8.48A0.6 0.6 0 0 1 13.54 9.06L13 9.18A0.6 0.6 0 0 0 12.58 9.54L12.19 10.47A0.6 0.6 0 0 0 12.24 11.03L12.54 11.49A0.6 0.6 0 0 1 12.45 12.24L11.97 12.72A0.6 0.6 0 0 1 11.22 12.8L10.76 12.51A0.6 0.6 0 0 0 10.21 12.46L9.27 12.85A0.6 0.6 0 0 0 8.91 13.27L8.8 13.81A0.6 0.6 0 0 1 8.21 14.28L7.52 14.28A0.6 0.6 0 0 1 6.94 13.81L6.82 13.27A0.6 0.6 0 0 0 6.46 12.85L5.53 12.46A0.6 0.6 0 0 0 4.97 12.51L4.51 12.8A0.6 0.6 0 0 1 3.76 12.72L3.28 12.24A0.6 0.6 0 0 1 3.2 11.49L3.49 11.03A0.6 0.6 0 0 0 3.54 10.47L3.15 9.54A0.6 0.6 0 0 0 2.73 9.18L2.19 9.06A0.6 0.6 0 0 1 1.72 8.48L1.72 7.79A0.6 0.6 0 0 1 2.19 7.2L2.73 7.09A0.6 0.6 0 0 0 3.15 6.73L3.54 5.79A0.6 0.6 0 0 0 3.49 5.24L3.2 4.78A0.6 0.6 0 0 1 3.28 4.03L3.76 3.55A0.6 0.6 0 0 1 4.51 3.46L4.97 3.76A0.6 0.6 0 0 0 5.53 3.81ZM9.85 8.13A1.98 1.98 0 0 1 5.88 8.13A1.98 1.98 0 0 1 9.85 8.13"
            }
            Self::ArrowClockwise => {
                "M13.01 10.84L12.76 11.46L12.44 12.05L12.05 12.58L11.58 13.07L11.07 13.48L10.5 13.83L9.89 14.11L9.25 14.3L8.59 14.41L7.93 14.44L7.26 14.38L6.61 14.23L5.99 14L5.39 13.7L4.85 13.32L4.35 12.88L3.91 12.37L3.55 11.81L3.26 11.21L3.04 10.58L2.91 9.93L2.86 9.26L2.9 8.6L3.03 7.94L3.23 7.31L3.52 6.71L3.88 6.15L4.31 5.64L4.8 5.19L5.35 4.8L5.94 4.49L6.56 4.26L7.21 4.11L7.87 4.04L8.54 4.06L9.2 4.16L9.84 4.35L10.45 4.62L11.02 4.96L11.54 5.37M7.89 5.8L11.54 5.37L11.12 1.72"
            }
            Self::ArrowCounterclockwise => {
                "M2.91 10.81L3.16 11.42L3.47 11.99L3.86 12.52L4.31 12.99L4.82 13.4L5.38 13.74L5.97 14.01L6.6 14.2L7.24 14.31L7.9 14.34L8.55 14.28L9.19 14.14L9.8 13.91L10.38 13.61L10.92 13.24L11.41 12.81L11.84 12.31L12.19 11.76L12.48 11.18L12.69 10.56L12.82 9.92L12.86 9.26L12.83 8.61L12.7 7.97L12.5 7.35L12.22 6.76L11.87 6.21L11.45 5.71L10.96 5.27L10.43 4.89L9.85 4.58L9.24 4.36L8.6 4.21L7.95 4.14L7.3 4.16L6.65 4.26L6.02 4.44L5.43 4.71L4.87 5.04L4.35 5.45M4.77 1.87L4.35 5.45L7.93 5.87"
            }
            Self::TextAlignLeft => {
                "M2.1 3.3L13.48 3.3M2.1 6.55L9.16 6.55M2.1 9.79L13.48 9.79M2.1 13.04L9.16 13.04"
            }
            Self::TextAlignCenter => {
                "M2.1 3.3L13.48 3.3M4.26 6.55L11.32 6.55M2.1 9.79L13.48 9.79M4.26 13.04L11.32 13.04"
            }
            Self::TextAlignRight => {
                "M2.1 3.3L13.48 3.3M6.42 6.55L13.48 6.55M2.1 9.79L13.48 9.79M6.42 13.04L13.48 13.04"
            }
            Self::TextJustify => {
                "M2.1 3.3L13.48 3.3M2.1 6.55L13.48 6.55M2.1 9.79L13.48 9.79M2.1 13.04L13.48 13.04"
            }
            Self::MacWindow => {
                "M1.15 5.07A2.2 2.2 0 0 1 3.35 2.87L12.54 2.87A2.2 2.2 0 0 1 14.74 5.07L14.74 11.21A2.2 2.2 0 0 1 12.54 13.41L3.35 13.41A2.2 2.2 0 0 1 1.15 11.21ZM1.15 6.27L14.74 6.27"
            }
            Self::SidebarRight => {
                "M1.15 5.27A2.4 2.4 0 0 1 3.55 2.87L12.34 2.87A2.4 2.4 0 0 1 14.74 5.27L14.74 11.01A2.4 2.4 0 0 1 12.34 13.41L3.55 13.41A2.4 2.4 0 0 1 1.15 11.01ZM9.98 2.87L9.98 13.41M11.34 5.77L13.38 5.77M11.34 8.14L13.38 8.14M11.34 10.51L13.38 10.51"
            }
            Self::DocOnDoc => {
                "M2.1 7.71A1.8 1.8 0 0 1 3.9 5.91L5.77 5.91A0.6 0.6 0 0 1 6.19 6.09L10.04 9.94A0.6 0.6 0 0 1 10.22 10.36L10.22 13.74A1.8 1.8 0 0 1 8.42 15.54L3.9 15.54A1.8 1.8 0 0 1 2.1 13.74ZM6.02 5.91L6.02 10.11L10.22 10.11M5.58 4.41L5.58 2.53A1.8 1.8 0 0 1 7.38 0.73L11.9 0.73A1.8 1.8 0 0 1 13.7 2.53L13.7 10.41A1.8 1.8 0 0 1 11.9 12.21L11.72 12.21"
            }
            Self::ArrowUturnBackward => {
                "M7.63 13.47L10.33 13.47A2.85 2.85 0 0 0 13.18 10.62L13.18 9.12A2.85 2.85 0 0 0 10.33 6.27L2.81 6.27M6.13 2.75L2.61 6.27L6.13 9.79"
            }
            Self::ArrowUturnForward => {
                "M8.2 13.48L5.52 13.48A2.87 2.87 0 0 1 2.64 10.61L2.64 9.09A2.87 2.87 0 0 1 5.52 6.21L13.02 6.21M9.67 9.76L13.22 6.21L9.67 2.67"
            }
            Self::DesktopComputer => {
                "M0.94 4.54A2 2 0 0 1 2.94 2.54L12.67 2.54A2 2 0 0 1 14.67 4.54L14.67 8.95A2 2 0 0 1 12.67 10.95L2.94 10.95A2 2 0 0 1 0.94 8.95ZM0.94 8.93L14.67 8.93M0.94 9.85L14.67 9.85M5.47 13.9L10.14 13.9M7.81 10.95L7.81 13.9"
            }
            Self::PersonCropCircle => {
                "M14.14 8.13A6.27 6.27 0 0 1 1.6 8.13A6.27 6.27 0 0 1 14.14 8.13M13.14 8.13A5.27 5.27 0 0 1 2.6 8.13A5.27 5.27 0 0 1 13.14 8.13M9.66 6.81A1.79 1.79 0 0 1 6.08 6.81A1.79 1.79 0 0 1 9.66 6.81M4.39 12.08C4.97 8.66 10.77 8.66 11.35 12.08A5.27 5.27 0 0 1 4.39 12.08"
            }
            Self::Eject => {
                "M6.83 3.35A1.2 1.2 0 0 1 8.85 3.35L12.02 8.31A1.2 1.2 0 0 1 11.01 10.16L4.67 10.16A1.2 1.2 0 0 1 3.66 8.31ZM2.48 12.36A1 1 0 0 1 3.48 11.36L12.2 11.36A1 1 0 0 1 13.2 12.36L13.2 12.46A1 1 0 0 1 12.2 13.46L3.48 13.46A1 1 0 0 1 2.48 12.46Z"
            }
            Self::SpeakerWave => {
                "M3 6h2l3-3v10l-3-3H3zM10.5 5.5a3.5 3.5 0 0 1 0 5M12.5 3.5a6.5 6.5 0 0 1 0 9"
            }
        }
    }
}

thread_local! {
    static CACHE: RefCell<HashMap<(Symbol, u32), Bytes>> = RefCell::new(HashMap::new());
}

fn source(symbol: Symbol, stroke: f32) -> Bytes {
    let solid = matches!(
        symbol,
        Symbol::Bolt
            | Symbol::CircleHalf
            | Symbol::StarFill
            | Symbol::TriangleLeft
            | Symbol::TriangleRight
            | Symbol::Dot
            | Symbol::XMarkCircleFill
            | Symbol::PlusCircleFill
            | Symbol::MinusCircleFill
            | Symbol::CheckmarkCircleFill
            | Symbol::ExclamationMarkCircleFill
            | Symbol::InfoCircleFill
            | Symbol::QuestionMarkCircleFill
            | Symbol::ExclamationMarkTriangleFill
            | Symbol::PersonCropCircle
    );
    let filled = solid
        || matches!(
            symbol,
            Symbol::StarFill
                | Symbol::CapsuleOnRectangleFill
                | Symbol::Wifi
                | Symbol::Battery
                | Symbol::MusicNote
                | Symbol::Scissors
        );
    CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry((symbol, stroke.to_bits()))
            .or_insert_with(|| {
                Bytes::from(format!(
                    r#"<svg viewBox="0 0 {grid} {grid}"><path d="{path}" fill="{fill}" fill-rule="evenodd" stroke="{paint}" stroke-width="{stroke}" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
                    grid = SYMBOL.grid,
                    path = symbol.path(),
                    fill = if filled { "currentColor" } else { "none" },
                    paint = if solid { "none" } else { "currentColor" },
                ))
            })
            .clone()
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TemplateRenderingMode {
    #[default]
    Original,
    Template,
}

#[derive(Clone, PartialEq)]
enum Source {
    System(Symbol),
    Data(Bytes),
    Rgba {
        width: u32,
        height: u32,
        pixels: Bytes,
    },
}

#[derive(Clone, PartialEq)]
pub struct Image {
    source: Source,
    rendering_mode: TemplateRenderingMode,
    icon_style: IconStyle,
    plated: bool,
    glass_plate: bool,
    raster_size: Option<f32>,
    frame: Option<(f32, f32)>,
    style: Theme,
}

impl Image {
    pub fn system(symbol: Symbol) -> Self {
        Self::from_source(Source::System(symbol))
    }

    pub fn data(bytes: impl Into<Bytes>) -> Self {
        Self::from_source(Source::Data(bytes.into()))
    }

    pub fn rgba(width: u32, height: u32, pixels: impl Into<Bytes>) -> Self {
        Self::from_source(Source::Rgba {
            width,
            height,
            pixels: pixels.into(),
        })
    }

    fn from_source(source: Source) -> Self {
        Self {
            source,
            rendering_mode: TemplateRenderingMode::Original,
            icon_style: IconStyle::Default,
            plated: false,
            glass_plate: false,
            raster_size: None,
            frame: None,
            style: Theme::tengen(),
        }
    }

    pub fn rendering_mode(mut self, rendering_mode: TemplateRenderingMode) -> Self {
        self.rendering_mode = rendering_mode;
        self
    }

    pub fn icon_style(mut self, icon_style: IconStyle) -> Self {
        self.icon_style = icon_style;
        self
    }

    /// Draws the image as an app icon: on a plate, in the user's icon style.
    pub fn app_icon(mut self) -> Self {
        self.plated = true;
        self
    }

    /// In the clear icon style the compositor draws the icon's plate as glass, so only the logo is drawn.
    pub fn glass_plate(mut self) -> Self {
        self.glass_plate = true;
        self
    }

    /// Rasterizes the image once at this size and lets the GPU scale it: for an icon whose `size` changes every frame.
    pub fn raster_size(mut self, size: f32) -> Self {
        self.raster_size = Some(size);
        self
    }

    pub fn style(mut self, restyle: impl Fn(ImageStyle) -> ImageStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn size(self, size: f32) -> Self {
        self.style(move |s| ImageStyle { size, ..s })
    }

    pub fn frame(mut self, width: f32, height: f32) -> Self {
        self.frame = Some((width, height));
        self
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
        let side = Size::px(style.size);
        let (width, height) = image
            .frame
            .map_or((side.clone(), side.clone()), |(width, height)| {
                (Size::px(width), Size::px(height))
            });
        let icon_style = image.icon_style;
        let raster_size = image.raster_size;
        let plated = image.plated;
        let plain = matches!(icon_style, IconStyle::Default | IconStyle::Dark)
            && !plated
            && raster_size.is_none();
        let on_glass = |style: IconStyle| image.glass_plate && style == IconStyle::Clear;
        match (image.source, image.rendering_mode) {
            (Source::System(glyph), _) => {
                viewer(source(glyph, style.stroke), style.size, color).into_element()
            }
            (Source::Data(bytes), TemplateRenderingMode::Original) if plain && is_svg(&bytes) => {
                SvgViewer::new(bytes)
                    .width(width)
                    .height(height)
                    .show_loader(false)
                    .into_element()
            }
            (Source::Data(bytes), TemplateRenderingMode::Original) if plain => {
                ImageViewer::new(bytes)
                    .width(width)
                    .height(height)
                    .into_element()
            }
            (source, mode) => {
                let resolved = crate::theme::resolved();
                let treatment = match mode {
                    _ if plated => Treatment::Plate(
                        resolved.icon_style,
                        resolved.icon_dark,
                        resolved.icon_tint,
                        on_glass(resolved.icon_style),
                    ),
                    TemplateRenderingMode::Template => Treatment::Tint(color),
                    TemplateRenderingMode::Original => {
                        Treatment::Style(icon_style, resolved.icon_tint, on_glass(icon_style))
                    }
                };
                match cached_image(&source, raster_size.unwrap_or(style.size), treatment) {
                    Some(handle) => freya_core::elements::image::image(handle)
                        .width(side.clone())
                        .height(side)
                        .into_element(),
                    None => rect().width(side.clone()).height(side).into_element(),
                }
            }
        }
    }
}

pub(crate) fn symbol(symbol: Symbol, size: f32, color: Rgba) -> SvgViewer {
    viewer(source(symbol, SYMBOL.stroke), size, color)
}

pub(crate) fn stroked_symbol(symbol: Symbol, size: f32, stroke: f32, color: Rgba) -> SvgViewer {
    viewer(source(symbol, stroke * SYMBOL.grid / size), size, color)
}

pub(crate) fn tile_symbol(symbol: Symbol, size: f32, color: Rgba) -> SvgViewer {
    viewer(source(symbol, SYMBOL.fine_stroke), size, color)
}

fn viewer(source: Bytes, size: f32, color: Rgba) -> SvgViewer {
    SvgViewer::new(source)
        .width(Size::px(size))
        .height(Size::px(size))
        .color(crate::theme::color(color))
        .show_loader(false)
}

const TEMPLATE_CACHE_CAPACITY: usize = 256;

type TemplateKey = (Bytes, u32, u32, (u8, [u32; 4]));

#[derive(Clone, Copy)]
enum Treatment {
    Tint(Rgba),
    Style(IconStyle, Rgba, bool),
    Plate(IconStyle, bool, Rgba, bool),
}

impl Treatment {
    fn key(self) -> (u8, [u32; 4]) {
        let bits = |color: Rgba| [color.r, color.g, color.b, color.a].map(f32::to_bits);
        match self {
            Self::Tint(color) => (0, bits(color)),
            Self::Style(style, tint, glass) => (1 + style as u8 + 16 * u8::from(glass), bits(tint)),
            Self::Plate(style, dark, tint, glass) => (
                5 + 2 * style as u8 + u8::from(dark) + 16 * u8::from(glass),
                bits(tint),
            ),
        }
    }
}

thread_local! {
    static TEMPLATE_CACHE: RefCell<HashMap<TemplateKey, Option<ImageHandle>>> =
        RefCell::new(HashMap::new());
}

fn is_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(512)];
    head.windows(4).any(|window| window == b"<svg")
}

fn cached_image(source: &Source, size: f32, treatment: Treatment) -> Option<ImageHandle> {
    let scale = *Platform::get().current_window().scale_factor.read() as f32;
    let exact = (size * scale).round().max(1.0) as u32;
    // A magnified Dock draws every size between two: steps of 1/32 to 1/16 of the size keep the cache from refilling.
    let side = exact.next_multiple_of(1 << exact.ilog2().saturating_sub(4));
    let (bytes, width) = match source {
        Source::Data(bytes) => (bytes.clone(), 0),
        Source::Rgba { width, pixels, .. } => (pixels.clone(), *width),
        Source::System(_) => return None,
    };
    let key = (bytes, width, side, treatment.key());
    TEMPLATE_CACHE.with(|cache| {
        if let Some(cached) = cache.borrow().get(&key) {
            return cached.clone();
        }
        let image = raster(source, side).and_then(|raster| match treatment {
            Treatment::Tint(color) => tint(&raster, side, color),
            Treatment::Style(IconStyle::Default | IconStyle::Dark, ..) => {
                Some(ImageHandle::new(raster, Bytes::new()))
            }
            Treatment::Style(style, tint, glass) => restyle(&raster, side, style, tint, glass),
            Treatment::Plate(style, dark, tint, glass) => {
                crate::app_icon::plated(&raster, side, style, dark, tint, glass)
            }
        });
        let mut cache = cache.borrow_mut();
        if cache.len() >= TEMPLATE_CACHE_CAPACITY {
            cache.clear();
        }
        cache.insert(key, image.clone());
        image
    })
}

fn raster(source: &Source, svg_side: u32) -> Option<SkImage> {
    match source {
        Source::Data(bytes) if is_svg(bytes) => rasterize_svg(bytes, svg_side.max(1)),
        Source::Data(bytes) => SkImage::from_encoded(SkData::new_copy(bytes)),
        Source::Rgba {
            width,
            height,
            pixels,
        } => {
            let info = ImageInfo::new(
                (*width as i32, *height as i32),
                ColorType::RGBA8888,
                AlphaType::Unpremul,
                None,
            );
            raster_from_data(&info, SkData::new_copy(pixels), *width as usize * 4)
        }
        Source::System(_) => None,
    }
}

fn rasterize_svg(bytes: &[u8], side: u32) -> Option<SkImage> {
    let mut dom = svg::Dom::from_bytes(bytes, FontMgr::empty()).ok()?;
    let side = side as i32;
    dom.set_container_size((side, side));
    let mut root = dom.root();
    root.set_width(svg::Length::new(side as f32, svg::LengthUnit::PX));
    root.set_height(svg::Length::new(side as f32, svg::LengthUnit::PX));
    let mut surface = raster_n32_premul((side, side))?;
    dom.render(surface.canvas());
    Some(surface.image_snapshot())
}

fn tint(raster: &SkImage, side: u32, color: Rgba) -> Option<ImageHandle> {
    let mut paint = Paint::default();
    paint.set_color_filter(color_filters::blend(
        SkColor::from(crate::theme::color(color)),
        BlendMode::SrcIn,
    ));
    filtered(raster, side, paint, false)
}

fn restyle(
    raster: &SkImage,
    side: u32,
    style: IconStyle,
    tint: Rgba,
    glass: bool,
) -> Option<ImageHandle> {
    let matrix = if glass {
        style.matrix_on_glass(tint)
    } else {
        style.matrix(tint)
    }?;
    let mut paint = Paint::default();
    paint.set_color_filter(color_filters::matrix_row_major(
        &matrix,
        color_filters::Clamp::Yes,
    ));
    filtered(raster, side, paint, style.luminance_to_alpha())
}

fn filtered(
    raster: &SkImage,
    side: u32,
    mut paint: Paint,
    cut_to_shape: bool,
) -> Option<ImageHandle> {
    let side = side as i32;
    let mut surface = raster_n32_premul((side, side))?;
    paint.set_anti_alias(true);
    let rect = SkRect::from_wh(side as f32, side as f32);
    surface.canvas().draw_image_rect_with_sampling_options(
        raster,
        None,
        rect,
        SamplingMode::Mitchell.sampling_options(),
        &paint,
    );
    if cut_to_shape {
        let mut cut = Paint::default();
        cut.set_blend_mode(BlendMode::DstIn);
        surface.canvas().draw_image_rect_with_sampling_options(
            raster,
            None,
            rect,
            SamplingMode::Mitchell.sampling_options(),
            &cut,
        );
    }
    Some(ImageHandle::new(surface.image_snapshot(), Bytes::new()))
}
