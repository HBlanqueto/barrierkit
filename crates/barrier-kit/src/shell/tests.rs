use crate::platform::app_menu_export::AppMenuAddress;
use crate::shell::{
    GtkMenusAddress,
    toplevels::{InfoEvent, ToplevelInfo, apply_info_event},
};

#[test]
fn interleaved_info_events_update_their_own_toplevel() {
    let mut infos = vec![ToplevelInfo::default(), ToplevelInfo::default()];
    apply_info_event(&mut infos, 0, InfoEvent::Title("Editor".into()));
    apply_info_event(&mut infos, 1, InfoEvent::Title("Navegador".into()));
    apply_info_event(&mut infos, 0, InfoEvent::AppId("editor".into()));
    apply_info_event(
        &mut infos,
        1,
        InfoEvent::State {
            active: true,
            maximized: true,
            minimized: false,
        },
    );
    apply_info_event(
        &mut infos,
        0,
        InfoEvent::State {
            active: false,
            maximized: false,
            minimized: true,
        },
    );

    assert_eq!(infos[0].title, "Editor");
    assert_eq!(infos[0].app_id, "editor");
    assert!(!infos[0].active);
    assert!(!infos[0].maximized);
    assert!(infos[0].minimized);
    assert_eq!(infos[1].title, "Navegador");
    assert!(infos[1].active);
    assert!(infos[1].maximized);
    assert!(!infos[1].minimized);

    apply_info_event(&mut infos, 0, InfoEvent::Closed);
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].title, "Navegador");
}

#[test]
fn app_menu_event_updates_only_its_toplevel() {
    let mut infos = vec![ToplevelInfo::default(), ToplevelInfo::default()];
    apply_info_event(
        &mut infos,
        0,
        InfoEvent::AppMenu(Some(AppMenuAddress {
            service: ":1.2".to_string(),
            path: "/MenuBar/1".to_string(),
        })),
    );
    assert_eq!(
        infos[0].app_menu.as_ref().map(|a| a.service.as_str()),
        Some(":1.2")
    );
    assert!(infos[1].app_menu.is_none());

    apply_info_event(&mut infos, 0, InfoEvent::AppMenu(None));
    assert!(infos[0].app_menu.is_none());
}

#[test]
fn gtk_menus_event_updates_only_its_toplevel() {
    let mut infos = vec![ToplevelInfo::default(), ToplevelInfo::default()];
    apply_info_event(
        &mut infos,
        0,
        InfoEvent::GtkMenus(GtkMenusAddress::new(
            ":1.3".to_string(),
            "org.gtk.Demo".to_string(),
            "/menubar".to_string(),
            "/appmenu".to_string(),
            "/application".to_string(),
            "/window".to_string(),
        )),
    );
    assert_eq!(
        infos[0]
            .gtk_menus
            .as_ref()
            .map(|a| a.unique_bus_name.as_str()),
        Some(":1.3")
    );
    assert!(infos[1].gtk_menus.is_none());

    apply_info_event(&mut infos, 0, InfoEvent::GtkMenus(None));
    assert!(infos[0].gtk_menus.is_none());
}

#[test]
fn gtk_menus_address_needs_a_bus_name() {
    assert!(
        GtkMenusAddress::new(
            "".to_string(),
            "org.gtk.Demo".to_string(),
            "/menubar".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        )
        .is_none()
    );
}

#[test]
fn gtk_menus_address_accepts_app_menu_only() {
    let address = GtkMenusAddress::new(
        ":1.2".to_string(),
        "org.gtk.Demo".to_string(),
        "".to_string(),
        "/appmenu".to_string(),
        "".to_string(),
        "".to_string(),
    );
    assert!(address.is_some());
    assert_eq!(address.unwrap().app_menu_path, "/appmenu");
}

#[test]
fn gtk_menus_address_accepts_menubar_only() {
    let address = GtkMenusAddress::new(
        ":1.2".to_string(),
        "org.gtk.Demo".to_string(),
        "/menubar".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    );
    assert!(address.is_some());
    assert_eq!(address.unwrap().menubar_path, "/menubar");
}

#[test]
fn gtk_menus_address_rejects_both_paths_empty() {
    assert!(
        GtkMenusAddress::new(
            ":1.2".to_string(),
            "org.gtk.Demo".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        )
        .is_none()
    );
}
