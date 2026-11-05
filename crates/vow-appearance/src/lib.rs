//! Appearance domain (dark/light, accent) and its resolved colours.

pub mod backdrop;
mod config;
mod resolve;
pub mod traffic;
pub mod watch;
pub mod window;
pub mod xdg;

pub use config::*;
pub use resolve::*;
