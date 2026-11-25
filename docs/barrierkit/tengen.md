# Tengen

**Goal:** change or add a design value of Tengen, the desktop's visual
language, and know which `ctdesign` constant carries it.
**Scope:** from the JSON files in `crates/ctdesign/tengen/` to the Rust
constants of `ctdesign` that `bk` and Malaysia read,
and how `ctappearance` combines them with the user's appearance. Using the
constants in an app is in
[4. Appearance and tokens](04-appearance-and-tokens.md).
**Reader:** whoever changes a colour, a metric, a font or a design threshold.
**Input:** `tengen/tokens.json` and `tengen/components.json`.
**Output:** `$OUT_DIR/tengen.rs`, compile-time constants that
`crates/ctdesign/src/lib.rs` includes with `include!`
([ADR 0011](../decisions.md)), and `docs/barrierkit/tengen.css`, the same values as
CSS variables for [artifacts](artifacts.md).

Tengen is the visual language (the role Aqua has in macOS). CTDesign is the
compiled tokens; CTAppearance is the resolved appearance
([ADR 0046](../decisions.md)). Only `build.rs` and `scripts/tengen-css.sh`
read the JSON files.

## Steps to change a value

1. Find the key. Base values are in `tokens.json`; component metrics, the
   shell and the appearance factors are in `components.json`
   ([value map](#value-map)).
2. Edit the value. A number may be a reference to another key, written as its
   dotted path: `"titlebar": "radius.lg"`. A colour may be a token name in
   braces, `"{ink}"`, which resolves in each theme.
3. Build. `build.rs` merges both files, reads the structs in `src/lib.rs` and
   writes `tengen.rs`. `cargo:rerun-if-changed` watches the three files.
4. Regenerate the CSS: `scripts/tengen-css.sh > docs/barrierkit/tengen.css`.
   `./dev.sh check` fails while the file differs from the JSON.
5. Run `./dev.sh check`.

Do not edit `src/lib.rs` to change a value: it holds the types and the API.

## Steps to add a constant

1. Add the key to `components.json` (or `tokens.json` for a base value).
2. Add a line to the `CONSTANTS` table in `crates/ctdesign/build.rs`:
   `NAME type path`, for example `SPACE_4 f32 space.4`. For a value that
   changes per theme, use the `THEMED` table and write `{theme}` in the path.
   A value only the desktop reads belongs to the desktop's own repository, not here.
3. If the type is a new struct, declare it in `src/lib.rs`, one `pub field:
   Type,` per line. Each field reads the key with its name in kebab case
   under the path (`hide_delay` → `hide-delay`); an underscore may also open
   a nested group (`separator_inset` → `separator.inset`).
4. When a field's key has another name, add `field=key` after the path in the
   table. `field==value` writes a literal instead.
5. Build. A missing key, struct or field stops the build with
   `ctdesign: …` and the path it looked for.
6. Regenerate `docs/barrierkit/tengen.css` (step 4 above).
7. Use the constant through `ctdesign::NAME` and run `./dev.sh check`.

## What the build writes

| Source | Writes |
|---|---|
| `CONSTANTS` table | One constant per line |
| `THEMED` table | `NAME_LIGHT`, `NAME_DARK`, `NAME_DARK_TINT`; `SHELL`, `SIDEBAR` and `TOOLGROUP_BG` also get `_OPAQUE` variants, composited over `bg-window`. `SIDEBAR` and `TOOLGROUP_BG` read them from the `-reduced` key, the color measured with Reduce Transparency |
| `emit_titlebar` | `TITLEBAR`, `WINDOW_MIN_SIZE`, `WINDOW_DEFAULT_SIZE` |
| `emit_tints` | `WINDOW_TINT_ACCENT`, `WALLPAPER_TINTING`, `WALLPAPER_TINTING_LIFT`, `TINT_STRENGTH` |
| `emit_palettes` | `LIGHT`, `DARK`, `DARK_TINT` (`Palette`) and `MENU_<THEME>_OPAQUE` |
| `emit_specials` | `SANS_FAMILIES`, `ACCENTS`, `ACCENT_SWATCHES`, `MULTICOLOR`, `LEVEL_INDICATOR_COLORS` and its `_DARK`, `_CONTRAST` and `_CONTRAST_DARK` twins |

`ACCENTS` comes from the list `components.settings.accents`, one entry per
accent with the four looks `light`, `dark`, `light-contrast` and
`dark-contrast`. A color there is a hex value or a theme reference such as
`{accent}` or `{contrast.accent}`, read in the theme of its look. A missing
role stops the build. `ACCENT_SWATCHES` is the name and light `control` of
each.

## Value map

| Values | Key | Constant |
|---|---|---|
| Themes | `themes.light`, `themes.dark`, `themes.dark-tint`; `default-theme` is `light` | `LIGHT`, `DARK`, `DARK_TINT` and the themed constants |
| Traffic lights | `traffic-lights` and `traffic-lights-contrast` per theme: the gradient stops, ring, glyph and halo of each light, measured on macOS 27 | `TRAFFIC_LIGHTS_*`, `TRAFFIC_LIGHTS_CONTRAST_*` |
| Traffic light geometry and glyphs | `components.window.traffic-lights` (`size` 14, `gap` 9, `ring-width`, `pressed`, `glyphs`) | `TITLEBAR.buttons`, `TRAFFIC_GLYPHS`, `TRAFFIC_*` |
| Titlebar colours | `titlebar-bg`, `titlebar-bg-inactive`, `titlebar-title`, `titlebar-title-inactive`, `titlebar-row-title`, `titlebar-row-title-inactive`, `titlebar-line` (the `separator` token) | `Palette.titlebar_bg`, `Palette.title`, `Palette.row_title`, `Palette.hairline` |
| Titlebar heights | `size.titlebar-h` 40 (own apps); `size.titlebar-ssd-h` 32 (the bar the compositor draws) | `TITLEBAR_CLIENT_HEIGHT`; `TITLEBAR.thickness` |
| Window corners | `components.window.corner`: `titlebar` → `radius.lg`, `unified-compact`, `unified` and `utility`, all 16; `expansion` 1.52866, `exponent` 3.34 | `WINDOW_CORNER`; the names match the `set_bar` bars |
| Fonts | `font.families.sans` starts with Adwaita Sans; `font.families.mono` with Geist Mono | `SANS_FAMILIES`; each `Font` takes the first family |
| Text styles | `font.styles` (size, line height and weight; values in [4](04-appearance-and-tokens.md#type)): `display`, `title-1`, `title-2`, `title-3`, `headline`, `body`, `callout`, `subheadline`, `caption`, `footnote`, `caption-2`, `mono`, `toolbar-title`, `about-name` | `LARGE_TITLE`, `TITLE`, `TITLE_2`, `TITLE_3`, `HEADLINE`, `BODY`, `CALLOUT`, `SUBHEADLINE`, `CAPTION`, `FOOTNOTE`, `CAPTION_2`, `MONO`, `TOOLBAR_TITLE`, `ABOUT_NAME`; the window title uses `headline` (`TITLEBAR.font`) |
| Spacing | `space.1`–`space.4`: 4, 8, 12, 16 | `SPACE_1`–`SPACE_4` |
| Durations | `duration.fast` 120, `base` 250, `slow` 420, `wait-max` 150 (ms) | `DURATION_FAST_MS`, `DURATION_BASE`, `DURATION_SLOW_MS`, `THEME_FADE_WAIT_MS` |
| Shadows | `shadow.layers.{light,dark}` (ambient and contact layers); `shadow.unfocused` (blur 0.45, offset 0.4, opacity 0.55) | `SHADOW`, `SHADOW_DARK` |
| Controls | `components.<control>` | `BUTTON`, `SWITCH`, `SLIDER`, `TEXT_FIELD`… |
| Shell | `components.shell` | What the kit reads: `MENUBAR_HEIGHT`, `MENUBAR_TINT_SPREAD`, the Dock's frame, label and menu, `MENU_*`, `SHELL_*`; the rest belongs to the desktop |
| Appearance factors | `appearance`: `dominant`, `tint-strength`, `modes`, `layer-factors`, `wallpaper-tinting`, `contrast`, `accessibility` | `DOMINANT_*`, `TINT_STRENGTH`, `WINDOW_TINT_*`, `ACCENT_CHROMA_GAIN`, `LAYER_FACTOR_*`, `WALLPAPER_TINTING*`, `ACCENT_MIN_CONTRAST_ON_WINDOW`, `ON_ACCENT_MIN_CONTRAST`, `TEXT_SIZES`, `POINTER_SIZES`, `DISPLAY_CONTRAST`, `COLOR_FILTER_INTENSITY` |

## Appearance and wallpaper

`ctappearance` turns the constants into the resolved appearance with
`resolve(&load_appearance(), load_dominant(), native_lights)`
([4](04-appearance-and-tokens.md#resolution)).

1. `appearance.color_scheme` picks the theme, unless the system's light or
   dark wins ([4](04-appearance-and-tokens.md#the-systems-light-or-dark)).
   The old value `auto` reads as `light`.
2. `reduce_transparency` or `contrast` picks the `_OPAQUE` shell, menu,
   sidebar and toolgroup colors.
3. `wallpaper_tinting` tints each window with the wallpaper behind it, as
   macOS 26 does: `WALLPAPER_TINTING` of its chroma and
   `WALLPAPER_TINTING_LIFT` of the way to `WALLPAPER_TINTING_TARGET`, times
   `tint_strength`, in the parts of `tint_parts`
   ([4](04-appearance-and-tokens.md#wallpaper-behind-a-window)).
4. The dominant colour and the wallpaper grid come from `wallpaper.state` in
   the data dir (`~/.local/share/malaysia/wallpaper.state`).
   `malaysia-wallpaper` writes it with `ctappearance::write_wallpaper_state`.
5. `ctappearance::backdrop::dominant_from_pixels` computes it: k-means in
   OKLab, drops the clusters with less chroma than `DOMINANT_MIN_CHROMA`,
   picks the one with the largest area × chroma and clamps its HSL lightness
   into `DOMINANT_LIGHTNESS`.

On Linux the kit, the compositor and the portal watch `appearance.domain` and
`wallpaper.state` and resolve again when either changes
([18. Platform](18-platform.md#the-runner)).

## Checks

`scripts/check-design-tokens.sh` runs in `./dev.sh check` and in the flake's
`design-tokens` check. In the production code of `crates/*/src`, except
`ctdesign`, it rejects:

| Pattern | Rule |
|---|---|
| A hex colour (`0xRRGGBB`, `"#RRGGBB"`), `Rgba::rgb8(…)` / `Rgba::rgba8(…)` or Freya's `Color::from_rgb(…)` with numbers, or an `r:`, `g:` or `b:` field set to a fraction | Colours live in `ctdesign` |
| A local `[u8; 4]` | Convert with `Rgba::to_rgba8` |
| `corner_radius(…)` or `CornerRadius::new(…)` with a number other than 0 | Radii live in `ctdesign` |
| `Duration::from_millis/secs/micros(…)` or `.time(…)` with a number in `crates/bk/src`, except `platform/`, `session_bus.rs`, `animation.rs` and `group.rs` | Durations live in `ctdesign` |
| `const NAME: i32/u32/f32 = <number>` in `password.rs`, `controls/`, `theme.rs` | Measurements live in `ctdesign` |

## Related

- [ADR 0011](../decisions.md): the values are Rust constants.
- [ADR 0032](../decisions.md) and [ADR 0046](../decisions.md): Tengen as the design source.
- [`interface.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/architecture/interface.md): the full visual map.
- [`windows.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/architecture/windows.md): titlebar and traffic lights.
- [`wallpaper.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/architecture/wallpaper.md): the background component and its state.
