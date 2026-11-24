use bk::prelude::*;
use bk_shell::{
    Anchors, KeyboardInteractivity, Layer, LayerSpec, StatusHost, TrayItem, run_layer,
    use_status_items,
};
use ctdesign::{MENUBAR_HEIGHT, SYMBOL};
use freya_core::integration::AppComponent;

fn tray() -> impl IntoElement {
    let (items, host) = use_status_items();
    HStack::new().children(items.read().iter().map(|item| status_icon(item, &host)))
}

fn status_icon(item: &TrayItem, host: &StatusHost) -> impl IntoElement {
    let (key, activate, menu) = (item.key.clone(), host.clone(), host.clone());
    let menu_key = key.clone();
    rect()
        .key(&item.key)
        .on_press(move |_| activate.activate(&key))
        .context_menu(item.menu.clone(), move |id: String| {
            menu.click(&menu_key, &id)
        })
        .maybe_child(item.pixmap.as_ref().map(|pixmap| {
            Image::rgba(pixmap.width, pixmap.height, pixmap.rgba.clone()).size(SYMBOL.grid)
        }))
        .maybe_child(item.pixmap.is_none().then(|| Text::new(item.title.clone())))
}

fn main() -> Result<(), RunError> {
    run_layer(
        AppComponent::from(tray),
        LayerSpec {
            namespace: "tray".to_string(),
            layer: Layer::Top,
            anchors: Anchors::TOP,
            exclusive_zone: MENUBAR_HEIGHT,
            keyboard: KeyboardInteractivity::None,
            size: Some((0, MENUBAR_HEIGHT as u32)),
        },
    )
}
