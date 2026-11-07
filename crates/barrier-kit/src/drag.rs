use std::path::PathBuf;

use ct_design::DRAG;
use freya_core::integration::{SingleThreadErasedEvent, UserEvent};
use freya_core::prelude::*;
use torin::prelude::CursorPoint;

use crate::portal::file_uri;

pub const URI_LIST: &str = "text/uri-list";
pub(crate) const TEXT_MIMES: [&str; 3] = ["text/plain;charset=utf-8", "text/plain", "UTF8_STRING"];

#[derive(Clone, Debug, PartialEq)]
pub enum DragPayload {
    Files(Vec<PathBuf>),
    Text(String),
}

impl DragPayload {
    pub(crate) fn mimes(&self) -> Vec<&'static str> {
        match self {
            DragPayload::Files(_) => std::iter::once(URI_LIST).chain(TEXT_MIMES).collect(),
            DragPayload::Text(_) => TEXT_MIMES.to_vec(),
        }
    }

    pub(crate) fn bytes(&self, mime: &str) -> Vec<u8> {
        match self {
            DragPayload::Files(paths) if mime == URI_LIST => paths
                .iter()
                .map(|path| file_uri(path) + "\r\n")
                .collect::<String>()
                .into_bytes(),
            DragPayload::Files(paths) => paths
                .iter()
                .map(|path| path.to_string_lossy())
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes(),
            DragPayload::Text(text) => text.clone().into_bytes(),
        }
    }
}

pub(crate) struct StartDrag(pub(crate) DragPayload);

#[derive(Clone, Copy)]
pub struct DragSource {
    origin: State<Option<CursorPoint>>,
}

pub fn use_drag_source() -> DragSource {
    DragSource {
        origin: use_state(|| None),
    }
}

impl DragSource {
    pub fn attach(self, element: Rect, payload: impl Fn() -> DragPayload + 'static) -> Rect {
        let mut origin = self.origin;
        let platform = Platform::get();
        element
            .on_pointer_down(move |event: Event<PointerEventData>| {
                if event.is_primary() {
                    origin.set(Some(event.global_location()));
                }
            })
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let Some(start) = *origin.peek() else {
                    return;
                };
                if (event.global_location() - start).length() as f32 > DRAG.threshold {
                    origin.set(None);
                    platform.send(UserEvent::Erased(SingleThreadErasedEvent(Box::new(
                        StartDrag(payload()),
                    ))));
                }
            })
            .on_global_pointer_press(move |_| {
                if origin.peek().is_some() {
                    origin.set(None);
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragged_files_offer_a_crlf_uri_list_and_plain_paths() {
        let payload =
            DragPayload::Files(vec![PathBuf::from("/tmp/a b.txt"), PathBuf::from("/tmp/c")]);
        assert_eq!(payload.mimes()[0], URI_LIST);
        assert_eq!(
            payload.bytes(URI_LIST),
            b"file:///tmp/a%20b.txt\r\nfile:///tmp/c\r\n"
        );
        assert_eq!(payload.bytes("text/plain"), b"/tmp/a b.txt\n/tmp/c");
    }
}
