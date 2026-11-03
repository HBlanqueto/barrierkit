use super::*;

pub(super) fn emit_tints(out: &mut String, tokens: &Value, components: &Value) {
    let accent_light = number(
        components,
        "appearance.modes.accent.windows.light",
        COMPONENTS,
    );
    let accent_dark = number(
        components,
        "appearance.modes.accent.windows.dark",
        COMPONENTS,
    );
    let wallpaper_light = number(
        components,
        "appearance.modes.wallpaper.scope.windows.windows.light",
        COMPONENTS,
    );
    let wallpaper_dark = number(
        components,
        "appearance.modes.wallpaper.scope.windows.windows.dark",
        COMPONENTS,
    );
    let panel_light = number(
        components,
        "appearance.modes.wallpaper.scope.desktop.panel-dock.light",
        COMPONENTS,
    );
    let panel_dark = number(
        components,
        "appearance.modes.wallpaper.scope.desktop.panel-dock.dark",
        COMPONENTS,
    );
    for (name, doc, light, dark) in [
        (
            "WINDOW_TINT_ACCENT",
            "How much the accent tints the window chrome, light and dark.",
            accent_light,
            accent_dark,
        ),
        (
            "WINDOW_TINT_WALLPAPER",
            "How much the wallpaper tints the window chrome, light and dark.",
            wallpaper_light,
            wallpaper_dark,
        ),
        (
            "PANEL_TINT_WALLPAPER",
            "How much the wallpaper tints the panels and dock, light and dark.",
            panel_light,
            panel_dark,
        ),
    ] {
        const_doc(out, name, doc);
        out.push_str(&format!(
            ": [f32; 2] = [{}, {}];\n\n",
            float(light),
            float(dark)
        ));
    }

    let separator_ink_mix = number(
        components,
        "components.shell.menu.separator.tint-ink-mix",
        COMPONENTS,
    );
    const_doc(
        out,
        "MENU_SEPARATOR_INK_MIX",
        "How far a tinted menu separator moves from its panel toward the ink.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(separator_ink_mix)));

    let tint_subtle = number(components, "appearance.tint-strength.subtle", COMPONENTS);
    let tint_normal = number(components, "appearance.tint-strength.normal", COMPONENTS);
    let tint_strong = number(components, "appearance.tint-strength.strong", COMPONENTS);
    let tint_intense = number(components, "appearance.tint-strength.intense", COMPONENTS);
    const_doc(
        out,
        "TINT_STRENGTH",
        "Tint strength multipliers for Subtle, Normal, Strong and Intense.",
    );
    out.push_str(&format!(
        ": [f32; 4] = [{}, {}, {}, {}];\n\n",
        float(tint_subtle),
        float(tint_normal),
        float(tint_strong),
        float(tint_intense)
    ));

    for (name, doc, key) in [
        (
            "LAYER_FACTOR_BG_SUNKEN",
            "Layer factor for bg-sunken.",
            "bg-sunken",
        ),
        (
            "LAYER_FACTOR_CONTROL",
            "Layer factor for controls (bg-control, bg-hover).",
            "bg-control",
        ),
        ("LAYER_FACTOR_LINE", "Layer factor for lines.", "line"),
        (
            "LAYER_FACTOR_SURFACE",
            "Layer factor for groups and cards (bg-surface).",
            "bg-surface",
        ),
        (
            "LAYER_FACTOR_RAISED",
            "Layer factor for menus and dialogs (bg-raised).",
            "bg-raised",
        ),
    ] {
        let factor = number(
            components,
            &format!("appearance.layer-factors.{key}"),
            COMPONENTS,
        );
        const_doc(out, name, doc);
        out.push_str(&format!(": f32 = {};\n\n", float(factor)));
    }

    let _ = tokens;
}

pub(super) fn emit_backdrop(out: &mut String, components: &Value) {
    let field = |key: &str| {
        float(
            value(
                components,
                &format!("components.backdrop.{key}"),
                COMPONENTS,
            )
            .as_f64()
            .expect("number"),
        )
    };
    let count = |key: &str| {
        integer(
            components,
            &format!("components.backdrop.{key}"),
            COMPONENTS,
        )
    };
    const_doc(
        out,
        "BACKDROP",
        "Tinted window backgrounds drawn from an image or chosen colours.",
    );
    out.push_str(&format!(
        ": BackdropMetrics = BackdropMetrics {{ angles: [{}], opacity: {}, stops: {}, lightness: [{}], chroma: {}, base_darken: {}, base_lightness: [{}], base_chroma: {}, samples: {}, clusters: {}, ink_secondary: {}, ink_tertiary: {}, glass: {}, glass_strong: {}, island: {}, sunken: {}, accent_lightness_on_dark: {}, accent_lightness_on_light: {}, accent_min_chroma: {} }};\n\n",
        floats(components, "components.backdrop.angles"),
        field("opacity"),
        count("stops"),
        floats(components, "components.backdrop.lightness"),
        field("chroma"),
        field("base-darken"),
        floats(components, "components.backdrop.base-lightness"),
        field("base-chroma"),
        count("samples"),
        count("clusters"),
        field("ink-secondary"),
        field("ink-tertiary"),
        field("glass"),
        field("glass-strong"),
        field("island"),
        field("sunken"),
        field("accent-lightness-on-dark"),
        field("accent-lightness-on-light"),
        field("accent-min-chroma"),
    ));
    const_doc(
        out,
        "BREAKPOINT",
        "Window widths where adaptive layouts change shape.",
    );
    out.push_str(&format!(
        ": BreakpointMetrics = BreakpointMetrics {{ medium: {}, wide: {} }};\n\n",
        float(
            value(components, "components.breakpoint.medium", COMPONENTS)
                .as_f64()
                .expect("number")
        ),
        float(
            value(components, "components.breakpoint.wide", COMPONENTS)
                .as_f64()
                .expect("number")
        ),
    ));
}

pub(super) fn emit_space(out: &mut String, tokens: &Value) {
    for step in ["1", "2", "3", "4"] {
        const_doc(
            out,
            &format!("SPACE_{step}"),
            "A step of the Tengen spacing scale.",
        );
        out.push_str(&format!(
            ": f32 = {};\n\n",
            float(number(tokens, &format!("space.{step}"), TOKENS))
        ));
    }
}

pub(super) fn emit_wallpaper(out: &mut String, tokens: &Value, components: &Value) {
    let _ = tokens;
    let accent_darken_light = number(
        components,
        "appearance.modes.wallpaper.accent-shift.light.darken",
        COMPONENTS,
    );
    let accent_lighten_dark = number(
        components,
        "appearance.modes.wallpaper.accent-shift.dark.lighten",
        COMPONENTS,
    );
    let accent_on_window = number(
        components,
        "appearance.modes.wallpaper.contrast.accent-on-window",
        COMPONENTS,
    );
    let on_accent = number(
        components,
        "appearance.modes.wallpaper.contrast.on-accent",
        COMPONENTS,
    );
    let muted_below = number(
        components,
        "appearance.modes.wallpaper.muted.below-saturation",
        COMPONENTS,
    );
    let muted_max = number(
        components,
        "appearance.modes.wallpaper.muted.max-saturation",
        COMPONENTS,
    );

    const_doc(
        out,
        "WALLPAPER_ACCENT_DARKEN_LIGHT",
        "How much the wallpaper accent darkens in light mode.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(accent_darken_light)));
    const_doc(
        out,
        "WALLPAPER_ACCENT_LIGHTEN_DARK",
        "How much the wallpaper accent lightens in dark modes.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(accent_lighten_dark)));
    const_doc(
        out,
        "ACCENT_MIN_CONTRAST_ON_WINDOW",
        "Minimum contrast of the wallpaper accent against the window background.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(accent_on_window)));
    const_doc(
        out,
        "ON_ACCENT_MIN_CONTRAST",
        "Minimum contrast of text drawn on the wallpaper accent.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(on_accent)));
    const_doc(
        out,
        "MUTED_BELOW_SATURATION",
        "Saturation below which the wallpaper accent gets a saturation boost.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(muted_below)));
    const_doc(
        out,
        "MUTED_MAX_SATURATION",
        "Maximum saturation after boosting a muted wallpaper accent.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(muted_max)));
}

pub(super) fn emit_font_families(out: &mut String, tokens: &Value) {
    let stack = string(tokens, "font.families.sans", TOKENS);
    let families: Vec<String> = stack
        .split(',')
        .map(|family| family.trim().trim_matches('"').to_string())
        .collect();
    const_doc(out, "SANS_FAMILIES", "The sans-serif family stack.");
    out.push_str(": [&str; ");
    out.push_str(&families.len().to_string());
    out.push_str("] = [");
    for (index, family) in families.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("{family:?}"));
    }
    out.push_str("];\n\n");
}

pub(super) fn emit_typography(out: &mut String, tokens: &Value) {
    const_doc(out, "TEXT", "The body text face.");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "body")));
    const_doc(
        out,
        "DISPLAY",
        "The display face (28/700), the largest title.",
    );
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "display")));
    const_doc(out, "TITLE_1", "The primary title face (20/650).");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "title-1")));
    const_doc(out, "TITLE_2", "The secondary title face (15/600).");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "title-2")));
    const_doc(out, "HEADLINE", "The headline face (13/600).");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "headline")));
    const_doc(out, "CALLOUT", "The callout face (12/400).");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "callout")));
    const_doc(out, "CAPTION", "The caption face (11/500).");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "caption")));
    let line_height = number(tokens, "font.line-height-scale", TOKENS);
    const_doc(
        out,
        "TEXT_LINE_HEIGHT",
        "The line-height multiplier used by the text engines.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(line_height)));
}

pub(super) fn emit_shadows(out: &mut String, tokens: &Value) {
    const_doc(out, "SHADOW", "The window shadow in light mode.");
    out.push_str(&format!(": ShadowStyle = {};\n\n", shadow(tokens, "light")));
    const_doc(
        out,
        "SHADOW_DARK",
        "The window shadow in dark and dark-tint modes.",
    );
    out.push_str(&format!(": ShadowStyle = {};\n\n", shadow(tokens, "dark")));
    const_doc(
        out,
        "KNOB_SHADOW",
        "The shadow under switch and slider knobs.",
    );
    out.push_str(&format!(
        ": ShadowLayer = ShadowLayer {{ blur: {}, y_offset: {}, opacity: 1.0, color: {} }};\n\n",
        integer(tokens, "shadow.slint.knob.blur", TOKENS),
        integer(tokens, "shadow.slint.knob.offset-y", TOKENS),
        rgba(color(tokens, "shadow.slint.knob.color", TOKENS)),
    ));
}

pub(super) fn palette(tokens: &Value, components: &Value, theme: &str) -> String {
    let inner = theme_color(tokens, theme, "window-inner");
    let inner = if inner.a == 0 {
        "None".to_string()
    } else {
        format!("Some({})", rgba(inner))
    };
    let inactive = color_ref(
        tokens,
        theme,
        string(
            components,
            "components.window.traffic-lights.inactive",
            COMPONENTS,
        ),
        "components.window.traffic-lights.inactive",
        COMPONENTS,
    );
    format!(
        "Palette {{\n\
         \x20   titlebar_bg: FocusPair {{ focused: {}, unfocused: {} }},\n\
         \x20   hairline: {},\n\
         \x20   inner_line: {inner},\n\
         \x20   outer_border: {},\n\
         \x20   title: FocusPair {{ focused: {}, unfocused: {} }},\n\
         \x20   text: FocusPair {{ focused: {}, unfocused: {} }},\n\
         \x20   ink_strong: {},\n\
         \x20   ink_muted: {},\n\
         \x20   ink_faint: {},\n\
         \x20   line: {},\n\
         \x20   line_strong: {},\n\
         \x20   accent: {},\n\
         \x20   accent_inactive: {},\n\
         \x20   on_accent: {},\n\
         \x20   danger: {},\n\
         \x20   traffic_lights_inactive: {},\n\
         \x20   scrim: {},\n\
         \x20   selected_control: {},\n\
         \x20   card_border: {},\n\
         }}",
        rgba(theme_color(tokens, theme, "titlebar-bg")),
        rgba(theme_color(tokens, theme, "titlebar-bg-inactive")),
        rgba(theme_color(tokens, theme, "titlebar-line")),
        rgba(theme_color(tokens, theme, "window-outer")),
        rgba(theme_color(tokens, theme, "titlebar-title")),
        rgba(theme_color(tokens, theme, "titlebar-title-inactive")),
        rgba(theme_color(tokens, theme, "ink")),
        rgba(theme_color(tokens, theme, "ink-muted")),
        rgba(theme_color(tokens, theme, "ink-strong")),
        rgba(theme_color(tokens, theme, "ink-muted")),
        rgba(theme_color(tokens, theme, "ink-faint")),
        rgba(theme_color(tokens, theme, "line")),
        rgba(theme_color(tokens, theme, "line-strong")),
        rgba(theme_color(tokens, theme, "accent")),
        rgba(theme_color(tokens, theme, "accent-inactive")),
        rgba(theme_color(tokens, theme, "on-accent")),
        rgba(theme_color(tokens, theme, "danger")),
        rgba(inactive),
        rgba(theme_color(tokens, theme, "scrim")),
        rgba(theme_color(tokens, theme, "bg-selected-control")),
        rgba(theme_color(tokens, theme, "card-border"))
    )
}

pub(super) fn emit_palettes(out: &mut String, tokens: &Value, components: &Value) {
    for (name, theme) in [
        ("LIGHT", "light"),
        ("DARK", "dark"),
        ("DARK_TINT", "dark-tint"),
    ] {
        const_doc(out, name, &format!("The {theme} mode palette."));
        out.push_str(&format!(
            ": Palette = {};\n\n",
            palette(tokens, components, theme)
        ));
    }

    for (name, theme) in [
        ("BG_WINDOW_LIGHT", "light"),
        ("BG_WINDOW_DARK", "dark"),
        ("BG_WINDOW_DARK_TINT", "dark-tint"),
    ] {
        const_doc(
            out,
            name,
            &format!("The {theme} window background (`bg-window`)."),
        );
        out.push_str(&format!(
            ": Rgba = {};\n\n",
            rgba(theme_color(tokens, theme, "bg-window"))
        ));
    }

    for (name, theme) in [
        ("BG_CONTROL_LIGHT", "light"),
        ("BG_CONTROL_DARK", "dark"),
        ("BG_CONTROL_DARK_TINT", "dark-tint"),
    ] {
        const_doc(
            out,
            name,
            &format!("The {theme} control background (`bg-control`)."),
        );
        out.push_str(&format!(
            ": Rgba = {};\n\n",
            rgba(theme_color(tokens, theme, "bg-control"))
        ));
    }

    for (name, theme) in [
        ("BG_SURFACE_LIGHT", "light"),
        ("BG_SURFACE_DARK", "dark"),
        ("BG_SURFACE_DARK_TINT", "dark-tint"),
    ] {
        const_doc(
            out,
            name,
            &format!("The {theme} raised surface background (`bg-surface`)."),
        );
        out.push_str(&format!(
            ": Rgba = {};\n\n",
            rgba(theme_color(tokens, theme, "bg-surface"))
        ));
    }

    for (name, theme, token) in [
        ("BG_SUNKEN_LIGHT", "light", "bg-sunken"),
        ("BG_SUNKEN_DARK", "dark", "bg-sunken"),
        ("BG_SUNKEN_DARK_TINT", "dark-tint", "bg-sunken"),
        ("BG_RAISED_LIGHT", "light", "bg-raised"),
        ("BG_RAISED_DARK", "dark", "bg-raised"),
        ("BG_RAISED_DARK_TINT", "dark-tint", "bg-raised"),
        ("BG_HOVER_LIGHT", "light", "bg-hover"),
        ("BG_HOVER_DARK", "dark", "bg-hover"),
        ("BG_HOVER_DARK_TINT", "dark-tint", "bg-hover"),
    ] {
        const_doc(out, name, &format!("The {theme} `{token}` background."));
        out.push_str(&format!(
            ": Rgba = {};\n\n",
            rgba(theme_color(tokens, theme, token))
        ));
    }
}
