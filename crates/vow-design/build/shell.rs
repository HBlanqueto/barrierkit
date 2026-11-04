use super::*;

pub(super) fn shell(tokens: &Value, components: &Value, theme: &str, opaque: bool) -> String {
    let bg_window = theme_color(tokens, theme, "bg-window");
    let color_at = |path: &str| {
        let color = color_ref(
            tokens,
            theme,
            string(components, path, COMPONENTS),
            path,
            COMPONENTS,
        );
        if opaque {
            opaque_over(color, bg_window)
        } else {
            color
        }
    };
    format!(
        "ShellPalette {{\n\
         \x20   bar: {},\n\
         \x20   text: {},\n\
         \x20   tile: {},\n\
         \x20   tile_active: {},\n\
         \x20   tile_text_active: {},\n\
         }}",
        rgba(color_at("components.shell.menubar.bg")),
        rgba(color_at("components.shell.menubar.text")),
        rgba(color_at("components.shell.dock.tile")),
        rgba(color_at("components.shell.dock.tile-active")),
        rgba(color_at("components.shell.dock.tile-text-active"))
    )
}

pub(super) fn emit_shell(out: &mut String, tokens: &Value, components: &Value) {
    for (name, theme) in [
        ("SHELL_LIGHT", "light"),
        ("SHELL_DARK", "dark"),
        ("SHELL_DARK_TINT", "dark-tint"),
    ] {
        const_doc(out, name, &format!("The {theme} shell palette."));
        out.push_str(&format!(
            ": ShellPalette = {};\n\n",
            shell(tokens, components, theme, false)
        ));
        const_doc(
            out,
            &format!("{name}_OPAQUE"),
            &format!("The {theme} shell palette with opaque surfaces."),
        );
        out.push_str(&format!(
            ": ShellPalette = {};\n\n",
            shell(tokens, components, theme, true)
        ));
    }

    for (name, theme) in [
        ("SIDEBAR_LIGHT", "light"),
        ("SIDEBAR_DARK", "dark"),
        ("SIDEBAR_DARK_TINT", "dark-tint"),
    ] {
        let bg_window = theme_color(tokens, theme, "bg-window");
        let sunken = theme_color(tokens, theme, "bg-sunken");
        const_doc(
            out,
            name,
            &format!("The {theme} flat sidebar background (bg-sunken)."),
        );
        out.push_str(&format!(": Rgba = {};\n\n", rgba(sunken)));
        const_doc(
            out,
            &format!("{name}_OPAQUE"),
            &format!("The {theme} flat sidebar background over bg-window."),
        );
        out.push_str(&format!(
            ": Rgba = {};\n\n",
            rgba(opaque_over(sunken, bg_window))
        ));
    }

    let lock = format!(
        "LockPalette {{\n\
         \x20   bg_surface: {},\n\
         \x20   ink: {},\n\
         \x20   ink_muted: {},\n\
         \x20   danger: {},\n\
         \x20   accent: {},\n\
         }}",
        rgba(theme_color(tokens, "dark-tint", "bg-surface")),
        rgba(theme_color(tokens, "dark-tint", "ink")),
        rgba(theme_color(tokens, "dark-tint", "ink-muted")),
        rgba(theme_color(tokens, "dark-tint", "danger")),
        rgba(theme_color(tokens, "dark-tint", "accent"))
    );
    const_doc(out, "LOCK", "The lock screen palette.");
    out.push_str(&format!(": LockPalette = {lock};\n\n"));
}

pub(super) fn emit_metrics(out: &mut String, tokens: &Value, components: &Value) {
    let duration_base = integer(tokens, "duration.base", TOKENS);
    const_doc(
        out,
        "DURATION_BASE",
        "The base animation duration, in milliseconds.",
    );
    out.push_str(&format!(": u64 = {duration_base};\n\n"));

    let duration_fast = integer(tokens, "duration.fast", TOKENS);
    const_doc(
        out,
        "DURATION_FAST_MS",
        "The fast animation duration, in milliseconds (Tengen `duration.fast`).",
    );
    out.push_str(&format!(": u64 = {duration_fast};\n\n"));

    let duration_slow = integer(tokens, "duration.slow", TOKENS);
    const_doc(
        out,
        "DURATION_SLOW_MS",
        "The slow animation duration, in milliseconds (Tengen `duration.slow`).",
    );
    out.push_str(&format!(": u64 = {duration_slow};\n\n"));

    let theme_fade_wait = integer(tokens, "duration.wait-max", TOKENS);
    const_doc(
        out,
        "THEME_FADE_WAIT_MS",
        "Maximum time to wait for surface commits before starting an appearance fade, in milliseconds.",
    );
    out.push_str(&format!(": u64 = {theme_fade_wait};\n\n"));

    let metrics = [
        ("MENUBAR_HEIGHT", "i32", "components.shell.menubar.height"),
        (
            "MENUBAR_FONT_SIZE",
            "f32",
            "components.shell.menubar.font-size",
        ),
        (
            "MENUBAR_APP_NAME_WEIGHT",
            "i32",
            "components.shell.menubar.app-name.weight",
        ),
        ("MENUBAR_MARGIN", "i32", "components.shell.menubar.margin"),
        (
            "MENUBAR_MARGIN_RIGHT",
            "i32",
            "components.shell.menubar.margin-right",
        ),
        (
            "MENUBAR_TITLE_PADDING_X",
            "f32",
            "components.shell.menubar.title-padding-x",
        ),
        (
            "MENUBAR_TEXT_LIFT",
            "f32",
            "components.shell.menubar.text-lift",
        ),
        (
            "MENUBAR_TITLE_MAX_WIDTH",
            "f32",
            "components.shell.menubar.title-max-width",
        ),
        ("DOCK_TILESIZE", "i32", "components.shell.dock.tilesize"),
        ("DOCK_GAP", "i32", "components.shell.dock.gap"),
        ("DOCK_PADDING", "i32", "components.shell.dock.padding"),
        ("DOCK_FONT_SIZE", "f32", "components.shell.dock.font-size"),
        ("LOCK_FONT_SIZE", "f32", "components.shell.lock.font-size"),
        (
            "LOCK_TITLE_SCALE",
            "f32",
            "components.shell.lock.title-scale",
        ),
        (
            "LOCK_FIELD_WIDTH",
            "i32",
            "components.shell.lock.field-width",
        ),
        (
            "LOCK_FIELD_HEIGHT",
            "i32",
            "components.shell.lock.field-height",
        ),
        (
            "LOCK_LABEL_OFFSET",
            "i32",
            "components.shell.lock.label-offset",
        ),
        (
            "LOCK_LABEL_FONT_SIZE",
            "f32",
            "components.shell.lock.label-font-size",
        ),
        (
            "LOCK_MASK_FONT_SIZE",
            "f32",
            "components.shell.lock.mask-font-size",
        ),
    ];
    for (name, ty, path) in metrics {
        let value = number(components, path, COMPONENTS);
        const_doc(out, name, &format!("Tengen shell metric `{path}`."));
        if ty == "f32" {
            out.push_str(&format!(": {ty} = {};\n\n", float(value)));
        } else {
            out.push_str(&format!(": {ty} = {value};\n\n"));
        }
    }

    for (name, path) in [
        ("LAUNCHER_WIDTH", "components.shell.launcher.width"),
        ("LAUNCHER_HEIGHT", "components.shell.launcher.height"),
        ("LAUNCHER_PADDING", "components.shell.launcher.padding"),
        (
            "LAUNCHER_SEARCH_HEIGHT",
            "components.shell.launcher.search-height",
        ),
        (
            "LAUNCHER_RESULT_HEIGHT",
            "components.shell.launcher.result-height",
        ),
        (
            "LAUNCHER_RESULT_GAP",
            "components.shell.launcher.result-gap",
        ),
        ("LAUNCHER_ICON_SIZE", "components.shell.launcher.icon-size"),
        (
            "LAUNCHER_MAX_RESULTS",
            "components.shell.launcher.max-results",
        ),
    ] {
        const_doc(out, name, &format!("Tengen launcher metric `{path}`."));
        out.push_str(&format!(
            ": i32 = {};\n\n",
            integer(components, path, COMPONENTS)
        ));
    }
    for (name, ty, path) in [
        (
            "LAUNCHER_CORNER_RADIUS",
            "f32",
            "components.shell.launcher.corner-radius",
        ),
        (
            "LAUNCHER_FONT_SIZE",
            "f32",
            "components.shell.launcher.font-size",
        ),
        (
            "LAUNCHER_SUBTITLE_FONT_SIZE",
            "f32",
            "components.shell.launcher.subtitle-font-size",
        ),
        (
            "LAUNCHER_SCALE_FROM",
            "f32",
            "components.shell.launcher.scale-from",
        ),
        (
            "LAUNCHER_RESULT_RADIUS",
            "f32",
            "components.shell.launcher.result-radius",
        ),
    ] {
        const_doc(out, name, &format!("Tengen launcher metric `{path}`."));
        out.push_str(&format!(
            ": {ty} = {};\n\n",
            float(number(components, path, COMPONENTS))
        ));
    }
    for (name, path) in [
        (
            "LAUNCHER_OPEN_MS",
            "components.shell.launcher.open-duration",
        ),
        (
            "LAUNCHER_CLOSE_MS",
            "components.shell.launcher.close-duration",
        ),
    ] {
        const_doc(out, name, &format!("Tengen launcher metric `{path}`."));
        out.push_str(&format!(
            ": u64 = {};\n\n",
            integer(components, path, COMPONENTS)
        ));
    }

    let tilesize = number(components, "components.shell.dock.tilesize", COMPONENTS) as i64;
    let padding = number(components, "components.shell.dock.padding", COMPONENTS) as i64;
    const_doc(
        out,
        "DOCK_BAR_HEIGHT",
        "The dock bar height, in logical pixels.",
    );
    out.push_str(&format!(": i32 = {};\n\n", tilesize + padding * 2));
}
