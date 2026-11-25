use objc2_app_kit::NSWindowStyleMask;

use super::super::{DecorationResponse, WindowStyle};

pub(super) fn frame(
    style: WindowStyle,
    resizable: bool,
) -> (NSWindowStyleMask, DecorationResponse) {
    let lights = NSWindowStyleMask::Closable | NSWindowStyleMask::Miniaturizable;
    let (mask, decoration) = match style {
        WindowStyle::TitleBar => (
            NSWindowStyleMask::Titled | lights,
            DecorationResponse::Server,
        ),
        WindowStyle::HiddenTitleBar => (
            NSWindowStyleMask::Titled | NSWindowStyleMask::FullSizeContentView | lights,
            DecorationResponse::Hybrid,
        ),
        WindowStyle::Plain => (
            NSWindowStyleMask::Borderless | lights,
            DecorationResponse::Client,
        ),
        WindowStyle::Borderless => {
            return (NSWindowStyleMask::Borderless, DecorationResponse::Client);
        }
    };
    let resize = if resizable {
        NSWindowStyleMask::Resizable
    } else {
        NSWindowStyleMask::empty()
    };
    (mask | resize, decoration)
}
