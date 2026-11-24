use bk::theme::{color, resolved};
use ctappearance::{AppearanceConfig, ColorScheme, Resolved, resolve};
use ctdesign::Rgba;
use freya_components::theming::component_themes::{ColorsSheet, Theme};
use freya_components::theming::hooks::use_init_theme;
use freya_components::theming::themes::{dark_theme, light_theme};
use freya_core::prelude::{State, WritableUtils, try_consume_root_context, use_side_effect};

const LINUX_LIGHTS: bool = false;

pub fn for_scheme(dark: bool) -> Theme {
    let appearance = AppearanceConfig {
        color_scheme: if dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        ..Default::default()
    };
    shell_theme(&resolve(&appearance, None, LINUX_LIGHTS))
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
        text_highlight: color(resolved.accent_colors.selected_text),
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
