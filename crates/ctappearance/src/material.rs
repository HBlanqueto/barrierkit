#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Titlebar,
    Selection,
    Menu,
    Popover,
    Sidebar,
    HeaderView,
    Sheet,
    WindowBackground,
    HudWindow,
    FullScreenUi,
    ToolTip,
    ContentBackground,
    UnderWindowBackground,
    UnderPageBackground,
}
