# 7. Glass and blur

**Goal:** give a shell surface the desktop's glass: the compositor's blur
behind it, the color the glass shows over that blur, and ink and a theme
that stay legible on it.
**Scope:** `bk_shell::set_blur_region`, the glass models of `ctshell`
(`menu`, `shell_glass`, `liquid_glass`, `resolve`, `material`),
`ctshell::traffic`, the shell fields of `ctappearance::Resolved` and
`bk_shell::{use_shell_theme, for_scheme}`. The glass inside an app's window is
in [15. Backdrops and media](../barrierkit/15-backdrops-and-media.md#who-draws-the-blur)
and [18. Platform](../barrierkit/18-platform.md#glass-over-the-windows-content);
the tokens are in [9](09-shell-tokens.md).
**Reader:** whoever draws a shell surface, or the glass in Malaysia's
compositor.
**Input:** a layer surface ([3](03-layer-surfaces.md)) and the user's
appearance ([4](../barrierkit/04-appearance-and-tokens.md#user-appearance-ctappearance)).
**Output:** a blurred backdrop behind the surface, the color the compositor
draws there, and ink and Freya components that follow it.

## Steps

1. Call `use_shell_theme()` first in the surface's root component, so Freya's
   own components take the shell's colors ([Shell theme](#shell-theme)).
2. Ask the compositor to blur behind the glass part of the surface, in
   surface coordinates ([`set_blur_region`](#set_blur_region)):

   ```rust
   set_blur_region(BlurRegion { rects: vec![(0, 0, width, height)] });
   ```

3. Read the resolved appearance: `bk::theme::resolved()` inside a component,
   `ctappearance::resolve(&load_appearance(), load_dominant(), false)` in the
   compositor ([The shell colors of `Resolved`](#the-shell-colors-of-resolved)).
4. Take the wallpaper color behind the surface with
   `load_wallpaper_state().grid_for(desktop)` and `grid.behind(output, rect)`
   ([4](../barrierkit/04-appearance-and-tokens.md#wallpaper-behind-a-window)).
5. Compute what the glass shows over it
   ([Veil and tone](#veil-and-tone)):

   ```rust
   let shown = ctshell::menu::panel_over(&resolved.menu_glass, behind);
   ```

6. Draw the ink for that color:
   `ctappearance::menu::ink_over(panel_luma(shown)).fg`
   ([4](../barrierkit/04-appearance-and-tokens.md#glass-over-a-backdrop)).

## Who draws the blur

On Linux the compositor draws every blur, as macOS's render server does
([ADR 0091](../decisions.md)). A shell surface paints no backdrop blur: it
asks for one and paints over it.

| Request | Global | Sent by | The compositor blurs |
|---|---|---|---|
| `set_blur_region` | `ext_background_effect_manager_v1` version 1 | A layer or lock surface of `bk-shell` | What lies behind the surface, inside the region |
| `set_backdrop`, `set_scroll_edge` | `infinity_window_style_v1` version 10 | The kit's runner, for the glass inside a window | What the window draws below its chrome ([15](../barrierkit/15-backdrops-and-media.md#who-draws-the-blur)) |

Neither request carries a blur radius. Malaysia blurs behind a surface with a dual
Kawase of `BACKGROUND_BLUR_PASSES` (3) halvings and a spread of
`BACKGROUND_BLUR_OFFSET` (3.0). `shell_glass::kawase_sigma(passes, offset)`
returns the Gaussian sigma that blur matches.

### `set_blur_region`

`set_blur_region(BlurRegion { rects })` takes `(x, y, width, height)`
rectangles in surface-local logical pixels. The runner applies it to the
surface that called it:

1. It builds a `wl_region` from `rects`.
2. It gets the surface's `ext_background_effect_surface_v1` the first time,
   and destroys those of surfaces that closed.
3. It sends `set_blur_region` with the region. An empty `rects` sends none,
   which removes the blur.
4. It commits the surface.

The region stays until the next call: send it again when the rectangles
change. Without the global the call does nothing.

## The glass a surface shows

`ctshell` holds three models of the color a glass shows over its backdrop.
The compositor draws with them; the shell reads them to know the color under
its ink.

| Model | Surfaces | Function | Values |
|---|---|---|---|
| [Veil and tone](#veil-and-tone) | The Dock, the menus, the notifications | `menu::panel_over(&surface, behind)` | `Resolved.dock`, `menu_glass`, `notification_glass` |
| [Measured pipeline](#measured-pipeline) | `menu` (also the banner), `spotlight`, `apps`, `notification-card`, `control-center`, `control-center-panel`, `dock` and `spaces-bar`, dark only | `shell_glass::over(&glass, blurred)` | `SHELL_GLASS_*` |
| [Liquid Glass](#liquid-glass) | The `regular` and `clear` glass a window requests | `liquid_glass::render(..)` | `LIQUID_GLASS_*` |

The pipeline misses the menu and the Dock by 1.7 levels at most, where a veil
and a cap miss them by over 6
([measurements](../research/macos-blur.md#14-desktop-surfaces-as-a-pipeline-measured-2026-11-22)).
The veil and tone stay while Malaysia reads them.

### Veil and tone

`menu::panel_over(&surface, behind)` takes a `DockSurface` and the color
behind it, and returns the color the compositor's shader shows:

1. The veil (`surface.glass`) mixes over `behind` at its alpha.
2. A surface without a `tone` stops here: the Dock with transparency reduced.
3. `tone.lift` adds to the OKLab lightness of the result.
4. A soft minimum keeps that lightness under `tone.cap`.
5. The color takes the OKLab hue of `behind`, its chroma times `tone.chroma`
   and scaled down by the share of lightness the cap removed.

| Surface | Light | Dark and dark tint | Transparency reduced |
|---|---|---|---|
| `dock` | The theme's `DOCK_GLASS` with `DOCK_TONE_LIGHT` | The theme's `DOCK_GLASS` with `DOCK_TONE_DARK` | The theme's `DOCK_GLASS_SOLID`, no tone |
| `menu_glass` | `MENU_GLASS_VEIL_LIGHT`, `MENU_GLASS_LIFT_LIGHT`, `MENU_GLASS_CAP_LIGHT` | `MENU_GLASS_VEIL_DARK`, `MENU_GLASS_LIFT_DARK`, `MENU_GLASS_CAP_DARK` | Unchanged |
| `notification_glass` | `NOTIFICATION_GLASS_VEIL_LIGHT` with the tone of `menu_glass` | The veil and tone of `menu_glass` | `dock` |

`menu_glass` takes `MENU_GLASS_CHROMA` as its chroma and `MENU_GLASS_SPREAD`
as its `soften` in every scheme. `design::dock_divider(surface, backdrop)` is
the white or black overlay that moves the glass's luminance over `backdrop` by
`surface.divider`.

### Measured pipeline

`design::ShellGlass` is the glass of one surface in one scheme, as macOS
27.2.1 composes it: `sigma`, `halo_sigma`, `halo_weight`, `tone` (12 levels)
and `chroma` (5 gains). The compositor draws it in two steps:

1. It blurs the backdrop by `sigma` px and adds `halo_weight` of a
   `halo_sigma` px blur when the weight is not 0. A Gaussian on a downsampled
   backdrop, or a dual Kawase of `kawase_sigma(passes, offset)`, gives the same
   sigma.
2. `shell_glass::over(&glass, blurred)` maps each channel `c` of the blurred
   pixel, in the display's device values, to `tone(Y) + chroma(Y) (c - Y)`.
   `Y` is its Rec. 709 luma; `tone` interpolates `glass.tone` at
   `SHELL_GLASS_LEVELS` and `chroma` interpolates `glass.chroma` at
   `SHELL_GLASS_CHROMA_KNOTS`. It does not clamp.

`SHELL_GLASS_SPACES_BAR_DARK` is a `MaterialLook`: draw it with
`ctappearance::material::over(&look, backdrop)`. The menu bar has no token: it
shows the backdrop unchanged. Light, Reduce Transparency and Increase Contrast
have no token, because they were not measured.

### Liquid Glass

`ctshell::liquid_glass` is the CPU reference of the glass a window requests.
Its values are device values at scale 1, as macOS 27.2.1 draws them; the
compositor applies them in its own output space
([the model](../research/macos-liquid-glass.md#6-the-model)).

1. `looks(glass, scheme)` returns the `GlassSet` of `Glass::Regular` or
   `Glass::Clear`; dark tint takes dark's.
2. `look(&set, state)` picks the look of a `GlassState`: `Active`,
   `Inactive`, `Reduced`, `IncreasedContrast` or `ReducedContrast`.
3. `adapted(&look, &set.scale, shorter)` applies the size rules for the
   shape's shorter side.
4. For the active look, `tint(glass, scheme)` returns the `GlassTint` of the
   Appearance › Liquid Glass slider, and `tinted(&set, &tint, amount, shorter)`
   the look at that position, with its blur fill (`Tinted`). At 0 the look is
   unchanged ([the slider](../research/macos-liquid-glass.md#13-the-appearance--liquid-glass-slider-nsglasstintamount-measured-2026-11-22)).
5. `render(&set, &tint, amount, state, &backdrop, &shape)` draws the glass of
   a `Shape` (`x`, `y`, `width`, `height`, `radius`) over an `Image` (`width`,
   `height`, `rgb`) and returns the result. Only `Active` takes the tint.

`face`, `blur_sigma`, `lens_offset`, `rim_levels`, `smooth_union` and
`rounded_rect_distance` are its stages, public for the compositor's parity
tests. The fields of each type are in
[9](09-shell-tokens.md#liquid-glass-types).

## Panel glass per scheme

The compositor draws the glass panel of the kit's sidebar
([Glass](../barrierkit/10-sidebar.md#glass)). `ctshell::resolve` reads its
per-scheme values from `ctdesign::GLASS`
([4](../barrierkit/04-appearance-and-tokens.md#view-metrics));
`ctappearance::glass_chroma(scheme)` gives its chroma and the chroma it
saturates towards.

| Function | Returns |
|---|---|
| `glass_lightness(scheme)` | The share of the lightness behind the glass that shows through |
| `glass_rim_lift(scheme)` | How much lighter the panel's border is than the fill under it, and how much more where the wallpaper shows |
| `glass_blur_passes(scheme)` | How many times the glass blurs what is behind it |
| `glass_edge(scheme) -> GlassEdge` | The wallpaper along the left, top and bottom edges: `intensity`, `span`, `chroma`, `lightness`, `cap` and `offset` |
| `glass_shadow(scheme) -> Option<GlassShadow>` | The panel's shadow on the window: `opacity`, `span`, `tint`, `lightness`; `None` in dark |

`material::blur_region(line, sigma)` is the Gaussian over one line of a region
that sees only itself: the weights are renormalised at the border, so nothing
outside the region reaches it, as macOS blurs with `inputNormalizeEdges`.

## Traffic-light group

`ctshell::traffic` adds two items to the geometry of `ctappearance::traffic`
([4](../barrierkit/04-appearance-and-tokens.md#traffic-light-geometry)):

| Item | What it does | Read by |
|---|---|---|
| `group_rect() -> (x, y, width, height)` | The rectangle of the three lights: from the first light's `button_rect(0)`, `TITLEBAR.buttons.order` and `spacing` | The compositor's server-side title bar |
| `glyph_svg(glyph, color) -> String` | The SVG of a `Glyph` in `color`, `TITLEBAR.buttons.diameter` (14) on a side | The portal, for the traffic lights it writes for GTK |

## The shell colors of `Resolved`

`ctappearance::resolve` fills these fields of `Resolved` for the shell and the
compositor; the rest is in
[4](../barrierkit/04-appearance-and-tokens.md#resolution). Malaysia passes
`false` as `native_lights`.

| `Resolved` field | What it holds |
|---|---|
| `bar_tint` | The tint of the menu bar and the Dock when `tint_parts` has `bars`: the accent tint, or the wallpaper's dominant color with `wallpaper_tinting` |
| `shell` | `ShellPalette`: `bar`, `text`, `tile`, `tile_active` (the accent) and `tile_text_active` |
| `dock` | The Dock's `DockSurface` ([Veil and tone](#veil-and-tone)) |
| `menu_glass` | The menus' `DockSurface` |
| `notification_glass` | The notifications' `DockSurface` |

## Shell theme

| Function | What it does |
|---|---|
| `use_shell_theme()` | Hook. Gives Freya's own components a theme built from `Resolved`, and builds it again when the root `State<Resolved>` changes |
| `for_scheme(dark) -> Theme` | The same theme for the default appearance in light or dark, without a hook |

The theme is `tengen-shell`, Freya's light or dark theme with these colors:

| Freya color | From |
|---|---|
| `primary`, `secondary`, `tertiary`, `success`, `warning`, `info`, `focus`, `active`, `border_focus` | `shell.tile_active` |
| `error` | The palette's `danger` |
| `background`, `surface_tertiary`, `overlay`, `text_inverse` | `shell.bar` |
| `surface_primary`, `surface_secondary` | `shell.tile` |
| `surface_inverse`, `surface_inverse_secondary`, `surface_inverse_tertiary`, `text_primary` | `shell.text` |
| `border`, `border_disabled`, `disabled` | `shell.tile` in dark, `shell.bar` in light |
| `text_secondary`, `text_placeholder` | The palette's `ink_muted` and `ink_faint` |
| `text_highlight` | `accent_colors.selected_text` |
| `shadow` | `shell.text` at `THEME_SHADOW_OPACITY` |

Next: [8. Desktop settings](08-desktop-settings.md).
