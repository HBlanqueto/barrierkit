use ctdesign::{
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
    SIDEBAR_DARK_TINT, SIDEBAR_DARK_TINT_OPAQUE, SIDEBAR_ISLAND_DARK, SIDEBAR_ISLAND_DARK_TINT,
    SIDEBAR_ISLAND_INNER_DARK, SIDEBAR_ISLAND_INNER_DARK_TINT, SIDEBAR_ISLAND_INNER_LIGHT,
    SIDEBAR_ISLAND_LIGHT, SIDEBAR_ISLAND_RIM_DARK, SIDEBAR_ISLAND_RIM_DARK_TINT,
    SIDEBAR_ISLAND_RIM_LIGHT, SIDEBAR_LIGHT, SIDEBAR_LIGHT_OPAQUE, SIDEBAR_PALETTE_DARK,
    SIDEBAR_PALETTE_DARK_TINT, SIDEBAR_PALETTE_LIGHT, ShellPalette, SidebarPalette, TITLEBAR,
    TOOLGROUP_BG_DARK, TOOLGROUP_BG_DARK_OPAQUE, TOOLGROUP_BG_DARK_TINT,
    TOOLGROUP_BG_DARK_TINT_OPAQUE, TOOLGROUP_BG_LIGHT, TOOLGROUP_BG_LIGHT_OPAQUE,
    TOOLGROUP_PALETTE_DARK, TOOLGROUP_PALETTE_DARK_TINT, TOOLGROUP_PALETTE_LIGHT,
    TRAFFIC_CLOSE_MINIMIZE_DARKEN, TRAFFIC_ZOOM_LIGHTEN, ToolgroupPalette,
    WALLPAPER_ACCENT_DARKEN_LIGHT, WALLPAPER_ACCENT_LIGHTEN_DARK, WINDOW_TINT_ACCENT,
    WINDOW_TINT_WALLPAPER,
};

use crate::{
    AppearanceConfig, ColorScheme, ScrollBars, SidebarStyle, TintParts, TrafficLightShape,
    TrafficLightStyle, WindowColorMode, WindowTintScope,
};

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
    pub on_accent: Rgba,
    pub focus_ring: Rgba,
    pub link: Rgba,
    pub window_tint: Option<(Rgba, f32)>,
    pub window_tint_scope: WindowTintScope,
    pub panel_tint: Option<(Rgba, f32)>,
    pub lights: [Rgba; 3],
    pub traffic_style: TrafficLightStyle,
    pub traffic_shape: TrafficLightShape,
    pub contrast: bool,
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
    scheme.resolved().pick([LIGHT, DARK, DARK_TINT])
}

const BG_WINDOW: [Rgba; 3] = [BG_WINDOW_LIGHT, BG_WINDOW_DARK, BG_WINDOW_DARK_TINT];

const SHELL: [[ShellPalette; 3]; 2] = [
    [SHELL_LIGHT, SHELL_DARK, SHELL_DARK_TINT],
    [
        SHELL_LIGHT_OPAQUE,
        SHELL_DARK_OPAQUE,
        SHELL_DARK_TINT_OPAQUE,
    ],
];

const MENU: [[MenuPalette; 3]; 2] = [
    [MENU_LIGHT, MENU_DARK, MENU_DARK_TINT],
    [MENU_LIGHT_OPAQUE, MENU_DARK_OPAQUE, MENU_DARK_TINT_OPAQUE],
];

const SIDEBAR_FLAT: [[Rgba; 3]; 2] = [
    [SIDEBAR_LIGHT, SIDEBAR_DARK, SIDEBAR_DARK_TINT],
    [
        SIDEBAR_LIGHT_OPAQUE,
        SIDEBAR_DARK_OPAQUE,
        SIDEBAR_DARK_TINT_OPAQUE,
    ],
];

const SIDEBAR_ISLAND: [[Rgba; 3]; 2] = [
    [
        SIDEBAR_ISLAND_LIGHT,
        SIDEBAR_ISLAND_DARK,
        SIDEBAR_ISLAND_DARK_TINT,
    ],
    [
        SIDEBAR_LIGHT_OPAQUE,
        SIDEBAR_DARK_OPAQUE,
        SIDEBAR_DARK_TINT_OPAQUE,
    ],
];

const TOOLGROUP_BG: [[Rgba; 3]; 2] = [
    [
        TOOLGROUP_BG_LIGHT,
        TOOLGROUP_BG_DARK,
        TOOLGROUP_BG_DARK_TINT,
    ],
    [
        TOOLGROUP_BG_LIGHT_OPAQUE,
        TOOLGROUP_BG_DARK_OPAQUE,
        TOOLGROUP_BG_DARK_TINT_OPAQUE,
    ],
];

fn chrome_surfaces(
    scheme: ColorScheme,
    style: SidebarStyle,
    opaque: bool,
) -> (ShellPalette, MenuPalette, SidebarSurface, ToolgroupSurface) {
    let layer = usize::from(opaque);
    let palette = scheme.pick([
        SIDEBAR_PALETTE_LIGHT,
        SIDEBAR_PALETTE_DARK,
        SIDEBAR_PALETTE_DARK_TINT,
    ]);
    let sidebar = match style {
        SidebarStyle::Flat => SidebarSurface {
            style,
            bg: scheme.pick(SIDEBAR_FLAT[layer]),
            rim: Rgba::TRANSPARENT,
            inner: Rgba::TRANSPARENT,
            palette,
        },
        SidebarStyle::Island => SidebarSurface {
            style,
            bg: scheme.pick(SIDEBAR_ISLAND[layer]),
            rim: scheme.pick([
                SIDEBAR_ISLAND_RIM_LIGHT,
                SIDEBAR_ISLAND_RIM_DARK,
                SIDEBAR_ISLAND_RIM_DARK_TINT,
            ]),
            inner: scheme.pick([
                SIDEBAR_ISLAND_INNER_LIGHT,
                SIDEBAR_ISLAND_INNER_DARK,
                SIDEBAR_ISLAND_INNER_DARK_TINT,
            ]),
            palette,
        },
    };
    let toolgroup = ToolgroupSurface {
        bg: scheme.pick(TOOLGROUP_BG[layer]),
        palette: scheme.pick([
            TOOLGROUP_PALETTE_LIGHT,
            TOOLGROUP_PALETTE_DARK,
            TOOLGROUP_PALETTE_DARK_TINT,
        ]),
    };
    (
        scheme.pick(SHELL[layer]),
        scheme.pick(MENU[layer]),
        sidebar,
        toolgroup,
    )
}

pub fn resolve(appearance: &AppearanceConfig, dominant: Option<Rgba>) -> Resolved {
    let scheme = appearance.color_scheme.resolved();
    let base = palette(scheme);
    let index = usize::from(scheme.is_dark());

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
    let (accent, on_accent) = legible_on_accent(accent, base.on_accent);

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

    let panel_tint = (appearance.window_color == WindowColorMode::Wallpaper
        && appearance.window_tint_scope == WindowTintScope::Desktop)
        .then(|| {
            (
                dominant,
                PANEL_TINT_WALLPAPER[index] * appearance.tint_strength_wallpaper.factor(),
            )
        });

    let lights = lights_for(
        appearance.traffic_light_style(),
        normal_accent,
        Some(dominant),
    );

    let (mut shell, mut menu, mut sidebar, mut toolgroup) = chrome_surfaces(
        scheme,
        appearance.sidebar_style,
        appearance.reduce_transparency,
    );
    let bg_window = scheme.pick(BG_WINDOW);
    let menu_parent = window_tint.map_or(bg_window, |(tint, factor)| bg_window.mix(tint, factor));
    let surfaces = tinted_surfaces(scheme, window_tint, &base, appearance.tint_parts);
    // WCAG 2.2: a focus ring needs 3:1 on the window (§1.4.11), link text 4.5:1 where it sits (§1.4.3).
    let focus_ring = reaching(
        accent,
        &[scheme.pick(BG_WINDOW)],
        ACCENT_MIN_CONTRAST_ON_WINDOW,
        scheme,
    );
    let link = reaching(
        accent,
        &[surfaces.window, surfaces.surface],
        ON_ACCENT_MIN_CONTRAST,
        scheme,
    );
    menu.hover_bg = accent;
    menu.hover_fg = on_accent;
    menu.search_ring = focus_ring;
    sidebar.palette.selected_bg = accent;
    sidebar.palette.item_icon = accent;
    sidebar.palette.selected_ink = on_accent;
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
        on_accent,
        focus_ring,
        link,
        window_tint,
        window_tint_scope: appearance.window_tint_scope,
        panel_tint,
        lights,
        traffic_style: appearance.traffic_light_style(),
        traffic_shape: appearance.traffic_light_shape,
        contrast: appearance.contrast,
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
    let [window, sunken, surface, raised, control, hover] = scheme.pick([
        [
            BG_WINDOW_LIGHT,
            BG_SUNKEN_LIGHT,
            BG_SURFACE_LIGHT,
            BG_RAISED_LIGHT,
            BG_CONTROL_LIGHT,
            BG_HOVER_LIGHT,
        ],
        [
            BG_WINDOW_DARK,
            BG_SUNKEN_DARK,
            BG_SURFACE_DARK,
            BG_RAISED_DARK,
            BG_CONTROL_DARK,
            BG_HOVER_DARK,
        ],
        [
            BG_WINDOW_DARK_TINT,
            BG_SUNKEN_DARK_TINT,
            BG_SURFACE_DARK_TINT,
            BG_RAISED_DARK_TINT,
            BG_CONTROL_DARK_TINT,
            BG_HOVER_DARK_TINT,
        ],
    ]);
    let mix = |color: Rgba, factor: f32, enabled: bool| {
        window_tint
            .filter(|_| enabled)
            .map_or(color, |(tint, tint_factor)| {
                color.mix(tint, tint_factor * factor)
            })
    };
    Surfaces {
        window: mix(window, 1.0, parts.window),
        titlebar_inactive: mix(base.titlebar_bg.unfocused, 1.0, parts.window),
        sunken: mix(sunken, LAYER_FACTOR_BG_SUNKEN, parts.window),
        surface: mix(surface, LAYER_FACTOR_SURFACE, parts.groups),
        raised: mix(raised, LAYER_FACTOR_RAISED, parts.menus),
        control: mix(control, LAYER_FACTOR_CONTROL, parts.controls),
        hover: mix(hover, LAYER_FACTOR_CONTROL, parts.controls),
        line: mix(base.line, LAYER_FACTOR_LINE, parts.groups),
    }
}

// WCAG 2.2 §1.4.3: text on the accent needs 4.5:1. The light ink stays while it reaches it, then
// the dark ink; a mid-tone accent that neither reaches is darkened just enough for the light ink.
fn legible_on_accent(accent: Rgba, light: Rgba) -> (Rgba, Rgba) {
    if accent.contrast_ratio(light) >= ON_ACCENT_MIN_CONTRAST {
        return (accent, light);
    }
    let ink = crate::backdrop::ink_on(accent);
    if accent.contrast_ratio(ink) >= ON_ACCENT_MIN_CONTRAST {
        return (accent, ink);
    }
    let (hue, saturation, mut lightness) = accent.to_hsl();
    let mut fill = accent;
    while fill.contrast_ratio(light) < ON_ACCENT_MIN_CONTRAST && lightness > 0.0 {
        lightness = (lightness - 0.01).max(0.0);
        fill = Rgba::from_hsl(hue, saturation, lightness, 1.0);
    }
    (fill, light)
}

// The accent with its lightness moved until it reaches `min` on every background; the hue stays.
fn reaching(accent: Rgba, backgrounds: &[Rgba], min: f32, scheme: ColorScheme) -> Rgba {
    if backgrounds
        .iter()
        .all(|bg| accent.contrast_ratio(*bg) >= min)
    {
        return accent;
    }
    let (hue, saturation, lightness) = accent.to_hsl();
    let lightness = reach_contrast(backgrounds, min, scheme, hue, saturation, lightness);
    Rgba::from_hsl(hue, saturation, lightness, 1.0)
}

fn reach_contrast(
    backgrounds: &[Rgba],
    min: f32,
    scheme: ColorScheme,
    hue: f32,
    saturation: f32,
    mut lightness: f32,
) -> f32 {
    let direction = if scheme.is_dark() { 1.0 } else { -1.0 };
    for _ in 0..100 {
        let accent = Rgba::from_hsl(hue, saturation, lightness, 1.0);
        if backgrounds
            .iter()
            .all(|bg| accent.contrast_ratio(*bg) >= min)
        {
            break;
        }
        lightness += direction * 0.01;
        if !(0.0..=1.0).contains(&lightness) {
            break;
        }
    }
    lightness
}

fn reach_window_contrast(scheme: ColorScheme, hue: f32, saturation: f32, lightness: f32) -> f32 {
    let window = [scheme.pick(BG_WINDOW)];
    reach_contrast(
        &window,
        ACCENT_MIN_CONTRAST_ON_WINDOW,
        scheme,
        hue,
        saturation,
        lightness,
    )
}

fn wallpaper_accent(dominant: Rgba, scheme: ColorScheme) -> Rgba {
    let dark = scheme.is_dark();
    let shifted = if dark {
        dominant.lighten(WALLPAPER_ACCENT_LIGHTEN_DARK)
    } else {
        dominant.darken(WALLPAPER_ACCENT_DARKEN_LIGHT)
    };
    let (hue, mut saturation, original_lightness) = shifted.to_hsl();
    let bg_window = scheme.pick(BG_WINDOW);
    let on_accent = palette(scheme).on_accent;

    let mut lightness = reach_window_contrast(scheme, hue, saturation, original_lightness);

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
        lightness = reach_window_contrast(scheme, hue, saturation, lightness);
    }

    Rgba::from_hsl(hue, saturation, lightness, 1.0)
}

pub fn lights_for(
    style: TrafficLightStyle,
    normal_accent: Rgba,
    dominant: Option<Rgba>,
) -> [Rgba; 3] {
    let derived = |source: Rgba| {
        [
            source.darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN),
            source.darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN),
            source.lighten(TRAFFIC_ZOOM_LIGHTEN),
        ]
    };
    match style {
        TrafficLightStyle::Auto | TrafficLightStyle::Standard => {
            TITLEBAR.buttons.order.map(|button| button.color)
        }
        TrafficLightStyle::Accent => derived(normal_accent),
        TrafficLightStyle::Wallpaper => derived(dominant.unwrap_or(normal_accent)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TintStrength;
    use ctdesign::{
        BG_CONTROL_LIGHT, BG_RAISED_LIGHT, BG_SURFACE_LIGHT, BG_WINDOW_LIGHT, PANEL_TINT_WALLPAPER,
        SIDEBAR_LIGHT_OPAQUE, TITLEBAR, TOOLGROUP_BG_LIGHT_OPAQUE, TRAFFIC_CLOSE_MINIMIZE_DARKEN,
        TRAFFIC_ZOOM_LIGHTEN, WINDOW_TINT_ACCENT, WINDOW_TINT_WALLPAPER,
    };

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
            window_color: mode,
            window_tint_scope: scope,
            traffic_lights: lights,
            ..AppearanceConfig::default()
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
    fn accent_tints_windows_but_not_panels() {
        let accent = [0.1, 0.3, 0.8];
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

        assert_eq!(neutral.sidebar.bg, SIDEBAR_LIGHT_OPAQUE);
        assert_eq!(neutral.toolgroup.bg, TOOLGROUP_BG_LIGHT_OPAQUE);
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
            let index = usize::from(scheme.is_dark());
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
    fn every_accent_swatch_gets_readable_ink_and_a_visible_focus_ring() {
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            for (name, swatch) in ctdesign::ACCENT_SWATCHES {
                let config = AppearanceConfig {
                    color_scheme: scheme,
                    accent_color: Some([swatch.r, swatch.g, swatch.b]),
                    ..Default::default()
                };
                let resolved = resolve(&config, None);
                if crate::backdrop::ink_on(swatch).contrast_ratio(swatch) >= ON_ACCENT_MIN_CONTRAST
                {
                    assert_eq!(
                        resolved.accent, swatch,
                        "{scheme:?} {name}: the fill is the user's"
                    );
                }
                assert!(
                    resolved.on_accent.contrast_ratio(resolved.accent) >= ON_ACCENT_MIN_CONTRAST,
                    "{scheme:?} {name}: ink on the accent"
                );
                assert!(
                    resolved.focus_ring.contrast_ratio(scheme.pick(BG_WINDOW))
                        >= ACCENT_MIN_CONTRAST_ON_WINDOW,
                    "{scheme:?} {name}: focus ring on the window"
                );
                for background in [resolved.surfaces.window, resolved.surfaces.surface] {
                    assert!(
                        resolved.link.contrast_ratio(background) >= ON_ACCENT_MIN_CONTRAST,
                        "{scheme:?} {name}: link text"
                    );
                }
            }
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
