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
    let margin = integer(
        components,
        "components.window.traffic-lights.margin-start",
        COMPONENTS,
    );
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
        "#[allow(missing_docs)]\n\
         /// The titlebar: server-decorated windows get the Tengen bar.\n\
         pub const TITLEBAR: Titlebar = Titlebar {{\n\
         \x20   position: TitlebarPosition::Top,\n\
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
         \x20       margin_start: {margin},\n\
         \x20   }},\n\
         \x20   font: {},\n\
         }};\n\n",
        rgba(close),
        rgba(minimize),
        rgba(zoom),
        font(tokens, "headline")
    ));
    const_doc(
        out,
        "TITLEBAR_CLIENT_HEIGHT",
        "The height of a BarrierKit app's own titlebar, in logical pixels.",
    );
    out.push_str(&format!(": u32 = {client_thickness};\n\n"));
    const_doc(
        out,
        "WINDOW_MIN_SIZE",
        "The smallest size a BarrierKit app window can take, in logical pixels.",
    );
    out.push_str(&format!(
        ": (u32, u32) = ({}, {});\n\n",
        integer(components, "components.window.min-width", COMPONENTS),
        integer(components, "components.window.min-height", COMPONENTS)
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
    const_doc(
        out,
        "TITLEBAR_DOUBLE_CLICK",
        "The double-click interval and pointer slop for the titlebar.",
    );
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
    const_doc(out, "WINDOW", "The window shape: Tengen's rounded corner.");
    out.push_str(&format!(
        ": WindowStyle = WindowStyle {{\n    corner_radius: {radius},\n    superellipse_exponent: {},\n}};\n\n",
        float(exponent)
    ));
}

pub(super) fn emit_dialog(out: &mut String, tokens: &Value, components: &Value) {
    let radius = metric(tokens, components, "components.dialog.radius", COMPONENTS) as u32;
    const_doc(
        out,
        "DIALOG",
        "The dialog shape: same language, smaller radius.",
    );
    out.push_str(&format!(
        ": WindowStyle = WindowStyle {{\n    corner_radius: {radius},\n    superellipse_exponent: 2.0,\n}};\n\n"
    ));
    let padding = number(components, "components.dialog.padding", COMPONENTS);
    const_doc(
        out,
        "DIALOG_PADDING",
        "Inner padding of a dialog, in logical px.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(padding)));
}

pub(super) fn emit_popup(out: &mut String, tokens: &Value, components: &Value) {
    let radius = metric(
        tokens,
        components,
        "components.shell.menu.panel.radius",
        COMPONENTS,
    ) as u32;
    const_doc(
        out,
        "POPUP",
        "The popup shape the compositor gives menus and popovers.",
    );
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
    const_doc(out, "TITLEBAR_RAIL", "The vertical titlebar metrics.");
    out.push_str(&format!(
        ": TitlebarRailMetrics = TitlebarRailMetrics {{ width: {}, width_wide: {}, actions_width: {}, padding_top: {}, padding_bottom: {}, title_size: {}, pill_inset: {}, pill_padding_x: {}, pill_padding_y: {}, pill_radius: {}, pill_clearance: {}, reveal_zone: {}, reveal_corner: {}, edge_line: {}, edge_accent: {} }};\n\n",
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
        field("pill-clearance"),
        field("reveal-zone"),
        field("reveal-corner"),
        field("edge-line"),
        field("edge-accent")
    ));
}

pub(super) fn emit_fullscreen_glass(out: &mut String, components: &Value) {
    let path = |key: &str| format!("components.window.titlebar.fullscreen-glass.{key}");
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
    const_doc(
        out,
        "FULLSCREEN_GLASS",
        "The progressive glass behind the titlebar in full screen.",
    );
    out.push_str(&format!(
        ": FullscreenGlassMetrics = FullscreenGlassMetrics {{ height: {}, blur: {}, bands: {}, content_top: {}, tint: &[{}], mask: &[{}] }};\n\n",
        field("height"),
        field("blur"),
        integer(components, &path("bands"), COMPONENTS),
        field("content-top"),
        stops("tint"),
        stops("mask")
    ));
}

pub(super) fn emit_traffic(out: &mut String, tokens: &Value, components: &Value) {
    let rim = color_ref(
        tokens,
        default_theme(tokens),
        string(
            components,
            "components.window.traffic-lights.rim.color",
            COMPONENTS,
        ),
        "components.window.traffic-lights.rim.color",
        COMPONENTS,
    );
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
    const_doc(
        out,
        "TRAFFIC_RIM",
        "The hairline rim around each traffic light.",
    );
    out.push_str(&format!(": Rgba = {};\n\n", rgba(rim)));
    const_doc(
        out,
        "TRAFFIC_RIM_WIDTH",
        "The traffic-light rim width, in logical pixels.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(width)));
    const_doc(
        out,
        "TRAFFIC_CLOSE_MINIMIZE_DARKEN",
        "How much the close/minimize lights darken their source colour.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(darken)));
    const_doc(
        out,
        "TRAFFIC_ZOOM_LIGHTEN",
        "How much the zoom light lightens its source colour.",
    );
    out.push_str(&format!(": f32 = {};\n\n", float(lighten)));
    let pressed = number(
        components,
        "components.window.traffic-lights.pressed-darken",
        COMPONENTS,
    );
    const_doc(
        out,
        "TRAFFIC_PRESSED_DARKEN",
        "How much a pressed traffic light darkens its fill.",
    );
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
    for (name, doc, color) in [
        (
            "TRAFFIC_GLYPH_STANDARD",
            "The glyph colour on standard traffic lights.",
            standard_glyph,
        ),
        (
            "TRAFFIC_GLYPH_ON_DARK",
            "The glyph colour on dark accent/wallpaper lights.",
            glyph_on_dark,
        ),
        (
            "TRAFFIC_GLYPH_ON_LIGHT",
            "The glyph colour on light accent/wallpaper lights.",
            glyph_on_light,
        ),
    ] {
        const_doc(out, name, doc);
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
    const_doc(
        out,
        "TRAFFIC_GLYPHS",
        "The close, minimize, zoom and plus glyph geometry for traffic lights.",
    );
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

    let _ = tokens;
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
