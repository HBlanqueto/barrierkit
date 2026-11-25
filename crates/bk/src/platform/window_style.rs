// Each enum declares the values of its limitless-window-style-v1 or limitless-effects-v1 namesake, which the Linux backend sends.

/// How a window lays out its title bar and toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bar {
    Titlebar = 0,
    UnifiedCompact = 1,
    Unified = 2,
    Utility = 3,
}

/// The outline the system draws around a popup.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PopoverShape {
    Popover = 0,
    Menu = 1,
    Label = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Knob {
    Slider = 0,
    Switch = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WireMaterial {
    Titlebar = 0,
    Selection = 1,
    Menu = 2,
    Popover = 3,
    Sidebar = 4,
    HeaderView = 5,
    Sheet = 6,
    WindowBackground = 7,
    HudWindow = 8,
    FullScreenUi = 9,
    ToolTip = 10,
    ContentBackground = 11,
    UnderWindowBackground = 12,
    UnderPageBackground = 13,
    GlassRegular = 14,
    GlassClear = 15,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edge {
    Top = 0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScrollEdgeEffectStyle {
    Soft = 1,
}
