use super::*;

pub(super) fn emit_tints(out: &mut String, components: &Value) {
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
    for (name, light, dark) in [
        ("WINDOW_TINT_ACCENT", accent_light, accent_dark),
        ("WINDOW_TINT_WALLPAPER", wallpaper_light, wallpaper_dark),
        ("PANEL_TINT_WALLPAPER", panel_light, panel_dark),
    ] {
        pub_const(out, name);
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
    pub_const(out, "MENU_SEPARATOR_INK_MIX");
    out.push_str(&format!(": f32 = {};\n\n", float(separator_ink_mix)));

    let tint_subtle = number(components, "appearance.tint-strength.subtle", COMPONENTS);
    let tint_normal = number(components, "appearance.tint-strength.normal", COMPONENTS);
    let tint_strong = number(components, "appearance.tint-strength.strong", COMPONENTS);
    let tint_intense = number(components, "appearance.tint-strength.intense", COMPONENTS);
    pub_const(out, "TINT_STRENGTH");
    out.push_str(&format!(
        ": [f32; 4] = [{}, {}, {}, {}];\n\n",
        float(tint_subtle),
        float(tint_normal),
        float(tint_strong),
        float(tint_intense)
    ));

    for (name, key) in [
        ("LAYER_FACTOR_BG_SUNKEN", "bg-sunken"),
        ("LAYER_FACTOR_CONTROL", "bg-control"),
        ("LAYER_FACTOR_LINE", "line"),
        ("LAYER_FACTOR_SURFACE", "bg-surface"),
        ("LAYER_FACTOR_RAISED", "bg-raised"),
    ] {
        let factor = number(
            components,
            &format!("appearance.layer-factors.{key}"),
            COMPONENTS,
        );
        pub_const(out, name);
        out.push_str(&format!(": f32 = {};\n\n", float(factor)));
    }
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
    pub_const(out, "BACKDROP");
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
    pub_const(out, "BREAKPOINT");
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
        pub_const(out, &format!("SPACE_{step}"));
        out.push_str(&format!(
            ": f32 = {};\n\n",
            float(number(tokens, &format!("space.{step}"), TOKENS))
        ));
    }
}

pub(super) fn emit_wallpaper(out: &mut String, components: &Value) {
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

    pub_const(out, "WALLPAPER_ACCENT_DARKEN_LIGHT");
    out.push_str(&format!(": f32 = {};\n\n", float(accent_darken_light)));
    pub_const(out, "WALLPAPER_ACCENT_LIGHTEN_DARK");
    out.push_str(&format!(": f32 = {};\n\n", float(accent_lighten_dark)));
    pub_const(out, "ACCENT_MIN_CONTRAST_ON_WINDOW");
    out.push_str(&format!(": f32 = {};\n\n", float(accent_on_window)));
    pub_const(out, "ON_ACCENT_MIN_CONTRAST");
    out.push_str(&format!(": f32 = {};\n\n", float(on_accent)));
    pub_const(out, "MUTED_BELOW_SATURATION");
    out.push_str(&format!(": f32 = {};\n\n", float(muted_below)));
    pub_const(out, "MUTED_MAX_SATURATION");
    out.push_str(&format!(": f32 = {};\n\n", float(muted_max)));
}

pub(super) fn emit_font_families(out: &mut String, tokens: &Value) {
    let stack = string(tokens, "font.families.sans", TOKENS);
    let families: Vec<String> = stack
        .split(',')
        .map(|family| family.trim().trim_matches('"').to_string())
        .collect();
    pub_const(out, "SANS_FAMILIES");
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
    pub_const(out, "TEXT");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "body")));
    pub_const(out, "DISPLAY");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "display")));
    pub_const(out, "TITLE_1");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "title-1")));
    pub_const(out, "TITLE_2");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "title-2")));
    pub_const(out, "HEADLINE");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "headline")));
    pub_const(out, "CALLOUT");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "callout")));
    pub_const(out, "CAPTION");
    out.push_str(&format!(": Font = {};\n\n", font(tokens, "caption")));
    let line_height = number(tokens, "font.line-height-scale", TOKENS);
    pub_const(out, "TEXT_LINE_HEIGHT");
    out.push_str(&format!(": f32 = {};\n\n", float(line_height)));
}

pub(super) fn emit_shadows(out: &mut String, tokens: &Value) {
    pub_const(out, "SHADOW");
    out.push_str(&format!(": ShadowStyle = {};\n\n", shadow(tokens, "light")));
    pub_const(out, "SHADOW_DARK");
    out.push_str(&format!(": ShadowStyle = {};\n\n", shadow(tokens, "dark")));
    pub_const(out, "KNOB_SHADOW");
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
    let panel_sides = theme_color(tokens, theme, "panel-inner");
    let panel_edges = theme_color(tokens, theme, "panel-inner-edge");
    let panel_inner = if panel_sides.a == 0 && panel_edges.a == 0 {
        "None".to_string()
    } else {
        format!(
            "Some(PanelLine {{ sides: {}, edges: {} }})",
            rgba(panel_sides),
            rgba(panel_edges)
        )
    };
    let traffic_ref = |path: &str| {
        color_ref(
            tokens,
            theme,
            string(components, path, COMPONENTS),
            path,
            COMPONENTS,
        )
    };
    let inactive = traffic_ref("components.window.traffic-lights.inactive");
    let rim_lit = traffic_ref("components.window.traffic-lights.rim.color");
    let rim_idle = traffic_ref("components.window.traffic-lights.rim.inactive");
    format!(
        "Palette {{\n\
         \x20   titlebar_bg: FocusPair {{ focused: {}, unfocused: {} }},\n\
         \x20   hairline: {},\n\
         \x20   inner_line: {inner},\n\
         \x20   panel_inner_line: {panel_inner},\n\
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
         \x20   traffic_rim: TrafficRim {{ lit: {}, idle: {} }},\n\
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
        rgba(rim_lit),
        rgba(rim_idle),
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
        pub_const(out, name);
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
        pub_const(out, name);
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
        pub_const(out, name);
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
        pub_const(out, name);
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
        pub_const(out, name);
        out.push_str(&format!(
            ": Rgba = {};\n\n",
            rgba(theme_color(tokens, theme, token))
        ));
    }
}
