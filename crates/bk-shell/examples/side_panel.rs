use bk::{
    prelude::*,
    theme::{SurfaceKind, resolved, surface_background},
};
use bk_shell::{
    Anchors, KeyboardInteractivity, Layer, LayerGeometry, LayerSpec, run_layer, set_layer_geometry,
};
use freya_core::integration::AppComponent;

const RIGHT_EDGE: Anchors = Anchors {
    top: true,
    bottom: true,
    left: false,
    right: true,
};
const NARROW: u32 = 280;
const WIDE: u32 = 420;

fn side_panel() -> impl IntoElement {
    let mut width = use_state(|| NARROW);
    VStack::new()
        .expanded()
        .background(color(surface_background(
            &resolved(),
            SurfaceKind::LayerShell,
        )))
        .child(Text::new("Side panel").font(TITLE))
        .child(Button::new("Change width").on_press(move |_| {
            let next = if *width.peek() == NARROW {
                WIDE
            } else {
                NARROW
            };
            width.set(next);
            set_layer_geometry(LayerGeometry {
                anchors: RIGHT_EDGE,
                exclusive_zone: next as i32,
                size: (next, 0),
                keyboard: KeyboardInteractivity::OnDemand,
            });
        }))
}

fn main() -> Result<(), RunError> {
    run_layer(
        AppComponent::from(side_panel),
        LayerSpec {
            namespace: "side-panel".to_string(),
            layer: Layer::Top,
            anchors: RIGHT_EDGE,
            exclusive_zone: NARROW as i32,
            keyboard: KeyboardInteractivity::OnDemand,
            size: Some((NARROW, 0)),
        },
    )
}
