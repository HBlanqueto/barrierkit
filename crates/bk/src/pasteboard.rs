use std::borrow::Cow;
use std::path::PathBuf;

use crate::drag::DragPayload;

/// A value that copies, pastes and drags as a typed payload.
pub trait Transferable: Sized {
    fn payload(&self) -> DragPayload;
    /// Takes the payload back when it holds this type.
    fn from_payload(payload: &DragPayload) -> Option<Self>;
}

impl Transferable for DragPayload {
    fn payload(&self) -> DragPayload {
        self.clone()
    }

    fn from_payload(payload: &DragPayload) -> Option<Self> {
        Some(payload.clone())
    }
}

impl Transferable for String {
    fn payload(&self) -> DragPayload {
        DragPayload::Text(self.clone())
    }

    fn from_payload(payload: &DragPayload) -> Option<Self> {
        match payload {
            DragPayload::Text(text) | DragPayload::Html { text, .. } => Some(text.clone()),
            _ => None,
        }
    }
}

impl Transferable for Vec<PathBuf> {
    fn payload(&self) -> DragPayload {
        DragPayload::Files(self.clone())
    }

    fn from_payload(payload: &DragPayload) -> Option<Self> {
        match payload {
            DragPayload::Files(paths) => Some(paths.clone()),
            _ => None,
        }
    }
}

/// The system clipboard, or a private board an app names; Linux holds text only on the system one.
#[derive(Clone, Debug, PartialEq)]
pub struct Pasteboard(Option<Cow<'static, str>>);

impl Pasteboard {
    pub fn general() -> Self {
        Self(None)
    }

    pub fn named(name: impl Into<Cow<'static, str>>) -> Self {
        Self(Some(name.into()))
    }

    /// Replaces the board with the value; false when the platform cannot hold its type.
    pub fn copy(&self, value: &impl Transferable) -> bool {
        crate::platform::pasteboard::write(self.0.as_deref(), &value.payload())
    }

    /// The richest item on the board as a `T`; none when the board holds nothing `T` reads.
    pub fn paste<T: Transferable>(&self) -> Option<T> {
        crate::platform::pasteboard::read(self.0.as_deref())
            .and_then(|payload| T::from_payload(&payload))
    }
}
