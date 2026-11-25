use crate::{drag::DragPayload, platform::linux::clipboard::Clipboard};
use freya_core::integration::GlobalContexts;

// The Wayland clipboard holds text and HTML here: images, files and private boards report that they cannot be held.
pub(crate) fn write(name: Option<&str>, payload: &DragPayload) -> bool {
    match (name, payload) {
        (None, DragPayload::Text(text)) => {
            freya_clipboard::prelude::Clipboard::set(text.clone()).is_ok()
        }
        (None, DragPayload::Html { text, html }) => GlobalContexts::try_get()
            .and_then(|contexts| contexts.try_get_context::<Clipboard>())
            .map(|clipboard| clipboard.set_rich(text.clone(), html.clone()))
            .is_some(),
        _ => false,
    }
}

pub(crate) fn read(name: Option<&str>) -> Option<DragPayload> {
    name.is_none()
        .then(|| freya_clipboard::prelude::Clipboard::get().ok())
        .flatten()
        .map(DragPayload::Text)
}
