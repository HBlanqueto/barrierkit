macro_rules! protocol {
    ($module:ident, $wayland:ident, $generate:ident, $file:literal) => {
        pub mod __interfaces {
            use $wayland::protocol::__interfaces::*;
            wayland_scanner::generate_interfaces!($file);
        }

        pub mod $module {
            use super::__interfaces::*;
            use $wayland;
            use $wayland::protocol::*;
            wayland_scanner::$generate!($file);
        }
    };
}

macro_rules! side {
    ($side:ident, $wayland:ident, $generate:ident) => {
        pub mod __interfaces {
            use wayland_protocols_wlr::foreign_toplevel::v1::$side::__interfaces::*;
            wayland_scanner::generate_interfaces!("protocols/infinity-window-manager-v1.xml");
        }

        pub mod infinity_window_manager_v1 {
            use super::__interfaces::*;
            use wayland_protocols_wlr::foreign_toplevel::v1::$side::*;
            use $wayland;
            wayland_scanner::$generate!("protocols/infinity-window-manager-v1.xml");
        }

        pub mod focus {
            protocol!(
                infinity_focus_v1,
                $wayland,
                $generate,
                "protocols/infinity-focus-v1.xml"
            );
        }

        pub mod style {
            protocol!(
                infinity_window_style_v1,
                $wayland,
                $generate,
                "protocols/infinity-window-style-v1.xml"
            );
        }
    };
}

#[cfg(feature = "client")]
pub mod client {
    side!(client, wayland_client, generate_client_code);
}

#[cfg(feature = "server")]
pub mod server {
    side!(server, wayland_server, generate_server_code);
}
