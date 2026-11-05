//! Bindings for the Infinity window-management protocol.

#[cfg(feature = "client")]
pub mod client {
    pub mod __interfaces {
        use wayland_protocols_wlr::foreign_toplevel::v1::client::__interfaces::*;
        wayland_scanner::generate_interfaces!("protocols/infinity-window-manager-v1.xml");
    }

    pub mod infinity_window_manager_v1 {
        use super::__interfaces::*;
        use wayland_client;
        use wayland_protocols_wlr::foreign_toplevel::v1::client::*;
        wayland_scanner::generate_client_code!("protocols/infinity-window-manager-v1.xml");
    }

    pub mod focus {
        pub mod __interfaces {
            use wayland_client::protocol::__interfaces::*;
            wayland_scanner::generate_interfaces!("protocols/infinity-focus-v1.xml");
        }

        pub mod infinity_focus_v1 {
            use super::__interfaces::*;
            use wayland_client;
            use wayland_client::protocol::*;
            wayland_scanner::generate_client_code!("protocols/infinity-focus-v1.xml");
        }
    }

    pub mod style {
        pub mod __interfaces {
            use wayland_client::protocol::__interfaces::*;
            wayland_scanner::generate_interfaces!("protocols/infinity-window-style-v1.xml");
        }

        pub mod infinity_window_style_v1 {
            use super::__interfaces::*;
            use wayland_client;
            use wayland_client::protocol::*;
            wayland_scanner::generate_client_code!("protocols/infinity-window-style-v1.xml");
        }
    }
}

#[cfg(feature = "server")]
pub mod server {
    pub mod __interfaces {
        use wayland_protocols_wlr::foreign_toplevel::v1::server::__interfaces::*;
        wayland_scanner::generate_interfaces!("protocols/infinity-window-manager-v1.xml");
    }

    pub mod infinity_window_manager_v1 {
        use super::__interfaces::*;
        use wayland_protocols_wlr::foreign_toplevel::v1::server::*;
        use wayland_server;
        wayland_scanner::generate_server_code!("protocols/infinity-window-manager-v1.xml");
    }

    pub mod focus {
        pub mod __interfaces {
            use wayland_server::protocol::__interfaces::*;
            wayland_scanner::generate_interfaces!("protocols/infinity-focus-v1.xml");
        }

        pub mod infinity_focus_v1 {
            use super::__interfaces::*;
            use wayland_server;
            use wayland_server::protocol::*;
            wayland_scanner::generate_server_code!("protocols/infinity-focus-v1.xml");
        }
    }

    pub mod style {
        pub mod __interfaces {
            use wayland_server::protocol::__interfaces::*;
            wayland_scanner::generate_interfaces!("protocols/infinity-window-style-v1.xml");
        }

        pub mod infinity_window_style_v1 {
            use super::__interfaces::*;
            use wayland_server;
            use wayland_server::protocol::*;
            wayland_scanner::generate_server_code!("protocols/infinity-window-style-v1.xml");
        }
    }
}
