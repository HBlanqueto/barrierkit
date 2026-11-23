use ctdesign::{CORNER_CONTROLS, ControlSize};

use crate::styles::{
    ButtonStyle, CheckboxStyle, ComboBoxStyle, ControlStyle, CornerStyle, PopUpButtonStyle,
    SegmentedControlStyle, Shape, TextFieldStyle,
};

const SIZES: [ControlSize; 5] = [
    ControlSize::Mini,
    ControlSize::Small,
    ControlSize::Regular,
    ControlSize::Large,
    ControlSize::ExtraLarge,
];

fn continuous_radius(shape: Shape, height: f32) -> f32 {
    match shape {
        Shape::RoundedRectangle {
            radius,
            style: CornerStyle::Continuous,
        } => radius,
        Shape::Capsule => height / 2.0,
        other => panic!("not a continuous corner: {other:?}"),
    }
}

#[test]
fn every_control_has_the_radius_and_curve_macos_gives_it_at_every_size() {
    for (index, size) in SIZES.into_iter().enumerate() {
        let button = ButtonStyle::tengen().sized(size);
        let popup = PopUpButtonStyle::tengen().sized(size);
        let segmented = SegmentedControlStyle::tengen().sized(size);
        let measured = |radii: [f32; 5]| radii[index];
        for (name, radius, expected) in [
            (
                "push",
                continuous_radius(button.shape, button.height),
                measured(CORNER_CONTROLS.push),
            ),
            (
                "popup",
                continuous_radius(popup.shape, popup.height),
                measured(CORNER_CONTROLS.popup),
            ),
            (
                "segmented",
                continuous_radius(segmented.shape, segmented.segment_height),
                measured(CORNER_CONTROLS.segmented_square),
            ),
            (
                "checkbox",
                continuous_radius(CheckboxStyle::tengen().sized(size).box_shape, 0.0),
                measured(CORNER_CONTROLS.checkbox),
            ),
            (
                "text field",
                continuous_radius(TextFieldStyle::tengen().sized(size).shape, 0.0),
                measured(CORNER_CONTROLS.text_field),
            ),
            (
                "combo button",
                continuous_radius(ComboBoxStyle::tengen().sized(size).button_shape, 0.0),
                measured(CORNER_CONTROLS.combo_button),
            ),
        ] {
            assert_eq!(radius, expected, "{name} at {size:?}");
        }
    }
}
