//! App-wide services: actions, preferences, restoration, pasteboard and single instance.

mod action;
mod application;
mod defaults;
mod pasteboard;

pub(crate) use action::MenuActions;
pub use action::{Action, Shortcut, ShortcutKey, use_menu_action};
pub use application::{Instance, Launch, launch};
pub(crate) use defaults::DefaultsRevision;
pub use defaults::{
    load_defaults, load_restoration, save_defaults, save_restoration, use_defaults,
};
pub use pasteboard::{copy, paste};
