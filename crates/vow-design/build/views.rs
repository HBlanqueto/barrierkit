use super::*;

pub(super) fn emit_split_view(out: &mut String, components: &Value) {
    let field = |key: &str| {
        float(number(
            components,
            &format!("components.split-view.{key}"),
            COMPONENTS,
        ))
    };
    pub_const(out, "SPLIT_VIEW");
    out.push_str(&format!(
        ": SplitViewMetrics = SplitViewMetrics {{ sidebar_min: {}, sidebar_max: {}, collapse_below: {}, inspector: {}, grip: {}, grip_line: {}, grip_line_inset: {}, key_step: {} }};\n\n",
        field("sidebar-min"),
        field("sidebar-max"),
        field("collapse-below"),
        field("inspector"),
        field("grip"),
        field("grip-line"),
        field("grip-line-inset"),
        field("key-step")
    ));
}

pub(super) fn emit_views(out: &mut String, tokens: &Value, components: &Value) {
    let field = |key: &str| {
        float(metric(
            tokens,
            components,
            &format!("components.{key}"),
            COMPONENTS,
        ))
    };
    pub_const(out, "TABLE");
    out.push_str(&format!(
        ": TableMetrics = TableMetrics {{ row_height: {}, header_height: {}, padding_x: {}, zebra: {} }};\n\n",
        field("table.row-height"),
        field("table.header-height"),
        field("table.padding-x"),
        field("table.zebra")
    ));
    pub_const(out, "COLLECTION");
    out.push_str(&format!(
        ": CollectionMetrics = CollectionMetrics {{ tile: {}, icon: {}, gap: {}, radius: {}, selection: {} }};\n\n",
        field("collection.tile"),
        field("collection.icon"),
        field("collection.gap"),
        field("collection.radius"),
        field("collection.selection")
    ));
    pub_const(out, "TOOLBAR_TABS");
    out.push_str(&format!(
        ": ToolbarTabsMetrics = ToolbarTabsMetrics {{ padding: {}, gap: {}, radius: {}, tab_min_width: {}, tab_padding_top: {}, tab_padding_bottom: {}, tab_radius: {}, icon: {}, icon_gap: {}, font_size: {}, font_weight: {} }};\n\n",
        field("toolbar-tabs.container.padding"),
        field("toolbar-tabs.container.gap"),
        field("toolbar-tabs.container.radius"),
        field("toolbar-tabs.tab.min-width"),
        field("toolbar-tabs.tab.padding-top"),
        field("toolbar-tabs.tab.padding-bottom"),
        field("toolbar-tabs.tab.radius"),
        field("toolbar-tabs.tab.icon"),
        field("toolbar-tabs.tab.gap"),
        field("toolbar-tabs.tab.font.size"),
        integer(components, "components.toolbar-tabs.tab.font.weight", COMPONENTS)
    ));
}
