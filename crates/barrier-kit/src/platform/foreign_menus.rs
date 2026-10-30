/// D-Bus address of a GTK global menu (ADR 0045).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GtkMenusAddress {
    pub unique_bus_name: String,
    pub application_id: String,
    pub menubar_path: String,
    pub app_menu_path: String,
    pub application_object_path: String,
    pub window_object_path: String,
}

impl GtkMenusAddress {
    /// Returns a `GtkMenusAddress` only when `unique_bus_name` is non-empty and
    /// at least one of `menubar_path` / `app_menu_path` is non-empty.
    pub(crate) fn new(
        unique_bus_name: String,
        application_id: String,
        menubar_path: String,
        app_menu_path: String,
        application_object_path: String,
        window_object_path: String,
    ) -> Option<Self> {
        if unique_bus_name.is_empty() || (menubar_path.is_empty() && app_menu_path.is_empty()) {
            return None;
        }
        Some(Self {
            unique_bus_name,
            application_id,
            menubar_path,
            app_menu_path,
            application_object_path,
            window_object_path,
        })
    }
}
