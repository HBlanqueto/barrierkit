use ctdesign::{
    ACCENT_CHROMA_GAIN, ACCENT_MIN_CONTRAST_ON_WINDOW, Accent, AccentColors, BG_HOVER_DARK,
    BG_HOVER_DARK_TINT, BG_HOVER_LIGHT, BG_RAISED_DARK, BG_RAISED_DARK_TINT, BG_RAISED_LIGHT,
    BG_SUNKEN_DARK, BG_SUNKEN_DARK_TINT, BG_SUNKEN_LIGHT, BG_SURFACE_DARK, BG_SURFACE_DARK_TINT,
    BG_SURFACE_LIGHT, BG_WINDOW_DARK, BG_WINDOW_DARK_TINT, BG_WINDOW_LIGHT, CONTRAST_DARK,
    CONTRAST_DARK_TINT, CONTRAST_LIGHT, DARK, DARK_TINT, GLASS, LAYER_FACTOR_BG_SUNKEN,
    LAYER_FACTOR_CONTROL, LAYER_FACTOR_LINE, LAYER_FACTOR_RAISED, LAYER_FACTOR_SURFACE, LIGHT,
    MENU_DARK, MENU_DARK_OPAQUE, MENU_DARK_TINT, MENU_DARK_TINT_OPAQUE, MENU_LIGHT,
    MENU_LIGHT_OPAQUE, MENU_SEPARATOR_INK_MIX, MenuPalette, ON_ACCENT_MIN_CONTRAST, Palette, Rgba,
    SHELL_DARK, SHELL_DARK_OPAQUE, SHELL_DARK_TINT, SHELL_DARK_TINT_OPAQUE, SHELL_LIGHT,
    SHELL_LIGHT_OPAQUE, SIDEBAR_DARK, SIDEBAR_DARK_OPAQUE, SIDEBAR_DARK_TINT,
    SIDEBAR_DARK_TINT_OPAQUE, SIDEBAR_GLASS_BASE_DARK, SIDEBAR_GLASS_BASE_DARK_TINT,
    SIDEBAR_GLASS_BASE_LIGHT, SIDEBAR_GLASS_SOLID_DARK, SIDEBAR_GLASS_SOLID_DARK_TINT,
    SIDEBAR_GLASS_SOLID_LIGHT, SIDEBAR_LIGHT, SIDEBAR_LIGHT_OPAQUE, SIDEBAR_PALETTE_DARK,
    SIDEBAR_PALETTE_DARK_TINT, SIDEBAR_PALETTE_LIGHT, ShellPalette, SidebarPalette,
    TOOLBAR_BG_DARK, TOOLBAR_BG_DARK_TINT, TOOLBAR_BG_LIGHT, TOOLBAR_LINE_DARK,
    TOOLBAR_LINE_DARK_TINT, TOOLBAR_LINE_LIGHT, TOOLGROUP_BG_DARK, TOOLGROUP_BG_DARK_OPAQUE,
    TOOLGROUP_BG_DARK_TINT, TOOLGROUP_BG_DARK_TINT_OPAQUE, TOOLGROUP_BG_LIGHT,
    TOOLGROUP_BG_LIGHT_OPAQUE, TOOLGROUP_PALETTE_DARK, TOOLGROUP_PALETTE_DARK_TINT,
    TOOLGROUP_PALETTE_LIGHT, TRAFFIC_LIGHTS_CONTRAST_DARK, TRAFFIC_LIGHTS_CONTRAST_DARK_TINT,
    TRAFFIC_LIGHTS_CONTRAST_LIGHT, TRAFFIC_LIGHTS_DARK, TRAFFIC_LIGHTS_DARK_TINT,
    TRAFFIC_LIGHTS_LIGHT, ToolgroupPalette, TrafficLooks, WALLPAPER_TINTING,
    WALLPAPER_TINTING_LIFT, WINDOW_TINT_ACCENT,
};

use crate::{
    AppearanceConfig, ColorScheme, ScrollBarClick, ScrollBars, TintParts, TrafficLightStyle,
    WindowColorMode, accent,
    backdrop::{WallpaperTinting, lift, shift_like, take_chroma, take_chroma_capped},
    icon_style::IconStyle,
    traffic,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarSurface {
    pub bg: Rgba,
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
    pub toolbar: Rgba,
    pub toolbar_line: Rgba,
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
    pub accent_colors: AccentColors,
    pub accent: Rgba,
    pub control_accent: Rgba,
    pub toggle_accent: Rgba,
    pub ring_accent: Rgba,
    pub toggle_accent_disabled: Rgba,
    pub automatic_accent: bool,
    pub on_accent: Rgba,
    pub focus_ring: Rgba,
    pub link: Rgba,
    pub window_tint: Option<(Rgba, f32)>,
    pub bar_tint: Option<(Rgba, f32)>,
    pub wallpaper_tinting: Option<WallpaperTinting>,
    pub traffic_lights: TrafficLooks,
    pub contrast: bool,
    pub reduce_transparency: bool,
    pub reduced_motion: bool,
    pub show_borders: bool,
    pub differentiate_without_color: bool,
    pub text_size: f32,
    pub icon_style: IconStyle,
    pub icon_dark: bool,
    pub icon_tint: Rgba,
    pub scroll_bars: ScrollBars,
    pub scroll_bar_click: ScrollBarClick,
    pub shell: ShellPalette,
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
                self.surfaces =
                    tinted_surfaces(scheme, &palette_in(scheme, self.contrast), parts, paint);
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
                self.surfaces =
                    tinted_surfaces(scheme, &palette_in(scheme, self.contrast), parts, paint);
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
        if self.contrast && !self.automatic_accent {
            self.focus_ring = reaching(
                self.accent,
                &[self.surfaces.window],
                ACCENT_MIN_CONTRAST_ON_WINDOW,
                scheme,
            );
        }
        self
    }

    pub fn palette(&self) -> Palette {
        palette_in(self.scheme, self.contrast)
    }

    pub fn menu_separator(&self, panel_bg: Rgba) -> Rgba {
        if self.window_tint.is_none() || panel_bg == self.menu.bg {
            self.menu.separator_color
        } else {
            panel_bg.mix(self.menu.fg, MENU_SEPARATOR_INK_MIX)
        }
    }
}

// How much of the wallpaper's chroma the sidebar glass takes, and the chroma it saturates towards (docs/barrierkit/10-sidebar.md#glass).
pub fn glass_chroma(scheme: ColorScheme) -> (f32, f32) {
    if scheme.is_dark() {
        (GLASS.chroma_dark, GLASS.chroma_cap_dark)
    } else {
        (GLASS.chroma_light, GLASS.chroma_cap_light)
    }
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

pub fn palette_in(scheme: ColorScheme, contrast: bool) -> Palette {
    let base = palette(scheme);
    if contrast {
        base.contrasted(scheme.pick([CONTRAST_LIGHT, CONTRAST_DARK, CONTRAST_DARK_TINT]))
    } else {
        base
    }
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

const SIDEBAR_BG: [[Rgba; 3]; 2] = [
    [SIDEBAR_LIGHT, SIDEBAR_DARK, SIDEBAR_DARK_TINT],
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
    let sidebar = SidebarSurface {
        bg: scheme.pick(SIDEBAR_BG[layer]),
        palette,
        glass,
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

enum Chosen {
    Named(&'static Accent),
    Custom(Rgba),
}

fn chosen_accent(appearance: &AppearanceConfig, dominant: Option<Rgba>) -> Option<Chosen> {
    let custom =
        |color: Rgba| accent::named_by_color(color).map_or(Chosen::Custom(color), Chosen::Named);
    appearance
        .accent
        .as_deref()
        .and_then(accent::named)
        .map(Chosen::Named)
        .or_else(|| {
            appearance.accent_color.map(|[r, g, b]| {
                custom(Rgba {
                    r: r.clamp(0.0, 1.0),
                    g: g.clamp(0.0, 1.0),
                    b: b.clamp(0.0, 1.0),
                    a: 1.0,
                })
            })
        })
        .or_else(|| {
            appearance
                .wallpaper_accent
                .then(|| dominant.map_or_else(|| Chosen::Named(accent::graphite()), Chosen::Custom))
        })
}

/// `native_lights`: the system draws its own traffic lights (macOS), so only the graphite accent changes them.
pub fn resolve(
    appearance: &AppearanceConfig,
    dominant: Option<Rgba>,
    native_lights: bool,
) -> Resolved {
    let scheme = appearance.color_scheme;
    let base = palette_in(scheme, appearance.contrast);
    let index = usize::from(scheme.is_dark());

    let chosen = chosen_accent(appearance, dominant);
    let colors = match &chosen {
        None => accent::blue().colors(scheme.is_dark(), appearance.contrast),
        Some(Chosen::Named(named)) => named.colors(scheme.is_dark(), appearance.contrast),
        Some(Chosen::Custom(color)) => {
            let control = if appearance.contrast {
                legible_on_accent(*color, base.on_accent)
            } else {
                *color
            };
            accent::derived(control, scheme.is_dark(), appearance.contrast)
        }
    };
    let normal_accent = colors.control;
    let on_accent = base.on_accent;
    let accent = match chosen {
        Some(Chosen::Custom(_)) => legible_on_accent(colors.selected_content, on_accent),
        _ => colors.selected_content,
    };

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

    let standard_lights = if appearance.contrast {
        scheme.pick([
            TRAFFIC_LIGHTS_CONTRAST_LIGHT,
            TRAFFIC_LIGHTS_CONTRAST_DARK,
            TRAFFIC_LIGHTS_CONTRAST_DARK_TINT,
        ])
    } else {
        scheme.pick([
            TRAFFIC_LIGHTS_LIGHT,
            TRAFFIC_LIGHTS_DARK,
            TRAFFIC_LIGHTS_DARK_TINT,
        ])
    };
    let lights = match (native_lights, &chosen) {
        (false, _) => appearance.traffic_lights,
        (true, Some(Chosen::Named(named))) if named.name == "graphite" => TrafficLightStyle::Accent,
        (true, _) => TrafficLightStyle::Standard,
    };
    let traffic_lights = traffic::looks(lights, &standard_lights, normal_accent, dominant);

    let (mut shell, mut menu, mut sidebar, toolgroup) =
        chrome_surfaces(scheme, appearance.transparency_reduced());
    let surfaces = tinted_surfaces(scheme, &base, appearance.tint_parts, |color, _| color);
    // WCAG 2.2 §1.4.11: with Increase Contrast a custom accent's focus ring needs 3:1 on the window.
    let focus_ring = match chosen {
        Some(Chosen::Custom(_)) if appearance.contrast => reaching(
            colors.focus_ring,
            &[scheme.pick(BG_WINDOW)],
            ACCENT_MIN_CONTRAST_ON_WINDOW,
            scheme,
        ),
        _ => colors.focus_ring,
    };
    let link = reaching(
        accent,
        &[surfaces.window, surfaces.surface],
        ON_ACCENT_MIN_CONTRAST,
        scheme,
    );
    menu.hover_bg = if appearance.contrast || matches!(chosen, Some(Chosen::Custom(_))) {
        accent
    } else {
        normal_accent
    };
    menu.hover_fg = on_accent;
    sidebar.palette.selected_bg = if matches!(chosen, Some(Chosen::Custom(_))) {
        accent
    } else {
        colors.sidebar_selection
    };
    sidebar.palette.item_icon = normal_accent;
    sidebar.palette.selected_ink = on_accent;
    shell.tile_active = accent;

    let untinted = Resolved {
        scheme,
        accent_colors: colors,
        accent,
        control_accent: normal_accent,
        toggle_accent: colors.toggle,
        ring_accent: colors.ring,
        toggle_accent_disabled: colors.toggle_disabled,
        automatic_accent: chosen.is_none(),
        on_accent,
        focus_ring,
        link,
        window_tint: None,
        bar_tint,
        wallpaper_tinting,
        traffic_lights,
        contrast: appearance.contrast,
        reduce_transparency: appearance.transparency_reduced(),
        reduced_motion: appearance.reduced_motion,
        show_borders: appearance.show_borders,
        differentiate_without_color: appearance.differentiate_without_color,
        text_size: appearance
            .text_sizes
            .barrierkit
            .unwrap_or(appearance.text_size),
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
    let [window, sunken, surface, raised, hover] = scheme.pick([
        [
            BG_WINDOW_LIGHT,
            BG_SUNKEN_LIGHT,
            BG_SURFACE_LIGHT,
            BG_RAISED_LIGHT,
            BG_HOVER_LIGHT,
        ],
        [
            BG_WINDOW_DARK,
            BG_SUNKEN_DARK,
            BG_SURFACE_DARK,
            BG_RAISED_DARK,
            BG_HOVER_DARK,
        ],
        [
            BG_WINDOW_DARK_TINT,
            BG_SUNKEN_DARK_TINT,
            BG_SURFACE_DARK_TINT,
            BG_RAISED_DARK_TINT,
            BG_HOVER_DARK_TINT,
        ],
    ]);
    let mix = |color: Rgba, layer: f32, enabled: bool| {
        if enabled { paint(color, layer) } else { color }
    };
    Surfaces {
        window: mix(window, 1.0, parts.window),
        titlebar_inactive: mix(base.titlebar_bg.unfocused, 1.0, parts.window),
        toolbar: mix(
            scheme.pick([TOOLBAR_BG_LIGHT, TOOLBAR_BG_DARK, TOOLBAR_BG_DARK_TINT]),
            1.0,
            parts.window,
        ),
        toolbar_line: scheme.pick([
            TOOLBAR_LINE_LIGHT,
            TOOLBAR_LINE_DARK,
            TOOLBAR_LINE_DARK_TINT,
        ]),
        sunken: mix(sunken, LAYER_FACTOR_BG_SUNKEN, parts.window),
        surface: mix(surface, LAYER_FACTOR_SURFACE, parts.groups),
        raised: mix(raised, LAYER_FACTOR_RAISED, parts.menus),
        control: base.control,
        hover: mix(hover, LAYER_FACTOR_CONTROL, parts.controls),
        line: mix(base.separator, LAYER_FACTOR_LINE, parts.groups),
    }
}

// WCAG 2.2 §1.4.3: light ink on the accent needs 4.5:1; as in macOS the ink stays and the accent darkens.
fn legible_on_accent(accent: Rgba, light: Rgba) -> Rgba {
    let (hue, saturation, mut lightness) = accent.to_hsl();
    let mut fill = accent;
    while fill.contrast_ratio(light) < ON_ACCENT_MIN_CONTRAST && lightness > 0.0 {
        lightness = (lightness - 0.01).max(0.0);
        fill = Rgba::from_hsl(hue, saturation, lightness, 1.0);
    }
    fill
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
    lightness.clamp(0.0, 1.0)
}
