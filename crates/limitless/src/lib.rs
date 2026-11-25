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
    ($wayland:ident, $generate:ident) => {
        pub mod effects {
            protocol!(
                limitless_effects_v1,
                $wayland,
                $generate,
                "protocols/limitless-effects-v1.xml"
            );
        }

        pub mod focus {
            protocol!(
                limitless_focus_v1,
                $wayland,
                $generate,
                "protocols/limitless-focus-v1.xml"
            );
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
    side!(wayland_client, generate_client_code);
}

#[cfg(feature = "server")]
pub mod server {
    side!(wayland_server, generate_server_code);
}
