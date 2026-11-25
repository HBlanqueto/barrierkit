use crate::platform::backdrop::SystemWindowBackground;
use crate::platform::{NATIVE_LIGHTS, Popups, appearance, dominant};
use ctappearance::menu::{MenuInk, ink_over};
use ctappearance::{AppearanceConfig, ColorScheme, Resolved, resolve};
use ctdesign::{
    BG_WINDOW_DARK, BG_WINDOW_DARK_TINT, BG_WINDOW_LIGHT, MenuPalette, Rgba, SHELL_DARK_OPAQUE,
    SHELL_DARK_TINT_OPAQUE, SHELL_LIGHT_OPAQUE,
};
use freya_core::current_context::CurrentContext;
use freya_core::prelude::{
    Color, State, WritableUtils, provide_context, try_consume_context, try_consume_root_context,
    use_hook, use_memo,
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
        SurfaceKind::LayerShell => resolved
            .bar_tint
            .map_or(resolved.shell.bar, |(tint, factor)| {
                resolved.shell.bar.mix(tint, factor)
            }),
    }
}

/// `palette` cleared for the compositor's glass, its ink following the glass under the popup; unchanged with Increase Contrast.
pub fn over_glass(palette: MenuPalette) -> MenuPalette {
    if !menus_keep_glass(&resolved()) {
        return palette;
    }
    let ink = try_consume_root_context::<Popups>()
        .and_then(|popups| *popups.glass_luma.read())
        .map_or(
            MenuInk {
                fg: palette.glass_fg,
                shortcut: palette.glass_shortcut_color,
                disabled: palette.glass_disabled_fg,
            },
            ink_over,
        );
    MenuPalette {
        bg: Rgba::TRANSPARENT,
        search_bg: ink.disabled,
        fg: ink.fg,
        shortcut_color: ink.shortcut,
        chevron_color: ink.fg,
        disabled_fg: ink.disabled,
        ..palette
    }
}

fn menus_keep_glass(resolved: &Resolved) -> bool {
    !resolved.contrast
}

// The platform's window material shows where the kit paints none, unless the theme tints the window.
pub(crate) fn window_material(resolved: &Resolved, system_draws: bool) -> bool {
    system_draws && resolved.window_tint.is_none()
}

pub(crate) fn window_material_here() -> bool {
    window_material(
        &resolved(),
        try_consume_root_context::<SystemWindowBackground>().is_some_and(|system| system.0),
    )
}

pub fn resolved() -> Resolved {
    if let Some(state) =
        CurrentContext::try_with(|_| try_consume_context::<State<Resolved>>()).flatten()
    {
        *state.read()
    } else {
        resolve(&appearance(), dominant(), NATIVE_LIGHTS)
    }
}

pub fn use_appearance(adjust: impl Fn(&mut AppearanceConfig) + 'static) {
    let root = try_consume_root_context::<State<Resolved>>();
    let adjusted = use_memo(move || {
        if let Some(root) = root {
            root.read();
        }
        let mut config = appearance();
        adjust(&mut config);
        resolve(&config, dominant(), NATIVE_LIGHTS)
    });
    let mut scoped = use_hook(|| {
        let scoped = State::create(*adjusted.read());
        provide_context(scoped);
        scoped
    });
    scoped.set_if_modified(*adjusted.read());
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
    if resolved.contrast {
        let ink = ctappearance::palette_in(resolved.scheme, true);
        palette.fg = ink.ink_strong;
        palette.chevron_color = ink.ink_strong;
        palette.shortcut_color = ink.ink_faint;
        palette.disabled_fg = ink.ink_faint;
        palette.separator_color = ink.separator_strong;
    }
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
