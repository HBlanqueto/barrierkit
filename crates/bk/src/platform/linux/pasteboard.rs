use crate::drag::DragPayload;

// The Wayland clipboard is text only here: images, files and private boards report that they cannot be held.
pub(crate) fn write(name: Option<&str>, payload: &DragPayload) -> bool {
    let (None, DragPayload::Text(text) | DragPayload::Html { text, .. }) = (name, payload) else {
        return false;
    };
    freya_clipboard::prelude::Clipboard::set(text.clone()).is_ok()
}

pub(crate) fn read(name: Option<&str>) -> Option<DragPayload> {
    name.is_none()
        .then(|| freya_clipboard::prelude::Clipboard::get().ok())
        .flatten()
        .map(DragPayload::Text)
}
