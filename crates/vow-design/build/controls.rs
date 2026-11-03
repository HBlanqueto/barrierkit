use super::*;

pub(super) fn emit_slider(out: &mut String, components: &Value) {
    let height = number(components, "components.slider.height", COMPONENTS);
    let track = number(components, "components.slider.track", COMPONENTS);
    let knob = number(components, "components.slider.knob", COMPONENTS);
    const_doc(out, "SLIDER", "The Tengen slider metrics and colours.");
    out.push_str(&format!(
        ": SliderMetrics = SliderMetrics {{
\
         \x20   height: {},
\
         \x20   track: {},
\
         \x20   knob: {},
}};\n\n",
        float(height),
        float(track),
        float(knob)
    ));
}

pub(super) fn emit_checkbox(out: &mut String, tokens: &Value, components: &Value) {
    let size = number(components, "components.checkbox.size", COMPONENTS);
    let radius = metric(tokens, components, "components.checkbox.radius", COMPONENTS);
    let check_stroke = number(components, "components.checkbox.check-stroke", COMPONENTS);
    const_doc(out, "CHECKBOX", "The Tengen checkbox metrics and colours.");
    out.push_str(&format!(
        ": CheckboxMetrics = CheckboxMetrics {{
\
         \x20   size: {},
\
         \x20   radius: {},
\
         \x20   check_stroke: {},
}};\n\n",
        float(size),
        float(radius),
        float(check_stroke)
    ));
}

pub(super) fn emit_switch(out: &mut String, tokens: &Value, components: &Value) {
    let field = |key: &str| {
        float(metric(
            tokens,
            components,
            &format!("components.switch.{key}"),
            COMPONENTS,
        ))
    };
    let knob_color = color(components, "components.switch.knob-color", COMPONENTS);
    const_doc(out, "SWITCH", "The Tengen switch metrics.");
    out.push_str(&format!(
        ": SwitchMetrics = SwitchMetrics {{
\
         \x20   width: {},
\
         \x20   height: {},
\
         \x20   knob: {},
\
         \x20   inset: {},
\
         \x20   travel: {},
\
         \x20   knob_color: {},
\
         \x20   transition_ms: {},
\
         \x20   disabled_opacity: {},
}};\n\n",
        field("width"),
        field("height"),
        field("knob"),
        field("inset"),
        field("travel"),
        rgba(knob_color),
        metric(
            tokens,
            components,
            "components.switch.transition",
            COMPONENTS
        ) as u64,
        field("disabled-opacity")
    ));
}

pub(super) fn emit_segmented(out: &mut String, tokens: &Value, components: &Value) {
    let field = |key: &str| {
        float(metric(
            tokens,
            components,
            &format!("components.segmented.{key}"),
            COMPONENTS,
        ))
    };
    const_doc(out, "SEGMENTED", "The Tengen segmented control metrics.");
    out.push_str(&format!(
        ": SegmentedMetrics = SegmentedMetrics {{
\
         \x20   padding: {},
\
         \x20   gap: {},
\
         \x20   radius: {},
\
         \x20   segment_radius: {},
\
         \x20   segment_height: {},
\
         \x20   segment_padding_x: {},
\
         \x20   font_size: {},
\
         \x20   font_weight: {},
}};\n\n",
        field("padding"),
        field("gap"),
        field("radius"),
        field("segment-radius"),
        field("segment-height"),
        field("segment-padding-x"),
        field("font.size"),
        integer(components, "components.segmented.font.weight", COMPONENTS)
    ));
}

pub(super) fn emit_focus_ring(out: &mut String, components: &Value) {
    const_doc(out, "FOCUS_RING", "The keyboard focus ring metrics.");
    out.push_str(&format!(
        ": FocusRingMetrics = FocusRingMetrics {{ width: {}, offset: {} }};\n\n",
        float(number(
            components,
            "components.focus-ring.width",
            COMPONENTS
        )),
        float(number(
            components,
            "components.focus-ring.offset",
            COMPONENTS
        ))
    ));
}

pub(super) fn emit_button(out: &mut String, tokens: &Value, components: &Value) {
    let field = |key: &str| {
        float(metric(
            tokens,
            components,
            &format!("components.button.{key}"),
            COMPONENTS,
        ))
    };
    const_doc(out, "BUTTON", "The Tengen text button metrics.");
    out.push_str(&format!(
        ": ButtonMetrics = ButtonMetrics {{
\
         \x20   height: {},
\
         \x20   padding_x: {},
\
         \x20   radius: {},
\
         \x20   font_size: {},
\
         \x20   font_weight: {},
\
         \x20   pressed_scale: {},
}};\n\n",
        field("height"),
        field("padding-x"),
        field("radius"),
        field("font.size"),
        integer(components, "components.button.font.weight", COMPONENTS),
        field("pressed-scale")
    ));
}

pub(super) fn emit_text_field(out: &mut String, tokens: &Value, components: &Value) {
    let field = |key: &str| {
        float(metric(
            tokens,
            components,
            &format!("components.text-field.{key}"),
            COMPONENTS,
        ))
    };
    const_doc(out, "TEXT_FIELD", "The Tengen text field metrics.");
    out.push_str(&format!(
        ": TextFieldMetrics = TextFieldMetrics {{ height: {}, padding_x: {}, radius: {} }};\n\n",
        field("height"),
        field("padding-x"),
        field("radius")
    ));
}

pub(super) fn emit_progress(out: &mut String, tokens: &Value, components: &Value) {
    let field = |key: &str| {
        float(metric(
            tokens,
            components,
            &format!("components.progress.{key}"),
            COMPONENTS,
        ))
    };
    const_doc(out, "PROGRESS", "The Tengen progress bar and ring metrics.");
    out.push_str(&format!(
        ": ProgressMetrics = ProgressMetrics {{
\
         \x20   height: {},
\
         \x20   ring_size: {},
\
         \x20   ring_stroke: {},
\
         \x20   indeterminate_segment: {},
\
         \x20   indeterminate_ms: {},
\
         \x20   spinner_size: {},
\
         \x20   spinner_spokes: {},
\
         \x20   spinner_ms: {},
}};\n\n",
        field("height"),
        field("ring.size"),
        field("ring.stroke"),
        field("indeterminate.segment"),
        integer(
            components,
            "components.progress.indeterminate.duration",
            COMPONENTS
        ),
        field("spinner.size"),
        integer(components, "components.progress.spinner.spokes", COMPONENTS),
        integer(
            components,
            "components.progress.spinner.duration",
            COMPONENTS
        )
    ));
}

pub(super) fn emit_tooltip(out: &mut String, tokens: &Value, components: &Value) {
    let field = |key: &str| {
        float(metric(
            tokens,
            components,
            &format!("components.tooltip.{key}"),
            COMPONENTS,
        ))
    };
    const_doc(out, "TOOLTIP", "The Tengen tooltip metrics.");
    out.push_str(&format!(
        ": TooltipMetrics = TooltipMetrics {{ delay_ms: {}, padding_x: {}, padding_y: {}, radius: {}, pointer_gap: {} }};\n\n",
        integer(components, "components.tooltip.delay", COMPONENTS),
        field("padding-x"),
        field("padding-y"),
        field("radius"),
        field("pointer-gap")
    ));
}

pub(super) fn emit_symbol(out: &mut String, components: &Value) {
    const_doc(out, "SYMBOL", "The symbol grid and stroke width.");
    out.push_str(&format!(
        ": SymbolMetrics = SymbolMetrics {{ stroke: {}, grid: {} }};\n\n",
        float(number(components, "components.symbol.stroke", COMPONENTS)),
        float(number(components, "components.symbol.grid", COMPONENTS))
    ));
}

pub(super) fn emit_slider_fat(out: &mut String, components: &Value) {
    let field = |key: &str| {
        float(number(
            components,
            &format!("components.slider.{key}"),
            COMPONENTS,
        ))
    };
    const_doc(
        out,
        "SLIDER_FAT",
        "The thick control-centre slider metrics.",
    );
    out.push_str(&format!(
        ": SliderFatMetrics = SliderFatMetrics {{ height: {}, fill_min: {}, lead_icon_x: {} }};\n\n",
        field("fat.height"),
        field("fat.fill-min"),
        field("fat.lead-icon-x")
    ));
    const_doc(
        out,
        "SLIDER_KEY_STEP",
        "The fraction an arrow key moves a continuous slider.",
    );
    out.push_str(&format!(": f32 = {};\n\n", field("keyboard-step")));
}

pub(super) fn emit_compact(out: &mut String, tokens: &Value, components: &Value) {
    let field = |key: &str| {
        float(metric(
            tokens,
            components,
            &format!("components.{key}"),
            COMPONENTS,
        ))
    };
    const_doc(out, "ICON_BUTTON", "The Tengen icon button metrics.");
    out.push_str(&format!(
        ": IconButtonMetrics = IconButtonMetrics {{ size: {}, radius: {}, icon: {} }};\n\n",
        field("icon-button.size"),
        field("icon-button.radius"),
        field("icon-button.icon")
    ));
    const_doc(out, "BADGE", "The Tengen badge metrics.");
    out.push_str(&format!(
        ": BadgeMetrics = BadgeMetrics {{ height: {}, padding_x: {} }};\n\n",
        field("badge.height"),
        field("badge.padding-x")
    ));
    const_doc(out, "SCROLL_BAR", "The Tengen scroll bar metrics.");
    out.push_str(&format!(
        ": ScrollBarMetrics = ScrollBarMetrics {{ thumb: {}, inset: {}, min_length: {}, opacity: {} }};\n\n",
        field("scroll-bar.thumb"),
        field("scroll-bar.inset"),
        field("scroll-bar.min-length"),
        field("scroll-bar.opacity")
    ));
}

pub(super) fn emit_list_group(out: &mut String, tokens: &Value, components: &Value) {
    let field = |key: &str| {
        float(metric(
            tokens,
            components,
            &format!("components.list-group.{key}"),
            COMPONENTS,
        ))
    };
    let header = value(
        components,
        "components.list-group.section-header.padding",
        COMPONENTS,
    )
    .as_array()
    .expect("list-group header padding")
    .iter()
    .map(|v| float(v.as_f64().expect("padding number")))
    .collect::<Vec<_>>()
    .join(", ");
    const_doc(out, "LIST_GROUP", "The Tengen grouped list (form) metrics.");
    out.push_str(&format!(
        ": ListGroupMetrics = ListGroupMetrics {{
\
         \x20   radius: {},
\
         \x20   row_min_height: {},
\
         \x20   row_padding_x: {},
\
         \x20   separator_inset: {},
\
         \x20   header_padding: [{header}],
\
         \x20   highlight: {},
\
         \x20   label_min_share: {},
\
         \x20   stack_below: {},
}};\n\n",
        field("radius"),
        field("row-min-height"),
        field("row-padding-x"),
        field("separator.inset-left"),
        field("highlight"),
        field("label-min-share"),
        field("stack-below"),
    ));
}
