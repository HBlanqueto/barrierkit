use bk::compact::IconButton;
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    let design = std::env::args().nth(1).unwrap_or_default();
    let (style, toolbar_style, shows_title) = match design.as_str() {
        "title-bar" => (WindowStyle::TitleBar, None, true),
        "hidden-title-bar" => (WindowStyle::HiddenTitleBar, None, true),
        "unified-no-title" => (
            WindowStyle::HiddenTitleBar,
            Some(WindowToolbarStyle::Unified),
            false,
        ),
        "compact" => (
            WindowStyle::HiddenTitleBar,
            Some(WindowToolbarStyle::UnifiedCompact),
            true,
        ),
        "compact-no-title" => (
            WindowStyle::HiddenTitleBar,
            Some(WindowToolbarStyle::UnifiedCompact),
            false,
        ),
        "expanded" => (
            WindowStyle::HiddenTitleBar,
            Some(WindowToolbarStyle::Expanded),
            true,
        ),
        _ => (
            WindowStyle::HiddenTitleBar,
            Some(WindowToolbarStyle::Unified),
            true,
        ),
    };
    let visibility = match design.as_str() {
        "on-hover" => WindowToolbarFullScreenVisibility::OnHover,
        _ => WindowToolbarFullScreenVisibility::Automatic,
    };
    Application::new("com.example.window_designs")
        .window("Hello", move || {
            rect()
                .expanded()
                .maybe_child(toolbar_style.map(|toolbar_style| {
                    Titlebar::new()
                        .toolbar_style(toolbar_style)
                        .shows_title(shows_title)
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::PrimaryAction,
                            ToolGroup::new().child(IconButton::new(Symbol::Plus, "Add")),
                        ))
                }))
                .child(
                    rect()
                        .expanded()
                        .center()
                        .child(Text::new("Hello, World").font(TITLE)),
                )
        })
        .options(move |window| {
            window
                .size((480, 300))
                .window_style(style)
                .window_toolbar_full_screen_visibility(visibility)
        })
        .run()
}
