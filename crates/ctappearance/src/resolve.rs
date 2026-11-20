use ctdesign::{
    ACCENT_CHROMA_GAIN, ACCENT_MIN_CONTRAST_ON_WINDOW, ACCENT_SWATCHES, BG_CONTROL_DARK,
    BG_CONTROL_DARK_TINT, BG_CONTROL_LIGHT, BG_HOVER_DARK, BG_HOVER_DARK_TINT, BG_HOVER_LIGHT,
    BG_RAISED_DARK, BG_RAISED_DARK_TINT, BG_RAISED_LIGHT, BG_SUNKEN_DARK, BG_SUNKEN_DARK_TINT,
    BG_SUNKEN_LIGHT, BG_SURFACE_DARK, BG_SURFACE_DARK_TINT, BG_SURFACE_LIGHT, BG_WINDOW_DARK,
    BG_WINDOW_DARK_TINT, BG_WINDOW_LIGHT, DARK, DARK_TINT, DOCK_DIVIDER_DARK, DOCK_DIVIDER_LIGHT,
    DOCK_GLASS_DARK, DOCK_GLASS_DARK_TINT, DOCK_GLASS_FROSTED_DARK, DOCK_GLASS_FROSTED_DARK_TINT,
    DOCK_GLASS_FROSTED_LIGHT, DOCK_GLASS_LIGHT, DOCK_GLASS_SOLID_DARK, DOCK_GLASS_SOLID_DARK_TINT,
    DOCK_GLASS_SOLID_LIGHT, DOCK_LENS_DARK, DOCK_LENS_LIGHT, DOCK_OUTLINE_DARK,
    DOCK_OUTLINE_DARK_TINT, DOCK_OUTLINE_LIGHT, DOCK_OUTLINE_SOLID_DARK,
    DOCK_OUTLINE_SOLID_DARK_TINT, DOCK_OUTLINE_SOLID_LIGHT, DOCK_RIM_CORNER_DARK,
    DOCK_RIM_CORNER_LIGHT, DOCK_RIM_DARK, DOCK_RIM_GLOW, DOCK_RIM_GLOW_LIGHT, DOCK_RIM_LIGHT,
    DOCK_RIM_SIDE_DARK, DOCK_RIM_SIDE_LIGHT, DOCK_TONE_DARK, DOCK_TONE_LIGHT, DockSurface,
    DockTone, GLASS, GLASS_BLUR_SPREAD, LAYER_FACTOR_BG_SUNKEN, LAYER_FACTOR_CONTROL,
    LAYER_FACTOR_LINE, LAYER_FACTOR_RAISED, LAYER_FACTOR_SURFACE, LIGHT, MENU_DARK,
    MENU_DARK_OPAQUE, MENU_DARK_TINT, MENU_DARK_TINT_OPAQUE, MENU_GLASS_CAP_DARK,
    MENU_GLASS_CAP_LIGHT, MENU_GLASS_CHROMA, MENU_GLASS_LIFT_DARK, MENU_GLASS_LIFT_LIGHT,
    MENU_GLASS_SPREAD, MENU_GLASS_VEIL_DARK, MENU_GLASS_VEIL_LIGHT, MENU_LIGHT, MENU_LIGHT_OPAQUE,
    MENU_SEPARATOR_INK_MIX, MenuPalette, NOTIFICATION_GLASS_VEIL_LIGHT, ON_ACCENT_MIN_CONTRAST,
    Palette, Rgba, SHELL_DARK, SHELL_DARK_OPAQUE, SHELL_DARK_TINT, SHELL_DARK_TINT_OPAQUE,
    SHELL_LIGHT, SHELL_LIGHT_OPAQUE, SIDEBAR_DARK, SIDEBAR_DARK_OPAQUE, SIDEBAR_DARK_TINT,
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
    WALLPAPER_TINTING_LIFT, WINDOW_TINT_ACCENT,
};

use crate::backdrop::{WallpaperTinting, lift, shift_like, take_chroma, take_chroma_capped};
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
    pub wallpaper_tinting: Option<WallpaperTinting>,
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
    pub icon_dark: bool,
    pub icon_tint: Rgba,
    pub scroll_bars: ScrollBars,
    pub scroll_bar_click: ScrollBarClick,
    pub shell: ShellPalette,
    pub dock: DockSurface,
    pub menu_glass: DockSurface,
    pub notification_glass: DockSurface,
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
            Some(tinting) => self.tinted((wallpaper, tinting.chroma), Tint::Wallpaper(tinting)),
            None => self,
        }
    }

    fn tinted(mut self, (tint, factor): (Rgba, f32), source: Tint) -> Self {
        let scheme = self.scheme;
        let parts = self.tint_parts;
        let bg_window = scheme.pick(BG_WINDOW);
        self.window_tint = Some((tint, factor));
        match source {
            Tint::Accent => {
                let paint =
                    |color: Rgba, layer: f32| take_accent_chroma(color, tint, factor * layer);
                self.menu_parent = paint(bg_window, 1.0);
                self.surfaces = tinted_surfaces(scheme, &palette(scheme), parts, paint);
                if parts.sidebars {
                    self.sidebar.bg = paint(self.sidebar.bg, LAYER_FACTOR_BG_SUNKEN);
                    self.sidebar.glass = self.sidebar.glass.map(|glass| {
                        let base = paint(glass.base, LAYER_FACTOR_BG_SUNKEN);
                        SidebarGlass {
                            base,
                            solid: base,
                            chroma: glass.chroma * GLASS.accent_share,
                        }
                    });
                }
                if parts.controls {
                    self.toolgroup.bg = paint(self.toolgroup.bg, LAYER_FACTOR_CONTROL);
                }
            }
            Tint::Wallpaper(tinting) => {
                let window = tinting.take(bg_window, tint);
                let paint = |color: Rgba, _layer: f32| shift_like(color, bg_window, window);
                self.menu_parent = window;
                self.surfaces = tinted_surfaces(scheme, &palette(scheme), parts, paint);
                if parts.sidebars {
                    let sidebar = tinting.sidebar();
                    let cap = glass_chroma(scheme).1;
                    let glass_tint = |color| {
                        lift(
                            take_chroma_capped(color, tint, sidebar.chroma, cap),
                            sidebar.lift,
                        )
                    };
                    self.sidebar.bg = sidebar.take(self.sidebar.bg, tint);
                    self.sidebar.glass = self.sidebar.glass.map(|glass| SidebarGlass {
                        base: glass_tint(glass.base),
                        solid: glass_tint(glass.solid),
                        ..glass
                    });
                }
                if parts.controls {
                    self.toolgroup.bg = paint(self.toolgroup.bg, LAYER_FACTOR_CONTROL);
                }
            }
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

// `glass_blur` widens a glass's Gaussian from `rest`, as designed, to GLASS_BLUR_SPREAD.
fn glass_soften(rest: f32, glass_blur: f32) -> f32 {
    rest + liquid_glass(glass_blur) * (GLASS_BLUR_SPREAD - rest)
}

// Measured on macOS 26's Liquid Glass slider: halfway the blur has grown a quarter of the way and the tint has hardly moved.
fn liquid_glass(glass_blur: f32) -> f32 {
    glass_blur * glass_blur
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

pub fn glass_shadow(scheme: ColorScheme) -> Option<GlassShadow> {
    (!scheme.is_dark()).then_some(GlassShadow {
        opacity: GLASS.shadow_opacity,
        span: GLASS.shadow_span,
        tint: GLASS.shadow_tint,
        lightness: GLASS.shadow_lightness,
    })
}

// The accent tints like the wallpaper does: only its chroma, so a window keeps its lightness.
fn take_accent_chroma(base: Rgba, accent: Rgba, amount: f32) -> Rgba {
    take_chroma(base, accent, amount * ACCENT_CHROMA_GAIN)
}

#[derive(Clone, Copy)]
enum Tint {
    Accent,
    Wallpaper(WallpaperTinting),
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

// Mixed premultiplied, so over any backdrop the glass lies `amount` of the way from `clear` to `frosted`.
fn frosted_glass(clear: Rgba, frosted: Rgba, amount: f32) -> Rgba {
    let amount = amount.clamp(0.0, 1.0);
    let [clear, frosted] = [clear.premultiplied(), frosted.premultiplied()];
    let [r, g, b, a] = std::array::from_fn(|i| clear[i] + (frosted[i] - clear[i]) * amount);
    if a <= 0.0 {
        return Rgba::TRANSPARENT;
    }
    Rgba {
        r: r / a,
        g: g / a,
        b: b / a,
        a,
    }
}

// With transparency reduced the Dock is a solid grey, as in macOS, not its glass over the window colour.
fn dock_surface(scheme: ColorScheme, opaque: bool, glass_blur: f32) -> DockSurface {
    let (rim, divider, corner, lens, tone) = if scheme.is_dark() {
        (
            DOCK_RIM_DARK,
            DOCK_DIVIDER_DARK,
            DOCK_RIM_CORNER_DARK,
            DOCK_LENS_DARK,
            DOCK_TONE_DARK,
        )
    } else {
        (
            DOCK_RIM_LIGHT,
            DOCK_DIVIDER_LIGHT,
            DOCK_RIM_CORNER_LIGHT,
            DOCK_LENS_LIGHT,
            DOCK_TONE_LIGHT,
        )
    };
    let glow = if opaque || scheme.is_dark() {
        DOCK_RIM_GLOW
    } else {
        DOCK_RIM_GLOW_LIGHT
    };
    let rim_side = match (opaque, scheme.is_dark()) {
        (true, _) => 0.0,
        (false, true) => DOCK_RIM_SIDE_DARK,
        (false, false) => DOCK_RIM_SIDE_LIGHT,
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
            frosted_glass(
                scheme.pick([DOCK_GLASS_LIGHT, DOCK_GLASS_DARK, DOCK_GLASS_DARK_TINT]),
                scheme.pick([
                    DOCK_GLASS_FROSTED_LIGHT,
                    DOCK_GLASS_FROSTED_DARK,
                    DOCK_GLASS_FROSTED_DARK_TINT,
                ]),
                liquid_glass(glass_blur),
            ),
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
        rim_side,
        glow,
        lens,
        tone: (!opaque).then_some(tone),
        soften: glass_soften(1.0, glass_blur),
    }
}

fn menu_glass(dock: DockSurface, scheme: ColorScheme, glass_blur: f32) -> DockSurface {
    let (glass, lift, cap) = if scheme.is_dark() {
        (
            MENU_GLASS_VEIL_DARK,
            MENU_GLASS_LIFT_DARK,
            MENU_GLASS_CAP_DARK,
        )
    } else {
        (
            MENU_GLASS_VEIL_LIGHT,
            MENU_GLASS_LIFT_LIGHT,
            MENU_GLASS_CAP_LIGHT,
        )
    };
    DockSurface {
        glass,
        corner: [dock.corner[0], -1.0, 0.0],
        rim_side: 0.0,
        lens: [dock.lens[0], 0.0],
        tone: Some(DockTone {
            chroma: MENU_GLASS_CHROMA,
            lift,
            cap,
            ..DOCK_TONE_DARK
        }),
        soften: glass_soften(MENU_GLASS_SPREAD, glass_blur),
        ..dock
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

fn graphite_accent() -> Rgba {
    ACCENT_SWATCHES
        .iter()
        .find_map(|(name, color)| (*name == "graphite").then_some(*color))
        .expect("ctdesign has a graphite swatch")
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
        .or_else(|| appearance.wallpaper_accent.then_some(dominant).flatten())
        .unwrap_or_else(|| {
            if appearance.wallpaper_accent {
                graphite_accent()
            } else {
                base.accent
            }
        });

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
        .then(|| WallpaperTinting {
            chroma: WALLPAPER_TINTING[index] * appearance.tint_strength.factor(),
            lift: WALLPAPER_TINTING_LIFT[index] * appearance.tint_strength.factor(),
        });
    // The menu bar and the Dock span the desktop, so the wallpaper source tints them with the dominant.
    let bar_tint = appearance
        .tint_parts
        .bars
        .then(|| {
            window_tint.or_else(|| dominant.zip(wallpaper_tinting.map(|tinting| tinting.chroma)))
        })
        .flatten();

    let lights = lights_for(appearance.traffic_lights, normal_accent, dominant);

    let (mut shell, mut menu, mut sidebar, toolgroup) = chrome_surfaces(
        scheme,
        appearance.sidebar_style,
        appearance.transparency_reduced(),
    );
    let surfaces = tinted_surfaces(scheme, &base, appearance.tint_parts, |color, _| color);
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

    let dock = dock_surface(
        scheme,
        appearance.transparency_reduced(),
        appearance.glass_blur,
    );
    let menu_glass = menu_glass(dock, scheme, appearance.glass_blur);
    let untinted = Resolved {
        scheme,
        accent,
        automatic_accent: appearance.accent_color.is_none()
            && !(appearance.wallpaper_accent && dominant.is_some()),
        on_accent,
        focus_ring,
        link,
        window_tint: None,
        bar_tint,
        wallpaper_tinting,
        lights,
        traffic_style: appearance.traffic_lights,
        traffic_shape: appearance.traffic_light_shape,
        contrast: appearance.contrast,
        reduce_transparency: appearance.transparency_reduced(),
        reduced_motion: appearance.reduced_motion,
        show_borders: appearance.show_borders,
        differentiate_without_color: appearance.differentiate_without_color,
        text_size: appearance.text_size_for(Toolkit::BarrierKit),
        menu_bar_size: appearance.menu_bar_size,
        icon_style: appearance.icon_style,
        icon_dark: appearance
            .icon_style
            .dark(appearance.icon_tone, scheme.is_dark()),
        icon_tint: appearance
            .icon_color
            .map_or(normal_accent, |[r, g, b]| Rgba { r, g, b, a: 1.0 }),
        scroll_bars: appearance.scroll_bars,
        scroll_bar_click: appearance.scroll_bar_click,
        shell,
        dock,
        menu_glass,
        notification_glass: if appearance.transparency_reduced() {
            dock
        } else {
            DockSurface {
                glass: scheme.pick([
                    NOTIFICATION_GLASS_VEIL_LIGHT,
                    menu_glass.glass,
                    menu_glass.glass,
                ]),
                tone: menu_glass.tone,
                soften: menu_glass.soften,
                rim_side: menu_glass.rim_side,
                ..dock
            }
        },
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
    base: &Palette,
    parts: TintParts,
    paint: impl Fn(Rgba, f32) -> Rgba,
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
    let mix = |color: Rgba, layer: f32, enabled: bool| {
        if enabled { paint(color, layer) } else { color }
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
        TrafficLightStyle::Standard => TITLEBAR.buttons.order.map(|button| button.color),
        TrafficLightStyle::Accent => derived(normal_accent),
        TrafficLightStyle::Wallpaper => derived(dominant.unwrap_or(normal_accent)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TintStrength;
    use ctdesign::{
        BG_CONTROL_LIGHT, BG_RAISED_LIGHT, BG_SURFACE_LIGHT, BG_WINDOW_LIGHT,
        DOCK_GLASS_FROSTED_LIGHT, DOCK_GLASS_LIGHT, GLASS_BLUR_FULL, SIDEBAR_LIGHT_OPAQUE,
        TITLEBAR, TOOLGROUP_BG_LIGHT_OPAQUE, TRAFFIC_CLOSE_MINIMIZE_DARKEN, TRAFFIC_ZOOM_LIGHTEN,
        WALLPAPER_TINTING_TARGET, WINDOW_TINT_ACCENT,
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
                TrafficLightStyle::Standard,
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
                TrafficLightStyle::Standard,
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
                    TrafficLightStyle::Standard,
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
    fn the_wallpaper_accent_takes_the_dominant_colour() {
        let resolved = resolve(
            &AppearanceConfig {
                wallpaper_accent: true,
                ..AppearanceConfig::default()
            },
            Some(dominant()),
        );
        assert!(!resolved.automatic_accent);
        assert!(resolved.accent.r > resolved.accent.g);
        assert!(resolved.accent.r > resolved.accent.b);
    }

    #[test]
    fn a_monochrome_wallpaper_falls_back_to_graphite_instead_of_blue() {
        let resolved = resolve(
            &AppearanceConfig {
                wallpaper_accent: true,
                ..AppearanceConfig::default()
            },
            None,
        );
        let (expected, _) = legible_on_accent(graphite_accent(), LIGHT.on_accent);
        assert_eq!(resolved.accent, expected);
        assert_ne!(resolved.accent, LIGHT.accent);
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
                    TrafficLightStyle::Standard,
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
                TrafficLightStyle::Standard,
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
            TrafficLightStyle::Standard,
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
    fn wallpaper_traffic_lights_follow_the_dominant_and_fall_back_to_the_accent() {
        let config = appearance(
            ColorScheme::Light,
            WindowColorMode::Accent,
            TrafficLightStyle::Wallpaper,
            Some([0.9, 0.3, 0.1]),
        );
        let from_wallpaper = resolve(&config, Some(dominant()));
        assert_eq!(
            from_wallpaper.lights[0],
            dominant().darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN)
        );
        assert_eq!(
            from_wallpaper.lights[2],
            dominant().lighten(TRAFFIC_ZOOM_LIGHTEN)
        );

        let accent = Rgba {
            r: 0.9,
            g: 0.3,
            b: 0.1,
            a: 1.0,
        };
        let without_wallpaper = resolve(&config, None);
        assert_eq!(
            without_wallpaper.lights[0],
            accent.darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN)
        );
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
            TrafficLightStyle::Standard,
            Some([accent.r, accent.g, accent.b]),
        );
        neutral.reduce_transparency = true;
        let neutral = resolve(&neutral, None);

        let mut accent_mode = appearance(
            ColorScheme::Light,
            WindowColorMode::Accent,
            TrafficLightStyle::Standard,
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
                    TrafficLightStyle::Standard,
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
                    TrafficLightStyle::Standard,
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
            TrafficLightStyle::Standard,
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
                TrafficLightStyle::Standard,
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
    fn the_wallpaper_behind_tints_the_window_and_the_sidebar_at_the_chosen_strength() {
        let red = Rgba::rgb8(200, 30, 30);
        for strength in [TintStrength::Subtle, TintStrength::Intense] {
            let config = AppearanceConfig {
                color_scheme: ColorScheme::Dark,
                tint_strength: strength,
                ..AppearanceConfig::default()
            };
            let neutral = resolve(&config, None);
            let tinting = WallpaperTinting {
                chroma: WALLPAPER_TINTING[1] * strength.factor(),
                lift: WALLPAPER_TINTING_LIFT[1] * strength.factor(),
            };
            assert_eq!(neutral.wallpaper_tinting, Some(tinting));
            let behind = neutral.behind_window(red);
            assert_eq!(behind.window_tint, Some((red, tinting.chroma)));
            assert_eq!(behind.menu_parent, tinting.take(BG_WINDOW_DARK, red));
            assert_close(
                behind.surfaces.window,
                tinting.take(BG_WINDOW_DARK, red),
                "the window background",
            );
            assert_eq!(
                behind.sidebar.bg,
                tinting.sidebar().take(neutral.sidebar.bg, red),
                "{strength:?}: the sidebar"
            );
        }
    }

    fn assert_close(actual: Rgba, expected: Rgba, what: &str) {
        let [r, g, b, _] = actual.to_rgba8().map(i32::from);
        let [er, eg, eb, _] = expected.to_rgba8().map(i32::from);
        assert!(
            (r - er).abs() <= 1 && (g - eg).abs() <= 1 && (b - eb).abs() <= 1,
            "{what}: {actual:?} is not {expected:?}"
        );
    }

    #[test]
    fn behind_the_wallpaper_of_the_macos_captures_a_window_turns_the_measured_color() {
        // ADR 0065: macOS 26 turns #1e1e1e into #23232e over a wallpaper that averages #46419b.
        let tinting = WallpaperTinting {
            chroma: WALLPAPER_TINTING[1],
            lift: WALLPAPER_TINTING_LIFT[1],
        };
        let window = tinting.take(Rgba::rgb8(0x1e, 0x1e, 0x1e), Rgba::rgb8(0x46, 0x41, 0x9b));
        let [r, g, b, _] = window.to_rgba8().map(i32::from);
        assert!(
            (r - 0x23).abs() <= 1 && (g - 0x23).abs() <= 1 && (b - 0x2e).abs() <= 1,
            "{window:?}"
        );
    }

    #[test]
    fn every_opaque_layer_shifts_as_much_as_the_window_and_a_translucent_one_shows_it() {
        use crate::backdrop::Oklab;
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            ..AppearanceConfig::default()
        };
        let neutral = resolve(&config, None);
        let behind = neutral.behind_window(Rgba::rgb8(70, 65, 155));
        let shift = |before: Rgba, after: Rgba| {
            let (p, q) = (Oklab::from_rgba(before), Oklab::from_rgba(after));
            [q.l - p.l, q.a - p.a, q.b - p.b]
        };
        let window = shift(neutral.surfaces.window, behind.surfaces.window);
        for (name, before, after) in [
            ("sunken", neutral.surfaces.sunken, behind.surfaces.sunken),
            ("surface", neutral.surfaces.surface, behind.surfaces.surface),
            ("raised", neutral.surfaces.raised, behind.surfaces.raised),
            ("control", neutral.surfaces.control, behind.surfaces.control),
            ("toolgroup", neutral.toolgroup.bg, behind.toolgroup.bg),
        ] {
            let layer = shift(before, after);
            for (taken, expected) in layer.iter().zip(window) {
                assert!(
                    (taken - expected).abs() < 0.003,
                    "{name}: {layer:?} vs {window:?}"
                );
            }
        }
        assert!(neutral.surfaces.line.a < 1.0);
        assert_eq!(behind.surfaces.line, neutral.surfaces.line);
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
                    TrafficLightStyle::Standard,
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
        let resolved = resolve(&config, None);
        let glass = resolved.sidebar.glass.unwrap();
        let sidebar = resolved.wallpaper_tinting.unwrap().sidebar();
        let behind = resolved.behind_window(red).sidebar.glass.unwrap();
        let tinted = |color| {
            lift(
                take_chroma_capped(
                    color,
                    red,
                    sidebar.chroma,
                    glass_chroma(ColorScheme::Dark).1,
                ),
                sidebar.lift,
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
            TrafficLightStyle::Standard,
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
            TrafficLightStyle::Standard,
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
    fn any_wallpaper_lifts_a_dark_window_by_the_same_lightness_and_gives_it_its_hue() {
        use crate::backdrop::Oklab;
        let neutral = resolve(
            &AppearanceConfig {
                color_scheme: ColorScheme::Dark,
                ..AppearanceConfig::default()
            },
            None,
        );
        let before = Oklab::from_rgba(neutral.surfaces.window);
        let lift = WALLPAPER_TINTING_LIFT[1] * (WALLPAPER_TINTING_TARGET - before.l);
        for wallpaper in [Rgba::rgb8(146, 197, 41), Rgba::rgb8(20, 24, 70)] {
            let after = Oklab::from_rgba(neutral.behind_window(wallpaper).surfaces.window);
            assert!((after.l - before.l - lift).abs() < 0.002, "{wallpaper:?}");
            let wall = Oklab::from_rgba(wallpaper);
            let taken = (after.a * wall.a + after.b * wall.b) / (wall.a.powi(2) + wall.b.powi(2));
            assert!((taken - WALLPAPER_TINTING[1]).abs() < 0.01, "{wallpaper:?}");
        }
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
            Some(WallpaperTinting {
                chroma: WALLPAPER_TINTING[0],
                lift: WALLPAPER_TINTING_LIFT[0],
            })
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
            resolve(&config, None)
                .wallpaper_tinting
                .map(|tinting| tinting.chroma),
            Some(WALLPAPER_TINTING[1])
        );
    }

    #[test]
    fn the_glass_blur_minimum_is_the_dock_as_designed() {
        let resolved = resolve(&AppearanceConfig::default(), None);
        assert_eq!(resolved.dock.glass, DOCK_GLASS_LIGHT);
        assert_eq!(resolved.dock.soften, 1.0);
    }

    #[test]
    fn only_the_translucent_dock_keeps_part_of_its_rim_on_the_sides() {
        let config = |color_scheme, reduce_transparency| AppearanceConfig {
            color_scheme,
            reduce_transparency,
            ..AppearanceConfig::default()
        };
        let light = resolve(&config(ColorScheme::Light, false), None);
        let dark = resolve(&config(ColorScheme::Dark, false), None);
        assert_eq!(light.dock.rim_side, DOCK_RIM_SIDE_LIGHT);
        assert_eq!(dark.dock.rim_side, DOCK_RIM_SIDE_DARK);
        assert_eq!(light.menu_glass.rim_side, 0.0);
        assert_eq!(light.notification_glass.rim_side, 0.0);
        assert_eq!(
            resolve(&config(ColorScheme::Light, true), None)
                .dock
                .rim_side,
            0.0
        );
    }

    fn over(glass: Rgba, backdrop: f32) -> [f32; 3] {
        [glass.r, glass.g, glass.b].map(|channel| backdrop + (channel - backdrop) * glass.a)
    }

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-5)
    }

    #[test]
    fn the_glass_blur_maximum_stops_short_of_the_frosted_glass_and_widens_its_taps() {
        let config = AppearanceConfig {
            glass_blur: GLASS_BLUR_FULL,
            ..AppearanceConfig::default()
        };
        let dock = resolve(&config, None).dock;
        let frost = GLASS_BLUR_FULL * GLASS_BLUR_FULL;
        let reached = frosted_glass(DOCK_GLASS_LIGHT, DOCK_GLASS_FROSTED_LIGHT, frost);
        assert!(close(over(dock.glass, 0.0), over(reached, 0.0)));
        assert!(dock.glass.a < 0.55);
        assert_eq!(dock.soften, 1.0 + frost * (GLASS_BLUR_SPREAD - 1.0));
    }

    #[test]
    fn halfway_the_glass_covers_any_backdrop_halfway_between_clear_and_frosted() {
        let half = frosted_glass(DOCK_GLASS_LIGHT, DOCK_GLASS_FROSTED_LIGHT, 0.5);
        for backdrop in [0.0, 0.5, 1.0] {
            let clear = over(DOCK_GLASS_LIGHT, backdrop);
            let frosted = over(DOCK_GLASS_FROSTED_LIGHT, backdrop);
            let between = std::array::from_fn(|i| (clear[i] + frosted[i]) / 2.0);
            assert!(close(over(half, backdrop), between));
        }
    }

    #[test]
    fn halfway_the_slider_has_moved_the_blur_and_the_tint_a_quarter_of_the_way() {
        let config = AppearanceConfig {
            glass_blur: GLASS_BLUR_FULL / 2.0,
            ..AppearanceConfig::default()
        };
        let dock = resolve(&config, None).dock;
        let quarter = GLASS_BLUR_FULL * GLASS_BLUR_FULL / 4.0;
        assert_eq!(dock.soften, 1.0 + quarter * (GLASS_BLUR_SPREAD - 1.0));
        let tint = frosted_glass(DOCK_GLASS_LIGHT, DOCK_GLASS_FROSTED_LIGHT, quarter);
        assert!(close(over(dock.glass, 0.5), over(tint, 0.5)));
    }

    #[test]
    fn reduced_transparency_keeps_the_dock_glass_solid_whatever_the_glass_blur() {
        let config = |glass_blur| AppearanceConfig {
            reduce_transparency: true,
            glass_blur,
            ..AppearanceConfig::default()
        };
        assert_eq!(
            resolve(&config(1.0), None).dock.glass,
            resolve(&config(0.0), None).dock.glass
        );
    }

    #[test]
    fn the_slider_widens_the_menu_blur_and_leaves_its_veil_and_chroma_as_designed() {
        let config = |glass_blur, reduce_transparency| AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            reduce_transparency,
            glass_blur,
            ..AppearanceConfig::default()
        };
        let clear = resolve(&config(0.0, false), None);
        let widened = resolve(&config(1.0, false), None);
        assert_eq!(clear.menu_glass.glass, MENU_GLASS_VEIL_DARK);
        assert!(
            clear.menu_glass.corner[2] <= 0.0,
            "the outline is whole at vertical"
        );
        assert_eq!(clear.menu_glass.soften, MENU_GLASS_SPREAD);
        assert_eq!(widened.menu_glass.soften, GLASS_BLUR_SPREAD);
        assert_eq!(widened.menu_glass.glass, clear.menu_glass.glass);
        assert_eq!(widened.menu_glass.tone, clear.menu_glass.tone);
        assert_eq!(clear.menu_glass.tone.unwrap().chroma, MENU_GLASS_CHROMA);
        let solid = resolve(&config(0.0, true), None);
        assert_eq!(solid.menu_glass.glass, clear.menu_glass.glass);
        assert_eq!(solid.menu_glass.tone, clear.menu_glass.tone);
        assert_eq!(solid.notification_glass, solid.dock);
    }

    #[test]
    fn a_menu_bends_nothing_and_has_its_own_chroma_lift_and_cap_per_scheme_and_the_dock_does_not() {
        for scheme in SCHEMES {
            let resolved = resolved_in(scheme, false);
            let tone = resolved.menu_glass.tone.unwrap();
            assert_eq!(resolved.menu_glass.lens[1], 0.0);
            assert_eq!(
                (tone.chroma, tone.lift, tone.cap),
                scheme.pick([
                    (
                        MENU_GLASS_CHROMA,
                        MENU_GLASS_LIFT_LIGHT,
                        MENU_GLASS_CAP_LIGHT
                    ),
                    (MENU_GLASS_CHROMA, MENU_GLASS_LIFT_DARK, MENU_GLASS_CAP_DARK),
                    (MENU_GLASS_CHROMA, MENU_GLASS_LIFT_DARK, MENU_GLASS_CAP_DARK),
                ])
            );
            assert_ne!(resolved.dock.tone.unwrap().cap, tone.cap);
        }
    }

    #[test]
    fn the_menu_veil_follows_the_scheme_and_not_the_transparency_setting() {
        for scheme in SCHEMES {
            let veil = scheme.pick([
                MENU_GLASS_VEIL_LIGHT,
                MENU_GLASS_VEIL_DARK,
                MENU_GLASS_VEIL_DARK,
            ]);
            for reduce_transparency in [false, true] {
                let resolved = resolved_in(scheme, reduce_transparency);
                assert_eq!(resolved.menu_glass.glass, veil);
            }
        }
    }

    const SCHEMES: [ColorScheme; 3] =
        [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint];
    const DARK_SCHEMES: [ColorScheme; 2] = [ColorScheme::Dark, ColorScheme::DarkTint];
    const WHITE: u8 = 0xff;
    const MACOS_GRAY: u8 = 0xa3;
    const BLACK: u8 = 0x00;
    const CHECKER: u8 = 0x80;
    const TOLERANCE: u8 = 8;
    type Panels = [(u8, u8); 4];
    const MENU_PANELS_DARK: Panels = [
        (WHITE, 0xa3),
        (MACOS_GRAY, 0x93),
        (BLACK, 0x2b),
        (CHECKER, 0x8c),
    ];
    const MENU_PANELS_LIGHT: Panels = [
        (WHITE, 0xf2),
        (MACOS_GRAY, 0xd1),
        (BLACK, 0x78),
        (CHECKER, 0xc2),
    ];
    const DOCK_PANELS_DARK: Panels = [
        (WHITE, 0xbe),
        (MACOS_GRAY, 0x8b),
        (BLACK, 0x0e),
        (CHECKER, 0x6c),
    ];
    const DOCK_PANELS_LIGHT: Panels = [
        (WHITE, 0xeb),
        (MACOS_GRAY, 0xa6),
        (BLACK, 0x34),
        (CHECKER, 0x91),
    ];
    const BANNER_OVER_THE_CHECKER_DARK: u8 = 0x89;
    const BANNER_OVER_THE_CHECKER_LIGHT: u8 = 0xb3;

    fn resolved_in(scheme: ColorScheme, reduce_transparency: bool) -> Resolved {
        resolve(
            &AppearanceConfig {
                color_scheme: scheme,
                reduce_transparency,
                ..AppearanceConfig::default()
            },
            None,
        )
    }

    fn grey_panel_over(glass: &DockSurface, backdrop: u8) -> Rgba {
        crate::menu::panel_over(glass, Rgba::rgb8(backdrop, backdrop, backdrop))
    }

    fn assert_panel_near(panel: Rgba, backdrop: u8, target: u8) {
        let [red, green, blue, _] = panel.to_rgba8();
        assert!(
            [red, green, blue]
                .iter()
                .all(|channel| channel.abs_diff(target) <= TOLERANCE),
            "over #{backdrop:02x}: #{red:02x}{green:02x}{blue:02x}, expected #{target:02x}"
        );
    }

    fn assert_panels(
        schemes: &[ColorScheme],
        panels: Panels,
        glass_of: fn(&Resolved) -> DockSurface,
    ) {
        for &scheme in schemes {
            let glass = glass_of(&resolved_in(scheme, false));
            for (backdrop, target) in panels {
                assert_panel_near(grey_panel_over(&glass, backdrop), backdrop, target);
            }
        }
    }

    fn assert_menu_ink_follows_the_panel(scheme: ColorScheme, panels: Panels) {
        let glass = resolved_in(scheme, false).menu_glass;
        for (backdrop, _) in panels {
            let luma = crate::menu::panel_luma(grey_panel_over(&glass, backdrop));
            let light_ink = scheme.is_dark() && backdrop == BLACK;
            assert_eq!(
                crate::menu::dark_ink_over(luma),
                !light_ink,
                "over #{backdrop:02x}"
            );
        }
    }

    #[test]
    fn the_dark_menu_panel_matches_macos_over_white_gray_black_and_the_checker() {
        assert_panels(&DARK_SCHEMES, MENU_PANELS_DARK, |resolved| {
            resolved.menu_glass
        });
    }

    #[test]
    fn the_light_menu_panel_matches_macos_over_white_gray_black_and_the_checker() {
        assert_panels(&[ColorScheme::Light], MENU_PANELS_LIGHT, |resolved| {
            resolved.menu_glass
        });
    }

    #[test]
    fn the_dark_menu_takes_light_ink_only_over_black() {
        for scheme in DARK_SCHEMES {
            assert_menu_ink_follows_the_panel(scheme, MENU_PANELS_DARK);
        }
    }

    #[test]
    fn the_light_menu_takes_dark_ink_over_every_backdrop_black_included() {
        assert_menu_ink_follows_the_panel(ColorScheme::Light, MENU_PANELS_LIGHT);
    }

    #[test]
    fn the_dark_dock_matches_macos_over_white_gray_black_and_the_checker() {
        assert_panels(&DARK_SCHEMES, DOCK_PANELS_DARK, |resolved| resolved.dock);
    }

    #[test]
    fn the_light_dock_matches_macos_over_white_gray_black_and_the_checker() {
        assert_panels(&[ColorScheme::Light], DOCK_PANELS_LIGHT, |resolved| {
            resolved.dock
        });
    }

    #[test]
    fn the_notification_banner_over_the_checker_matches_macos_in_every_scheme() {
        for scheme in SCHEMES {
            let target = scheme.pick([
                BANNER_OVER_THE_CHECKER_LIGHT,
                BANNER_OVER_THE_CHECKER_DARK,
                BANNER_OVER_THE_CHECKER_DARK,
            ]);
            let banner = resolved_in(scheme, false).notification_glass;
            assert_panel_near(grey_panel_over(&banner, CHECKER), CHECKER, target);
        }
    }

    #[test]
    fn with_transparency_reduced_the_dock_is_one_solid_per_scheme_over_any_backdrop() {
        for scheme in SCHEMES {
            let codes = scheme.pick([0xf1..=0xf1, 0x03..=0x0d, 0x03..=0x0d]);
            let solid = resolved_in(scheme, true).dock;
            let over_white = grey_panel_over(&solid, WHITE).to_rgba8();
            assert_eq!(over_white, grey_panel_over(&solid, BLACK).to_rgba8());
            let [red, green, blue, _] = over_white;
            assert!(
                [red, green, blue]
                    .iter()
                    .all(|channel| codes.contains(channel)),
                "{scheme:?}: #{red:02x}{green:02x}{blue:02x}"
            );
        }
    }

    #[test]
    fn with_transparency_reduced_the_menu_keeps_its_glass_and_the_notifications_take_the_dock_solid()
     {
        for scheme in SCHEMES {
            let clear = resolved_in(scheme, false);
            let solid = resolved_in(scheme, true);
            assert_eq!(
                (
                    solid.menu_glass.glass,
                    solid.menu_glass.tone,
                    solid.menu_glass.soften
                ),
                (
                    clear.menu_glass.glass,
                    clear.menu_glass.tone,
                    clear.menu_glass.soften
                )
            );
            assert_eq!(solid.notification_glass, solid.dock);
        }
    }

    #[test]
    fn the_notifications_wear_the_menus_tone_and_blur_in_the_docks_frame() {
        for scheme in SCHEMES {
            for glass_blur in [0.0, 0.5, 1.0] {
                let resolved = resolve(
                    &AppearanceConfig {
                        color_scheme: scheme,
                        glass_blur,
                        ..AppearanceConfig::default()
                    },
                    None,
                );
                let cards = resolved.notification_glass;
                let menu = resolved.menu_glass;
                assert_eq!((cards.tone, cards.soften), (menu.tone, menu.soften));
                assert_eq!(
                    (cards.lens, cards.corner),
                    (resolved.dock.lens, resolved.dock.corner)
                );
            }
        }
    }

    #[test]
    fn the_notifications_wear_the_menus_veil_in_dark_and_their_own_in_light() {
        for scheme in SCHEMES {
            let resolved = resolved_in(scheme, false);
            assert_eq!(
                resolved.notification_glass.glass,
                scheme.pick([
                    NOTIFICATION_GLASS_VEIL_LIGHT,
                    resolved.menu_glass.glass,
                    resolved.menu_glass.glass,
                ])
            );
        }
        assert_ne!(
            resolved_in(ColorScheme::Light, false)
                .notification_glass
                .glass,
            resolved_in(ColorScheme::Light, false).menu_glass.glass
        );
    }

    #[test]
    fn the_window_surface_is_within_8_of_the_measured_macos_title_bar_in_light_and_dark() {
        for (scheme, measured) in [(ColorScheme::Light, 0xfa), (ColorScheme::Dark, 0x28)] {
            let [red, green, blue, _] = resolved_in(scheme, false).surfaces.window.to_rgba8();
            for channel in [red, green, blue] {
                assert!(
                    channel.abs_diff(measured) <= 8,
                    "{scheme:?}: {channel:#04x} against {measured:#04x}"
                );
            }
        }
    }
}
