use super::*;

pub(super) fn emit_titlebar(out: &mut String, tokens: &Value, components: &Value) {
    let thickness = integer(tokens, "size.titlebar-ssd-h", TOKENS);
    let client_thickness = integer(tokens, "size.titlebar-h", TOKENS);
    let diameter = integer(
        components,
        "components.window.traffic-lights.size",
        COMPONENTS,
    );
    let gap = integer(
        components,
        "components.window.traffic-lights.gap",
        COMPONENTS,
    );
    let radius = metric(
        tokens,
        components,
        "components.window.traffic-lights.radius",
        COMPONENTS,
    ) as i64;
    let theme = default_theme(tokens);
    let close = color_ref(
        tokens,
        theme,
        string(
            components,
            "components.window.traffic-lights.styles.standard.close",
            COMPONENTS,
        ),
        "components.window.traffic-lights.styles.standard.close",
        COMPONENTS,
    );
    let minimize = color_ref(
        tokens,
        theme,
        string(
            components,
            "components.window.traffic-lights.styles.standard.minimize",
            COMPONENTS,
        ),
        "components.window.traffic-lights.styles.standard.minimize",
        COMPONENTS,
    );
    let zoom = color_ref(
        tokens,
        theme,
        string(
            components,
            "components.window.traffic-lights.styles.standard.zoom",
            COMPONENTS,
        ),
        "components.window.traffic-lights.styles.standard.zoom",
        COMPONENTS,
    );
    out.push_str(&format!(
        "pub const TITLEBAR: Titlebar = Titlebar {{\n\
         \x20   thickness: {thickness},\n\
         \x20   buttons: TitlebarButtons {{\n\
         \x20       order: [\n\
         \x20           Button {{ kind: ButtonKind::Close, color: {} }},\n\
         \x20           Button {{ kind: ButtonKind::Minimize, color: {} }},\n\
         \x20           Button {{ kind: ButtonKind::Maximize, color: {} }},\n\
         \x20       ],\n\
         \x20       diameter: {diameter},\n\
         \x20       spacing: {gap},\n\
         \x20       corner_radius: {radius},\n\
         \x20   }},\n\
         \x20   font: {},\n\
         }};\n\n",
        rgba(close),
        rgba(minimize),
        rgba(zoom),
        font(tokens, "headline")
    ));
    pub_const(out, "TITLEBAR_CLIENT_HEIGHT");
    out.push_str(&format!(": u32 = {client_thickness};\n\n"));
    pub_const(out, "WINDOW_MIN_SIZE");
    out.push_str(&format!(
        ": (u32, u32) = ({}, {});\n\n",
        integer(components, "components.window.min-width", COMPONENTS),
        integer(components, "components.window.min-height", COMPONENTS)
    ));
    pub_const(out, "WINDOW_DEFAULT_SIZE");
    out.push_str(&format!(
        ": (u32, u32) = ({}, {});\n\n",
        integer(components, "components.window.default-width", COMPONENTS),
        integer(components, "components.window.default-height", COMPONENTS)
    ));

    let interval = integer(
        components,
        "components.window.titlebar.double-click.interval-ms",
        COMPONENTS,
    );
    let slop = number(
        components,
        "components.window.titlebar.double-click.slop",
        COMPONENTS,
    );
    pub_const(out, "TITLEBAR_DOUBLE_CLICK");
    out.push_str(&format!(
        ": DoubleClick = DoubleClick {{ interval_ms: {interval}, slop: {} }};\n\n",
        float(slop)
    ));
}

pub(super) fn emit_window(out: &mut String, tokens: &Value, components: &Value) {
    let radius = metric(tokens, components, "components.window.radius", COMPONENTS) as u32;
    let exponent = number(
        components,
        "components.window.superellipse-exponent",
        COMPONENTS,
    );
    pub_const(out, "WINDOW");
    out.push_str(&format!(
        ": WindowStyle = WindowStyle {{\n    corner_radius: {radius},\n    superellipse_exponent: {},\n}};\n\n",
        float(exponent)
    ));
    let toolbar_radius = metric(
        tokens,
        components,
        "components.window.toolbar-radius",
        COMPONENTS,
    ) as u32;
    pub_const(out, "WINDOW_TOOLBAR");
    out.push_str(&format!(
        ": WindowStyle = WindowStyle {{\n    corner_radius: {toolbar_radius},\n    superellipse_exponent: {},\n}};\n\n",
        float(exponent)
    ));
    let edge = number(components, "components.window.resize-edge", COMPONENTS);
    let corner = number(components, "components.window.resize-corner", COMPONENTS);
    pub_const(out, "WINDOW_RESIZE");
    out.push_str(&format!(
        ": WindowResize = WindowResize {{ edge: {}, corner: {} }};\n\n",
        float(edge),
        float(corner)
    ));
}

pub(super) fn emit_dialog(out: &mut String, tokens: &Value, components: &Value) {
    let radius = metric(tokens, components, "components.dialog.radius", COMPONENTS) as u32;
    pub_const(out, "DIALOG");
    out.push_str(&format!(
        ": WindowStyle = WindowStyle {{\n    corner_radius: {radius},\n    superellipse_exponent: 2.0,\n}};\n\n"
    ));
    let padding = number(components, "components.dialog.padding", COMPONENTS);
    pub_const(out, "DIALOG_PADDING");
    out.push_str(&format!(": f32 = {};\n\n", float(padding)));
}

pub(super) fn emit_popup(out: &mut String, tokens: &Value, components: &Value) {
    let radius = metric(
        tokens,
        components,
        "components.shell.menu.panel.radius",
        COMPONENTS,
    ) as u32;
    pub_const(out, "POPUP");
    out.push_str(&format!(
        ": WindowStyle = WindowStyle {{\n    corner_radius: {radius},\n    superellipse_exponent: 2.0,\n}};\n\n"
    ));
}

pub(super) fn emit_titlebar_rail(out: &mut String, components: &Value) {
    let field = |key: &str| {
        float(number(
            components,
            &format!("components.window.titlebar.rail.{key}"),
            COMPONENTS,
        ))
    };
    pub_const(out, "TITLEBAR_RAIL");
    out.push_str(&format!(
        ": TitlebarRailMetrics = TitlebarRailMetrics {{ width: {}, width_wide: {}, actions_width: {}, padding_top: {}, padding_bottom: {}, title_size: {}, pill_inset: {}, pill_padding_x: {}, pill_padding_y: {}, pill_radius: {}, reveal_zone: {}, reveal_corner: {}, edge_line: {}, edge_accent: {} }};\n\n",
        field("width"),
        field("width-wide"),
        field("actions-width"),
        field("padding-top"),
        field("padding-bottom"),
        field("title-size"),
        field("pill-inset"),
        field("pill-padding-x"),
        field("pill-padding-y"),
        field("pill-radius"),
        field("reveal-zone"),
        field("reveal-corner"),
        field("edge-line"),
        field("edge-accent")
    ));
}

pub(super) fn emit_fullscreen_glass(out: &mut String, components: &Value) {
    emit_glass(out, components, "fullscreen-glass", "FULLSCREEN_GLASS");
    emit_glass(out, components, "fullscreen-band", "FULLSCREEN_BAND");
    let path = |key: &str| format!("components.window.titlebar.fullscreen-bar.{key}");
    let field = |key: &str| float(number(components, &path(key), COMPONENTS));
    pub_const(out, "FULLSCREEN_BAR");
    out.push_str(&format!(
        ": FullscreenBarMetrics = FullscreenBarMetrics {{ content_top: {}, band_line: {} }};\n\n",
        field("content-top"),
        field("band-line")
    ));
}

fn emit_glass(out: &mut String, components: &Value, key: &str, name: &str) {
    let path = |field: &str| format!("components.window.titlebar.{key}.{field}");
    let field = |key: &str| float(number(components, &path(key), COMPONENTS));
    let stops = |key: &str| {
        value(components, &path(key), COMPONENTS)
            .as_array()
            .unwrap_or_else(|| panic!("vow-design: `{}` must be a list", path(key)))
            .iter()
            .map(|stop| {
                let pair = stop.as_array().expect("[offset, alpha]");
                format!(
                    "({}, {})",
                    float(pair[0].as_f64().expect("number")),
                    float(pair[1].as_f64().expect("number"))
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    pub_const(out, name);
    out.push_str(&format!(
        ": FullscreenGlassMetrics = FullscreenGlassMetrics {{ height: {}, blur: {}, layers: {}, content_top: {}, tint: &[{}], mask: &[{}] }};\n\n",
        field("height"),
        field("blur"),
        integer(components, &path("layers"), COMPONENTS),
        field("content-top"),
        stops("tint"),
        stops("mask")
    ));
}

pub(super) fn emit_traffic(out: &mut String, tokens: &Value, components: &Value) {
    let width = number(
        components,
        "components.window.traffic-lights.rim.width",
        COMPONENTS,
    );
    let darken = number(
        components,
        "components.window.traffic-lights.styles.accent.close-minimize-darken",
        COMPONENTS,
    );
    let lighten = number(
        components,
        "components.window.traffic-lights.styles.accent.zoom-lighten",
        COMPONENTS,
    );
    pub_const(out, "TRAFFIC_RIM_WIDTH");
    out.push_str(&format!(": f32 = {};\n\n", float(width)));
    pub_const(out, "TRAFFIC_CLOSE_MINIMIZE_DARKEN");
    out.push_str(&format!(": f32 = {};\n\n", float(darken)));
    pub_const(out, "TRAFFIC_ZOOM_LIGHTEN");
    out.push_str(&format!(": f32 = {};\n\n", float(lighten)));
    let pressed = number(
        components,
        "components.window.traffic-lights.pressed-darken",
        COMPONENTS,
    );
    pub_const(out, "TRAFFIC_PRESSED_DARKEN");
    out.push_str(&format!(": f32 = {};\n\n", float(pressed)));

    let standard_glyph = color_ref(
        tokens,
        default_theme(tokens),
        string(
            components,
            "components.window.traffic-lights.styles.standard.glyph",
            COMPONENTS,
        ),
        "components.window.traffic-lights.styles.standard.glyph",
        COMPONENTS,
    );
    let glyph_on_dark = color_ref(
        tokens,
        default_theme(tokens),
        string(
            components,
            "components.window.traffic-lights.styles.accent.glyph-on-dark",
            COMPONENTS,
        ),
        "components.window.traffic-lights.styles.accent.glyph-on-dark",
        COMPONENTS,
    );
    let glyph_on_light = color_ref(
        tokens,
        default_theme(tokens),
        string(
            components,
            "components.window.traffic-lights.styles.accent.glyph-on-light",
            COMPONENTS,
        ),
        "components.window.traffic-lights.styles.accent.glyph-on-light",
        COMPONENTS,
    );
    for (name, color) in [
        ("TRAFFIC_GLYPH_STANDARD", standard_glyph),
        ("TRAFFIC_GLYPH_ON_DARK", glyph_on_dark),
        ("TRAFFIC_GLYPH_ON_LIGHT", glyph_on_light),
    ] {
        pub_const(out, name);
        out.push_str(&format!(": Rgba = {};\n\n", rgba(color)));
    }

    let box_size = number(
        components,
        "components.window.traffic-lights.glyphs.box",
        COMPONENTS,
    );
    let stroke = number(
        components,
        "components.window.traffic-lights.glyphs.stroke",
        COMPONENTS,
    );
    let close = glyph_segments(components, "components.window.traffic-lights.glyphs.close");
    let minimize = glyph_segments(
        components,
        "components.window.traffic-lights.glyphs.minimize",
    );
    let zoom = glyph_triangles(components, "components.window.traffic-lights.glyphs.zoom");
    let plus = glyph_segments(components, "components.window.traffic-lights.glyphs.plus");
    pub_const(out, "TRAFFIC_GLYPHS");
    out.push_str(&format!(
        ": TrafficGlyphs = TrafficGlyphs {{\n\
         \x20   box_size: {},\n\
         \x20   stroke: {},\n\
         \x20   close: &[{}],\n\
         \x20   minimize: &[{}],\n\
         \x20   zoom: &[{}],\n\
         \x20   plus: &[{}],\n\
         }};\n\n",
        float(box_size),
        float(stroke),
        close,
        minimize,
        zoom,
        plus
    ));
}

pub(super) fn glyph_segments(components: &Value, path: &str) -> String {
    let array_path = format!("{path}.segments");
    let values = value(components, &array_path, COMPONENTS)
        .as_array()
        .unwrap_or_else(|| panic!("vow-design: `{array_path}` in {COMPONENTS} is not an array"));
    values
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item = item.as_array().unwrap_or_else(|| {
                panic!("vow-design: `{array_path}[{index}]` in {COMPONENTS} is not an array")
            });
            if item.len() != 4 {
                panic!(
                    "vow-design: `{array_path}[{index}]` in {COMPONENTS} must have 4 numbers"
                );
            }
            let point = |offset: usize, label: &str| {
                item[offset].as_f64().unwrap_or_else(|| {
                    panic!(
                        "vow-design: `{array_path}[{index}][{label}]` in {COMPONENTS} is not a number"
                    )
                })
            };
            format!(
                "Segment {{ a: ({}, {}), b: ({}, {}) }}",
                float(point(0, "0")),
                float(point(1, "1")),
                float(point(2, "2")),
                float(point(3, "3"))
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn glyph_triangles(components: &Value, path: &str) -> String {
    let array_path = format!("{path}.triangles");
    let values = value(components, &array_path, COMPONENTS)
        .as_array()
        .unwrap_or_else(|| panic!("vow-design: `{array_path}` in {COMPONENTS} is not an array"));
    values
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item = item.as_array().unwrap_or_else(|| {
                panic!("vow-design: `{array_path}[{index}]` in {COMPONENTS} is not an array")
            });
            if item.len() != 6 {
                panic!(
                    "vow-design: `{array_path}[{index}]` in {COMPONENTS} must have 6 numbers"
                );
            }
            let point = |offset: usize| {
                let x = item[offset].as_f64().unwrap_or_else(|| {
                    panic!(
                        "vow-design: `{array_path}[{index}][{offset}]` in {COMPONENTS} is not a number"
                    )
                });
                let y = item[offset + 1].as_f64().unwrap_or_else(|| {
                    panic!(
                        "vow-design: `{array_path}[{index}][{}]` in {COMPONENTS} is not a number",
                        offset + 1
                    )
                });
                (x, y)
            };
            let (x0, y0) = point(0);
            let (x1, y1) = point(2);
            let (x2, y2) = point(4);
            format!(
                "Triangle([({}, {}), ({}, {}), ({}, {})])",
                float(x0),
                float(y0),
                float(x1),
                float(y1),
                float(x2),
                float(y2)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}
