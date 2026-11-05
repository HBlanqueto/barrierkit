//! Appearance domain (dark/light, accent) and its resolved colours.

use std::path::{Path, PathBuf};

use serde::Deserialize;
pub mod backdrop;
pub mod traffic;
pub mod watch;

use serde::Deserializer;
use vow_design::{
    ACCENT_MIN_CONTRAST_ON_WINDOW, BG_CONTROL_DARK, BG_CONTROL_DARK_TINT, BG_CONTROL_LIGHT,
    BG_HOVER_DARK, BG_HOVER_DARK_TINT, BG_HOVER_LIGHT, BG_RAISED_DARK, BG_RAISED_DARK_TINT,
    BG_RAISED_LIGHT, BG_SUNKEN_DARK, BG_SUNKEN_DARK_TINT, BG_SUNKEN_LIGHT, BG_SURFACE_DARK,
    BG_SURFACE_DARK_TINT, BG_SURFACE_LIGHT, BG_WINDOW_DARK, BG_WINDOW_DARK_TINT, BG_WINDOW_LIGHT,
    DARK, DARK_TINT, LAYER_FACTOR_BG_SUNKEN, LAYER_FACTOR_CONTROL, LAYER_FACTOR_LINE,
    LAYER_FACTOR_RAISED, LAYER_FACTOR_SURFACE, LIGHT, MENU_DARK, MENU_DARK_OPAQUE, MENU_DARK_TINT,
    MENU_DARK_TINT_OPAQUE, MENU_LIGHT, MENU_LIGHT_OPAQUE, MENU_SEPARATOR_INK_MIX,
    MUTED_BELOW_SATURATION, MUTED_MAX_SATURATION, MenuPalette, ON_ACCENT_MIN_CONTRAST,
    PANEL_TINT_WALLPAPER, Palette, Rgba, SHELL_DARK, SHELL_DARK_OPAQUE, SHELL_DARK_TINT,
    SHELL_DARK_TINT_OPAQUE, SHELL_LIGHT, SHELL_LIGHT_OPAQUE, SIDEBAR_DARK, SIDEBAR_DARK_OPAQUE,
    SIDEBAR_DARK_TINT, SIDEBAR_DARK_TINT_OPAQUE, SIDEBAR_ISLAND_DARK, SIDEBAR_ISLAND_DARK_OPAQUE,
    SIDEBAR_ISLAND_DARK_TINT, SIDEBAR_ISLAND_DARK_TINT_OPAQUE, SIDEBAR_ISLAND_INNER_DARK,
    SIDEBAR_ISLAND_INNER_DARK_TINT, SIDEBAR_ISLAND_INNER_LIGHT, SIDEBAR_ISLAND_LIGHT,
    SIDEBAR_ISLAND_LIGHT_OPAQUE, SIDEBAR_ISLAND_RIM_DARK, SIDEBAR_ISLAND_RIM_DARK_TINT,
    SIDEBAR_ISLAND_RIM_LIGHT, SIDEBAR_LIGHT, SIDEBAR_LIGHT_OPAQUE, SIDEBAR_PALETTE_DARK,
    SIDEBAR_PALETTE_DARK_TINT, SIDEBAR_PALETTE_LIGHT, ShellPalette, SidebarPalette, TINT_STRENGTH,
    TITLEBAR, TOOLGROUP_BG_DARK, TOOLGROUP_BG_DARK_OPAQUE, TOOLGROUP_BG_DARK_TINT,
    TOOLGROUP_BG_DARK_TINT_OPAQUE, TOOLGROUP_BG_LIGHT, TOOLGROUP_BG_LIGHT_OPAQUE,
    TOOLGROUP_PALETTE_DARK, TOOLGROUP_PALETTE_DARK_TINT, TOOLGROUP_PALETTE_LIGHT,
    TRAFFIC_CLOSE_MINIMIZE_DARKEN, TRAFFIC_ZOOM_LIGHTEN, ToolgroupPalette,
    WALLPAPER_ACCENT_DARKEN_LIGHT, WALLPAPER_ACCENT_LIGHTEN_DARK, WINDOW_TINT_ACCENT,
    WINDOW_TINT_WALLPAPER,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    #[default]
    Auto,
    Dark,
    #[serde(rename = "dark_tint")]
    DarkTint,
    Light,
}

impl ColorScheme {
    pub fn resolved(self) -> Self {
        match self {
            Self::Auto => Self::Light,
            scheme => scheme,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowColorMode {
    #[default]
    Neutral,
    Accent,
    Wallpaper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowTintScope {
    #[default]
    Windows,
    Desktop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TintStrength {
    Subtle,
    #[default]
    Normal,
    Strong,
    Intense,
}

impl TintStrength {
    pub fn factor(self) -> f32 {
        TINT_STRENGTH[self as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TintParts {
    pub sidebars: bool,
    pub window: bool,
    pub groups: bool,
    pub controls: bool,
    pub menus: bool,
}

impl Default for TintParts {
    fn default() -> Self {
        Self {
            sidebars: true,
            window: true,
            groups: true,
            controls: true,
            menus: true,
        }
    }
}

fn deserialize_tint_parts<'de, D>(deserializer: D) -> Result<TintParts, D::Error>
where
    D: Deserializer<'de>,
{
    let list: Vec<String> = Vec::deserialize(deserializer)?;
    let mut parts = TintParts {
        sidebars: false,
        window: false,
        groups: false,
        controls: false,
        menus: false,
    };
    for name in list {
        match name.as_str() {
            "sidebars" => parts.sidebars = true,
            "window" => parts.window = true,
            "groups" => parts.groups = true,
            "controls" => parts.controls = true,
            "menus" => parts.menus = true,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "unknown tint part: {name}"
                )));
            }
        }
    }
    Ok(parts)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarStyle {
    #[default]
    Island,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrafficLightShape {
    #[default]
    Square,
    Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrafficLightStyle {
    #[default]
    Auto,
    #[serde(alias = "macos")]
    Standard,
    Accent,
    Wallpaper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollBars {
    #[default]
    WhenScrolling,
    Always,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub color_scheme: ColorScheme,
    pub accent_color: Option<[f32; 3]>,
    pub contrast: bool,
    pub reduced_motion: bool,
    pub scroll_bars: ScrollBars,
    pub reduce_transparency: bool,
    pub window_color: WindowColorMode,
    pub window_tint_scope: WindowTintScope,
    pub traffic_lights: TrafficLightStyle,
    pub traffic_light_shape: TrafficLightShape,
    pub sidebar_style: SidebarStyle,
    #[serde(default)]
    pub tint_strength_accent: TintStrength,
    #[serde(default)]
    pub tint_strength_wallpaper: TintStrength,
    #[serde(default, deserialize_with = "deserialize_tint_parts")]
    pub tint_parts: TintParts,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default(),
            accent_color: None,
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::default(),
            reduce_transparency: true,
            window_color: WindowColorMode::default(),
            window_tint_scope: WindowTintScope::default(),
            traffic_lights: TrafficLightStyle::default(),
            traffic_light_shape: TrafficLightShape::default(),
            sidebar_style: SidebarStyle::default(),
            tint_strength_accent: TintStrength::default(),
            tint_strength_wallpaper: TintStrength::default(),
            tint_parts: TintParts::default(),
        }
    }
}

impl AppearanceConfig {
    pub fn traffic_light_style(&self) -> TrafficLightStyle {
        match self.traffic_lights {
            TrafficLightStyle::Auto => match self.window_color {
                WindowColorMode::Neutral => TrafficLightStyle::Standard,
                WindowColorMode::Accent => TrafficLightStyle::Accent,
                WindowColorMode::Wallpaper => TrafficLightStyle::Wallpaper,
            },
            style => style,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarSurface {
    pub style: SidebarStyle,
    pub bg: Rgba,
    pub rim: Rgba,
    pub inner: Rgba,
    pub palette: SidebarPalette,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolgroupSurface {
    pub bg: Rgba,
    pub palette: ToolgroupPalette,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surfaces {
    pub window: Rgba,
    pub titlebar_inactive: Rgba,
    pub sunken: Rgba,
    pub surface: Rgba,
    pub raised: Rgba,
    pub control: Rgba,
    pub hover: Rgba,
    pub line: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolved {
    pub scheme: ColorScheme,
    pub accent: Rgba,
    pub window_tint: Option<(Rgba, f32)>,
    pub window_tint_scope: WindowTintScope,
    pub panel_tint: Option<(Rgba, f32)>,
    pub lights: [Rgba; 3],
    pub traffic_style: TrafficLightStyle,
    pub traffic_shape: TrafficLightShape,
    pub reduce_transparency: bool,
    pub reduced_motion: bool,
    pub scroll_bars: ScrollBars,
    pub shell: ShellPalette,
    pub menu: MenuPalette,
    pub sidebar: SidebarSurface,
    pub toolgroup: ToolgroupSurface,
    pub surfaces: Surfaces,
    pub tint_parts: TintParts,
    pub menu_parent: Rgba,
}

impl Resolved {
    pub fn menu_separator(&self, panel_bg: Rgba) -> Rgba {
        if self.window_tint.is_none() || panel_bg == self.menu.bg {
            self.menu.separator_color
        } else {
            panel_bg.mix(self.menu.fg, MENU_SEPARATOR_INK_MIX)
        }
    }
}

pub fn palette(scheme: ColorScheme) -> Palette {
    match scheme.resolved() {
        ColorScheme::DarkTint => DARK_TINT,
        ColorScheme::Dark => DARK,
        _ => LIGHT,
    }
}

fn chrome_surfaces(
    scheme: ColorScheme,
    style: SidebarStyle,
    opaque: bool,
) -> (ShellPalette, MenuPalette, SidebarSurface, ToolgroupSurface) {
    let sidebar_palette = match scheme {
        ColorScheme::Dark => SIDEBAR_PALETTE_DARK,
        ColorScheme::DarkTint => SIDEBAR_PALETTE_DARK_TINT,
        _ => SIDEBAR_PALETTE_LIGHT,
    };
    let toolgroup_palette = match scheme {
        ColorScheme::Dark => TOOLGROUP_PALETTE_DARK,
        ColorScheme::DarkTint => TOOLGROUP_PALETTE_DARK_TINT,
        _ => TOOLGROUP_PALETTE_LIGHT,
    };
    (
        match (scheme, opaque) {
            (ColorScheme::Dark, false) => SHELL_DARK,
            (ColorScheme::Dark, true) => SHELL_DARK_OPAQUE,
            (ColorScheme::DarkTint, false) => SHELL_DARK_TINT,
            (ColorScheme::DarkTint, true) => SHELL_DARK_TINT_OPAQUE,
            (_, false) => SHELL_LIGHT,
            (_, true) => SHELL_LIGHT_OPAQUE,
        },
        match (scheme, opaque) {
            (ColorScheme::Dark, false) => MENU_DARK,
            (ColorScheme::Dark, true) => MENU_DARK_OPAQUE,
            (ColorScheme::DarkTint, false) => MENU_DARK_TINT,
            (ColorScheme::DarkTint, true) => MENU_DARK_TINT_OPAQUE,
            (_, false) => MENU_LIGHT,
            (_, true) => MENU_LIGHT_OPAQUE,
        },
        sidebar_surface(scheme, style, opaque, sidebar_palette),
        toolgroup_surface(scheme, opaque, toolgroup_palette),
    )
}

fn sidebar_surface(
    scheme: ColorScheme,
    style: SidebarStyle,
    opaque: bool,
    palette: SidebarPalette,
) -> SidebarSurface {
    let transparent = Rgba::rgba8(0, 0, 0, 0.0);

    if style == SidebarStyle::Flat {
        let bg = match (scheme, opaque) {
            (ColorScheme::Dark, false) => SIDEBAR_DARK,
            (ColorScheme::Dark, true) => SIDEBAR_DARK_OPAQUE,
            (ColorScheme::DarkTint, false) => SIDEBAR_DARK_TINT,
            (ColorScheme::DarkTint, true) => SIDEBAR_DARK_TINT_OPAQUE,
            (_, false) => SIDEBAR_LIGHT,
            (_, true) => SIDEBAR_LIGHT_OPAQUE,
        };
        return SidebarSurface {
            style,
            bg,
            rim: transparent,
            inner: transparent,
            palette,
        };
    }

    let (bg, rim, inner) = match (scheme, opaque) {
        (ColorScheme::Dark, false) => (
            SIDEBAR_ISLAND_DARK,
            SIDEBAR_ISLAND_RIM_DARK,
            SIDEBAR_ISLAND_INNER_DARK,
        ),
        (ColorScheme::Dark, true) => (
            SIDEBAR_ISLAND_DARK_OPAQUE,
            SIDEBAR_ISLAND_RIM_DARK,
            SIDEBAR_ISLAND_INNER_DARK,
        ),
        (ColorScheme::DarkTint, false) => (
            SIDEBAR_ISLAND_DARK_TINT,
            SIDEBAR_ISLAND_RIM_DARK_TINT,
            SIDEBAR_ISLAND_INNER_DARK_TINT,
        ),
        (ColorScheme::DarkTint, true) => (
            SIDEBAR_ISLAND_DARK_TINT_OPAQUE,
            SIDEBAR_ISLAND_RIM_DARK_TINT,
            SIDEBAR_ISLAND_INNER_DARK_TINT,
        ),
        (_, false) => (
            SIDEBAR_ISLAND_LIGHT,
            SIDEBAR_ISLAND_RIM_LIGHT,
            SIDEBAR_ISLAND_INNER_LIGHT,
        ),
        (_, true) => (
            SIDEBAR_ISLAND_LIGHT_OPAQUE,
            SIDEBAR_ISLAND_RIM_LIGHT,
            SIDEBAR_ISLAND_INNER_LIGHT,
        ),
    };
    SidebarSurface {
        style,
        bg,
        rim,
        inner,
        palette,
    }
}

fn toolgroup_surface(
    scheme: ColorScheme,
    opaque: bool,
    palette: ToolgroupPalette,
) -> ToolgroupSurface {
    let bg = match (scheme, opaque) {
        (ColorScheme::Dark, false) => TOOLGROUP_BG_DARK,
        (ColorScheme::Dark, true) => TOOLGROUP_BG_DARK_OPAQUE,
        (ColorScheme::DarkTint, false) => TOOLGROUP_BG_DARK_TINT,
        (ColorScheme::DarkTint, true) => TOOLGROUP_BG_DARK_TINT_OPAQUE,
        (_, false) => TOOLGROUP_BG_LIGHT,
        (_, true) => TOOLGROUP_BG_LIGHT_OPAQUE,
    };
    ToolgroupSurface { bg, palette }
}

pub fn resolve(appearance: &AppearanceConfig, dominant: Option<Rgba>) -> Resolved {
    let scheme = appearance.color_scheme.resolved();
    let base = palette(scheme);
    let dark = matches!(scheme, ColorScheme::Dark | ColorScheme::DarkTint);
    let index = usize::from(dark);

    let normal_accent = appearance
        .accent_color
        .map(|[r, g, b]| Rgba {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: 1.0,
        })
        .unwrap_or(base.accent);

    let dominant = dominant.unwrap_or(normal_accent);

    let accent = if appearance.window_color == WindowColorMode::Wallpaper {
        wallpaper_accent(dominant, scheme)
    } else {
        normal_accent
    };

    let window_tint = match appearance.window_color {
        WindowColorMode::Neutral => None,
        WindowColorMode::Accent => Some((
            normal_accent,
            WINDOW_TINT_ACCENT[index] * appearance.tint_strength_accent.factor(),
        )),
        WindowColorMode::Wallpaper => Some((
            dominant,
            WINDOW_TINT_WALLPAPER[index] * appearance.tint_strength_wallpaper.factor(),
        )),
    };

    let panel_tint = if appearance.window_color == WindowColorMode::Wallpaper
        && appearance.window_tint_scope == WindowTintScope::Desktop
    {
        Some((
            dominant,
            PANEL_TINT_WALLPAPER[index] * appearance.tint_strength_wallpaper.factor(),
        ))
    } else {
        None
    };

    let lights = match appearance.traffic_light_style() {
        TrafficLightStyle::Auto | TrafficLightStyle::Standard => [
            TITLEBAR.buttons.order[0].color,
            TITLEBAR.buttons.order[1].color,
            TITLEBAR.buttons.order[2].color,
        ],
        TrafficLightStyle::Accent => derived_lights(normal_accent),
        TrafficLightStyle::Wallpaper => derived_lights(dominant),
    };

    let (mut shell, mut menu, mut sidebar, mut toolgroup) = chrome_surfaces(
        scheme,
        appearance.sidebar_style,
        appearance.reduce_transparency,
    );
    let menu_parent = match window_tint {
        Some((tint, factor)) => {
            let bg_window = match scheme {
                ColorScheme::DarkTint => BG_WINDOW_DARK_TINT,
                ColorScheme::Dark => BG_WINDOW_DARK,
                _ => BG_WINDOW_LIGHT,
            };
            bg_window.mix(tint, factor)
        }
        None => match scheme {
            ColorScheme::DarkTint => BG_WINDOW_DARK_TINT,
            ColorScheme::Dark => BG_WINDOW_DARK,
            _ => BG_WINDOW_LIGHT,
        },
    };
    let surfaces = tinted_surfaces(scheme, window_tint, &base, appearance.tint_parts);
    menu.hover_bg = accent;
    menu.hover_fg = base.on_accent;
    menu.search_ring = accent;
    sidebar.palette.selected_bg = accent;
    sidebar.palette.item_icon = accent;
    sidebar.palette.selected_ink = base.on_accent;
    if let Some((tint, factor)) = panel_tint {
        shell.bar = shell.bar.mix(tint, factor);
        shell.tile = shell.tile.mix(tint, factor);
    }
    if let Some((tint, factor)) = window_tint {
        if appearance.tint_parts.sidebars {
            sidebar.bg = sidebar.bg.mix(tint, factor * LAYER_FACTOR_BG_SUNKEN);
        }
        if appearance.tint_parts.controls {
            toolgroup.bg = toolgroup.bg.mix(tint, factor * LAYER_FACTOR_CONTROL);
        }
    }
    shell.tile_active = accent;

    Resolved {
        scheme,
        accent,
        window_tint,
        window_tint_scope: appearance.window_tint_scope,
        panel_tint,
        lights,
        traffic_style: appearance.traffic_light_style(),
        traffic_shape: appearance.traffic_light_shape,
        reduce_transparency: appearance.reduce_transparency,
        reduced_motion: appearance.reduced_motion,
        scroll_bars: appearance.scroll_bars,
        shell,
        menu,
        sidebar,
        toolgroup,
        surfaces,
        tint_parts: appearance.tint_parts,
        menu_parent,
    }
}

fn tinted_surfaces(
    scheme: ColorScheme,
    window_tint: Option<(Rgba, f32)>,
    base: &Palette,
    parts: TintParts,
) -> Surfaces {
    let (window, sunken, surface, raised, control, hover) = match scheme {
        ColorScheme::DarkTint => (
            BG_WINDOW_DARK_TINT,
            BG_SUNKEN_DARK_TINT,
            BG_SURFACE_DARK_TINT,
            BG_RAISED_DARK_TINT,
            BG_CONTROL_DARK_TINT,
            BG_HOVER_DARK_TINT,
        ),
        ColorScheme::Dark => (
            BG_WINDOW_DARK,
            BG_SUNKEN_DARK,
            BG_SURFACE_DARK,
            BG_RAISED_DARK,
            BG_CONTROL_DARK,
            BG_HOVER_DARK,
        ),
        _ => (
            BG_WINDOW_LIGHT,
            BG_SUNKEN_LIGHT,
            BG_SURFACE_LIGHT,
            BG_RAISED_LIGHT,
            BG_CONTROL_LIGHT,
            BG_HOVER_LIGHT,
        ),
    };
    let line = base.line;
    let mix = |color: Rgba, factor: f32, enabled: bool| {
        if !enabled {
            return color;
        }
        window_tint
            .map(|(tint, tint_factor)| color.mix(tint, tint_factor * factor))
            .unwrap_or(color)
    };
    Surfaces {
        window: mix(window, 1.0, parts.window),
        titlebar_inactive: mix(base.titlebar_bg.unfocused, 1.0, parts.window),
        sunken: mix(sunken, LAYER_FACTOR_BG_SUNKEN, parts.window),
        surface: mix(surface, LAYER_FACTOR_SURFACE, parts.groups),
        raised: mix(raised, LAYER_FACTOR_RAISED, parts.menus),
        control: mix(control, LAYER_FACTOR_CONTROL, parts.controls),
        hover: mix(hover, LAYER_FACTOR_CONTROL, parts.controls),
        line: mix(line, LAYER_FACTOR_LINE, parts.groups),
    }
}

fn wallpaper_accent(dominant: Rgba, scheme: ColorScheme) -> Rgba {
    let dark = matches!(scheme, ColorScheme::Dark | ColorScheme::DarkTint);
    let shifted = if dark {
        dominant.lighten(WALLPAPER_ACCENT_LIGHTEN_DARK)
    } else {
        dominant.darken(WALLPAPER_ACCENT_DARKEN_LIGHT)
    };
    let (hue, mut saturation, original_lightness) = shifted.to_hsl();
    let bg_window = match scheme {
        ColorScheme::DarkTint => BG_WINDOW_DARK_TINT,
        ColorScheme::Dark => BG_WINDOW_DARK,
        _ => BG_WINDOW_LIGHT,
    };
    let on_accent = palette(scheme).on_accent;

    let window_dir = if dark { 1.0 } else { -1.0 };
    let reach_window_contrast = |saturation: f32, mut lightness: f32| {
        for _ in 0..100 {
            let accent = Rgba::from_hsl(hue, saturation, lightness, 1.0);
            if accent.contrast_ratio(bg_window) >= ACCENT_MIN_CONTRAST_ON_WINDOW {
                break;
            }
            lightness += window_dir * 0.01;
            if !(0.0..=1.0).contains(&lightness) {
                break;
            }
        }
        lightness
    };
    let mut lightness = reach_window_contrast(saturation, original_lightness);

    let mut accent = Rgba::from_hsl(hue, saturation, lightness, 1.0);
    let mut on_contrast = accent.contrast_ratio(on_accent);
    for _ in 0..100 {
        if on_contrast >= ON_ACCENT_MIN_CONTRAST {
            break;
        }
        let accent_lum = accent.relative_luminance();
        let on_lum = on_accent.relative_luminance();
        let improve_dir = if accent_lum < on_lum { -1.0 } else { 1.0 };
        let next_lightness = lightness + improve_dir * 0.01;
        if !(0.0..=1.0).contains(&next_lightness) {
            break;
        }
        let next = Rgba::from_hsl(hue, saturation, next_lightness, 1.0);
        if next.contrast_ratio(bg_window) < ACCENT_MIN_CONTRAST_ON_WINDOW {
            break;
        }
        let next_on_contrast = next.contrast_ratio(on_accent);
        if next_on_contrast <= on_contrast {
            break;
        }
        lightness = next_lightness;
        accent = next;
        on_contrast = next_on_contrast;
    }

    let delta_lightness = (lightness - original_lightness).abs();
    if saturation < MUTED_BELOW_SATURATION && delta_lightness > 0.10 {
        saturation = (saturation * (1.0 + delta_lightness)).min(MUTED_MAX_SATURATION);
        lightness = reach_window_contrast(saturation, lightness);
    }

    Rgba::from_hsl(hue, saturation, lightness, 1.0)
}

pub fn derived_lights(source: Rgba) -> [Rgba; 3] {
    [
        source.darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN),
        source.darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN),
        source.lighten(TRAFFIC_ZOOM_LIGHTEN),
    ]
}

pub fn config_dir() -> PathBuf {
    std::env::var_os("MALAYSIA_CONFIG_DIR").map_or_else(
        || {
            let base = std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME")
                        .map(PathBuf::from)
                        .map(|home| home.join(".config"))
                })
                .unwrap_or_else(|| PathBuf::from(".config"));
            base.join("malaysia")
        },
        PathBuf::from,
    )
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("MALAYSIA_DATA_DIR").map_or_else(
        || {
            let base = std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME")
                        .map(PathBuf::from)
                        .map(|home| home.join(".local/share"))
                })
                .unwrap_or_else(|| PathBuf::from(".local/share"));
            base.join("malaysia")
        },
        PathBuf::from,
    )
}

pub fn appearance_path() -> PathBuf {
    config_dir().join("appearance.domain")
}

pub fn wallpaper_state_path() -> PathBuf {
    data_dir().join("wallpaper.state")
}

pub fn read_appearance() -> Result<Option<AppearanceConfig>, toml::de::Error> {
    read_appearance_from(&appearance_path())
}

pub fn read_appearance_from(path: &Path) -> Result<Option<AppearanceConfig>, toml::de::Error> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            warn_legacy_traffic_light_name(&text);
            toml::from_str(&text).map(Some)
        }
        Err(_) => Ok(None),
    }
}

pub fn write_appearance(path: &Path, config: &AppearanceConfig) -> std::io::Result<()> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let parsed = text.parse::<toml_edit::DocumentMut>();
    if !text.is_empty() && parsed.is_err() {
        let backup = path.with_extension("domain.bak");
        std::fs::copy(path, &backup)?;
        tracing::warn!(
            path = %path.display(),
            backup = %backup.display(),
            "unreadable appearance domain backed up before overwrite"
        );
    }
    let mut doc = parsed.unwrap_or_else(|_| toml_edit::DocumentMut::new());

    doc["color_scheme"] = toml_edit::value(color_scheme_str(config.color_scheme));
    if let Some(accent) = config.accent_color {
        doc["accent_color"] = toml_edit::value(accent_array(accent));
    } else {
        doc.remove("accent_color");
    }
    doc["contrast"] = toml_edit::value(config.contrast);
    doc["reduced_motion"] = toml_edit::value(config.reduced_motion);
    doc["reduce_transparency"] = toml_edit::value(config.reduce_transparency);
    doc["scroll_bars"] = toml_edit::value(scroll_bars_str(config.scroll_bars));
    doc["window_color"] = toml_edit::value(window_color_str(config.window_color));
    doc["window_tint_scope"] = toml_edit::value(window_tint_scope_str(config.window_tint_scope));
    doc["traffic_lights"] = toml_edit::value(traffic_lights_str(config.traffic_lights));
    doc["traffic_light_shape"] = toml_edit::value(match config.traffic_light_shape {
        TrafficLightShape::Square => "square",
        TrafficLightShape::Round => "round",
    });
    doc["sidebar_style"] = toml_edit::value(sidebar_style_str(config.sidebar_style));
    doc["tint_strength_accent"] = toml_edit::value(tint_strength_str(config.tint_strength_accent));
    doc["tint_strength_wallpaper"] =
        toml_edit::value(tint_strength_str(config.tint_strength_wallpaper));
    doc["tint_parts"] = toml_edit::value(tint_parts_value(config.tint_parts));

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, doc.to_string())?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn write_appearance_default() -> std::io::Result<()> {
    write_appearance(&appearance_path(), &AppearanceConfig::default())
}

fn color_scheme_str(scheme: ColorScheme) -> &'static str {
    match scheme {
        ColorScheme::Auto => "auto",
        ColorScheme::Dark => "dark",
        ColorScheme::DarkTint => "dark_tint",
        ColorScheme::Light => "light",
    }
}

fn window_color_str(mode: WindowColorMode) -> &'static str {
    match mode {
        WindowColorMode::Neutral => "neutral",
        WindowColorMode::Accent => "accent",
        WindowColorMode::Wallpaper => "wallpaper",
    }
}

fn window_tint_scope_str(scope: WindowTintScope) -> &'static str {
    match scope {
        WindowTintScope::Windows => "windows",
        WindowTintScope::Desktop => "desktop",
    }
}

fn scroll_bars_str(scroll_bars: ScrollBars) -> &'static str {
    match scroll_bars {
        ScrollBars::WhenScrolling => "when_scrolling",
        ScrollBars::Always => "always",
    }
}

fn traffic_lights_str(style: TrafficLightStyle) -> &'static str {
    match style {
        TrafficLightStyle::Auto => "auto",
        TrafficLightStyle::Standard => "standard",
        TrafficLightStyle::Accent => "accent",
        TrafficLightStyle::Wallpaper => "wallpaper",
    }
}

fn sidebar_style_str(style: SidebarStyle) -> &'static str {
    match style {
        SidebarStyle::Island => "island",
        SidebarStyle::Flat => "flat",
    }
}

fn tint_strength_str(strength: TintStrength) -> &'static str {
    match strength {
        TintStrength::Subtle => "subtle",
        TintStrength::Normal => "normal",
        TintStrength::Strong => "strong",
        TintStrength::Intense => "intense",
    }
}

fn tint_parts_value(parts: TintParts) -> toml_edit::Value {
    let mut arr = toml_edit::Array::new();
    if parts.sidebars {
        arr.push("sidebars");
    }
    if parts.window {
        arr.push("window");
    }
    if parts.groups {
        arr.push("groups");
    }
    if parts.controls {
        arr.push("controls");
    }
    if parts.menus {
        arr.push("menus");
    }
    toml_edit::Value::Array(arr)
}

fn accent_array(accent: [f32; 3]) -> toml_edit::Array {
    let mut arr = toml_edit::Array::new();
    for channel in accent {
        let rounded = (channel.clamp(0.0, 1.0) * 1000.0).round() / 1000.0;
        arr.push(rounded as f64);
    }
    arr
}

fn warn_legacy_traffic_light_name(text: &str) {
    static WARNED: std::sync::Once = std::sync::Once::new();
    let legacy = text.lines().any(|line| {
        let line = line.split('#').next().unwrap_or(line).trim();
        line.starts_with("traffic_lights")
            && line
                .split_once('=')
                .map(|(_, value)| value.trim().trim_matches('"') == "macos")
                .unwrap_or(false)
    });
    if legacy {
        WARNED.call_once(|| {
            tracing::warn!(
                "traffic_lights = \"macos\" is deprecated; the style is named \"standard\""
            );
        });
    }
}

pub fn load_appearance() -> AppearanceConfig {
    read_appearance().ok().flatten().unwrap_or_default()
}

#[derive(Debug, Deserialize)]
struct WallpaperState {
    dominant: Option<[f32; 3]>,
}

pub fn load_dominant() -> Option<Rgba> {
    read_dominant()
}

pub fn read_dominant() -> Option<Rgba> {
    read_dominant_from(&wallpaper_state_path())
}

pub fn read_dominant_from(path: &Path) -> Option<Rgba> {
    let text = std::fs::read_to_string(path).ok()?;
    let state: WallpaperState = toml::from_str(&text).ok()?;
    let [r, g, b] = state.dominant?;
    Some(Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vow_design::{
        BG_CONTROL_LIGHT, BG_HOVER_LIGHT, BG_RAISED_LIGHT, BG_SUNKEN_LIGHT, BG_SURFACE_LIGHT,
        BG_WINDOW_LIGHT, MENU_LIGHT_OPAQUE, PANEL_TINT_WALLPAPER, SHELL_LIGHT_OPAQUE,
        SIDEBAR_ISLAND_INNER_LIGHT, SIDEBAR_ISLAND_LIGHT_OPAQUE, SIDEBAR_ISLAND_RIM_LIGHT,
        SIDEBAR_PALETTE_LIGHT, TITLEBAR, TOOLGROUP_BG_LIGHT_OPAQUE, TOOLGROUP_PALETTE_LIGHT,
        TRAFFIC_CLOSE_MINIMIZE_DARKEN, TRAFFIC_ZOOM_LIGHTEN, WINDOW_TINT_ACCENT,
        WINDOW_TINT_WALLPAPER,
    };

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn appearance(
        scheme: ColorScheme,
        mode: WindowColorMode,
        scope: WindowTintScope,
        lights: TrafficLightStyle,
        accent: Option<[f32; 3]>,
    ) -> AppearanceConfig {
        AppearanceConfig {
            color_scheme: scheme,
            accent_color: accent,
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::WhenScrolling,
            reduce_transparency: true,
            window_color: mode,
            window_tint_scope: scope,
            traffic_lights: lights,
            traffic_light_shape: TrafficLightShape::Square,
            sidebar_style: SidebarStyle::Island,
            tint_strength_accent: TintStrength::default(),
            tint_strength_wallpaper: TintStrength::default(),
            tint_parts: TintParts::default(),
        }
    }

    fn dominant() -> Rgba {
        Rgba {
            r: 0.8,
            g: 0.2,
            b: 0.1,
            a: 1.0,
        }
    }

    #[test]
    fn auto_resolves_to_light() {
        let resolved = resolve(
            &appearance(
                ColorScheme::Auto,
                WindowColorMode::Neutral,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                None,
            ),
            None,
        );
        assert_eq!(resolved.scheme, ColorScheme::Light);
    }

    #[test]
    fn neutral_desktop_does_not_tint() {
        let resolved = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Neutral,
                WindowTintScope::Desktop,
                TrafficLightStyle::Auto,
                None,
            ),
            Some(dominant()),
        );
        assert_eq!(resolved.window_tint, None);
        assert_eq!(resolved.panel_tint, None);
        assert_eq!(resolved.accent, LIGHT.accent);
    }

    #[test]
    fn neutral_windows_does_not_tint() {
        let resolved = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Neutral,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                None,
            ),
            Some(dominant()),
        );
        assert_eq!(resolved.window_tint, None);
        assert_eq!(resolved.panel_tint, None);
        assert_eq!(resolved.accent, LIGHT.accent);
    }

    #[test]
    fn accent_windows_tints_only_the_window_chrome() {
        let resolved = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Accent,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                Some([0.9, 0.3, 0.1]),
            ),
            Some(dominant()),
        );
        assert!(resolved.window_tint.is_some());
        assert_eq!(resolved.panel_tint, None);
    }

    #[test]
    fn accent_tints_windows_but_not_panels() {
        let accent = [0.9, 0.3, 0.1];
        let resolved = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Accent,
                WindowTintScope::Desktop,
                TrafficLightStyle::Auto,
                Some(accent),
            ),
            Some(dominant()),
        );
        let expected = Rgba {
            r: accent[0],
            g: accent[1],
            b: accent[2],
            a: 1.0,
        };
        assert_eq!(resolved.accent, expected);
        assert_eq!(
            resolved.window_tint,
            Some((expected, WINDOW_TINT_ACCENT[0]))
        );
        assert_eq!(resolved.panel_tint, None);
    }

    #[test]
    fn wallpaper_desktop_tints_panels_and_uses_the_dominant_accent() {
        let resolved = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Wallpaper,
                WindowTintScope::Desktop,
                TrafficLightStyle::Auto,
                Some([0.9, 0.3, 0.1]),
            ),
            Some(dominant()),
        );
        assert_ne!(resolved.accent, dominant());
        assert!(
            resolved.accent.contrast_ratio(resolved.surfaces.window) >= 3.0,
            "wallpaper accent must contrast with the window background"
        );
        assert_eq!(
            resolved.window_tint,
            Some((dominant(), WINDOW_TINT_WALLPAPER[0]))
        );
        assert_eq!(
            resolved.panel_tint,
            Some((dominant(), PANEL_TINT_WALLPAPER[0]))
        );
    }

    #[test]
    fn the_selected_sidebar_row_follows_the_wallpaper_accent() {
        let resolved = resolve(
            &appearance(
                ColorScheme::Dark,
                WindowColorMode::Wallpaper,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                Some([0.9, 0.3, 0.1]),
            ),
            Some(dominant()),
        );
        assert_eq!(resolved.sidebar.palette.selected_bg, resolved.accent);
    }

    #[test]
    fn wallpaper_windows_never_tints_panels() {
        let resolved = resolve(
            &appearance(
                ColorScheme::Dark,
                WindowColorMode::Wallpaper,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                None,
            ),
            Some(dominant()),
        );
        assert_eq!(resolved.panel_tint, None);
        assert_ne!(resolved.accent, dominant());
        assert!(
            resolved.accent.contrast_ratio(resolved.surfaces.window) >= 3.0,
            "wallpaper accent must contrast with the window background"
        );
        assert_eq!(
            resolved.window_tint,
            Some((dominant(), WINDOW_TINT_WALLPAPER[1]))
        );
    }

    #[test]
    fn wallpaper_without_dominant_falls_back_to_the_normal_accent() {
        let resolved = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Wallpaper,
                WindowTintScope::Desktop,
                TrafficLightStyle::Auto,
                Some([0.9, 0.3, 0.1]),
            ),
            None,
        );
        let normal = Rgba {
            r: 0.9,
            g: 0.3,
            b: 0.1,
            a: 1.0,
        };
        assert!(
            resolved.accent.contrast_ratio(resolved.surfaces.window) >= 3.0,
            "fallback accent must contrast with the window background"
        );
        assert_eq!(resolved.panel_tint, Some((normal, PANEL_TINT_WALLPAPER[0])));
        assert_eq!(
            resolved.window_tint,
            Some((normal, WINDOW_TINT_WALLPAPER[0]))
        );
    }

    #[test]
    fn dark_and_dark_tint_use_the_dark_factors() {
        for scheme in [ColorScheme::Dark, ColorScheme::DarkTint] {
            let resolved = resolve(
                &appearance(
                    scheme,
                    WindowColorMode::Wallpaper,
                    WindowTintScope::Desktop,
                    TrafficLightStyle::Auto,
                    None,
                ),
                Some(dominant()),
            );
            assert_eq!(
                resolved.window_tint,
                Some((dominant(), WINDOW_TINT_WALLPAPER[1]))
            );
            assert_eq!(
                resolved.panel_tint,
                Some((dominant(), PANEL_TINT_WALLPAPER[1]))
            );
        }
    }

    #[test]
    fn wallpaper_traffic_lights_follow_the_dominant_even_in_accent_mode() {
        let accent = [0.9, 0.3, 0.1];
        let with_dominant = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Accent,
                WindowTintScope::Windows,
                TrafficLightStyle::Wallpaper,
                Some(accent),
            ),
            Some(dominant()),
        );
        assert_eq!(
            with_dominant.lights[0],
            dominant().darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN)
        );
        assert_eq!(
            with_dominant.lights[2],
            dominant().lighten(TRAFFIC_ZOOM_LIGHTEN)
        );

        let without_dominant = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Accent,
                WindowTintScope::Windows,
                TrafficLightStyle::Wallpaper,
                Some(accent),
            ),
            None,
        );
        let normal = Rgba {
            r: accent[0],
            g: accent[1],
            b: accent[2],
            a: 1.0,
        };
        assert_eq!(
            without_dominant.lights[0],
            normal.darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN)
        );
        assert_eq!(
            without_dominant.lights[2],
            normal.lighten(TRAFFIC_ZOOM_LIGHTEN)
        );
    }

    #[test]
    fn the_legacy_style_name_deserializes_to_standard() {
        let config: AppearanceConfig = toml::from_str("traffic_lights = \"macos\"\n").unwrap();
        assert_eq!(config.traffic_lights, TrafficLightStyle::Standard);
        let config: AppearanceConfig = toml::from_str("traffic_lights = \"standard\"\n").unwrap();
        assert_eq!(config.traffic_lights, TrafficLightStyle::Standard);
    }

    #[test]
    fn reduce_transparency_defaults_to_true() {
        let without_key: AppearanceConfig = toml::from_str(
            r#"
color_scheme = "dark"
accent_color = [0.2, 0.5, 0.8]
contrast = false
reduced_motion = false
window_color = "neutral"
window_tint_scope = "windows"
traffic_lights = "auto"
"#,
        )
        .unwrap();
        assert!(without_key.reduce_transparency);

        let with_false: AppearanceConfig = toml::from_str("reduce_transparency = false\n").unwrap();
        assert!(!with_false.reduce_transparency);

        assert!(AppearanceConfig::default().reduce_transparency);
    }

    #[test]
    fn reduced_transparency_makes_shell_menu_and_sidebar_opaque() {
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            let resolved = resolve(
                &appearance(
                    scheme,
                    WindowColorMode::Neutral,
                    WindowTintScope::Windows,
                    TrafficLightStyle::Auto,
                    None,
                ),
                Some(dominant()),
            );
            assert!(resolved.reduce_transparency);
            for (name, color) in [
                ("shell bar", resolved.shell.bar),
                ("menu panel", resolved.menu.bg),
                ("sidebar", resolved.sidebar.bg),
            ] {
                assert_eq!(color.a, 1.0, "{scheme:?} {name} is still translucent");
            }
        }
    }

    #[test]
    fn transparency_keeps_the_translucent_bar_when_disabled() {
        let mut config = appearance(
            ColorScheme::Light,
            WindowColorMode::Neutral,
            WindowTintScope::Windows,
            TrafficLightStyle::Auto,
            None,
        );
        config.reduce_transparency = false;
        let resolved = resolve(&config, None);
        assert!(!resolved.reduce_transparency);
        assert!(resolved.shell.bar.a < 1.0);
    }

    #[test]
    fn traffic_light_styles_are_resolved() {
        let standard = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Accent,
                WindowTintScope::Windows,
                TrafficLightStyle::Standard,
                None,
            ),
            None,
        );
        assert_eq!(
            standard.lights,
            [
                TITLEBAR.buttons.order[0].color,
                TITLEBAR.buttons.order[1].color,
                TITLEBAR.buttons.order[2].color,
            ]
        );

        let accent = Rgba {
            r: 0.9,
            g: 0.3,
            b: 0.1,
            a: 1.0,
        };
        let derived = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Accent,
                WindowTintScope::Windows,
                TrafficLightStyle::Accent,
                Some([0.9, 0.3, 0.1]),
            ),
            None,
        );
        assert_eq!(
            derived.lights[0],
            accent.darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN)
        );
        assert_eq!(derived.lights[2], accent.lighten(TRAFFIC_ZOOM_LIGHTEN));
    }

    #[test]
    fn sidebar_style_defaults_to_island() {
        let config: AppearanceConfig = toml::from_str("color_scheme = \"light\"\n").unwrap();
        assert_eq!(config.sidebar_style, SidebarStyle::Island);
    }

    #[test]
    fn flat_sidebar_style_deserializes() {
        let config: AppearanceConfig = toml::from_str(
            r#"
color_scheme = "light"
sidebar_style = "flat"
"#,
        )
        .unwrap();
        assert_eq!(config.sidebar_style, SidebarStyle::Flat);
    }

    #[test]
    fn sidebar_style_chooses_the_right_background_per_scheme_and_transparency() {
        let check = |scheme, style, opaque, expected: Rgba| {
            let mut config = appearance(
                scheme,
                WindowColorMode::Neutral,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                None,
            );
            config.sidebar_style = style;
            config.reduce_transparency = opaque;
            let resolved = resolve(&config, None);
            assert_eq!(resolved.sidebar.style, style);
            assert_eq!(resolved.sidebar.bg, expected);
            if opaque {
                assert_eq!(
                    resolved.sidebar.bg.a, 1.0,
                    "{scheme:?} {style:?} must be opaque"
                );
            }
        };

        check(
            ColorScheme::Light,
            SidebarStyle::Island,
            true,
            SIDEBAR_ISLAND_LIGHT_OPAQUE,
        );
        check(
            ColorScheme::Light,
            SidebarStyle::Flat,
            true,
            SIDEBAR_LIGHT_OPAQUE,
        );
        check(
            ColorScheme::Dark,
            SidebarStyle::Island,
            true,
            SIDEBAR_ISLAND_DARK_OPAQUE,
        );
        check(
            ColorScheme::Dark,
            SidebarStyle::Flat,
            true,
            SIDEBAR_DARK_OPAQUE,
        );
        check(
            ColorScheme::DarkTint,
            SidebarStyle::Island,
            true,
            SIDEBAR_ISLAND_DARK_TINT_OPAQUE,
        );
        check(
            ColorScheme::DarkTint,
            SidebarStyle::Flat,
            true,
            SIDEBAR_DARK_TINT_OPAQUE,
        );

        check(
            ColorScheme::Light,
            SidebarStyle::Island,
            false,
            SIDEBAR_ISLAND_LIGHT,
        );
        check(ColorScheme::Light, SidebarStyle::Flat, false, SIDEBAR_LIGHT);
    }

    #[test]
    fn flat_neutral_matches_the_legacy_sidebar_colour() {
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            for opaque in [false, true] {
                let mut config = appearance(
                    scheme,
                    WindowColorMode::Neutral,
                    WindowTintScope::Windows,
                    TrafficLightStyle::Auto,
                    None,
                );
                config.sidebar_style = SidebarStyle::Flat;
                config.reduce_transparency = opaque;
                let resolved = resolve(&config, None);
                let legacy = match (scheme, opaque) {
                    (ColorScheme::Dark, false) => SIDEBAR_DARK,
                    (ColorScheme::Dark, true) => SIDEBAR_DARK_OPAQUE,
                    (ColorScheme::DarkTint, false) => SIDEBAR_DARK_TINT,
                    (ColorScheme::DarkTint, true) => SIDEBAR_DARK_TINT_OPAQUE,
                    (_, false) => SIDEBAR_LIGHT,
                    (_, true) => SIDEBAR_LIGHT_OPAQUE,
                };
                assert_eq!(
                    resolved.sidebar.bg, legacy,
                    "{scheme:?} opaque={opaque} flat sidebar must match legacy value"
                );
                assert_eq!(resolved.sidebar.rim.a, 0.0);
                assert_eq!(resolved.sidebar.inner.a, 0.0);
            }
        }
    }

    #[test]
    fn accent_and_wallpaper_tint_sidebar_and_toolgroup_but_neutral_does_not() {
        let accent = Rgba {
            r: 0.9,
            g: 0.3,
            b: 0.1,
            a: 1.0,
        };
        let mut neutral = appearance(
            ColorScheme::Light,
            WindowColorMode::Neutral,
            WindowTintScope::Windows,
            TrafficLightStyle::Auto,
            Some([accent.r, accent.g, accent.b]),
        );
        neutral.reduce_transparency = true;
        let neutral = resolve(&neutral, None);

        let mut accent_mode = appearance(
            ColorScheme::Light,
            WindowColorMode::Accent,
            WindowTintScope::Windows,
            TrafficLightStyle::Auto,
            Some([accent.r, accent.g, accent.b]),
        );
        accent_mode.reduce_transparency = true;
        let accent_resolved = resolve(&accent_mode, None);

        let expected_factor = WINDOW_TINT_ACCENT[0] * LAYER_FACTOR_BG_SUNKEN;
        let expected_sidebar_bg = neutral.sidebar.bg.mix(accent, expected_factor);
        assert_eq!(accent_resolved.sidebar.bg, expected_sidebar_bg);

        let expected_toolgroup_factor = WINDOW_TINT_ACCENT[0] * LAYER_FACTOR_CONTROL;
        let expected_toolgroup_bg = neutral.toolgroup.bg.mix(accent, expected_toolgroup_factor);
        assert_eq!(accent_resolved.toolgroup.bg, expected_toolgroup_bg);

        let mut wallpaper = accent_mode.clone();
        wallpaper.window_color = WindowColorMode::Wallpaper;
        let wallpaper_resolved = resolve(&wallpaper, Some(dominant()));
        let tint = dominant();
        let wallpaper_factor = WINDOW_TINT_WALLPAPER[0] * LAYER_FACTOR_BG_SUNKEN;
        assert_eq!(
            wallpaper_resolved.sidebar.bg,
            neutral.sidebar.bg.mix(tint, wallpaper_factor)
        );

        assert_eq!(neutral.sidebar.bg, SIDEBAR_ISLAND_LIGHT_OPAQUE);
        assert_eq!(neutral.toolgroup.bg, TOOLGROUP_BG_LIGHT_OPAQUE);
    }

    #[test]
    fn neutral_surfaces_match_the_base_colours() {
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            let resolved = resolve(
                &appearance(
                    scheme,
                    WindowColorMode::Neutral,
                    WindowTintScope::Windows,
                    TrafficLightStyle::Auto,
                    None,
                ),
                None,
            );
            let expected = match scheme {
                ColorScheme::DarkTint => (
                    BG_WINDOW_DARK_TINT,
                    BG_SUNKEN_DARK_TINT,
                    BG_SURFACE_DARK_TINT,
                    BG_RAISED_DARK_TINT,
                    BG_CONTROL_DARK_TINT,
                    BG_HOVER_DARK_TINT,
                ),
                ColorScheme::Dark => (
                    BG_WINDOW_DARK,
                    BG_SUNKEN_DARK,
                    BG_SURFACE_DARK,
                    BG_RAISED_DARK,
                    BG_CONTROL_DARK,
                    BG_HOVER_DARK,
                ),
                _ => (
                    BG_WINDOW_LIGHT,
                    BG_SUNKEN_LIGHT,
                    BG_SURFACE_LIGHT,
                    BG_RAISED_LIGHT,
                    BG_CONTROL_LIGHT,
                    BG_HOVER_LIGHT,
                ),
            };
            assert_eq!(resolved.surfaces.window, expected.0, "{scheme:?} window");
            assert_eq!(resolved.surfaces.sunken, expected.1, "{scheme:?} sunken");
            assert_eq!(resolved.surfaces.surface, expected.2, "{scheme:?} surface");
            assert_eq!(resolved.surfaces.raised, expected.3, "{scheme:?} raised");
            assert_eq!(resolved.surfaces.control, expected.4, "{scheme:?} control");
            assert_eq!(resolved.surfaces.hover, expected.5, "{scheme:?} hover");
            assert_eq!(
                resolved.surfaces.line,
                palette(scheme).line,
                "{scheme:?} line"
            );
        }
    }

    #[test]
    fn menu_separator_follows_the_tinted_panel_but_not_the_neutral_one() {
        let tinted_panel = Rgba::rgb8(0x3b, 0x2e, 0x2e);
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            let neutral = resolve(
                &appearance(
                    scheme,
                    WindowColorMode::Neutral,
                    WindowTintScope::Desktop,
                    TrafficLightStyle::Auto,
                    None,
                ),
                None,
            );
            assert_eq!(
                neutral.menu_separator(tinted_panel),
                neutral.menu.separator_color,
                "{scheme:?}: neutral keeps the theme line"
            );

            let tinted = resolve(
                &appearance(
                    scheme,
                    WindowColorMode::Accent,
                    WindowTintScope::Desktop,
                    TrafficLightStyle::Auto,
                    Some([0.93, 0.27, 0.27]),
                ),
                None,
            );
            assert_eq!(
                tinted.menu_separator(tinted_panel),
                tinted_panel.mix(tinted.menu.fg, MENU_SEPARATOR_INK_MIX),
                "{scheme:?}: the separator follows the tinted panel"
            );
            assert_eq!(
                tinted.menu_separator(tinted.menu.bg),
                tinted.menu.separator_color,
                "{scheme:?}: an untinted panel keeps the theme line"
            );
        }
    }

    #[test]
    fn accent_tints_each_surface_by_its_layer_factor() {
        let purple = Rgba {
            r: 0.557,
            g: 0.357,
            b: 0.847,
            a: 1.0,
        };
        let resolved = resolve(
            &appearance(
                ColorScheme::Dark,
                WindowColorMode::Accent,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                Some([purple.r, purple.g, purple.b]),
            ),
            None,
        );
        let factor = WINDOW_TINT_ACCENT[1];
        assert_eq!(resolved.surfaces.window, BG_WINDOW_DARK.mix(purple, factor));
        assert_eq!(
            resolved.surfaces.sunken,
            BG_SUNKEN_DARK.mix(purple, factor * LAYER_FACTOR_BG_SUNKEN)
        );
        assert_eq!(
            resolved.surfaces.surface,
            BG_SURFACE_DARK.mix(purple, factor * LAYER_FACTOR_SURFACE)
        );
        assert_eq!(
            resolved.surfaces.raised,
            BG_RAISED_DARK.mix(purple, factor * LAYER_FACTOR_RAISED)
        );
        assert_eq!(
            resolved.surfaces.control,
            BG_CONTROL_DARK.mix(purple, factor * LAYER_FACTOR_CONTROL)
        );
        assert_eq!(
            resolved.surfaces.hover,
            BG_HOVER_DARK.mix(purple, factor * LAYER_FACTOR_CONTROL)
        );
        assert_eq!(
            resolved.surfaces.line,
            palette(ColorScheme::Dark)
                .line
                .mix(purple, factor * LAYER_FACTOR_LINE)
        );
    }

    #[test]
    fn sidebar_and_toolgroup_palettes_come_from_the_theme() {
        for (scheme, expected_sidebar, expected_toolgroup) in [
            (
                ColorScheme::Light,
                SIDEBAR_PALETTE_LIGHT,
                TOOLGROUP_PALETTE_LIGHT,
            ),
            (
                ColorScheme::Dark,
                SIDEBAR_PALETTE_DARK,
                TOOLGROUP_PALETTE_DARK,
            ),
            (
                ColorScheme::DarkTint,
                SIDEBAR_PALETTE_DARK_TINT,
                TOOLGROUP_PALETTE_DARK_TINT,
            ),
        ] {
            let resolved = resolve(
                &appearance(
                    scheme,
                    WindowColorMode::Neutral,
                    WindowTintScope::Windows,
                    TrafficLightStyle::Auto,
                    None,
                ),
                None,
            );
            assert_eq!(
                resolved.toolgroup.palette, expected_toolgroup,
                "{scheme:?} toolgroup palette"
            );
            assert_eq!(
                resolved.sidebar.palette.item_icon, resolved.accent,
                "{scheme:?} sidebar item icon"
            );

            let expected_sidebar = SidebarPalette {
                item_icon: resolved.accent,
                selected_bg: resolved.accent,
                selected_ink: palette(scheme).on_accent,
                ..expected_sidebar
            };
            assert_eq!(resolved.sidebar.palette, expected_sidebar);
        }
    }

    #[test]
    fn reduced_motion_is_passed_through() {
        let mut config = appearance(
            ColorScheme::Light,
            WindowColorMode::Neutral,
            WindowTintScope::Windows,
            TrafficLightStyle::Auto,
            None,
        );
        config.reduced_motion = true;
        let resolved = resolve(&config, None);
        assert!(resolved.reduced_motion);
    }

    #[test]
    fn dark_selected_row_mixes_accent_with_sidebar_bg() {
        let check = |scheme, accent: [f32; 3]| {
            let mut config = appearance(
                scheme,
                WindowColorMode::Neutral,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                Some(accent),
            );
            config.reduce_transparency = true;
            let resolved = resolve(&config, None);
            let accent_rgba = Rgba {
                r: accent[0],
                g: accent[1],
                b: accent[2],
                a: 1.0,
            };
            assert_eq!(
                resolved.sidebar.palette.selected_bg, accent_rgba,
                "{scheme:?} selected_bg should be the accent"
            );
            assert_eq!(
                resolved.sidebar.palette.selected_ink,
                palette(scheme).on_accent,
                "{scheme:?} selected_ink should be on-accent"
            );
        };
        check(ColorScheme::Dark, [0.184, 0.435, 0.894]);
        check(ColorScheme::DarkTint, [0.557, 0.357, 0.847]);
        check(ColorScheme::Dark, [0.898, 0.271, 0.561]);
    }

    fn assert_configs_equal(a: &AppearanceConfig, b: &AppearanceConfig) {
        assert_eq!(a.color_scheme, b.color_scheme);
        assert_eq!(a.accent_color, b.accent_color);
        assert_eq!(a.contrast, b.contrast);
        assert_eq!(a.reduced_motion, b.reduced_motion);
        assert_eq!(a.reduce_transparency, b.reduce_transparency);
        assert_eq!(a.window_color, b.window_color);
        assert_eq!(a.window_tint_scope, b.window_tint_scope);
        assert_eq!(a.traffic_lights, b.traffic_lights);
        assert_eq!(a.traffic_light_shape, b.traffic_light_shape);
        assert_eq!(a.sidebar_style, b.sidebar_style);
        assert_eq!(a.tint_strength_accent, b.tint_strength_accent);
        assert_eq!(a.tint_strength_wallpaper, b.tint_strength_wallpaper);
        assert_eq!(a.tint_parts, b.tint_parts);
    }

    #[test]
    fn write_appearance_roundtrips() {
        let dir = std::env::temp_dir().join(format!("vow-appearance-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            accent_color: Some([0.184, 0.435, 0.894]),
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::WhenScrolling,
            reduce_transparency: true,
            window_color: WindowColorMode::Accent,
            window_tint_scope: WindowTintScope::Desktop,
            traffic_lights: TrafficLightStyle::Accent,
            traffic_light_shape: TrafficLightShape::Round,
            sidebar_style: SidebarStyle::Flat,
            tint_strength_accent: TintStrength::Strong,
            tint_strength_wallpaper: TintStrength::Subtle,
            tint_parts: TintParts {
                sidebars: true,
                window: false,
                groups: true,
                controls: false,
                menus: true,
            },
        };
        write_appearance(&path, &config).unwrap();
        let read = read_appearance_from(&path).unwrap().unwrap();
        assert_configs_equal(&config, &read);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_preserves_unknown_keys_and_comments() {
        let dir =
            std::env::temp_dir().join(format!("vow-appearance-preserve-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        std::fs::write(
            &path,
            "# legacy comment preserved\nlegacy_key = true\ncolor_scheme = \"light\"\n",
        )
        .unwrap();
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            ..AppearanceConfig::default()
        };
        write_appearance(&path, &config).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("legacy comment preserved"));
        assert!(text.contains("legacy_key = true"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_backs_up_an_unreadable_domain() {
        let dir =
            std::env::temp_dir().join(format!("vow-appearance-backup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        let original = "color_scheme = ";
        std::fs::write(&path, original).unwrap();
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            ..AppearanceConfig::default()
        };

        write_appearance(&path, &config).unwrap();

        let backup = dir.join("appearance.domain.bak");
        assert_eq!(std::fs::read(&backup).unwrap(), original.as_bytes());
        let read = read_appearance_from(&path).unwrap().unwrap();
        assert_eq!(read.color_scheme, ColorScheme::Dark);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_does_not_back_up_a_readable_domain() {
        let dir =
            std::env::temp_dir().join(format!("vow-appearance-nobackup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        std::fs::write(&path, "color_scheme = \"light\"\n").unwrap();

        write_appearance(&path, &AppearanceConfig::default()).unwrap();

        assert!(!dir.join("appearance.domain.bak").exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_creates_missing_file_and_leaves_no_temp() {
        let dir =
            std::env::temp_dir().join(format!("vow-appearance-create-{}", std::process::id()));
        let path = dir.join("appearance.domain");
        assert!(!path.exists());
        write_appearance(&path, &AppearanceConfig::default()).unwrap();
        assert!(path.exists());
        assert!(!dir.join("appearance.tmp").exists());
        read_appearance_from(&path)
            .unwrap()
            .expect("created file parses");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_default_uses_config_dir() {
        let _env = ENV_LOCK.lock().unwrap();
        let dir =
            std::env::temp_dir().join(format!("vow-appearance-default-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let old = std::env::var_os("MALAYSIA_CONFIG_DIR");
        unsafe { std::env::set_var("MALAYSIA_CONFIG_DIR", &dir) };
        write_appearance_default().unwrap();
        let path = dir.join("appearance.domain");
        assert!(path.exists());
        let read = read_appearance_from(&path).unwrap().unwrap();
        assert_configs_equal(&AppearanceConfig::default(), &read);
        match old {
            Some(v) => unsafe { std::env::set_var("MALAYSIA_CONFIG_DIR", v) },
            None => unsafe { std::env::remove_var("MALAYSIA_CONFIG_DIR") },
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn config_and_data_dirs_use_the_documented_malaysia_directory() {
        let _env = ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("vow-appearance-dirs-{}", std::process::id()));
        let config_home = dir.join("config");
        let data_home = dir.join("data");
        let old_config = std::env::var_os("MALAYSIA_CONFIG_DIR");
        let old_data = std::env::var_os("MALAYSIA_DATA_DIR");
        let old_xdg_config = std::env::var_os("XDG_CONFIG_HOME");
        let old_xdg_data = std::env::var_os("XDG_DATA_HOME");
        let restore = |name: &str, value: Option<std::ffi::OsString>| unsafe {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        };

        unsafe {
            std::env::remove_var("MALAYSIA_CONFIG_DIR");
            std::env::remove_var("MALAYSIA_DATA_DIR");
            std::env::set_var("XDG_CONFIG_HOME", &config_home);
            std::env::set_var("XDG_DATA_HOME", &data_home);
        }
        assert_eq!(config_dir(), config_home.join("malaysia"));
        assert_eq!(data_dir(), data_home.join("malaysia"));

        restore("MALAYSIA_CONFIG_DIR", old_config);
        restore("MALAYSIA_DATA_DIR", old_data);
        restore("XDG_CONFIG_HOME", old_xdg_config);
        restore("XDG_DATA_HOME", old_xdg_data);
    }

    #[test]
    fn wallpaper_accent_for_saturated_yellow_meets_contrast() {
        let raw = Rgba::rgb8(0xF5, 0xD0, 0x20);
        let (hue, saturation, _) = raw.to_hsl();
        let yellow = Rgba::from_hsl(hue, saturation, 0.65, 1.0);
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            let resolved = resolve(
                &appearance(
                    scheme,
                    WindowColorMode::Wallpaper,
                    WindowTintScope::Windows,
                    TrafficLightStyle::Auto,
                    None,
                ),
                Some(yellow),
            );
            let bg_window = match scheme {
                ColorScheme::DarkTint => BG_WINDOW_DARK_TINT,
                ColorScheme::Dark => BG_WINDOW_DARK,
                _ => BG_WINDOW_LIGHT,
            };
            assert!(
                resolved.accent.contrast_ratio(bg_window) >= 3.0,
                "{scheme:?}: accent must contrast with the untinted window background"
            );
            let on_accent = palette(scheme).on_accent;
            let on_accent_ratio = on_accent.contrast_ratio(resolved.accent);
            if matches!(scheme, ColorScheme::Light) {
                assert!(
                    on_accent_ratio >= 3.0,
                    "{scheme:?}: on-accent must contrast with the accent"
                );
            } else {
                assert!(
                    on_accent_ratio >= 1.5,
                    "{scheme:?}: on-accent should at least remain readable-ish"
                );
            }
        }
    }

    #[test]
    fn wallpaper_window_tint_uses_the_dominant_not_the_accent() {
        let raw = Rgba::rgb8(0xF5, 0xD0, 0x20);
        let (hue, saturation, _) = raw.to_hsl();
        let yellow = Rgba::from_hsl(hue, saturation, 0.65, 1.0);
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            let index = usize::from(matches!(scheme, ColorScheme::Dark | ColorScheme::DarkTint));
            let resolved = resolve(
                &appearance(
                    scheme,
                    WindowColorMode::Wallpaper,
                    WindowTintScope::Windows,
                    TrafficLightStyle::Auto,
                    None,
                ),
                Some(yellow),
            );
            assert_eq!(
                resolved.window_tint,
                Some((yellow, WINDOW_TINT_WALLPAPER[index])),
                "{scheme:?}: window tint must be the raw dominant"
            );
        }
    }

    #[test]
    fn the_wallpaper_accent_always_contrasts_with_the_window() {
        let channels = (0..=0xff).step_by(0x11).map(|c| c as u8);
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            let bg_window = match scheme {
                ColorScheme::DarkTint => BG_WINDOW_DARK_TINT,
                ColorScheme::Dark => BG_WINDOW_DARK,
                _ => BG_WINDOW_LIGHT,
            };
            for r in channels.clone() {
                for g in channels.clone() {
                    for b in channels.clone() {
                        let dominant = Rgba::rgb8(r, g, b);
                        let ratio = wallpaper_accent(dominant, scheme).contrast_ratio(bg_window);
                        assert!(
                            ratio >= ACCENT_MIN_CONTRAST_ON_WINDOW,
                            "{scheme:?} {dominant:?}: {ratio}"
                        );
                    }
                }
            }
        }
    }

    fn hue_diff_degrees(a: Rgba, b: Rgba) -> f32 {
        let (h_a, _, _) = a.to_hsl();
        let (h_b, _, _) = b.to_hsl();
        let diff = ((h_a - h_b) * 360.0).abs();
        diff.min(360.0 - diff)
    }

    #[test]
    fn wallpaper_accent_for_forest_green_keeps_hue_and_shifts_lightness() {
        let forest = Rgba::rgb8(0x2E, 0x7D, 0x32);
        let light = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Wallpaper,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                None,
            ),
            Some(forest),
        );
        let dark = resolve(
            &appearance(
                ColorScheme::Dark,
                WindowColorMode::Wallpaper,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                None,
            ),
            Some(forest),
        );

        assert!(
            light.accent.relative_luminance() < forest.relative_luminance(),
            "light accent must be darker than the dominant"
        );
        assert!(
            dark.accent.relative_luminance() > forest.relative_luminance(),
            "dark accent must be lighter than the dominant"
        );
        assert!(
            hue_diff_degrees(light.accent, forest) <= 2.0,
            "light accent hue drift is too large"
        );
        assert!(
            hue_diff_degrees(dark.accent, forest) <= 2.0,
            "dark accent hue drift is too large"
        );
    }

    #[test]
    fn neutral_mode_unchanged_for_a_fixed_case() {
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Light,
            accent_color: None,
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::WhenScrolling,
            reduce_transparency: true,
            window_color: WindowColorMode::Neutral,
            window_tint_scope: WindowTintScope::Windows,
            traffic_lights: TrafficLightStyle::Auto,
            sidebar_style: SidebarStyle::Island,
            ..AppearanceConfig::default()
        };
        let resolved = resolve(&config, Some(dominant()));
        let expected = Resolved {
            scheme: ColorScheme::Light,
            accent: LIGHT.accent,
            window_tint: None,
            window_tint_scope: WindowTintScope::Windows,
            panel_tint: None,
            lights: [
                TITLEBAR.buttons.order[0].color,
                TITLEBAR.buttons.order[1].color,
                TITLEBAR.buttons.order[2].color,
            ],
            traffic_style: TrafficLightStyle::Standard,
            traffic_shape: TrafficLightShape::Square,
            reduce_transparency: true,
            reduced_motion: false,
            scroll_bars: ScrollBars::WhenScrolling,
            shell: SHELL_LIGHT_OPAQUE,
            menu: MENU_LIGHT_OPAQUE,
            sidebar: SidebarSurface {
                style: SidebarStyle::Island,
                bg: SIDEBAR_ISLAND_LIGHT_OPAQUE,
                rim: SIDEBAR_ISLAND_RIM_LIGHT,
                inner: SIDEBAR_ISLAND_INNER_LIGHT,
                palette: SidebarPalette {
                    item_icon: LIGHT.accent,
                    selected_ink: LIGHT.on_accent,
                    ..SIDEBAR_PALETTE_LIGHT
                },
            },
            toolgroup: ToolgroupSurface {
                bg: TOOLGROUP_BG_LIGHT_OPAQUE,
                palette: TOOLGROUP_PALETTE_LIGHT,
            },
            surfaces: Surfaces {
                window: BG_WINDOW_LIGHT,
                titlebar_inactive: LIGHT.titlebar_bg.unfocused,
                sunken: BG_SUNKEN_LIGHT,
                surface: BG_SURFACE_LIGHT,
                raised: BG_RAISED_LIGHT,
                control: BG_CONTROL_LIGHT,
                hover: BG_HOVER_LIGHT,
                line: LIGHT.line,
            },
            tint_parts: TintParts::default(),
            menu_parent: BG_WINDOW_LIGHT,
        };
        assert_eq!(resolved, expected);
    }

    #[test]
    fn accent_mode_unchanged_for_a_fixed_case() {
        let accent = Rgba {
            r: 0.9,
            g: 0.3,
            b: 0.1,
            a: 1.0,
        };
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Light,
            accent_color: Some([accent.r, accent.g, accent.b]),
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::WhenScrolling,
            reduce_transparency: true,
            window_color: WindowColorMode::Accent,
            window_tint_scope: WindowTintScope::Windows,
            traffic_lights: TrafficLightStyle::Accent,
            sidebar_style: SidebarStyle::Island,
            ..AppearanceConfig::default()
        };
        let resolved = resolve(&config, Some(dominant()));
        let tint_factor = WINDOW_TINT_ACCENT[0];
        let mut expected_shell = SHELL_LIGHT_OPAQUE;
        expected_shell.tile_active = accent;
        let expected = Resolved {
            scheme: ColorScheme::Light,
            accent,
            window_tint: Some((accent, WINDOW_TINT_ACCENT[0])),
            window_tint_scope: WindowTintScope::Windows,
            panel_tint: None,
            lights: derived_lights(accent),
            traffic_style: TrafficLightStyle::Accent,
            traffic_shape: TrafficLightShape::Square,
            reduce_transparency: true,
            reduced_motion: false,
            scroll_bars: ScrollBars::WhenScrolling,
            shell: expected_shell,
            menu: MenuPalette {
                hover_bg: accent,
                hover_fg: LIGHT.on_accent,
                search_ring: accent,
                ..MENU_LIGHT_OPAQUE
            },
            sidebar: SidebarSurface {
                style: SidebarStyle::Island,
                bg: SIDEBAR_ISLAND_LIGHT_OPAQUE.mix(accent, tint_factor * LAYER_FACTOR_BG_SUNKEN),
                rim: SIDEBAR_ISLAND_RIM_LIGHT,
                inner: SIDEBAR_ISLAND_INNER_LIGHT,
                palette: SidebarPalette {
                    item_icon: accent,
                    selected_bg: accent,
                    selected_ink: LIGHT.on_accent,
                    ..SIDEBAR_PALETTE_LIGHT
                },
            },
            toolgroup: ToolgroupSurface {
                bg: TOOLGROUP_BG_LIGHT_OPAQUE.mix(accent, tint_factor * LAYER_FACTOR_CONTROL),
                palette: TOOLGROUP_PALETTE_LIGHT,
            },
            surfaces: Surfaces {
                window: BG_WINDOW_LIGHT.mix(accent, tint_factor),
                titlebar_inactive: LIGHT.titlebar_bg.unfocused.mix(accent, tint_factor),
                sunken: BG_SUNKEN_LIGHT.mix(accent, tint_factor * LAYER_FACTOR_BG_SUNKEN),
                surface: BG_SURFACE_LIGHT.mix(accent, tint_factor * LAYER_FACTOR_SURFACE),
                raised: BG_RAISED_LIGHT.mix(accent, tint_factor * LAYER_FACTOR_RAISED),
                control: BG_CONTROL_LIGHT.mix(accent, tint_factor * LAYER_FACTOR_CONTROL),
                hover: BG_HOVER_LIGHT.mix(accent, tint_factor * LAYER_FACTOR_CONTROL),
                line: LIGHT.line.mix(accent, tint_factor * LAYER_FACTOR_LINE),
            },
            tint_parts: TintParts::default(),
            menu_parent: BG_WINDOW_LIGHT.mix(accent, tint_factor),
        };
        assert_eq!(resolved, expected);
    }

    #[test]
    fn old_domain_without_new_fields_uses_defaults() {
        let config: AppearanceConfig = toml::from_str(
            r#"
color_scheme = "light"
accent_color = [0.2, 0.5, 0.8]
contrast = false
reduced_motion = false
window_color = "accent"
window_tint_scope = "windows"
traffic_lights = "auto"
sidebar_style = "island"
"#,
        )
        .unwrap();
        assert_eq!(config.tint_strength_accent, TintStrength::Normal);
        assert_eq!(config.tint_strength_wallpaper, TintStrength::Normal);
        assert_eq!(config.tint_parts, TintParts::default());
    }

    #[test]
    fn tint_parts_roundtrip_through_write_appearance() {
        let dir = std::env::temp_dir().join(format!(
            "vow-appearance-parts-roundtrip-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        let config = AppearanceConfig {
            tint_strength_accent: TintStrength::Intense,
            tint_strength_wallpaper: TintStrength::Subtle,
            tint_parts: TintParts {
                sidebars: true,
                window: false,
                groups: true,
                controls: false,
                menus: true,
            },
            ..AppearanceConfig::default()
        };
        write_appearance(&path, &config).unwrap();
        let read = read_appearance_from(&path).unwrap().unwrap();
        assert_configs_equal(&config, &read);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn only_sidebars_tinted_leaves_other_surfaces_neutral() {
        let mut config = appearance(
            ColorScheme::Light,
            WindowColorMode::Accent,
            WindowTintScope::Windows,
            TrafficLightStyle::Auto,
            Some([0.9, 0.3, 0.1]),
        );
        config.tint_parts = TintParts {
            sidebars: true,
            window: false,
            groups: false,
            controls: false,
            menus: false,
        };
        let resolved = resolve(&config, None);
        let neutral = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Neutral,
                WindowTintScope::Windows,
                TrafficLightStyle::Auto,
                Some([0.9, 0.3, 0.1]),
            ),
            None,
        );
        assert_ne!(resolved.sidebar.bg, neutral.sidebar.bg);
        assert_eq!(resolved.surfaces.window, BG_WINDOW_LIGHT);
        assert_eq!(resolved.surfaces.surface, BG_SURFACE_LIGHT);
        assert_eq!(resolved.surfaces.control, BG_CONTROL_LIGHT);
        assert_eq!(resolved.surfaces.raised, BG_RAISED_LIGHT);
    }

    #[test]
    fn intense_wallpaper_dark_doubles_the_tint_factors() {
        let mut config = appearance(
            ColorScheme::Dark,
            WindowColorMode::Wallpaper,
            WindowTintScope::Desktop,
            TrafficLightStyle::Auto,
            None,
        );
        config.tint_strength_wallpaper = TintStrength::Intense;
        let dominant = Rgba {
            r: 0.8,
            g: 0.2,
            b: 0.1,
            a: 1.0,
        };
        let resolved = resolve(&config, Some(dominant));
        assert_eq!(
            resolved.window_tint,
            Some((dominant, WINDOW_TINT_WALLPAPER[1] * 2.0))
        );
        assert_eq!(
            resolved.panel_tint,
            Some((dominant, PANEL_TINT_WALLPAPER[1] * 2.0))
        );
    }
}
