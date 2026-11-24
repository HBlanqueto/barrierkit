#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GtkMenusAddress {
    pub unique_bus_name: String,
    pub application_id: String,
    pub menubar_path: String,
    pub app_menu_path: String,
    pub application_object_path: String,
    pub window_object_path: String,
}
