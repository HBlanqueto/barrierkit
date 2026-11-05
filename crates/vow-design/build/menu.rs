use super::*;

pub(super) fn menu_palette_value(
    tokens: &Value,
    components: &Value,
    theme: &str,
    opaque: bool,
) -> String {
    let color_at = |path: &str| {
        color_ref(
            tokens,
            theme,
            string(components, path, COMPONENTS),
            path,
            COMPONENTS,
        )
    };
    let bg_window = theme_color(tokens, theme, "bg-window");
    let panel_bg = color_at("components.shell.menu.panel.bg");
    let panel_bg = if opaque {
        opaque_over(panel_bg, bg_window)
    } else {
        panel_bg
    };
    let item_radius = metric(
        tokens,
        components,
        "components.shell.menu.item.radius",
        COMPONENTS,
    );
    format!(
        "MenuPalette {{\n\
         \x20   bg: {},\n\
         \x20   padding: {},\n\
         \x20   shadow: {},\n\
         \x20   min_width: {},\n\
         \x20   max_width: {},\n\
         \x20   item_height: {},\n\
         \x20   item_radius: {},\n\
         \x20   item_padding_x: {},\n\
         \x20   font_size: {},\n\
         \x20   shortcut_gap: {},\n\
         \x20   fg: {},\n\
         \x20   shortcut_color: {},\n\
         \x20   hover_bg: {},\n\
         \x20   hover_fg: {},\n\
         \x20   disabled_fg: {},\n\
         \x20   separator_color: {},\n\
         \x20   separator_height: {},\n\
         \x20   separator_margin_y: {},\n\
         \x20   leading_width: {},\n\
         \x20   mark_dot: {},\n\
         \x20   mark_check: {},\n\
         \x20   chevron_size: {},\n\
         \x20   chevron_color: {},\n\
         \x20   bar_hover_bg: {},\n\
         \x20   bar_title_open_bg: {},\n\
         \x20   bar_title_open_radius: {},\n\
         \x20   bar_title_open_height: {},\n\
         \x20   icon_size: {},\n\
         \x20   header_font_size: {},\n\
         \x20   header_weight: {},\n\
         \x20   header_height: {},\n\
         \x20   search_height: {},\n\
         \x20   search_margin_y: {},\n\
         \x20   search_ring_width: {},\n\
         \x20   search_radius: {},\n\
         \x20   search_bg: {},\n\
         \x20   search_ring: {},\n\
         }}",
        rgba(panel_bg),
        float(number(
            components,
            "components.shell.menu.panel.padding",
            COMPONENTS
        )),
        rgba(color_at("components.shell.menu.panel.shadow")),
        float(number(
            components,
            "components.shell.menu.panel.min-width",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.panel.max-width",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.item.height",
            COMPONENTS
        )),
        float(item_radius),
        float(number(
            components,
            "components.shell.menu.item.padding-x",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.item.font-size",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.item.shortcut-gap",
            COMPONENTS
        )),
        rgba(color_at("components.shell.menu.item.fg")),
        rgba(color_at("components.shell.menu.item.shortcut-color")),
        rgba(color_at("components.shell.menu.item.hover-bg")),
        rgba(color_at("components.shell.menu.item.hover-fg")),
        rgba(color_at("components.shell.menu.item.disabled-fg")),
        rgba(color_at("components.shell.menu.separator.color")),
        float(number(
            components,
            "components.shell.menu.separator.height",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.separator.margin-y",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.mark.width",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.mark.dot",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.mark.check",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.chevron.size",
            COMPONENTS
        )),
        rgba(color_at("components.shell.menu.chevron.color")),
        rgba(color_at("components.shell.menubar.hover-bg")),
        rgba(color_at("components.shell.menubar.title-open.bg")),
        float(metric(
            tokens,
            components,
            "components.shell.menubar.title-open.radius",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menubar.title-open.height",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.icon.size",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.header.font-size",
            COMPONENTS
        )),
        number(
            components,
            "components.shell.menu.header.weight",
            COMPONENTS
        ) as i32,
        float(number(
            components,
            "components.shell.menu.header.height",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.search.height",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.search.margin-y",
            COMPONENTS
        )),
        float(number(
            components,
            "components.shell.menu.search.ring-width",
            COMPONENTS
        )),
        float(metric(
            tokens,
            components,
            "components.shell.menu.search.radius",
            COMPONENTS
        )),
        rgba(color_at("components.shell.menu.search.bg")),
        rgba(color_at("components.shell.menu.search.ring")),
    )
}

pub(super) fn emit_menu(out: &mut String, tokens: &Value, components: &Value) {
    for (name, theme) in [
        ("MENU_LIGHT", "light"),
        ("MENU_DARK", "dark"),
        ("MENU_DARK_TINT", "dark-tint"),
    ] {
        pub_const(out, name);
        out.push_str(&format!(
            ": MenuPalette = {};\n\n",
            menu_palette_value(tokens, components, theme, false)
        ));
        pub_const(out, &format!("{name}_OPAQUE"));
        out.push_str(&format!(
            ": MenuPalette = {};\n\n",
            menu_palette_value(tokens, components, theme, true)
        ));
    }
}

pub(super) fn emit_menu_popover(out: &mut String, tokens: &Value, components: &Value) {
    let number_at = |path: &str| number(components, path, COMPONENTS);

    pub_const(out, "MENU_POPOVER");
    out.push_str(&format!(
        ": MenuPopoverMetrics = MenuPopoverMetrics {{\n\
         \x20   item_height: {},\n\
         \x20   item_radius: {},\n\
         \x20   padding: {},\n\
         \x20   min_width: {},\n\
         \x20   item_padding_start: {},\n\
         \x20   item_padding_x: {},\n\
         \x20   check: {},\n\
         \x20   leading_column: {},\n\
         \x20   swatch_grid_size: {},\n\
         \x20   swatch_grid_dot: {},\n\
         \x20   swatch_grid_gap: {},\n\
         \x20   separator_margin: {},\n\
         }};\n\n",
        float(number_at("components.menu-popover.item-height")),
        float(metric(
            tokens,
            components,
            "components.menu-popover.item-radius",
            COMPONENTS
        )),
        float(number_at("components.menu-popover.padding")),
        float(number_at("components.menu-popover.min-width")),
        float(number_at("components.menu-popover.item-padding-start")),
        float(number_at("components.menu-popover.item-padding-x")),
        float(number_at("components.menu-popover.check")),
        float(number_at("components.menu-popover.leading-column")),
        float(number_at("components.menu-popover.swatch-grid-size")),
        float(number_at("components.menu-popover.swatch-grid-dot")),
        float(number_at("components.menu-popover.swatch-grid-gap")),
        float(number_at("components.menu-popover.separator-margin")),
    ));
}
