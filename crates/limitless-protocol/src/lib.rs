#[cfg(any(feature = "client", feature = "server"))]
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

#[cfg(any(feature = "client", feature = "server"))]
macro_rules! side {
    ($side:ident, $wayland:ident, $generate:ident) => {
        #[cfg(feature = "shell")]
        pub mod __interfaces {
            use wayland_protocols_wlr::foreign_toplevel::v1::$side::__interfaces::*;
            wayland_scanner::generate_interfaces!("protocols/limitless-window-manager-v1.xml");
        }

        #[cfg(feature = "shell")]
        pub mod limitless_window_manager_v1 {
            use super::__interfaces::*;
            use wayland_protocols_wlr::foreign_toplevel::v1::$side::*;
            use $wayland;
            wayland_scanner::$generate!("protocols/limitless-window-manager-v1.xml");
        }

        pub mod focus {
            protocol!(
                limitless_focus_v1,
                $wayland,
                $generate,
                "protocols/limitless-focus-v1.xml"
            );
        }

        #[cfg(feature = "shell")]
        pub mod session {
            pub mod __interfaces {
                wayland_scanner::generate_interfaces!("protocols/limitless-session-v1.xml");
            }

            pub mod limitless_session_v1 {
                use super::__interfaces::*;
                use $wayland;
                wayland_scanner::$generate!("protocols/limitless-session-v1.xml");
            }
        }

        pub mod style {
            protocol!(
                limitless_window_style_v1,
                $wayland,
                $generate,
                "protocols/limitless-window-style-v1.xml"
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
