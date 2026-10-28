//! Client-side Infinity protocol bindings (ADR 0039).
//!
//! The shell uses this private protocol to ask the compositor for window
//! geometry: halves, quarters, pairs, fill, center, restore and fullscreen.

pub use infinity_protocol::client::infinity_window_manager_v1::infinity_window_manager_v1::{
    InfinityWindowManagerV1, Layout, PairLayout,
};

/// Re-export of the generated protocol module, matching the old layout.
pub mod protocol {
    pub use infinity_protocol::client::infinity_window_manager_v1::infinity_window_manager_v1;
}
