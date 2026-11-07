use ctdesign::{
    BODY, CAPTION, Font, HAIRLINE, HEADLINE, LIST_GROUP, Rgba, SPACE_2, SPACE_3, SPACE_4,
};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};
use crate::style::Palette;

/// `FormStyle` draws the whole `Form`: its sections and their rows. `Section`
/// and `Row` are data and take no style.
///
/// States: the rows of `Row::navigation` and `Row::destructive` take hovered,
/// pressed and focused; the rest only `normal`. The width below which a row puts
/// its control under the label stays `LIST_GROUP.stack_below`: it is layout.
///
/// States: the header of `DisclosureGroup` takes hovered, pressed and focused;
/// `GroupBox` only `normal`. The children open and close at once.
#[derive(Clone, Debug)]
pub struct FormStyle {
    /// Tengen, with `header_padding`: `LIST_GROUP.padding` around the form;

    /// `LIST_GROUP.header_padding` around a section title. Top, right, bottom, left. A footer

    /// takes the bottom, right and left of `header_padding`.
    pub padding: [f32; 4],
    /// Tengen: `SPACE_4` between sections.
    pub section_spacing: f32,
    /// Tengen, with `padding`: `LIST_GROUP.padding` around the form;
    /// `LIST_GROUP.header_padding` around a section title. Top, right, bottom, left. A footer
    /// takes the bottom, right and left of `header_padding`.
    pub header_padding: [f32; 4],
    /// Tengen, with `row_padding`, `row_spacing`: `LIST_GROUP.row_min_height`;
    /// `(LIST_GROUP.row_padding_x, SPACE_2)`; `SPACE_3` between the labels and the accessory.
    pub row_min_height: f32,
    /// Tengen, with `row_min_height`, `row_spacing`: `LIST_GROUP.row_min_height`;
    /// `(LIST_GROUP.row_padding_x, SPACE_2)`; `SPACE_3` between the labels and the accessory.
    pub row_padding: (f32, f32),
    /// Tengen, with `row_min_height`, `row_padding`: `LIST_GROUP.row_min_height`;
    /// `(LIST_GROUP.row_padding_x, SPACE_2)`; `SPACE_3` between the labels and the accessory.
    pub row_spacing: f32,
    /// Tengen: `LIST_GROUP.separator_inset`, the left inset of the line between rows; the line
    /// is a `Divider` and takes `DividerStyle`.
    pub separator_inset: f32,
    /// Tengen: `CAPTION.size`, the chevron of a navigation row.
    pub chevron_size: f32,
    /// Tengen: `RoundedRectangle { radius: LIST_GROUP.radius, style: Circular }`, the card of a
    /// section; a `Capsule` or `Circle` has no size to round by and stays square.
    pub shape: Shape,
    /// Tengen: `Role(ContentBackground)`, the card.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered `Role(ControlHover)`.
    pub row_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the accent at `LIST_GROUP.highlight`, the row of `.highlighted(..)`.
    pub highlight: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub header: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub footer: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub title: StateValue<ShapeStyle>,
    /// Tengen: `Role(Destructive)`.
    pub destructive_title: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub detail: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub value: StateValue<ShapeStyle>,
    /// Tengen: `Role(TertiaryLabel)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(CardBorder), HAIRLINE))`, drawn inside the card.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `HEADLINE`.
    pub header_font: Font,
    /// Tengen: `CAPTION`.
    pub footer_font: Font,
    /// Tengen: `BODY`.
    pub title_font: Font,
    /// Tengen: `CAPTION`.
    pub detail_font: Font,
    /// Tengen: `BODY`.
    pub value_font: Font,
}

impl ControlStyle for FormStyle {
    fn tengen() -> Self {
        Self {
            padding: LIST_GROUP.padding,
            section_spacing: SPACE_4,
            header_padding: LIST_GROUP.header_padding,
            row_min_height: LIST_GROUP.row_min_height,
            row_padding: (LIST_GROUP.row_padding_x, SPACE_2),
            row_spacing: SPACE_3,
            separator_inset: LIST_GROUP.separator_inset,
            chevron_size: CAPTION.size as f32,
            shape: rounded(LIST_GROUP.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            row_background: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlHover)),
            highlight: StateValue::new(ShapeStyle::Color(Rgba {
                a: LIST_GROUP.highlight,
                ..Palette::current().accent
            })),
            header: StateValue::new(ShapeStyle::Role(Role::Label)),
            footer: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            title: StateValue::new(ShapeStyle::Role(Role::Label)),
            destructive_title: StateValue::new(ShapeStyle::Role(Role::Destructive)),
            detail: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            value: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            chevron: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            effect: system_effect,
            header_font: HEADLINE,
            footer_font: CAPTION,
            title_font: BODY,
            detail_font: CAPTION,
            value_font: BODY,
        }
    }
}
