use ctdesign::{CAPTION, COLLECTION, Font, Rgba};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};
use crate::style::Palette;

#[derive(Clone, Debug)]
pub struct CollectionViewStyle {
    /// Tengen: `COLLECTION.tile`.
    pub tile_width: f32,
    /// Tengen: `COLLECTION.icon`.
    pub icon_size: f32,
    /// Tengen, with `padding`, `label_spacing`, `label_padding_x`: `COLLECTION.gap` between
    /// tiles, around a tile's content and on each side of the name; half of it between the icon
    /// and the name.
    pub spacing: f32,
    /// Tengen, with `spacing`, `label_spacing`, `label_padding_x`: `COLLECTION.gap` between
    /// tiles, around a tile's content and on each side of the name; half of it between the icon
    /// and the name.
    pub padding: f32,
    /// Tengen, with `spacing`, `padding`, `label_padding_x`: `COLLECTION.gap` between tiles,
    /// around a tile's content and on each side of the name; half of it between the icon and
    /// the name.
    pub label_spacing: f32,
    /// Tengen, with `spacing`, `padding`, `label_spacing`: `COLLECTION.gap` between tiles,
    /// around a tile's content and on each side of the name; half of it between the icon and
    /// the name.
    pub label_padding_x: f32,
    /// Tengen, with `label_shape`: `RoundedRectangle { radius: COLLECTION.radius, style:
    /// Circular }`, the tile and the icon area; `RoundedRectangle { radius: COLLECTION.gap,
    /// style: Circular }`, the name. A `Capsule` rounds the tile by half `tile_width`, the icon
    /// area by half `icon_size` and the name by half the `line_height` of `font`.
    pub shape: Shape,
    /// Tengen, with `shape`: `RoundedRectangle { radius: COLLECTION.radius, style: Circular }`,
    /// the tile and the icon area; `RoundedRectangle { radius: COLLECTION.gap, style: Circular
    /// }`, the name. A `Capsule` rounds the tile by half `tile_width`, the icon area by half
    /// `icon_size` and the name by half the `line_height` of `font`.
    pub label_shape: Shape,
    /// Tengen, with `selected_icon_background`: `Clear`; `Color` of the label color at
    /// `COLLECTION.selection`.
    pub icon_background: StateValue<ShapeStyle>,
    /// Tengen, with `icon_background`: `Clear`; `Color` of the label color at
    /// `COLLECTION.selection`.
    pub selected_icon_background: StateValue<ShapeStyle>,
    /// Tengen, with `selected_label_background`: `Clear`; `Accent`.
    pub label_background: StateValue<ShapeStyle>,
    /// Tengen, with `label_background`: `Clear`; `Accent`.
    pub selected_label_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `CAPTION`.
    pub font: Font,
}

impl ControlStyle for CollectionViewStyle {
    fn tengen() -> Self {
        Self {
            tile_width: COLLECTION.tile,
            icon_size: COLLECTION.icon,
            spacing: COLLECTION.gap,
            padding: COLLECTION.gap,
            label_spacing: COLLECTION.gap / 2.0,
            label_padding_x: COLLECTION.gap,
            shape: rounded(COLLECTION.radius),
            label_shape: rounded(COLLECTION.gap),
            icon_background: StateValue::new(ShapeStyle::Clear),
            selected_icon_background: StateValue::new(ShapeStyle::Color(Rgba {
                a: COLLECTION.selection,
                ..Palette::current().label
            })),
            label_background: StateValue::new(ShapeStyle::Clear),
            selected_label_background: StateValue::new(ShapeStyle::Accent),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            effect: system_effect,
            font: CAPTION,
        }
    }
}
