//! Infinity window-management protocol bindings (ADR 0039).
#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_unsafe,
    unused_variables,
    unused_imports,
    missing_docs,
    clippy::all
)]

#[cfg(feature = "client")]
pub mod client {
    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        use wayland_protocols_wlr::foreign_toplevel::v1::client::__interfaces::*;
        wayland_scanner::generate_interfaces!("../../protocols/infinity-window-manager-v1.xml");
    }

    pub mod infinity_window_manager_v1 {
        use super::__interfaces::*;
        use wayland_backend;
        use wayland_client;
        use wayland_client::protocol::*;
        use wayland_protocols_wlr::foreign_toplevel::v1::client::*;
        wayland_scanner::generate_client_code!("../../protocols/infinity-window-manager-v1.xml");
    }

    pub use infinity_window_manager_v1::infinity_window_manager_v1::{
        InfinityWindowManagerV1, Layout, PairLayout,
    };
}

#[cfg(feature = "server")]
pub mod server {
    pub mod __interfaces {
        use wayland_protocols_wlr::foreign_toplevel::v1::server::__interfaces::*;
        use wayland_server::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("../../protocols/infinity-window-manager-v1.xml");
    }

    pub mod infinity_window_manager_v1 {
        use super::__interfaces::*;
        use wayland_backend;
        use wayland_protocols_wlr::foreign_toplevel::v1::server::*;
        use wayland_server;
        use wayland_server::protocol::*;
        wayland_scanner::generate_server_code!("../../protocols/infinity-window-manager-v1.xml");
    }
}

#[cfg(feature = "client")]
pub use client::infinity_window_manager_v1::infinity_window_manager_v1::{
    InfinityWindowManagerV1, Layout, PairLayout,
};
