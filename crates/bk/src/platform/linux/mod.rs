mod activation;
#[cfg(feature = "a11y")]
pub(super) mod atspi;
mod chrome_surface;
pub(crate) mod clipboard;
mod dialog;
#[cfg(test)]
mod doc_shots;
pub(crate) mod extension;
#[cfg(test)]
pub(super) mod fake_server;
mod focus;
mod ime;
pub(crate) mod instance;
pub(crate) mod keychain;
pub(crate) mod notifications;
pub(crate) mod panels;
pub(crate) mod pasteboard;
mod portal;
mod primary;
pub(crate) mod runtime;
#[cfg(test)]
mod scene_tests;
pub(crate) mod surface;
mod system;
#[cfg(test)]
mod tests;
mod vulkan;

pub use sctk::reexports::client::backend::ObjectId;

pub(crate) const SYSTEM_MOMENTUM: bool = false;
