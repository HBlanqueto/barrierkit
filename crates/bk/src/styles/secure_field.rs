use ctdesign::TEXT_FIELD;

use super::ControlStyle;

/// The field itself draws from the `TextFieldStyle` in context.
#[derive(Clone, Debug)]
pub struct SecureFieldStyle {
    /// Tengen: `(TEXT_FIELD.secure_dot, TEXT_FIELD.secure_pitch)`: the diameter of each dot of a
    /// hidden secret and the distance between their centers.
    pub dot: (f32, f32),
    /// Tengen: `(TEXT_FIELD.secure_inset, TEXT_FIELD.secure_drop)`: the space before the first dot
    /// and how far the dots sit below the center of the field.
    pub dot_offset: (f32, f32),
}

impl ControlStyle for SecureFieldStyle {
    fn tengen() -> Self {
        Self {
            dot: (TEXT_FIELD.secure_dot, TEXT_FIELD.secure_pitch),
            dot_offset: (TEXT_FIELD.secure_inset, TEXT_FIELD.secure_drop),
        }
    }
}
