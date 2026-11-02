//! App-wide services: actions, preferences, restoration, undo, pasteboard and single instance.

mod action;
mod application;
mod defaults;
mod pasteboard;
mod undo;

pub use action::{Action, Shortcut, ShortcutKey};
pub use application::{Launch, launch};
pub(crate) use defaults::DefaultsRevision;
pub use defaults::{
    load_defaults, load_restoration, save_defaults, save_restoration, use_defaults,
};
pub use pasteboard::{copy, paste};
pub use undo::UndoStack;
