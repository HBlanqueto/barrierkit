use wayland_backend::protocol::{AllowNull, ArgumentType, Interface};

const REQUESTS: [(&str, u32); 12] = [
    ("destroy", 1),
    ("set_bar", 1),
    ("set_popover", 2),
    ("set_panel", 3),
    ("set_cards", 4),
    ("set_desktop", 5),
    ("set_glass", 6),
    ("set_popover_shape", 7),
    ("set_plates", 8),
    ("set_knobs", 9),
    ("set_backdrop", 10),
    ("set_scroll_edge", 10),
];

fn assert_version_ten_keeps_every_request_in_place(interface: &Interface) {
    assert_eq!(interface.version, 10);
    let requests: Vec<_> = interface
        .requests
        .iter()
        .map(|request| (request.name, request.since))
        .collect();
    assert_eq!(requests, REQUESTS);

    let signature = |name| {
        interface
            .requests
            .iter()
            .find(|request| request.name == name)
            .unwrap()
            .signature
    };
    assert_eq!(
        signature("set_backdrop"),
        [
            ArgumentType::Object(AllowNull::No),
            ArgumentType::Uint,
            ArgumentType::Uint,
            ArgumentType::Uint,
            ArgumentType::Array,
        ]
    );
    assert_eq!(
        signature("set_scroll_edge"),
        [
            ArgumentType::Object(AllowNull::No),
            ArgumentType::Uint,
            ArgumentType::Uint,
            ArgumentType::Uint,
        ]
    );
}

#[cfg(feature = "client")]
mod client {
    use infinity_protocol::client::style::infinity_window_style_v1::infinity_window_style_v1::{
        BlendingMode, Edge, InfinityWindowStyleV1, Material, ScrollEdgeEffectStyle,
    };
    use wayland_client::Proxy;
    use wayland_client::protocol::wl_surface::WlSurface;

    #[test]
    fn the_client_exposes_set_backdrop_and_set_scroll_edge() {
        let _: fn(&InfinityWindowStyleV1, &WlSurface, BlendingMode, Material, u32, Vec<u8>) =
            InfinityWindowStyleV1::set_backdrop;
        let _: fn(&InfinityWindowStyleV1, &WlSurface, Edge, ScrollEdgeEffectStyle, u32) =
            InfinityWindowStyleV1::set_scroll_edge;
        super::assert_version_ten_keeps_every_request_in_place(InfinityWindowStyleV1::interface());
    }

    #[test]
    fn the_material_has_the_fourteen_semantic_materials_and_the_two_glass_styles() {
        assert_eq!(Material::try_from(13), Ok(Material::UnderPageBackground));
        assert_eq!(Material::try_from(15), Ok(Material::GlassClear));
        assert!(Material::try_from(16).is_err());
        assert_eq!(BlendingMode::WithinWindow as u32, 1);
    }
}

#[cfg(feature = "server")]
mod server {
    use infinity_protocol::server::style::infinity_window_style_v1::infinity_window_style_v1::{
        InfinityWindowStyleV1, Request,
    };
    use wayland_server::Resource;

    #[test]
    fn the_server_decodes_set_backdrop_and_set_scroll_edge() {
        let _ = |request: Request| {
            matches!(
                request,
                Request::SetBackdrop { .. } | Request::SetScrollEdge { .. }
            )
        };
        super::assert_version_ten_keeps_every_request_in_place(InfinityWindowStyleV1::interface());
    }
}
