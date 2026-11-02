use super::*;

pub(super) fn emit_sidebar(out: &mut String, tokens: &Value, components: &Value) {
    for (name, theme) in [
        ("SIDEBAR_ISLAND_LIGHT", "light"),
        ("SIDEBAR_ISLAND_DARK", "dark"),
        ("SIDEBAR_ISLAND_DARK_TINT", "dark-tint"),
    ] {
        let bg_window = theme_color(tokens, theme, "bg-window");
        let island = theme_color(tokens, theme, "sidebar-island-bg");
        const_doc(
            out,
            name,
            &format!("The {theme} island sidebar background."),
        );
        out.push_str(&format!(": Rgba = {};\n\n", rgba(island)));
        const_doc(
            out,
            &format!("{name}_OPAQUE"),
            &format!("The {theme} island sidebar background over bg-window."),
        );
        out.push_str(&format!(
            ": Rgba = {};\n\n",
            rgba(opaque_over(island, bg_window))
        ));
    }

    for (name, theme) in [
        ("SIDEBAR_ISLAND_RIM_LIGHT", "light"),
        ("SIDEBAR_ISLAND_RIM_DARK", "dark"),
        ("SIDEBAR_ISLAND_RIM_DARK_TINT", "dark-tint"),
    ] {
        let rim = color_ref(
            tokens,
            theme,
            string(
                components,
                "components.window.sidebar.island.rim",
                COMPONENTS,
            ),
            "components.window.sidebar.island.rim",
            COMPONENTS,
        );
        const_doc(out, name, &format!("The {theme} island sidebar rim."));
        out.push_str(&format!(": Rgba = {};\n\n", rgba(rim)));
    }

    for (name, theme) in [
        ("SIDEBAR_ISLAND_INNER_LIGHT", "light"),
        ("SIDEBAR_ISLAND_INNER_DARK", "dark"),
        ("SIDEBAR_ISLAND_INNER_DARK_TINT", "dark-tint"),
    ] {
        let inner = color_ref(
            tokens,
            theme,
            string(
                components,
                "components.window.sidebar.island.inner",
                COMPONENTS,
            ),
            "components.window.sidebar.island.inner",
            COMPONENTS,
        );
        const_doc(
            out,
            name,
            &format!("The {theme} island sidebar inner line."),
        );
        out.push_str(&format!(": Rgba = {};\n\n", rgba(inner)));
    }

    for (name, theme) in [
        ("TOOLGROUP_BG_LIGHT", "light"),
        ("TOOLGROUP_BG_DARK", "dark"),
        ("TOOLGROUP_BG_DARK_TINT", "dark-tint"),
    ] {
        let bg_window = theme_color(tokens, theme, "bg-window");
        let glass = theme_color(tokens, theme, "toolbar-glass-bg");
        const_doc(
            out,
            name,
            &format!("The {theme} toolbar glass capsule background."),
        );
        out.push_str(&format!(": Rgba = {};\n\n", rgba(glass)));
        const_doc(
            out,
            &format!("{name}_OPAQUE"),
            &format!("The {theme} toolbar glass capsule background over bg-window."),
        );
        out.push_str(&format!(
            ": Rgba = {};\n\n",
            rgba(opaque_over(glass, bg_window))
        ));
    }

    fn sidebar_palette_value(tokens: &Value, components: &Value, theme: &str) -> String {
        let color_at = |path: &str| {
            rgba(color_ref(
                tokens,
                theme,
                string(components, path, COMPONENTS),
                path,
                COMPONENTS,
            ))
        };
        format!(
            "SidebarPalette {{\n\
             \x20   item_ink: {},\n\
             \x20   item_icon: {},\n\
             \x20   selected_bg: {},\n\
             \x20   selected_ink: {},\n\
             \x20   selected_icon_rim: {},\n\
             \x20   inactive_selected_bg: {},\n\
             \x20   inactive_ink: {},\n\
             \x20   hover_bg: {},\n\
             \x20   section_ink: {},\n\
             \x20   count_ink: {},\n\
             \x20   line: {},\n\
             }}",
            color_at("components.window.sidebar.item.ink"),
            color_at("components.window.sidebar.item.icon-ink"),
            color_at("components.window.sidebar.item.selected-bg"),
            color_at("components.window.sidebar.item.selected-ink"),
            color_at("components.window.sidebar.item.selected-icon-rim"),
            color_at("components.window.sidebar.item.inactive-selected-bg"),
            color_at("components.window.sidebar.item.inactive-ink"),
            color_at("components.window.sidebar.item.hover-bg"),
            color_at("components.window.sidebar.section.ink"),
            color_at("components.window.sidebar.item.count-ink"),
            color_at("components.window.sidebar.flat.line"),
        )
    }

    for (name, theme) in [
        ("SIDEBAR_PALETTE_LIGHT", "light"),
        ("SIDEBAR_PALETTE_DARK", "dark"),
        ("SIDEBAR_PALETTE_DARK_TINT", "dark-tint"),
    ] {
        const_doc(
            out,
            name,
            &format!("The {theme} sidebar item/section palette."),
        );
        out.push_str(&format!(
            ": SidebarPalette = {};\n\n",
            sidebar_palette_value(tokens, components, theme)
        ));
    }

    fn toolgroup_palette_value(tokens: &Value, components: &Value, theme: &str) -> String {
        let color_at = |path: &str| {
            rgba(color_ref(
                tokens,
                theme,
                string(components, path, COMPONENTS),
                path,
                COMPONENTS,
            ))
        };
        format!(
            "ToolgroupPalette {{\n\
             \x20   ink: {},\n\
             \x20   disabled_ink: {},\n\
             \x20   separator: {},\n\
             }}",
            color_at("components.window.toolgroup.ink"),
            color_at("components.window.toolgroup.disabled-ink"),
            color_at("components.window.toolgroup.separator"),
        )
    }

    for (name, theme) in [
        ("TOOLGROUP_PALETTE_LIGHT", "light"),
        ("TOOLGROUP_PALETTE_DARK", "dark"),
        ("TOOLGROUP_PALETTE_DARK_TINT", "dark-tint"),
    ] {
        const_doc(out, name, &format!("The {theme} toolbar capsule palette."));
        out.push_str(&format!(
            ": ToolgroupPalette = {};\n\n",
            toolgroup_palette_value(tokens, components, theme)
        ));
    }

    let width = integer(tokens, "size.sidebar-w", TOKENS) as u32;
    let inset = metric(
        tokens,
        components,
        "components.window.sidebar.island.inset",
        COMPONENTS,
    ) as u32;
    let padding_x = integer(
        components,
        "components.window.sidebar.island.padding-x",
        COMPONENTS,
    ) as u32;
    let radius = metric(
        tokens,
        components,
        "components.window.sidebar.island.radius",
        COMPONENTS,
    ) as u32;
    let rim_width = number(
        components,
        "components.window.sidebar.island.rim-width",
        COMPONENTS,
    );
    let inner_width = number(
        components,
        "components.window.sidebar.island.inner-width",
        COMPONENTS,
    );
    let blur = integer(
        components,
        "components.window.sidebar.island.blur",
        COMPONENTS,
    ) as u32;
    let traffic_x = integer(
        components,
        "components.window.sidebar.island.traffic-x",
        COMPONENTS,
    ) as u32;
    let traffic_y = integer(
        components,
        "components.window.sidebar.island.traffic-y",
        COMPONENTS,
    ) as u32;
    let item_height = integer(
        components,
        "components.window.sidebar.item.height",
        COMPONENTS,
    ) as u32;
    let item_radius = metric(
        tokens,
        components,
        "components.window.sidebar.item.radius",
        COMPONENTS,
    ) as u32;
    let item_padding_x = integer(
        components,
        "components.window.sidebar.item.padding-x",
        COMPONENTS,
    ) as u32;
    let item_gap = integer(components, "components.window.sidebar.item.gap", COMPONENTS) as u32;
    let item_text_size = integer(
        components,
        "components.window.sidebar.item.text-size",
        COMPONENTS,
    ) as u32;
    let count_size = integer(
        components,
        "components.window.sidebar.item.count-size",
        COMPONENTS,
    ) as u32;
    let selected_weight = integer(
        components,
        "components.window.sidebar.item.selected-weight",
        COMPONENTS,
    ) as u16;
    let item_icon = integer(
        components,
        "components.window.sidebar.item.icon",
        COMPONENTS,
    ) as u32;
    let app_icon = integer(
        components,
        "components.window.sidebar.item.app-icon",
        COMPONENTS,
    ) as u32;
    let section_height = integer(
        components,
        "components.window.sidebar.section.height",
        COMPONENTS,
    ) as u32;
    let section_margin_top = integer(
        components,
        "components.window.sidebar.section.margin-top",
        COMPONENTS,
    ) as u32;
    let section_padding_x = integer(
        components,
        "components.window.sidebar.section.padding-x",
        COMPONENTS,
    ) as u32;
    let section_size = integer(
        components,
        "components.window.sidebar.section.size",
        COMPONENTS,
    ) as u32;
    let section_weight = integer(
        components,
        "components.window.sidebar.section.weight",
        COMPONENTS,
    ) as u16;
    let chevron = integer(
        components,
        "components.window.sidebar.section.chevron",
        COMPONENTS,
    ) as u32;
    let chevron_stroke = number(
        components,
        "components.window.sidebar.section.chevron-stroke",
        COMPONENTS,
    );
    let default_island = string(
        components,
        "components.window.sidebar.default-style",
        COMPONENTS,
    ) == "island";

    const_doc(out, "SIDEBAR", "The Tengen sidebar metrics.");
    out.push_str(&format!(
        ": SidebarMetrics = SidebarMetrics {{\n\
         \x20   default_island: {default_island},\n\
         \x20   width: {width},\n\
         \x20   inset: {inset},\n\
         \x20   padding_x: {padding_x},\n\
         \x20   radius: {radius},\n\
         \x20   rim_width: {},\n\
         \x20   inner_width: {},\n\
         \x20   blur: {blur},\n\
         \x20   traffic_x: {traffic_x},\n\
         \x20   traffic_y: {traffic_y},\n\
         \x20   item_height: {item_height},\n\
         \x20   item_radius: {item_radius},\n\
         \x20   item_padding_x: {item_padding_x},\n\
         \x20   item_gap: {item_gap},\n\
         \x20   item_text_size: {item_text_size},\n\
         \x20   selected_weight: {selected_weight},\n\
         \x20   count_size: {count_size},\n\
         \x20   item_icon: {item_icon},\n\
         \x20   app_icon: {app_icon},\n\
         \x20   section_height: {section_height},\n\
         \x20   section_margin_top: {section_margin_top},\n\
         \x20   section_padding_x: {section_padding_x},\n\
         \x20   section_size: {section_size},\n\
         \x20   section_weight: {section_weight},\n\
         \x20   chevron: {chevron},\n\
         \x20   chevron_stroke: {},\n\
         }};\n\n",
        float(rim_width),
        float(inner_width),
        float(chevron_stroke)
    ));

    let toolgroup_height = metric(
        tokens,
        components,
        "components.window.toolgroup.height",
        COMPONENTS,
    ) as u32;
    let toolgroup_padding_x = integer(
        components,
        "components.window.toolgroup.padding-x",
        COMPONENTS,
    ) as u32;
    let button_w = integer(
        components,
        "components.window.toolgroup.button-w",
        COMPONENTS,
    ) as u32;
    let button_h = integer(
        components,
        "components.window.toolgroup.button-h",
        COMPONENTS,
    ) as u32;
    let separator_w = integer(
        components,
        "components.window.toolgroup.separator-w",
        COMPONENTS,
    ) as u32;
    let separator_h = integer(
        components,
        "components.window.toolgroup.separator-h",
        COMPONENTS,
    ) as u32;
    let flat_button_w = integer(
        components,
        "components.window.toolgroup.flat-button-w",
        COMPONENTS,
    ) as u32;
    let flat_button_h = integer(
        components,
        "components.window.toolgroup.flat-button-h",
        COMPONENTS,
    ) as u32;
    let flat_radius = metric(
        tokens,
        components,
        "components.window.toolgroup.flat-radius",
        COMPONENTS,
    ) as u32;
    let icon = integer(components, "components.window.toolgroup.icon", COMPONENTS) as u32;

    const_doc(out, "TOOLGROUP", "The Tengen toolbar capsule metrics.");
    out.push_str(&format!(
        ": ToolgroupMetrics = ToolgroupMetrics {{\n\
         \x20   height: {toolgroup_height},\n\
         \x20   padding_x: {toolgroup_padding_x},\n\
         \x20   button_w: {button_w},\n\
         \x20   button_h: {button_h},\n\
         \x20   separator_w: {separator_w},\n\
         \x20   separator_h: {separator_h},\n\
         \x20   flat_button_w: {flat_button_w},\n\
         \x20   flat_button_h: {flat_button_h},\n\
         \x20   flat_radius: {flat_radius},\n\
         \x20   icon: {icon},\n\
         }};\n\n"
    ));
}
