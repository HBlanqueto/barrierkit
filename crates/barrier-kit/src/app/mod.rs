//! App-wide services: typed actions, per-app preferences and single instance.

mod action;
mod application;
mod defaults;

pub use action::{Action, Shortcut, ShortcutKey};
pub use application::{Launch, launch};
pub(crate) use defaults::DefaultsRevision;
pub use defaults::{load_defaults, save_defaults, use_defaults};
