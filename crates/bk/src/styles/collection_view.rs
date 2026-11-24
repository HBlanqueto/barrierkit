use ctdesign::{CAPTION, COLLECTION, Font, Rgba};

use crate::style::Palette;
use crate::styles::button::rounded;
use crate::styles::{ControlStyle, Role, Shape, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct CollectionViewStyle {
    /// Tengen: `COLLECTION.tile`.
    pub tile_width: f32,
    /// Tengen: `COLLECTION.icon`.
    pub icon_size: f32,
    pub spacing: f32,
    pub padding: f32,
    pub label_spacing: f32,
    pub label_padding_x: f32,
    /// Tengen: `RoundedRectangle { radius: COLLECTION.radius, style: Circular }`, the tile and
    /// the icon area.
    pub shape: Shape,
    /// Tengen: `RoundedRectangle { radius: COLLECTION.gap, style: Circular }`, the name. A
    /// `Capsule` rounds the tile by half `tile_width`, the icon area by half `icon_size` and
    /// the name by half the `line_height` of `font`.
    pub label_shape: Shape,
    /// Tengen: `Clear`.
    pub icon_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the label color at `COLLECTION.selection`.
    pub selected_icon_background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`.
    pub label_background: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub selected_label_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_label: StateValue<ShapeStyle>,
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
            font: CAPTION,
        }
    }
}
