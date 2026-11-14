use ctdesign::{
    ACCENT_CHROMA_GAIN, ACCENT_MIN_CONTRAST_ON_WINDOW, BG_CONTROL_DARK, BG_CONTROL_DARK_TINT,
    BG_CONTROL_LIGHT, BG_HOVER_DARK, BG_HOVER_DARK_TINT, BG_HOVER_LIGHT, BG_RAISED_DARK,
    BG_RAISED_DARK_TINT, BG_RAISED_LIGHT, BG_SUNKEN_DARK, BG_SUNKEN_DARK_TINT, BG_SUNKEN_LIGHT,
    BG_SURFACE_DARK, BG_SURFACE_DARK_TINT, BG_SURFACE_LIGHT, BG_WINDOW_DARK, BG_WINDOW_DARK_TINT,
    BG_WINDOW_LIGHT, DARK, DARK_TINT, DOCK_DIVIDER_DARK, DOCK_DIVIDER_LIGHT, DOCK_GLASS_DARK,
    DOCK_GLASS_DARK_TINT, DOCK_GLASS_LIGHT, DOCK_GLASS_SOLID_DARK, DOCK_GLASS_SOLID_DARK_TINT,
    DOCK_GLASS_SOLID_LIGHT, DOCK_OUTLINE_DARK, DOCK_OUTLINE_DARK_TINT, DOCK_OUTLINE_LIGHT,
    DOCK_OUTLINE_SOLID_DARK, DOCK_OUTLINE_SOLID_DARK_TINT, DOCK_OUTLINE_SOLID_LIGHT,
    DOCK_RIM_CORNER_DARK, DOCK_RIM_CORNER_LIGHT, DOCK_RIM_DARK, DOCK_RIM_GLOW, DOCK_RIM_GLOW_LIGHT,
    DOCK_RIM_LIGHT, DOCK_TONE_DARK, DOCK_TONE_LIGHT, DockSurface, GLASS, LAYER_FACTOR_BG_SUNKEN,
    LAYER_FACTOR_CONTROL, LAYER_FACTOR_LINE, LAYER_FACTOR_RAISED, LAYER_FACTOR_SURFACE, LIGHT,
    MENU_DARK, MENU_DARK_OPAQUE, MENU_DARK_TINT, MENU_DARK_TINT_OPAQUE, MENU_LIGHT,
    MENU_LIGHT_OPAQUE, MENU_SEPARATOR_INK_MIX, MenuPalette, ON_ACCENT_MIN_CONTRAST, Palette, Rgba,
    SHELL_DARK, SHELL_DARK_OPAQUE, SHELL_DARK_TINT, SHELL_DARK_TINT_OPAQUE, SHELL_LIGHT,
    SHELL_LIGHT_OPAQUE, SIDEBAR_DARK, SIDEBAR_DARK_OPAQUE, SIDEBAR_DARK_TINT,
    SIDEBAR_DARK_TINT_OPAQUE, SIDEBAR_GLASS_BASE_DARK, SIDEBAR_GLASS_BASE_DARK_TINT,
    SIDEBAR_GLASS_BASE_LIGHT, SIDEBAR_GLASS_SOLID_DARK, SIDEBAR_GLASS_SOLID_DARK_TINT,
    SIDEBAR_GLASS_SOLID_LIGHT, SIDEBAR_ISLAND_DARK, SIDEBAR_ISLAND_DARK_TINT,
    SIDEBAR_ISLAND_INNER_DARK, SIDEBAR_ISLAND_INNER_DARK_TINT, SIDEBAR_ISLAND_INNER_LIGHT,
    SIDEBAR_ISLAND_LIGHT, SIDEBAR_ISLAND_RIM_DARK, SIDEBAR_ISLAND_RIM_DARK_TINT,
    SIDEBAR_ISLAND_RIM_LIGHT, SIDEBAR_LIGHT, SIDEBAR_LIGHT_OPAQUE, SIDEBAR_PALETTE_DARK,
    SIDEBAR_PALETTE_DARK_TINT, SIDEBAR_PALETTE_LIGHT, ShellPalette, SidebarPalette, TITLEBAR,
    TOOLGROUP_BG_DARK, TOOLGROUP_BG_DARK_OPAQUE, TOOLGROUP_BG_DARK_TINT,
    TOOLGROUP_BG_DARK_TINT_OPAQUE, TOOLGROUP_BG_LIGHT, TOOLGROUP_BG_LIGHT_OPAQUE,
    TOOLGROUP_PALETTE_DARK, TOOLGROUP_PALETTE_DARK_TINT, TOOLGROUP_PALETTE_LIGHT,
    TRAFFIC_CLOSE_MINIMIZE_DARKEN, TRAFFIC_ZOOM_LIGHTEN, ToolgroupPalette, WALLPAPER_TINTING,
    WALLPAPER_TINTING_SIDEBAR, WINDOW_TINT_ACCENT,
};

use crate::backdrop::{take_chroma, take_chroma_capped};
use crate::icon_style::IconStyle;
use crate::{
    AppearanceConfig, ColorScheme, MenuBarSize, ScrollBarClick, ScrollBars, SidebarStyle,
    TintParts, Toolkit, TrafficLightShape, TrafficLightStyle, WindowColorMode,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarSurface {
    pub style: SidebarStyle,
    pub bg: Rgba,
    pub rim: Rgba,
    pub inner: Rgba,
    pub palette: SidebarPalette,
    pub glass: Option<SidebarGlass>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarGlass {
    pub base: Rgba,
    pub solid: Rgba,
    pub chroma: f32,
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
    pub automatic_accent: bool,
    pub on_accent: Rgba,
    pub focus_ring: Rgba,
    pub link: Rgba,
    pub window_tint: Option<(Rgba, f32)>,
    pub bar_tint: Option<(Rgba, f32)>,
    pub wallpaper_tinting: Option<f32>,
    pub lights: [Rgba; 3],
    pub traffic_style: TrafficLightStyle,
    pub traffic_shape: TrafficLightShape,
    pub contrast: bool,
    pub reduce_transparency: bool,
    pub reduced_motion: bool,
    pub show_borders: bool,
    pub differentiate_without_color: bool,
    pub text_size: f32,
    pub menu_bar_size: MenuBarSize,
    pub icon_style: IconStyle,
    pub scroll_bars: ScrollBars,
    pub scroll_bar_click: ScrollBarClick,
    pub shell: ShellPalette,
    pub dock: DockSurface,
    pub menu: MenuPalette,
    pub sidebar: SidebarSurface,
    pub toolgroup: ToolgroupSurface,
    pub surfaces: Surfaces,
    pub tint_parts: TintParts,
    pub menu_parent: Rgba,
}

impl Resolved {
    pub fn behind_window(self, wallpaper: Rgba) -> Self {
        match self.wallpaper_tinting {
            Some(factor) => self.tinted((wallpaper, factor), Tint::Wallpaper),
            None => self,
        }
    }

    fn tinted(mut self, (tint, factor): (Rgba, f32), source: Tint) -> Self {
        let scheme = self.scheme;
        let bg_window = scheme.pick(BG_WINDOW);
        let (mix, sidebar_factor) = match source {
            Tint::Accent => (take_accent_chroma as Mix, LAYER_FACTOR_BG_SUNKEN),
            Tint::Wallpaper => (take_chroma as Mix, WALLPAPER_TINTING_SIDEBAR),
        };
        self.window_tint = Some((tint, factor));
        self.menu_parent = mix(bg_window, tint, factor);
        self.surfaces = tinted_surfaces(
            scheme,
            self.window_tint,
            &palette(scheme),
            self.tint_parts,
            mix,
        );
        if self.tint_parts.sidebars {
            self.sidebar.bg = mix(self.sidebar.bg, tint, factor * sidebar_factor);
        }
        self.sidebar.glass = match source {
            Tint::Accent => self.sidebar.glass.map(|glass| {
                if !self.tint_parts.sidebars {
                    return glass;
                }
                let base = mix(glass.base, tint, factor * sidebar_factor);
                SidebarGlass {
                    base,
                    solid: base,
                    chroma: glass.chroma * GLASS.accent_share,
                }
            }),
            Tint::Wallpaper => self.sidebar.glass.map(|glass| {
                if !self.tint_parts.sidebars {
                    return glass;
                }
                let tinted = |color| {
                    take_chroma_capped(color, tint, factor * sidebar_factor, glass_chroma(scheme).1)
                };
                SidebarGlass {
                    base: tinted(glass.base),
                    solid: tinted(glass.solid),
                    ..glass
                }
            }),
        };
        if self.tint_parts.controls {
            self.toolgroup.bg = mix(self.toolgroup.bg, tint, factor * LAYER_FACTOR_CONTROL);
        }
        self.link = reaching(
            self.accent,
            &[self.surfaces.window, self.surfaces.surface],
            ON_ACCENT_MIN_CONTRAST,
            scheme,
        );
        self.focus_ring = reaching(
            self.accent,
            &[self.surfaces.window],
            ACCENT_MIN_CONTRAST_ON_WINDOW,
            scheme,
        );
        self
    }

    pub fn menu_separator(&self, panel_bg: Rgba) -> Rgba {
        if self.window_tint.is_none() || panel_bg == self.menu.bg {
            self.menu.separator_color
        } else {
            panel_bg.mix(self.menu.fg, MENU_SEPARATOR_INK_MIX)
        }
    }
}

// How much of the wallpaper's chroma the sidebar glass takes, and the chroma it saturates towards (ADR 0078).
pub fn glass_chroma(scheme: ColorScheme) -> (f32, f32) {
    if scheme.is_dark() {
        (GLASS.chroma_dark, GLASS.chroma_cap_dark)
    } else {
        (GLASS.chroma_light, GLASS.chroma_cap_light)
    }
}

// How much of the lightness of what is behind the glass shows through it.
pub fn glass_lightness(scheme: ColorScheme) -> f32 {
    if scheme.is_dark() {
        GLASS.lightness_dark
    } else {
        GLASS.lightness_light
    }
}

// How much lighter the island's border is than the fill under it, and how much more where the wallpaper shows.
pub fn glass_rim_lift(scheme: ColorScheme) -> (f32, f32) {
    if scheme.is_dark() {
        (GLASS.rim_lift_dark, GLASS.rim_lift_edge_dark)
    } else {
        (GLASS.rim_lift_light, GLASS.rim_lift_edge_light)
    }
}

// How many times the glass blurs what is behind it.
pub fn glass_blur_passes(scheme: ColorScheme) -> u32 {
    if scheme.is_dark() {
        GLASS.blur_passes_dark
    } else {
        GLASS.blur_passes_light
    }
}

// The wallpaper shows along the left, top and bottom edges of a panel: `intensity` at the edge, fading over `span` logical pixels, with `chroma` and `lightness` of what is `offset` logical pixels outside the window (ADR 0078).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassEdge {
    pub intensity: f32,
    pub span: f32,
    pub chroma: f32,
    pub lightness: f32,
    pub cap: f32,
    pub offset: f32,
}

pub fn glass_edge(scheme: ColorScheme) -> GlassEdge {
    if scheme.is_dark() {
        GlassEdge {
            intensity: GLASS.edge_intensity_dark,
            span: GLASS.edge_span_dark,
            chroma: GLASS.edge_chroma_dark,
            lightness: GLASS.edge_lightness_dark,
            cap: GLASS.edge_cap_dark,
            offset: GLASS.edge_offset,
        }
    } else {
        GlassEdge {
            intensity: GLASS.edge_intensity_light,
            span: GLASS.edge_span_light,
            chroma: GLASS.edge_chroma_light,
            lightness: GLASS.edge_lightness_light,
            cap: GLASS.edge_cap_light,
            offset: GLASS.edge_offset,
        }
    }
}

// The island's shadow on the window around it: `opacity` next to the island, fading over `span` logical pixels, as dark as `lightness` and with `tint` of the wallpaper's chroma.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassShadow {
    pub opacity: f32,
    pub span: f32,
    pub tint: f32,
    pub lightness: f32,
}

pub fn glass_shadow(scheme: ColorScheme) -> GlassShadow {
    let (opacity, lightness) = if scheme.is_dark() {
        (GLASS.shadow_opacity_dark, GLASS.shadow_lightness_dark)
    } else {
        (GLASS.shadow_opacity_light, GLASS.shadow_lightness_light)
    };
    GlassShadow {
        opacity,
        span: GLASS.shadow_span,
        tint: GLASS.shadow_tint,
        lightness,
    }
}

// The accent tints like the wallpaper does: only its chroma, so a window keeps its lightness.
fn take_accent_chroma(base: Rgba, accent: Rgba, amount: f32) -> Rgba {
    take_chroma(base, accent, amount * ACCENT_CHROMA_GAIN)
}

type Mix = fn(Rgba, Rgba, f32) -> Rgba;

#[derive(Clone, Copy)]
enum Tint {
    Accent,
    Wallpaper,
}

pub fn palette(scheme: ColorScheme) -> Palette {
    scheme.pick([LIGHT, DARK, DARK_TINT])
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

// With transparency reduced the Dock is a solid grey, as in macOS, not its glass over the window colour.
fn dock_surface(scheme: ColorScheme, opaque: bool) -> DockSurface {
    let (rim, divider, corner, tone) = if scheme.is_dark() {
        (
            DOCK_RIM_DARK,
            DOCK_DIVIDER_DARK,
            DOCK_RIM_CORNER_DARK,
            DOCK_TONE_DARK,
        )
    } else {
        (
            DOCK_RIM_LIGHT,
            DOCK_DIVIDER_LIGHT,
            DOCK_RIM_CORNER_LIGHT,
            DOCK_TONE_LIGHT,
        )
    };
    let glow = if opaque || scheme.is_dark() {
        DOCK_RIM_GLOW
    } else {
        DOCK_RIM_GLOW_LIGHT
    };
    let (glass, outline) = if opaque {
        (
            scheme.pick([
                DOCK_GLASS_SOLID_LIGHT,
                DOCK_GLASS_SOLID_DARK,
                DOCK_GLASS_SOLID_DARK_TINT,
            ]),
            scheme.pick([
                DOCK_OUTLINE_SOLID_LIGHT,
                DOCK_OUTLINE_SOLID_DARK,
                DOCK_OUTLINE_SOLID_DARK_TINT,
            ]),
        )
    } else {
        (
            scheme.pick([DOCK_GLASS_LIGHT, DOCK_GLASS_DARK, DOCK_GLASS_DARK_TINT]),
            scheme.pick([
                DOCK_OUTLINE_LIGHT,
                DOCK_OUTLINE_DARK,
                DOCK_OUTLINE_DARK_TINT,
            ]),
        )
    };
    DockSurface {
        glass,
        outline,
        rim,
        divider,
        corner,
        glow,
        tone: (!opaque).then_some(tone),
    }
}

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
    let glass = (!opaque).then(|| {
        let base = scheme.pick([
            SIDEBAR_GLASS_BASE_LIGHT,
            SIDEBAR_GLASS_BASE_DARK,
            SIDEBAR_GLASS_BASE_DARK_TINT,
        ]);
        SidebarGlass {
            base,
            solid: scheme.pick([
                SIDEBAR_GLASS_SOLID_LIGHT,
                SIDEBAR_GLASS_SOLID_DARK,
                SIDEBAR_GLASS_SOLID_DARK_TINT,
            ]),
            chroma: glass_chroma(scheme).0,
        }
    });
    let sidebar = match style {
        SidebarStyle::Flat => SidebarSurface {
            style,
            bg: scheme.pick(SIDEBAR_FLAT[layer]),
            rim: Rgba::TRANSPARENT,
            inner: Rgba::TRANSPARENT,
            palette,
            glass,
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
            glass,
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
    let scheme = appearance.color_scheme;
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

    let (accent, on_accent) = legible_on_accent(normal_accent, base.on_accent);

    let window_tint = match appearance.window_color {
        WindowColorMode::Neutral => None,
        WindowColorMode::Accent => Some((
            normal_accent,
            WINDOW_TINT_ACCENT[index] * appearance.tint_strength.factor(),
        )),
    };
    let wallpaper_tinting = (appearance.wallpaper_tinting
        && !appearance.transparency_reduced()
        && window_tint.is_none()
        && (scheme.is_dark() || appearance.wallpaper_tinting_in_light))
        .then_some(WALLPAPER_TINTING[index] * appearance.tint_strength.factor());
    // The menu bar and the Dock span the desktop, so the wallpaper source tints them with the dominant.
    let bar_tint = appearance
        .tint_parts
        .bars
        .then(|| window_tint.or_else(|| dominant.zip(wallpaper_tinting)))
        .flatten();

    let lights = lights_for(appearance.traffic_light_style(), normal_accent);

    let (mut shell, mut menu, mut sidebar, toolgroup) = chrome_surfaces(
        scheme,
        appearance.sidebar_style,
        appearance.transparency_reduced(),
    );
    let surfaces = tinted_surfaces(scheme, None, &base, appearance.tint_parts, Rgba::mix);
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
    sidebar.palette.selected_bg = accent;
    sidebar.palette.item_icon = accent;
    sidebar.palette.selected_ink = on_accent;
    shell.tile_active = accent;

    let untinted = Resolved {
        scheme,
        accent,
        automatic_accent: appearance.accent_color.is_none(),
        on_accent,
        focus_ring,
        link,
        window_tint: None,
        bar_tint,
        wallpaper_tinting,
        lights,
        traffic_style: appearance.traffic_light_style(),
        traffic_shape: appearance.traffic_light_shape,
        contrast: appearance.contrast,
        reduce_transparency: appearance.transparency_reduced(),
        reduced_motion: appearance.reduced_motion,
        show_borders: appearance.show_borders,
        differentiate_without_color: appearance.differentiate_without_color,
        text_size: appearance.text_size_for(Toolkit::BarrierKit),
        menu_bar_size: appearance.menu_bar_size,
        icon_style: appearance.icon_style,
        scroll_bars: appearance.scroll_bars,
        scroll_bar_click: appearance.scroll_bar_click,
        shell,
        dock: dock_surface(scheme, appearance.transparency_reduced()),
        menu,
        sidebar,
        toolgroup,
        surfaces,
        tint_parts: appearance.tint_parts,
        menu_parent: scheme.pick(BG_WINDOW),
    };
    match window_tint {
        Some(tint) => untinted.tinted(tint, Tint::Accent),
        None => untinted,
    }
}

fn tinted_surfaces(
    scheme: ColorScheme,
    window_tint: Option<(Rgba, f32)>,
    base: &Palette,
    parts: TintParts,
    blend: Mix,
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
                blend(color, tint, tint_factor * factor)
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

// WCAG 2.2 §1.4.3: text on the accent needs 4.5:1. The ink stays light, as in macOS; an accent
// that does not reach it is darkened just enough.
fn legible_on_accent(accent: Rgba, light: Rgba) -> (Rgba, Rgba) {
    let (hue, saturation, mut lightness) = accent.to_hsl();
    let mut fill = accent;
    while fill.contrast_ratio(light) < ON_ACCENT_MIN_CONTRAST && lightness > 0.0 {
        lightness = (lightness - 0.01).max(0.0);
        fill = Rgba::from_hsl(hue, saturation, lightness, 1.0);
    }
    (fill, light)
}

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

pub fn lights_for(style: TrafficLightStyle, normal_accent: Rgba) -> [Rgba; 3] {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TintStrength;
    use ctdesign::{
        BG_CONTROL_LIGHT, BG_RAISED_LIGHT, BG_SURFACE_LIGHT, BG_WINDOW_LIGHT, SIDEBAR_LIGHT_OPAQUE,
        TITLEBAR, TOOLGROUP_BG_LIGHT_OPAQUE, TRAFFIC_CLOSE_MINIMIZE_DARKEN, TRAFFIC_ZOOM_LIGHTEN,
        WINDOW_TINT_ACCENT,
    };

    fn appearance(
        scheme: ColorScheme,
        mode: WindowColorMode,
        lights: TrafficLightStyle,
        accent: Option<[f32; 3]>,
    ) -> AppearanceConfig {
        AppearanceConfig {
            color_scheme: scheme,
            accent_color: accent,
            window_color: mode,
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
    fn neutral_desktop_does_not_tint() {
        let resolved = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Neutral,
                TrafficLightStyle::Auto,
                None,
            ),
            Some(dominant()),
        );
        assert_eq!(resolved.window_tint, None);
        assert_eq!(resolved.accent, LIGHT.accent);
    }

    #[test]
    fn accent_tints_windows() {
        let accent = [0.1, 0.3, 0.8];
        let resolved = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Accent,
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
    }

    #[test]
    fn dark_and_dark_tint_use_the_dark_factors() {
        for scheme in [ColorScheme::Dark, ColorScheme::DarkTint] {
            let resolved = resolve(
                &appearance(
                    scheme,
                    WindowColorMode::Accent,
                    TrafficLightStyle::Auto,
                    None,
                ),
                None,
            );
            assert_eq!(
                resolved.window_tint,
                Some((palette(scheme).accent, WINDOW_TINT_ACCENT[1]))
            );
        }
    }

    #[test]
    fn only_the_automatic_accent_leaves_an_apps_own_accent_alone() {
        let automatic = resolve(&AppearanceConfig::default(), Some(dominant()));
        assert!(automatic.automatic_accent);
        assert_eq!(automatic.accent, LIGHT.accent);
        let blue = [LIGHT.accent.r, LIGHT.accent.g, LIGHT.accent.b];
        let forced = resolve(
            &AppearanceConfig {
                accent_color: Some(blue),
                ..AppearanceConfig::default()
            },
            Some(dominant()),
        );
        assert!(!forced.automatic_accent);
        assert_eq!(forced.accent, automatic.accent);
    }

    #[test]
    fn the_bars_take_the_accent_or_the_dominant_only_when_their_part_is_on() {
        let bars = TintParts {
            bars: true,
            ..TintParts::default()
        };
        let dark = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            ..AppearanceConfig::default()
        };
        assert_eq!(resolve(&dark, Some(dominant())).bar_tint, None);
        let from_wallpaper = AppearanceConfig {
            tint_parts: bars,
            ..dark.clone()
        };
        assert_eq!(
            resolve(&from_wallpaper, Some(dominant())).bar_tint,
            Some((dominant(), WALLPAPER_TINTING[1]))
        );
        let from_accent = AppearanceConfig {
            window_color: WindowColorMode::Accent,
            ..from_wallpaper
        };
        let resolved = resolve(&from_accent, Some(dominant()));
        assert_eq!(resolved.bar_tint, resolved.window_tint);
    }

    #[test]
    fn reduced_transparency_makes_shell_menu_and_sidebar_opaque() {
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            let config = AppearanceConfig {
                reduce_transparency: true,
                ..appearance(
                    scheme,
                    WindowColorMode::Neutral,
                    TrafficLightStyle::Auto,
                    None,
                )
            };
            let resolved = resolve(&config, Some(dominant()));
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
    fn the_dock_is_glass_and_with_transparency_reduced_a_solid_grey() {
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            let config = appearance(
                scheme,
                WindowColorMode::Neutral,
                TrafficLightStyle::Auto,
                None,
            );
            let glass = resolve(&config, None).dock;
            let solid = resolve(
                &AppearanceConfig {
                    reduce_transparency: true,
                    ..config
                },
                None,
            )
            .dock;
            assert!(glass.glass.a < 1.0, "{scheme:?} glass is solid");
            assert_eq!(solid.glass.a, 1.0, "{scheme:?} is still translucent");
            assert_eq!(glass.rim, solid.rim);
            assert_eq!(glass.divider < 0.0, !scheme.is_dark());
        }
    }

    #[test]
    fn increase_contrast_also_reduces_transparency() {
        let config = AppearanceConfig {
            contrast: true,
            reduce_transparency: false,
            ..AppearanceConfig::default()
        };
        let resolved = resolve(&config, None);
        assert!(resolved.reduce_transparency);
        assert_eq!(resolved.shell.bar.a, 1.0);
    }

    #[test]
    fn transparency_keeps_the_translucent_bar_when_disabled() {
        let config = appearance(
            ColorScheme::Light,
            WindowColorMode::Neutral,
            TrafficLightStyle::Auto,
            None,
        );
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
    fn accent_tints_sidebar_and_toolgroup_but_neutral_does_not() {
        let accent = Rgba {
            r: 0.9,
            g: 0.3,
            b: 0.1,
            a: 1.0,
        };
        let mut neutral = appearance(
            ColorScheme::Light,
            WindowColorMode::Neutral,
            TrafficLightStyle::Auto,
            Some([accent.r, accent.g, accent.b]),
        );
        neutral.reduce_transparency = true;
        let neutral = resolve(&neutral, None);

        let mut accent_mode = appearance(
            ColorScheme::Light,
            WindowColorMode::Accent,
            TrafficLightStyle::Auto,
            Some([accent.r, accent.g, accent.b]),
        );
        accent_mode.reduce_transparency = true;
        let accent_resolved = resolve(&accent_mode, None);

        let expected_factor = WINDOW_TINT_ACCENT[0] * LAYER_FACTOR_BG_SUNKEN;
        let expected_sidebar_bg = take_accent_chroma(neutral.sidebar.bg, accent, expected_factor);
        assert_eq!(accent_resolved.sidebar.bg, expected_sidebar_bg);

        let expected_toolgroup_factor = WINDOW_TINT_ACCENT[0] * LAYER_FACTOR_CONTROL;
        let expected_toolgroup_bg =
            take_accent_chroma(neutral.toolgroup.bg, accent, expected_toolgroup_factor);
        assert_eq!(accent_resolved.toolgroup.bg, expected_toolgroup_bg);

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
    fn every_accent_swatch_gets_readable_ink_and_a_visible_focus_ring() {
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            for (name, swatch) in ctdesign::ACCENT_SWATCHES {
                let mut config = AppearanceConfig {
                    color_scheme: scheme,
                    accent_color: Some([swatch.r, swatch.g, swatch.b]),
                    window_color: WindowColorMode::Accent,
                    tint_strength: TintStrength::Intense,
                    ..Default::default()
                };
                let tinted = resolve(&config, None);
                assert!(
                    tinted.focus_ring.contrast_ratio(tinted.surfaces.window)
                        >= ACCENT_MIN_CONTRAST_ON_WINDOW,
                    "{scheme:?} {name}: focus ring on the tinted window"
                );
                config.window_color = WindowColorMode::Neutral;
                let resolved = resolve(&config, None);
                if palette(scheme).on_accent.contrast_ratio(swatch) >= ON_ACCENT_MIN_CONTRAST {
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
    fn only_sidebars_tinted_leaves_other_surfaces_neutral() {
        let mut config = appearance(
            ColorScheme::Light,
            WindowColorMode::Accent,
            TrafficLightStyle::Auto,
            Some([0.9, 0.3, 0.1]),
        );
        config.tint_parts = TintParts {
            sidebars: true,
            window: false,
            groups: false,
            controls: false,
            menus: false,
            bars: false,
        };
        let resolved = resolve(&config, None);
        let neutral = resolve(
            &appearance(
                ColorScheme::Light,
                WindowColorMode::Neutral,
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
    fn the_wallpaper_behind_tints_the_chosen_parts_like_the_accent_does() {
        let red = Rgba::rgb8(200, 30, 30);
        for strength in [TintStrength::Subtle, TintStrength::Intense] {
            let config = AppearanceConfig {
                color_scheme: ColorScheme::Dark,
                tint_strength: strength,
                ..AppearanceConfig::default()
            };
            let neutral = resolve(&config, None);
            let factor = WALLPAPER_TINTING[1] * strength.factor();
            assert_eq!(neutral.wallpaper_tinting, Some(factor));
            let behind = neutral.behind_window(red);
            assert_eq!(behind.window_tint, Some((red, factor)));
            assert_eq!(
                behind.surfaces.window,
                take_chroma(BG_WINDOW_DARK, red, factor),
                "{strength:?}: the window background"
            );
            assert_eq!(
                behind.sidebar.bg,
                take_chroma(neutral.sidebar.bg, red, factor * WALLPAPER_TINTING_SIDEBAR),
                "{strength:?}: the sidebar"
            );
        }
    }

    #[test]
    fn reduced_transparency_and_contrast_leave_the_wallpaper_without_tint() {
        let red = Rgba::rgb8(200, 30, 30);
        for (reduce_transparency, contrast) in [(true, false), (false, true)] {
            let config = AppearanceConfig {
                color_scheme: ColorScheme::Dark,
                reduce_transparency,
                contrast,
                ..AppearanceConfig::default()
            };
            let resolved = resolve(&config, None);
            assert_eq!(resolved.wallpaper_tinting, None);
            assert_eq!(resolved.behind_window(red).sidebar.bg, resolved.sidebar.bg);
        }
    }

    #[test]
    fn the_sidebar_has_glass_only_while_transparency_is_allowed() {
        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            let config = |reduce_transparency| AppearanceConfig {
                reduce_transparency,
                ..appearance(
                    scheme,
                    WindowColorMode::Neutral,
                    TrafficLightStyle::Auto,
                    None,
                )
            };
            let glass = resolve(&config(false), None).sidebar.glass.unwrap();
            assert_eq!(
                glass.solid,
                scheme.pick([
                    SIDEBAR_GLASS_SOLID_LIGHT,
                    SIDEBAR_GLASS_SOLID_DARK,
                    SIDEBAR_GLASS_SOLID_DARK_TINT
                ]),
                "{scheme:?}: nothing tints it yet"
            );
            assert_eq!(
                resolve(&config(true), None).sidebar.glass,
                None,
                "{scheme:?}"
            );
        }
    }

    #[test]
    fn the_wallpaper_tints_the_glass_at_rest_and_the_solid_one() {
        let red = Rgba::rgb8(200, 30, 30);
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            ..AppearanceConfig::default()
        };
        let glass = resolve(&config, None).sidebar.glass.unwrap();
        let factor = WALLPAPER_TINTING[1] * config.tint_strength.factor();
        let behind = resolve(&config, None)
            .behind_window(red)
            .sidebar
            .glass
            .unwrap();
        let tinted = |color| {
            take_chroma_capped(
                color,
                red,
                factor * WALLPAPER_TINTING_SIDEBAR,
                glass_chroma(ColorScheme::Dark).1,
            )
        };
        assert_eq!(behind.base, tinted(glass.base));
        assert_eq!(behind.solid, tinted(glass.solid));
    }

    #[test]
    fn the_solid_glass_stays_the_base_when_the_sidebars_do_not_take_the_tint() {
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            tint_parts: TintParts {
                sidebars: false,
                ..AppearanceConfig::default().tint_parts
            },
            ..AppearanceConfig::default()
        };
        let behind = resolve(&config, None)
            .behind_window(Rgba::rgb8(200, 30, 30))
            .sidebar
            .glass
            .unwrap();
        assert_eq!(behind.solid, behind.base);
    }

    #[test]
    fn an_accent_tint_changes_the_hue_but_not_the_lightness_of_the_window() {
        use crate::backdrop::Oklab;
        let accent = Rgba::rgb8(10, 132, 255);
        let config = appearance(
            ColorScheme::Dark,
            WindowColorMode::Accent,
            TrafficLightStyle::Auto,
            Some([accent.r, accent.g, accent.b]),
        );
        let window = resolve(&config, None).surfaces.window;
        let (before, after) = (Oklab::from_rgba(BG_WINDOW_DARK), Oklab::from_rgba(window));
        assert!((before.l - after.l).abs() < 0.005);
        assert!(after.b.hypot(after.a) > before.b.hypot(before.a));
    }

    #[test]
    fn an_accent_window_keeps_the_glass_with_the_accent_as_its_base() {
        let accent = Rgba {
            r: 0.9,
            g: 0.3,
            b: 0.1,
            a: 1.0,
        };
        let config = appearance(
            ColorScheme::Dark,
            WindowColorMode::Accent,
            TrafficLightStyle::Auto,
            Some([accent.r, accent.g, accent.b]),
        );
        let neutral = resolve(
            &AppearanceConfig {
                color_scheme: ColorScheme::Dark,
                ..AppearanceConfig::default()
            },
            None,
        );
        let glass = resolve(&config, None).sidebar.glass.unwrap();
        let neutral = neutral.sidebar.glass.unwrap();
        assert_eq!(glass.base, glass.solid);
        assert_ne!(glass.base, neutral.base);
        assert_eq!(glass.chroma, neutral.chroma * GLASS.accent_share);
    }

    #[test]
    fn a_bright_wallpaper_changes_the_hue_but_not_the_lightness_of_the_window() {
        use crate::backdrop::Oklab;
        let neutral = resolve(
            &AppearanceConfig {
                color_scheme: ColorScheme::Dark,
                ..AppearanceConfig::default()
            },
            None,
        );
        let green = Rgba::rgb8(146, 197, 41);
        let behind = neutral.behind_window(green);
        let (before, after) = (
            Oklab::from_rgba(neutral.surfaces.window),
            Oklab::from_rgba(behind.surfaces.window),
        );
        assert!((after.l - before.l).abs() < 0.002);
        let wall = Oklab::from_rgba(green);
        let taken = (after.a * wall.a + after.b * wall.b) / (wall.a.powi(2) + wall.b.powi(2));
        assert!((taken - WALLPAPER_TINTING[1]).abs() < 0.01);
    }

    #[test]
    fn unchecked_parts_stay_neutral_behind_the_wallpaper() {
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            tint_parts: TintParts {
                sidebars: true,
                window: false,
                groups: false,
                controls: false,
                menus: false,
                bars: false,
            },
            ..AppearanceConfig::default()
        };
        let neutral = resolve(&config, None);
        let behind = neutral.behind_window(Rgba::rgb8(200, 30, 30));
        assert_ne!(behind.sidebar.bg, neutral.sidebar.bg);
        assert_eq!(behind.surfaces, neutral.surfaces);
    }

    #[test]
    fn light_windows_take_the_wallpaper_only_with_the_accessibility_option() {
        let light = AppearanceConfig {
            color_scheme: ColorScheme::Light,
            ..AppearanceConfig::default()
        };
        assert_eq!(resolve(&light, None).wallpaper_tinting, None);
        let opted_in = AppearanceConfig {
            wallpaper_tinting_in_light: true,
            ..light
        };
        assert_eq!(
            resolve(&opted_in, None).wallpaper_tinting,
            Some(WALLPAPER_TINTING[0])
        );
    }

    #[test]
    fn the_accent_window_colour_and_the_switch_turn_wallpaper_tinting_off() {
        let dark = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            ..AppearanceConfig::default()
        };
        for config in [
            AppearanceConfig {
                window_color: WindowColorMode::Accent,
                ..dark.clone()
            },
            AppearanceConfig {
                wallpaper_tinting: false,
                ..dark.clone()
            },
        ] {
            let resolved = resolve(&config, None);
            assert_eq!(resolved.wallpaper_tinting, None);
            assert_eq!(resolved.behind_window(Rgba::rgb8(200, 30, 30)), resolved);
        }
    }

    #[test]
    fn a_graphite_accent_still_lets_the_wallpaper_tint() {
        let graphite = ctdesign::ACCENT_SWATCHES
            .iter()
            .find(|(name, _)| *name == "graphite")
            .map(|(_, swatch)| [swatch.r, swatch.g, swatch.b]);
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            accent_color: graphite,
            ..AppearanceConfig::default()
        };
        assert_eq!(
            resolve(&config, None).wallpaper_tinting,
            Some(WALLPAPER_TINTING[1])
        );
    }
}
