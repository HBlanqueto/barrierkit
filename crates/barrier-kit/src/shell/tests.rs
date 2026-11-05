use crate::shell::GtkMenusAddress;

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
