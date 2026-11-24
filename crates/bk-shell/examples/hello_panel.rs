use bk::prelude::*;
use bk_shell::{Anchors, KeyboardInteractivity, Layer, LayerSpec, run_layer};
use ctdesign::MENUBAR_HEIGHT;
use freya_core::integration::AppComponent;

fn main() -> Result<(), RunError> {
    let panel = AppComponent::from(|| {
        rect()
            .expanded()
            .center()
            .background(color(Palette::current().window_background))
            .child(Text::new("Hello, panel"))
    });
    run_layer(
        panel,
        LayerSpec {
            namespace: "hello-panel".to_string(),
            layer: Layer::Top,
            anchors: Anchors::TOP,
            exclusive_zone: MENUBAR_HEIGHT,
            keyboard: KeyboardInteractivity::None,
            size: Some((0, MENUBAR_HEIGHT as u32)),
        },
    )
}
