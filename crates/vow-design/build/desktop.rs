use super::*;

pub(super) fn emit_desktop(out: &mut String, components: &Value) {
    emit_numbers(
        out,
        components,
        &[
            ("HAIRLINE", "f32", "components.hairline"),
            ("CURSOR_SHAKE_SCALE", "f32", "components.cursor-shake.scale"),
            (
                "CURSOR_SHAKE_GROW_MS",
                "u64",
                "components.cursor-shake.grow-ms",
            ),
            (
                "CURSOR_SHAKE_SHRINK_MS",
                "u64",
                "components.cursor-shake.shrink-ms",
            ),
            (
                "CURSOR_SHAKE_TICK_MS",
                "u64",
                "components.cursor-shake.tick-ms",
            ),
            (
                "SCREENSHOT_BORDER_WIDTH",
                "i32",
                "components.screenshot.border-width",
            ),
            ("SPACE_GAP", "i32", "components.spaces.gap"),
            ("SPACE_SLIDE_MS", "u64", "components.spaces.slide-ms"),
            ("TILE_MARGIN", "i32", "components.tile.margin"),
            ("TILE_GAP", "i32", "components.tile.gap"),
            (
                "TITLEBAR_PADDING_X",
                "f32",
                "components.window.titlebar.padding-x",
            ),
            (
                "TITLEBAR_TITLE_GAP",
                "f32",
                "components.window.titlebar.title-gap",
            ),
            (
                "MENU_MARK_TEXT_GAP",
                "f32",
                "components.shell.menu.mark.text-gap",
            ),
            (
                "MENU_SUBMENU_INTENT_MS",
                "u64",
                "components.shell.menu.submenu.intent-ms",
            ),
            (
                "MENU_SUBMENU_REVEAL_GAP",
                "f32",
                "components.shell.menu.submenu.reveal-gap",
            ),
            (
                "MENU_SCROLL_TICK_MS",
                "u64",
                "components.shell.menu.scroll-tick-ms",
            ),
            (
                "MENU_SEARCH_CLEAR",
                "f32",
                "components.shell.menu.search.clear",
            ),
            (
                "MENU_SEARCH_CLEAR_GLYPH",
                "f32",
                "components.shell.menu.search.clear-glyph",
            ),
            (
                "MENU_SEARCH_CLEAR_MARGIN",
                "f32",
                "components.shell.menu.search.clear-margin",
            ),
            ("MENU_HELP_ICON", "f32", "components.shell.menu.help-icon"),
        ],
    );
    for (name, path) in [
        ("GLASS_MASK", "components.glass-mask"),
        ("SCREENSHOT_DIM", "components.screenshot.dim"),
        ("SCREENSHOT_IDLE_DIM", "components.screenshot.idle-dim"),
        ("SCREENSHOT_BORDER", "components.screenshot.border"),
        ("SPACE_GAP_COLOR", "components.spaces.gap-color"),
        ("FOCUS_BAR_BACKGROUND", "components.spaces.focus-bar"),
        ("FOCUS_BAR_TEXT", "components.spaces.focus-bar-text"),
        ("DESKTOP_CLEAR", "components.desktop-clear"),
    ] {
        pub_const(out, name);
        out.push_str(&format!(
            ": Rgba = {};\n\n",
            rgba(color(components, path, COMPONENTS))
        ));
    }
    for (name, path) in [
        ("SPACE_SLIDE_EASE", "components.spaces.slide-ease"),
        ("MENU_SEARCH_ICON", "components.shell.menu.search.icon"),
    ] {
        let values = floats(components, path);
        let count = values.split(", ").count();
        pub_const(out, name);
        out.push_str(&format!(": [f32; {count}] = [{values}];\n\n"));
    }
}
