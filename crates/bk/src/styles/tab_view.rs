use ctdesign::{BODY, CAPTION, ControlSize, Font, Rgba, SPACE_2, TAB_VIEW, TOOLBAR_TABS};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Gradient, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};
use crate::style::Palette;

/// Where the tabs are, like `NSTabView.TabType`: a strip of segments on one side of a bezel box,
/// the icon-over-title tabs of a settings window, or a box without tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabType {
    Top,
    Bottom,
    Left,
    Toolbar,
    NoTabsBezelBorder,
    NoTabsLineBorder,
    NoTabsNoBorder,
}

/// States: a tab of either view takes hovered, pressed and focused.
#[derive(Clone, Debug)]
pub struct TabViewStyle {
    /// Tengen: `Top`.
    pub tab_type: TabType,
    /// Tengen, with `spacing`, `content_spacing`: `0` inside the strip; `TOOLBAR_TABS.padding`
    /// in `toolbar`; `TOOLBAR_TABS.gap` between tabs and `SPACE_2` between the strip and the
    /// page there.
    pub padding: f32,
    /// Tengen, with `padding`, `content_spacing`: `0` between tabs; `TOOLBAR_TABS.gap` in
    /// `toolbar`.
    pub spacing: f32,
    /// Tengen, with `padding`, `spacing`: `0`, the page sits in the box; `SPACE_2` in `toolbar`.
    pub content_spacing: f32,
    /// Tengen: `0`; `TOOLBAR_TABS.tab_min_width` in `toolbar`.
    pub tab_min_width: f32,
    /// Tengen: `TAB_VIEW.strip_padding_x` on each side of a title along a horizontal strip;
    /// `TOOLBAR_TABS.tab_padding_x` in `toolbar`.
    pub tab_padding_x: f32,
    /// Tengen: `(0, 0)` with a strip of `strip_height`; `(tab_padding_top, tab_padding_bottom)`
    /// in `toolbar`.
    pub tab_padding: (f32, f32),
    /// Tengen: `TAB_VIEW.strip_padding_y` at each end of a title along a vertical strip.
    pub tab_padding_y: f32,
    /// Tengen: `TOOLBAR_TABS.icon`, only drawn in `toolbar`.
    pub icon_size: f32,
    /// Tengen: `TOOLBAR_TABS.icon_gap`.
    pub icon_spacing: f32,
    /// Tengen: `RoundedRectangle` of `TAB_VIEW.strip_radius`; `TOOLBAR_TABS.radius` in
    /// `toolbar`. `Circular`.
    pub shape: Shape,
    /// Tengen: `RoundedRectangle` of `TAB_VIEW.pill_radius`; `TOOLBAR_TABS.tab_radius` in
    /// `toolbar`. A `Capsule` rounds by half the height of the tab.
    pub tab_shape: Shape,
    /// Tengen: the label color at `TAB_VIEW.strip_opacity` over the window background, opaque like
    /// AppKit's, the strip; `Clear` in `toolbar`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered `Role(ControlBackground)` in `toolbar`.
    pub tab_background: StateValue<ShapeStyle>,
    /// Tengen: a gradient from `TAB_VIEW.pill_highlight` to `TAB_VIEW.pill_fill`, the chosen
    /// tab; `Role(ControlHover)` in `toolbar`.
    pub selected_tab_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`; `Role(SecondaryLabel)` in `toolbar`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`; `Accent` in `toolbar`, where the icon takes the same paint.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `None`, around the strip.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY` at `TAB_VIEW`'s label size; `CAPTION` at `TOOLBAR_TABS.font_size` with
    /// `TOOLBAR_TABS.font_weight` in `toolbar`; one line, so `line_height` only sets the tab's
    /// height there.
    pub font: Font,
    /// Tengen: `TAB_VIEW.strip_height`, the thickness of the strip.
    pub strip_height: f32,
    /// Tengen: `TAB_VIEW.strip_inset`, from the edge of the view to the strip.
    pub strip_inset: f32,
    /// Tengen: `TAB_VIEW.page_gap`, from the strip to the page on a `Top` strip (`Bottom` takes
    /// 2 less, `Left` 1 less).
    pub page_gap: f32,
    /// Tengen: `(TAB_VIEW.pill_inset_x, TAB_VIEW.pill_inset)`: the chosen tab inside its
    /// segment; the ring of `tab_shadows` takes 1 of the vertical inset.
    pub tab_inset: (f32, f32),
    /// Tengen: the 1 thick ring around the chosen tab, black at `TAB_VIEW.pill_ring`.
    pub tab_shadows: Vec<ShadowStyle>,
    /// Tengen: `RoundedRectangle` of `TAB_VIEW.box_radius`, the box of the page.
    pub box_shape: Shape,
    /// Tengen: the label color at `TAB_VIEW.box_fill_opacity`; `Clear` in `toolbar`.
    pub box_background: StateValue<ShapeStyle>,
    /// Tengen: the label color at `TAB_VIEW.box_border_opacity`, 1 inside the box.
    pub box_border: Option<(ShapeStyle, f32)>,
    /// Tengen: the label color at `TAB_VIEW.line_border_opacity`, the 1 line of
    /// `NoTabsLineBorder`, which is square.
    pub line_border: Option<(ShapeStyle, f32)>,
}

fn label_at(opacity: f32) -> ShapeStyle {
    ShapeStyle::Color(Rgba {
        a: opacity,
        ..Palette::current().label
    })
}

fn over_window(opacity: f32) -> ShapeStyle {
    let palette = Palette::current();
    ShapeStyle::Color(mix(
        palette.window_background,
        Rgba {
            a: palette.window_background.a,
            ..palette.label
        },
        opacity,
    ))
}

fn mix(from: Rgba, to: Rgba, amount: f32) -> Rgba {
    let blend = |from: f32, to: f32| from + (to - from) * amount;
    Rgba {
        r: blend(from.r, to.r),
        g: blend(from.g, to.g),
        b: blend(from.b, to.b),
        a: blend(from.a, to.a),
    }
}

fn pill() -> StateValue<ShapeStyle> {
    let (fill, highlight) = (
        TAB_VIEW.pill_fill[super::appearance_index()],
        TAB_VIEW.pill_highlight[super::appearance_index()],
    );
    let depth = TAB_VIEW.pill_highlight_depth;
    let (edge, soft, base) = (
        ShapeStyle::Color(highlight),
        ShapeStyle::Color(mix(fill, highlight, 0.25)),
        ShapeStyle::Color(fill),
    );
    StateValue::new(ShapeStyle::Gradient(Gradient::new(
        0.0,
        vec![
            (edge.clone(), 0.0),
            (soft.clone(), 0.4 * depth),
            (base.clone(), depth),
            (base, 1.0 - depth),
            (soft, 1.0 - 0.4 * depth),
            (edge, 1.0),
        ],
    )))
}

impl TabViewStyle {
    pub fn toolbar() -> Self {
        Self {
            tab_type: TabType::Toolbar,
            padding: TOOLBAR_TABS.padding,
            spacing: TOOLBAR_TABS.gap,
            content_spacing: SPACE_2,
            tab_min_width: TOOLBAR_TABS.tab_min_width,
            tab_padding_x: TOOLBAR_TABS.tab_padding_x,
            tab_padding: (
                TOOLBAR_TABS.tab_padding_top,
                TOOLBAR_TABS.tab_padding_bottom,
            ),
            shape: rounded(TOOLBAR_TABS.radius),
            tab_shape: rounded(TOOLBAR_TABS.tab_radius),
            background: StateValue::new(ShapeStyle::Clear),
            tab_background: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlBackground)),
            selected_tab_background: StateValue::new(ShapeStyle::Role(Role::ControlHover)),
            label: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            selected_label: StateValue::new(ShapeStyle::Accent),
            tab_shadows: Vec::new(),
            box_background: StateValue::new(ShapeStyle::Clear),
            box_border: None,
            line_border: None,
            font: Font {
                size: TOOLBAR_TABS.font_size as u32,
                weight: TOOLBAR_TABS.font_weight as u16,
                ..CAPTION
            },
            ..Self::tengen()
        }
    }

    pub(crate) fn sized(self, size: ControlSize) -> Self {
        if self.tab_type == TabType::Toolbar {
            return self;
        }
        let row = TAB_VIEW.size(size);
        Self {
            strip_height: row.strip_height,
            strip_inset: row.strip_inset,
            page_gap: row.page_gap,
            tab_padding_x: row.strip_padding_x,
            shape: rounded(row.strip_radius),
            ..self
        }
    }
}

impl ControlStyle for TabViewStyle {
    fn tengen() -> Self {
        let mode = super::appearance_index();
        Self {
            tab_type: TabType::Top,
            padding: 0.0,
            spacing: 0.0,
            content_spacing: 0.0,
            tab_min_width: 0.0,
            tab_padding_x: TAB_VIEW.strip_padding_x,
            tab_padding: (0.0, 0.0),
            tab_padding_y: TAB_VIEW.strip_padding_y,
            icon_size: TOOLBAR_TABS.icon,
            icon_spacing: TOOLBAR_TABS.icon_gap,
            shape: rounded(TAB_VIEW.strip_radius),
            tab_shape: rounded(TAB_VIEW.pill_radius),
            background: StateValue::new(over_window(TAB_VIEW.strip_opacity[mode])),
            tab_background: StateValue::new(ShapeStyle::Clear),
            selected_tab_background: pill(),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::Label)),
            border: None,
            effect: system_effect,
            font: BODY,
            strip_height: TAB_VIEW.strip_height,
            strip_inset: TAB_VIEW.strip_inset,
            page_gap: TAB_VIEW.page_gap,
            tab_inset: (TAB_VIEW.pill_inset_x, TAB_VIEW.pill_inset),
            tab_shadows: vec![ShadowStyle {
                color: Rgba {
                    a: TAB_VIEW.pill_ring[mode],
                    ..Rgba::TRANSPARENT
                },
                x: 0.0,
                y: 0.0,
                blur: 0.0,
                spread: 1.0,
                inner: false,
            }],
            box_shape: rounded(TAB_VIEW.box_radius),
            box_background: StateValue::new(label_at(TAB_VIEW.box_fill_opacity)),
            box_border: Some((
                label_at(TAB_VIEW.box_border_opacity[mode]),
                ctdesign::HAIRLINE,
            )),
            line_border: Some((
                label_at(TAB_VIEW.line_border_opacity[mode]),
                ctdesign::HAIRLINE,
            )),
        }
    }
}
