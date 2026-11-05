use super::*;

pub(super) fn emit_settings(out: &mut String, tokens: &Value, components: &Value) {
    let window_width = integer(components, "components.settings.window.width", COMPONENTS) as u32;
    let window_height = integer(components, "components.settings.window.height", COMPONENTS) as u32;
    let window_min_height = integer(
        components,
        "components.settings.window.min-height",
        COMPONENTS,
    ) as u32;

    let strength_width =
        integer(components, "components.settings.strength.width", COMPONENTS) as u32;
    let strength_track =
        integer(components, "components.settings.strength.track", COMPONENTS) as u32;
    let strength_tick = integer(components, "components.settings.strength.tick", COMPONENTS) as u32;
    let strength_tick_gap = integer(
        components,
        "components.settings.strength.tick-gap",
        COMPONENTS,
    ) as u32;
    let strength_label_gap = integer(
        components,
        "components.settings.strength.label-gap",
        COMPONENTS,
    ) as u32;
    let check_gap = integer(components, "components.settings.check.gap", COMPONENTS) as u32;
    let check_row_gap = integer(components, "components.settings.check.row-gap", COMPONENTS) as u32;

    let search_height = metric(
        tokens,
        components,
        "components.settings.search.height",
        COMPONENTS,
    ) as u32;
    let search_padding_x = integer(
        components,
        "components.settings.search.padding-x",
        COMPONENTS,
    ) as u32;
    let search_icon = integer(components, "components.settings.search.icon", COMPONENTS) as u32;

    let account_avatar =
        integer(components, "components.settings.account.avatar", COMPONENTS) as u32;
    let account_gap = integer(components, "components.settings.account.gap", COMPONENTS) as u32;
    let account_name_font = string(components, "components.settings.account.name", COMPONENTS);
    let account_detail_font = string(components, "components.settings.account.detail", COMPONENTS);

    let section_icon_bg = color(
        components,
        "components.settings.section-icon-bg",
        COMPONENTS,
    );

    let swatch_size = integer(components, "components.settings.swatch-size", COMPONENTS) as u32;
    let swatch_gap = integer(components, "components.settings.swatch-gap", COMPONENTS) as u32;

    let card_width = integer(components, "components.settings.card.width", COMPONENTS) as u32;
    let card_height = integer(components, "components.settings.card.height", COMPONENTS) as u32;
    let card_radius = metric(
        tokens,
        components,
        "components.settings.card.radius",
        COMPONENTS,
    ) as u32;
    let card_gap = integer(components, "components.settings.card.gap", COMPONENTS) as u32;
    let card_caption_gap = integer(
        components,
        "components.settings.card.caption-gap",
        COMPONENTS,
    ) as u32;
    let card_caption_font = string(components, "components.settings.card.caption", COMPONENTS);

    let preview_window_radius = integer(
        components,
        "components.settings.preview.window-radius",
        COMPONENTS,
    ) as u32;
    let preview_light = integer(components, "components.settings.preview.light", COMPONENTS) as u32;
    let preview_tint_window_x = integer(
        components,
        "components.settings.preview.tint-window-x",
        COMPONENTS,
    ) as u32;
    let preview_tint_window_y = integer(
        components,
        "components.settings.preview.tint-window-y",
        COMPONENTS,
    ) as u32;
    let preview_tint_side_w = integer(
        components,
        "components.settings.preview.tint-side-w",
        COMPONENTS,
    ) as u32;

    let preview_panel_h = integer(
        components,
        "components.settings.preview.panel-h",
        COMPONENTS,
    ) as u32;
    let preview_panel_w = integer(
        components,
        "components.settings.preview.panel-w",
        COMPONENTS,
    ) as u32;
    let preview_menu_x =
        integer(components, "components.settings.preview.menu-x", COMPONENTS) as u32;
    let preview_menu_y =
        integer(components, "components.settings.preview.menu-y", COMPONENTS) as u32;
    let preview_menu_w =
        integer(components, "components.settings.preview.menu-w", COMPONENTS) as u32;
    let preview_menu_h =
        integer(components, "components.settings.preview.menu-h", COMPONENTS) as u32;
    let preview_menu_padding = integer(
        components,
        "components.settings.preview.menu-padding",
        COMPONENTS,
    ) as u32;
    let preview_menu_line_h = integer(
        components,
        "components.settings.preview.menu-line-h",
        COMPONENTS,
    ) as u32;
    let preview_menu_line_radius = integer(
        components,
        "components.settings.preview.menu-line-radius",
        COMPONENTS,
    ) as u32;
    let preview_app_w = integer(components, "components.settings.preview.app-w", COMPONENTS) as u32;
    let preview_app_h = integer(components, "components.settings.preview.app-h", COMPONENTS) as u32;
    let preview_app_inset = integer(
        components,
        "components.settings.preview.app-inset",
        COMPONENTS,
    ) as u32;
    let preview_app_side_w = integer(
        components,
        "components.settings.preview.app-side-w",
        COMPONENTS,
    ) as u32;
    let preview_dots_inset = integer(
        components,
        "components.settings.preview.dots-inset",
        COMPONENTS,
    ) as u32;
    let preview_dot_gap = integer(
        components,
        "components.settings.preview.dot-gap",
        COMPONENTS,
    ) as u32;
    let preview_dot_radius = integer(
        components,
        "components.settings.preview.dot-radius",
        COMPONENTS,
    ) as u32;
    let preview_radius =
        integer(components, "components.settings.preview.radius", COMPONENTS) as u32;
    let preview_wall_angle = integer(
        components,
        "components.settings.preview.wall-angle",
        COMPONENTS,
    ) as u32;
    let preview_wall_light = number(
        components,
        "components.settings.preview.wall-light",
        COMPONENTS,
    );
    let preview_wall_dark = number(
        components,
        "components.settings.preview.wall-dark",
        COMPONENTS,
    );

    let radio_size = integer(components, "components.settings.radio.size", COMPONENTS) as u32;
    let radio_dot = integer(components, "components.settings.radio.dot", COMPONENTS) as u32;
    let radio_gap = integer(components, "components.settings.radio.gap", COMPONENTS) as u32;
    let radio_spacing = integer(components, "components.settings.radio.spacing", COMPONENTS) as u32;

    let popup_height = integer(components, "components.settings.popup.height", COMPONENTS) as u32;
    let popup_chevron = integer(components, "components.settings.popup.chevron", COMPONENTS) as u32;
    let popup_text_gap =
        integer(components, "components.settings.popup.text-gap", COMPONENTS) as u32;

    let sidebar_lights_gap = integer(
        components,
        "components.settings.sidebar.lights-gap",
        COMPONENTS,
    ) as u32;
    let sidebar_width = integer(components, "components.settings.sidebar.width", COMPONENTS) as u32;
    let sidebar_search_gap = integer(
        components,
        "components.settings.sidebar.search-gap",
        COMPONENTS,
    ) as u32;
    let sidebar_padding_top = integer(
        components,
        "components.settings.sidebar.padding-top",
        COMPONENTS,
    ) as u32;
    let search_gap = integer(components, "components.settings.search.gap", COMPONENTS) as u32;
    let search_popover_gap = integer(
        components,
        "components.settings.search.popover-gap",
        COMPONENTS,
    ) as u32;
    let search_popover_max_height = integer(
        components,
        "components.settings.search.popover-max-height",
        COMPONENTS,
    ) as u32;
    let search_result_height = integer(
        components,
        "components.settings.search.result-height",
        COMPONENTS,
    ) as u32;
    let search_header_height = integer(
        components,
        "components.settings.search.header-height",
        COMPONENTS,
    ) as u32;
    let account_height =
        integer(components, "components.settings.account.height", COMPONENTS) as u32;
    let account_padding_x = integer(
        components,
        "components.settings.account.padding-x",
        COMPONENTS,
    ) as u32;
    let row_gap = integer(components, "components.settings.row-gap", COMPONENTS) as u32;
    let dropdown_chevron_glyph = integer(
        components,
        "components.settings.dropdown.chevron-glyph",
        COMPONENTS,
    ) as u32;
    let empty_padding_y = integer(
        components,
        "components.settings.empty.padding-y",
        COMPONENTS,
    ) as u32;
    let empty_icon = integer(components, "components.settings.empty.icon", COMPONENTS) as u32;
    let empty_gap = integer(components, "components.settings.empty.gap", COMPONENTS) as u32;

    let toolbar_gap = integer(components, "components.settings.toolbar.gap", COMPONENTS) as u32;
    let toolbar_height =
        integer(components, "components.settings.toolbar.height", COMPONENTS) as u32;
    let toolbar_padding_top = metric(
        tokens,
        components,
        "components.settings.toolbar.padding-top",
        COMPONENTS,
    ) as u32;
    let toolbar_padding_x = integer(
        components,
        "components.settings.toolbar.padding-x",
        COMPONENTS,
    ) as u32;

    let swatch_column = integer(components, "components.settings.swatch.column", COMPONENTS) as u32;
    let swatch_name_gap = integer(
        components,
        "components.settings.swatch.name-gap",
        COMPONENTS,
    ) as u32;

    let card_outline = color(components, "components.settings.card.outline", COMPONENTS);

    pub_const(out, "SETTINGS");
    out.push_str(&format!(
        ": SettingsMetrics = SettingsMetrics {{\n\
         \x20   window_width: {window_width},\n\
         \x20   window_height: {window_height},\n\
         \x20   window_min_height: {window_min_height},\n\
         \x20   search_height: {search_height},\n\
         \x20   search_padding_x: {search_padding_x},\n\
         \x20   search_icon: {search_icon},\n\
         \x20   search_gap: {search_gap},\n\
         \x20   search_popover_gap: {search_popover_gap},\n\
         \x20   search_popover_max_height: {search_popover_max_height},\n\
         \x20   search_result_height: {search_result_height},\n\
         \x20   search_header_height: {search_header_height},\n\
         \x20   account_avatar: {account_avatar},\n\
         \x20   account_gap: {account_gap},\n\
         \x20   account_height: {account_height},\n\
         \x20   account_padding_x: {account_padding_x},\n\
         \x20   account_name_font: {},\n\
         \x20   account_detail_font: {},\n\
         \x20   section_icon_bg: {},\n\
         \x20   swatch_size: {swatch_size},\n\
         \x20   swatch_gap: {swatch_gap},\n\
         \x20   card_width: {card_width},\n\
         \x20   card_height: {card_height},\n\
         \x20   card_radius: {card_radius},\n\
         \x20   card_gap: {card_gap},\n\
         \x20   card_caption_gap: {card_caption_gap},\n\
         \x20   card_caption_font: {},\n\
         \x20   preview_window_radius: {preview_window_radius},\n\
         \x20   preview_light: {preview_light},\n\
         \x20   preview_tint_window_x: {preview_tint_window_x},\n\
         \x20   preview_tint_window_y: {preview_tint_window_y},\n\
         \x20   preview_tint_side_w: {preview_tint_side_w},\n\
         \x20   preview_panel_h: {preview_panel_h},\n\
         \x20   preview_panel_w: {preview_panel_w},\n\
         \x20   preview_menu_x: {preview_menu_x},\n\
         \x20   preview_menu_y: {preview_menu_y},\n\
         \x20   preview_menu_w: {preview_menu_w},\n\
         \x20   preview_menu_h: {preview_menu_h},\n\
         \x20   preview_menu_padding: {preview_menu_padding},\n\
         \x20   preview_menu_line_h: {preview_menu_line_h},\n\
         \x20   preview_menu_line_radius: {preview_menu_line_radius},\n\
         \x20   preview_app_w: {preview_app_w},\n\
         \x20   preview_app_h: {preview_app_h},\n\
         \x20   preview_app_inset: {preview_app_inset},\n\
         \x20   preview_app_side_w: {preview_app_side_w},\n\
         \x20   preview_dots_inset: {preview_dots_inset},\n\
         \x20   preview_dot_gap: {preview_dot_gap},\n\
         \x20   preview_dot_radius: {preview_dot_radius},\n\
         \x20   preview_radius: {preview_radius},\n\
         \x20   preview_wall_angle: {preview_wall_angle},\n\
         \x20   preview_wall_light: {},\n\
         \x20   preview_wall_dark: {},\n\
         \x20   radio_size: {radio_size},\n\
         \x20   radio_dot: {radio_dot},\n\
         \x20   radio_gap: {radio_gap},\n\
         \x20   radio_spacing: {radio_spacing},\n\
         \x20   popup_height: {popup_height},\n\
         \x20   popup_chevron: {popup_chevron},\n\
         \x20   popup_text_gap: {popup_text_gap},\n\
         \x20   sidebar_width: {sidebar_width},\n\
         \x20   sidebar_lights_gap: {sidebar_lights_gap},\n\
         \x20   sidebar_search_gap: {sidebar_search_gap},\n\
         \x20   sidebar_padding_top: {sidebar_padding_top},\n\
         \x20   row_gap: {row_gap},\n\
         \x20   strength: SettingsStrength {{ width: {strength_width}, track: {strength_track}, tick: {strength_tick}, tick_gap: {strength_tick_gap}, label_gap: {strength_label_gap} }},\n\
         \x20   check: SettingsCheck {{ gap: {check_gap}, row_gap: {check_row_gap} }},\n\
         \x20   dropdown_chevron_glyph: {dropdown_chevron_glyph},\n\
         \x20   empty_padding_y: {empty_padding_y},\n\
         \x20   empty_icon: {empty_icon},\n\
         \x20   empty_gap: {empty_gap},\n\
         \x20   toolbar_gap: {toolbar_gap},\n\
         \x20   toolbar_height: {toolbar_height},\n\
         \x20   toolbar_padding_top: {toolbar_padding_top},\n\
         \x20   toolbar_padding_x: {toolbar_padding_x},\n\
         \x20   swatch_column: {swatch_column},\n\
         \x20   swatch_name_gap: {swatch_name_gap},\n\
         \x20   card_outline: {},\n\
         }};\n\n",
        font(tokens, account_name_font),
        font(tokens, account_detail_font),
        rgba(section_icon_bg),
        font(tokens, card_caption_font),
        float(preview_wall_light),
        float(preview_wall_dark),
        rgba(card_outline)
    ));

    let swatches = value(
        components,
        "components.settings.accent-swatches",
        COMPONENTS,
    )
    .as_array()
    .expect("accent-swatches must be an array");
    let mut swatch_consts = Vec::with_capacity(swatches.len());
    for swatch in swatches {
        let name = swatch
            .get("name")
            .and_then(|v| v.as_str())
            .expect("accent swatch must have a name");
        let color = swatch
            .get("color")
            .and_then(|v| v.as_str())
            .expect("accent swatch must have a color");
        swatch_consts.push(format!(
            "({name:?}, {})",
            rgba(parse_color(
                color,
                "components.settings.accent-swatches",
                COMPONENTS
            ))
        ));
    }
    pub_const(out, "ACCENT_SWATCHES");
    out.push_str(&format!(
        ": [(&str, Rgba); {}] = [{}];\n\n",
        swatch_consts.len(),
        swatch_consts.join(", ")
    ));
}
