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
    };
    let resize = if resizable {
        NSWindowStyleMask::Resizable
    } else {
        NSWindowStyleMask::empty()
    };
    (mask | resize, decoration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_bar_window_is_titled_and_server_decorated() {
        let (mask, decoration) = frame(WindowStyle::TitleBar, true);
        assert!(mask.contains(NSWindowStyleMask::Titled | NSWindowStyleMask::Resizable));
        assert!(!mask.contains(NSWindowStyleMask::FullSizeContentView));
        assert_eq!(decoration, DecorationResponse::Server);
    }

    #[test]
    fn a_hidden_title_bar_window_keeps_the_native_lights_and_the_kit_draws_its_bar() {
        let (mask, decoration) = frame(WindowStyle::HiddenTitleBar, true);
        assert!(mask.contains(NSWindowStyleMask::Titled | NSWindowStyleMask::FullSizeContentView));
        assert_eq!(decoration, DecorationResponse::Hybrid);
    }

    #[test]
    fn a_plain_window_is_borderless_and_draws_its_own_title_bar() {
        let (mask, decoration) = frame(WindowStyle::Plain, true);
        assert!(!mask.contains(NSWindowStyleMask::Titled));
        assert_eq!(decoration, DecorationResponse::Client);
    }

    #[test]
    fn a_fixed_size_window_is_not_resizable() {
        for style in [
            WindowStyle::TitleBar,
            WindowStyle::HiddenTitleBar,
            WindowStyle::Plain,
        ] {
            assert!(!frame(style, false).0.contains(NSWindowStyleMask::Resizable));
        }
    }
}
