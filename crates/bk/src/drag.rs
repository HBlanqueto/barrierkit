use std::path::PathBuf;

use ctdesign::DRAG;
use freya_core::prelude::*;
use torin::prelude::CursorPoint;

pub const URI_LIST: &str = "text/uri-list";

#[derive(Clone, Debug, PartialEq)]
pub enum DragPayload {
    Files(Vec<PathBuf>),
    Text(String),
    /// Rich text: the markup and the plain text a reader that does not take it gets.
    Html {
        text: String,
        html: String,
    },
    /// A PNG file's bytes.
    Image(Vec<u8>),
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
        let runtime = crate::platform::KitSender::get();
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
                    runtime.send(StartDrag(payload()));
                }
            })
            .on_global_pointer_up(move |_| {
                if origin.peek().is_some() {
                    origin.set(None);
                }
            })
    }
}
