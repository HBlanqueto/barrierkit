use ctappearance::{
    AppearanceConfig, ColorScheme, Resolved, WindowTintScope, load_appearance, load_dominant,
    resolve,
};
use ctdesign::{
    BG_WINDOW_DARK, BG_WINDOW_DARK_TINT, BG_WINDOW_LIGHT, LockPalette, MenuPalette, Rgba,
    SHELL_DARK_OPAQUE, SHELL_DARK_TINT_OPAQUE, SHELL_LIGHT_OPAQUE,
};
use freya_components::theming::component_themes::{ColorsSheet, Theme};
use freya_components::theming::hooks::use_init_theme;
use freya_components::theming::themes::{dark_theme, light_theme};
use freya_core::prelude::{
    Color, State, WritableUtils, provide_context, try_consume_context, try_consume_root_context,
    use_hook, use_memo, use_side_effect,
};

pub fn color(c: Rgba) -> Color {
    let [r, g, b, a] = c.to_rgba8();
    Color::from_argb(a, r, g, b)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Toplevel,
    LayerShell,
}

pub fn surface_background(resolved: &Resolved, kind: SurfaceKind) -> Rgba {
    match kind {
        SurfaceKind::Toplevel => resolved.surfaces.window,
        SurfaceKind::LayerShell => {
            let mut bar = resolved.shell.bar;
            if resolved.window_tint_scope == WindowTintScope::Desktop
                && resolved.panel_tint.is_none()
                && let Some((tint, factor)) = resolved.window_tint
            {
                bar = bar.mix(tint, factor);
            }
            bar
        }
    }
}

pub fn resolved() -> Resolved {
    if let Some(state) = try_consume_context::<State<Resolved>>() {
        *state.read()
    } else {
        resolve(&load_appearance(), load_dominant())
    }
}

pub fn use_appearance(adjust: impl Fn(&mut AppearanceConfig) + 'static) {
    let root = try_consume_root_context::<State<Resolved>>();
    let adjusted = use_memo(move || {
        if let Some(root) = root {
            root.read();
        }
        let mut config = load_appearance();
        adjust(&mut config);
        resolve(&config, load_dominant())
    });
    let mut scoped = use_hook(|| {
        let scoped = State::create(*adjusted.read());
        provide_context(scoped);
        scoped
    });
    scoped.set_if_modified(*adjusted.read());
}

pub fn for_scheme(dark: bool) -> Theme {
    let appearance = AppearanceConfig {
        color_scheme: if dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        ..Default::default()
    };
    shell_theme(&resolve(&appearance, None))
}

pub fn use_shell_theme() {
    let mut theme = use_init_theme(|| shell_theme(&resolved()));
    let root = try_consume_root_context::<State<Resolved>>();
    use_side_effect(move || {
        if let Some(root) = root {
            theme.set(shell_theme(&root.read()));
        }
    });
}

fn shell_theme(resolved: &Resolved) -> Theme {
    let dark = matches!(resolved.scheme, ColorScheme::Dark | ColorScheme::DarkTint);
    let shell = resolved.shell;
    let ink = ctappearance::palette(resolved.scheme);
    let border = if dark { shell.tile } else { shell.bar };
    let accent = shell.tile_active;
    let mut theme = if dark { dark_theme() } else { light_theme() };
    theme.name = "tengen-shell";
    theme.colors = ColorsSheet {
        primary: color(accent),
        secondary: color(accent),
        tertiary: color(accent),
        success: color(accent),
        warning: color(accent),
        error: color(ink.danger),
        info: color(accent),
        background: color(shell.bar),
        surface_primary: color(shell.tile),
        surface_secondary: color(shell.tile),
        surface_tertiary: color(shell.bar),
        surface_inverse: color(shell.text),
        surface_inverse_secondary: color(shell.text),
        surface_inverse_tertiary: color(shell.text),
        border: color(border),
        border_focus: color(accent),
        border_disabled: color(border),
        text_primary: color(shell.text),
        text_secondary: color(ink.ink_muted),
        text_placeholder: color(ink.ink_faint),
        text_inverse: color(shell.bar),
        text_highlight: color(accent),
        focus: color(accent),
        active: color(accent),
        disabled: color(border),
        overlay: color(shell.bar),
        shadow: color(Rgba {
            a: ctdesign::THEME_SHADOW_OPACITY,
            ..shell.text
        }),
    };
    theme
}

fn neutral_background(scheme: ColorScheme, kind: SurfaceKind) -> Rgba {
    match (scheme, kind) {
        (ColorScheme::Dark, SurfaceKind::Toplevel) => BG_WINDOW_DARK,
        (ColorScheme::DarkTint, SurfaceKind::Toplevel) => BG_WINDOW_DARK_TINT,
        (_, SurfaceKind::Toplevel) => BG_WINDOW_LIGHT,
        (ColorScheme::Dark, SurfaceKind::LayerShell) => SHELL_DARK_OPAQUE.bar,
        (ColorScheme::DarkTint, SurfaceKind::LayerShell) => SHELL_DARK_TINT_OPAQUE.bar,
        (_, SurfaceKind::LayerShell) => SHELL_LIGHT_OPAQUE.bar,
    }
}

fn menu_background(resolved: &Resolved, kind: SurfaceKind) -> Rgba {
    if kind == SurfaceKind::Toplevel && !resolved.tint_parts.menus {
        return resolved.menu.bg;
    }
    let parent = match kind {
        SurfaceKind::Toplevel => resolved.menu_parent,
        _ => surface_background(resolved, kind),
    };
    let neutral = neutral_background(resolved.scheme, kind);
    let menu = resolved.menu.bg;
    if parent.r == neutral.r && parent.g == neutral.g && parent.b == neutral.b {
        return menu;
    }
    let lift = |parent: f32, menu: f32, neutral: f32| (parent + menu - neutral).clamp(0.0, 1.0);
    Rgba {
        r: lift(parent.r, menu.r, neutral.r),
        g: lift(parent.g, menu.g, neutral.g),
        b: lift(parent.b, menu.b, neutral.b),
        a: menu.a,
    }
}

pub fn menu_palette(resolved: &Resolved, kind: SurfaceKind) -> MenuPalette {
    let mut palette = resolved.menu;
    palette.bg = menu_background(resolved, kind);
    palette.separator_color = resolved.menu_separator(palette.bg);
    palette
}

pub fn apply_popover_metrics(mut palette: MenuPalette) -> MenuPalette {
    palette.padding = ctdesign::MENU_POPOVER.padding;
    palette.min_width = ctdesign::MENU_POPOVER.min_width;
    palette.item_height = ctdesign::MENU_POPOVER.item_height;
    palette.item_radius = ctdesign::MENU_POPOVER.item_radius;
    palette.item_padding_x = ctdesign::MENU_POPOVER.item_padding_start;
    palette.leading_width = ctdesign::MENU_POPOVER.leading_column;
    palette.mark_check = ctdesign::MENU_POPOVER.check;
    palette.separator_margin_y = ctdesign::MENU_POPOVER.separator_margin;
    palette
}

pub fn popover_palette(resolved: &Resolved, kind: SurfaceKind) -> MenuPalette {
    apply_popover_metrics(menu_palette(resolved, kind))
}

pub fn lock_palette(resolved: &Resolved) -> LockPalette {
    let mut palette = ctdesign::LOCK;
    palette.accent = resolved.accent;
    palette
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctappearance::{AppearanceConfig, ColorScheme, resolve};

    #[test]
    fn surface_backgrounds_follow_the_window_colour_mode_and_scope() {
        use ctappearance::{WindowColorMode, WindowTintScope};
        use ctdesign::{
            BG_WINDOW_LIGHT, PANEL_TINT_WALLPAPER, SHELL_LIGHT_OPAQUE, WINDOW_TINT_ACCENT,
            WINDOW_TINT_WALLPAPER,
        };

        let accent = Rgba {
            r: 0.2,
            g: 0.4,
            b: 0.9,
            a: 1.0,
        };
        let dominant = Rgba {
            r: 0.8,
            g: 0.2,
            b: 0.1,
            a: 1.0,
        };
        let config = |window_color, window_tint_scope| AppearanceConfig {
            color_scheme: ColorScheme::Light,
            accent_color: Some([accent.r, accent.g, accent.b]),
            reduce_transparency: true,
            window_color,
            window_tint_scope,
            ..Default::default()
        };

        let cases: [(&str, WindowColorMode, WindowTintScope, Rgba, Rgba); 6] = [
            (
                "neutral/windows",
                WindowColorMode::Neutral,
                WindowTintScope::Windows,
                BG_WINDOW_LIGHT,
                SHELL_LIGHT_OPAQUE.bar,
            ),
            (
                "neutral/desktop",
                WindowColorMode::Neutral,
                WindowTintScope::Desktop,
                BG_WINDOW_LIGHT,
                SHELL_LIGHT_OPAQUE.bar,
            ),
            (
                "accent/windows",
                WindowColorMode::Accent,
                WindowTintScope::Windows,
                BG_WINDOW_LIGHT.mix(accent, WINDOW_TINT_ACCENT[0]),
                SHELL_LIGHT_OPAQUE.bar,
            ),
            (
                "accent/desktop",
                WindowColorMode::Accent,
                WindowTintScope::Desktop,
                BG_WINDOW_LIGHT.mix(accent, WINDOW_TINT_ACCENT[0]),
                SHELL_LIGHT_OPAQUE.bar.mix(accent, WINDOW_TINT_ACCENT[0]),
            ),
            (
                "wallpaper/windows",
                WindowColorMode::Wallpaper,
                WindowTintScope::Windows,
                BG_WINDOW_LIGHT.mix(dominant, WINDOW_TINT_WALLPAPER[0]),
                SHELL_LIGHT_OPAQUE.bar,
            ),
            (
                "wallpaper/desktop",
                WindowColorMode::Wallpaper,
                WindowTintScope::Desktop,
                BG_WINDOW_LIGHT.mix(dominant, WINDOW_TINT_WALLPAPER[0]),
                SHELL_LIGHT_OPAQUE
                    .bar
                    .mix(dominant, PANEL_TINT_WALLPAPER[0]),
            ),
        ];

        for (name, window_color, window_tint_scope, toplevel, layer) in cases {
            let resolved = resolve(&config(window_color, window_tint_scope), Some(dominant));
            assert_eq!(
                surface_background(&resolved, SurfaceKind::Toplevel),
                toplevel,
                "{name}: toplevel background"
            );
            assert_eq!(
                surface_background(&resolved, SurfaceKind::LayerShell),
                layer,
                "{name}: layer-shell background"
            );
        }
    }

    #[test]
    fn selection_colours_follow_the_accent() {
        let orange = Rgba {
            r: 0.95,
            g: 0.5,
            b: 0.1,
            a: 1.0,
        };
        let config = |scheme| AppearanceConfig {
            color_scheme: scheme,
            accent_color: Some([orange.r, orange.g, orange.b]),
            ..Default::default()
        };

        let light = resolve(&config(ColorScheme::Light), None);
        assert_eq!(light.menu.hover_bg, orange, "light menu selection");
        assert!(
            light
                .menu
                .search_ring
                .contrast_ratio(ctdesign::BG_WINDOW_LIGHT)
                >= 3.0,
            "light focus ring"
        );
        assert_eq!(
            light.sidebar.palette.selected_bg, orange,
            "light sidebar selection"
        );
        assert_eq!(
            light.sidebar.palette.item_icon, orange,
            "light sidebar icon"
        );
        assert!(light.sidebar.palette.selected_ink.contrast_ratio(orange) >= 4.5);
        assert_ne!(light.menu.hover_bg, ctdesign::LIGHT.accent);

        let dark = resolve(&config(ColorScheme::Dark), None);
        assert_eq!(dark.menu.hover_bg, orange, "dark menu selection");
        assert!(
            dark.menu
                .search_ring
                .contrast_ratio(ctdesign::BG_WINDOW_DARK)
                >= 3.0,
            "dark focus ring"
        );
        assert_eq!(
            dark.sidebar.palette.selected_bg, orange,
            "dark sidebar selection"
        );
        assert_eq!(dark.sidebar.palette.item_icon, orange, "dark sidebar icon");
        assert!(dark.sidebar.palette.selected_ink.contrast_ratio(orange) >= 4.5);
    }

    #[test]
    fn a_layer_shell_menu_stays_neutral_when_only_windows_are_tinted_and_never_exceeds_white() {
        use ctappearance::{WindowColorMode, WindowTintScope};

        let config = |accent: Rgba, scope| AppearanceConfig {
            color_scheme: ColorScheme::Light,
            accent_color: Some([accent.r, accent.g, accent.b]),
            reduce_transparency: true,
            window_color: WindowColorMode::Accent,
            window_tint_scope: scope,
            ..Default::default()
        };
        let red = Rgba::rgb8(230, 26, 26);
        let windows = resolve(&config(red, WindowTintScope::Windows), None);
        assert_eq!(
            menu_palette(&windows, SurfaceKind::LayerShell).bg,
            windows.menu.bg
        );

        let pale = resolve(
            &config(Rgba::rgb8(255, 255, 255), WindowTintScope::Desktop),
            None,
        );
        let pale_menu = menu_palette(&pale, SurfaceKind::LayerShell).bg;
        assert!(pale_menu.r <= 1.0 && pale_menu.g <= 1.0 && pale_menu.b <= 1.0);
    }

    #[test]
    fn a_toplevel_menu_is_tinted_only_by_the_menus_part() {
        use ctappearance::{TintParts, WindowColorMode, WindowTintScope};

        let config = AppearanceConfig {
            color_scheme: ColorScheme::Light,
            accent_color: Some([0.9, 0.1, 0.1]),
            reduce_transparency: true,
            window_color: WindowColorMode::Accent,
            window_tint_scope: WindowTintScope::Windows,
            ..Default::default()
        };
        for menus in [false, true] {
            let mut resolved = resolve(&config, None);
            resolved.tint_parts = TintParts {
                sidebars: !menus,
                window: !menus,
                groups: !menus,
                controls: !menus,
                menus,
            };
            let tinted = menu_palette(&resolved, SurfaceKind::Toplevel).bg != resolved.menu.bg;
            assert_eq!(tinted, menus);
        }
    }

    #[test]
    fn tinted_menu_separators_stay_visible() {
        use ctappearance::WindowColorMode;
        use ctdesign::ACCENT_SWATCHES;

        let wallpapers = [
            ("dark", Rgba::rgb8(13, 13, 13)),
            ("mid", Rgba::rgb8(128, 115, 102)),
            ("light", Rgba::rgb8(235, 230, 219)),
        ];
        let config = |scheme, mode, accent: Option<[f32; 3]>| AppearanceConfig {
            color_scheme: scheme,
            accent_color: accent,
            reduce_transparency: true,
            window_color: mode,
            window_tint_scope: WindowTintScope::Desktop,
            ..Default::default()
        };

        for scheme in [ColorScheme::Light, ColorScheme::Dark, ColorScheme::DarkTint] {
            for (name, accent) in ACCENT_SWATCHES {
                let resolved = resolve(
                    &config(
                        scheme,
                        WindowColorMode::Accent,
                        Some([accent.r, accent.g, accent.b]),
                    ),
                    None,
                );
                for kind in [SurfaceKind::Toplevel, SurfaceKind::LayerShell] {
                    let palette = menu_palette(&resolved, kind);
                    let ratio = palette.separator_color.contrast_ratio(palette.bg);
                    assert!(
                        ratio >= 1.25,
                        "{scheme:?}/acento {name}/{kind:?}: {ratio:.3}:1"
                    );
                }
            }

            for (name, tint) in wallpapers {
                let resolved = resolve(
                    &config(scheme, WindowColorMode::Wallpaper, None),
                    Some(tint),
                );
                for kind in [SurfaceKind::Toplevel, SurfaceKind::LayerShell] {
                    let palette = menu_palette(&resolved, kind);
                    let ratio = palette.separator_color.contrast_ratio(palette.bg);
                    assert!(
                        ratio >= 1.25,
                        "{scheme:?}/fondo {name}/{kind:?}: {ratio:.3}:1"
                    );
                }
            }

            let neutral = resolve(&config(scheme, WindowColorMode::Neutral, None), None);
            for kind in [SurfaceKind::Toplevel, SurfaceKind::LayerShell] {
                assert_eq!(
                    menu_palette(&neutral, kind).separator_color,
                    neutral.menu.separator_color,
                    "{scheme:?}/{kind:?}: neutro keeps today's colour"
                );
            }
        }
    }
}
