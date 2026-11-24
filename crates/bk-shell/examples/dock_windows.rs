use bk::{
    prelude::*,
    theme::{SurfaceKind, resolved, surface_background},
};
use bk_shell::{Anchors, KeyboardInteractivity, Layer, LayerSpec, Toplevels, run_layer};
use freya_core::integration::AppComponent;

const HEIGHT: u32 = 48;

fn dock() -> impl IntoElement {
    let toplevels = Toplevels::get();
    let windows = toplevels.list.read().clone();
    HStack::new()
        .expanded()
        .background(color(surface_background(
            &resolved(),
            SurfaceKind::LayerShell,
        )))
        .children(windows.into_iter().map(|window| {
            let toplevels = toplevels.clone();
            Button::new(window.title.clone())
                .toggled(window.active)
                .key(window.id)
                .on_press(move |_| {
                    if window.minimized {
                        toplevels.unset_minimized(window.id);
                    }
                    toplevels.activate(window.id);
                })
        }))
}

fn main() -> Result<(), RunError> {
    run_layer(
        AppComponent::from(dock),
        LayerSpec {
            namespace: "dock-windows".to_string(),
            layer: Layer::Top,
            anchors: Anchors::BOTTOM,
            exclusive_zone: HEIGHT as i32,
            keyboard: KeyboardInteractivity::None,
            size: Some((0, HEIGHT)),
        },
    )
}
