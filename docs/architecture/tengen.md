# Tengen

**Goal:** change or add a design value of Tengen, the desktop's visual
language, and know which `ctdesign` constant carries it.
**Scope:** from the JSON files in `crates/ctdesign/tengen/` to the Rust
constants that `bk` and Malaysia (compositor, shell, portal) read,
and how `ctappearance` combines them with the user's appearance. Using the
constants in an app is in
[4. Appearance and tokens](../barrierkit/04-appearance-and-tokens.md).
**Reader:** whoever changes a colour, a metric, a font or a design threshold.
**Input:** `tengen/tokens.json` and `tengen/components.json`.
**Output:** `$OUT_DIR/tengen.rs`, compile-time constants that
`crates/ctdesign/src/lib.rs` includes with `include!`
([ADR 0011](../decisions/0011-design-values-as-rust-constants.md)).

Tengen is the visual language (the role Aqua has in macOS). CTDesign is the
compiled tokens; CTAppearance is the resolved appearance
([ADR 0046](../decisions/0046-tengen-visual-language.md)). The JSON files
produce nothing but these constants, and only `build.rs` reads them.

## Steps to change a value

1. Find the key. Base values are in `tokens.json`; component metrics, the
   shell and the appearance factors are in `components.json`
   ([value map](#value-map)).
2. Edit the value. A number may be a reference to another key, written as its
   dotted path: `"titlebar": "radius.lg"`. A colour may be a token name in
   braces, `"{traffic-close}"`, which resolves in each theme.
3. Build. `build.rs` merges both files, reads the structs in `src/lib.rs` and
   writes `tengen.rs`. `cargo:rerun-if-changed` watches the three files.
4. Run `./dev.sh check`.

Do not edit `src/lib.rs` to change a value: it holds the types and the API.

## Steps to add a constant

1. Add the key to `components.json` (or `tokens.json` for a base value).
2. Add a line to the `CONSTANTS` table in `crates/ctdesign/build.rs`:
   `NAME type path`, for example `SPACE_4 f32 space.4`. For a value that
   changes per theme, use the `THEMED` table and write `{theme}` in the path.
3. If the type is a new struct, declare it in `src/lib.rs`, one `pub field:
   Type,` per line. Each field reads the key with its name in kebab case
   under the path (`hide_delay` → `hide-delay`); an underscore may also open
   a nested group (`separator_inset` → `separator.inset`).
4. When a field's key has another name, add `field=key` after the path in the
   table. `field==value` writes a literal instead.
5. Build. A missing key, struct or field stops the build with
   `ctdesign: …` and the path it looked for.
6. Use the constant through `ctdesign::NAME` and run `./dev.sh check`.

## What the build writes

| Source | Writes |
|---|---|
| `CONSTANTS` table | One constant per line |
| `THEMED` table | `NAME_LIGHT`, `NAME_DARK`, `NAME_DARK_TINT`; `SHELL`, `SIDEBAR` and `TOOLGROUP_BG` also get `_OPAQUE` variants, composited over `bg-window` |
| `emit_titlebar` | `TITLEBAR`, `WINDOW_MIN_SIZE`, `WINDOW_DEFAULT_SIZE` |
| `emit_tints` | `WINDOW_TINT_ACCENT`, `WALLPAPER_TINTING`, `TINT_STRENGTH` |
| `emit_palettes` | `LIGHT`, `DARK`, `DARK_TINT` (`Palette`) and `MENU_<THEME>_OPAQUE` |
| `emit_specials` | `SANS_FAMILIES`, `ACCENT_SWATCHES`, `CHART_SERIES`, `LEVEL_INDICATOR_COLORS` |

`CHART_SERIES` and `LEVEL_INDICATOR_COLORS` list swatch names of
`components.settings.accent-swatches`; a name that is not a swatch stops the
build.

## Value map

| Values | Key | Constant |
|---|---|---|
| Themes | `themes.light`, `themes.dark`, `themes.dark-tint`; `default-theme` is `light` | `LIGHT`, `DARK`, `DARK_TINT` and the themed constants |
| Traffic lights | `traffic-close` `#ff5f57`, `traffic-min` `#febc2e`, `traffic-zoom` `#28c840` in every theme; `traffic-inactive`, `traffic-rim`, `traffic-rim-inactive` per theme | `TITLEBAR.buttons`, `Palette.traffic_lights_inactive`, `Palette.traffic_rim` |
| Traffic light geometry and glyphs | `components.window.traffic-lights` (`size` 14, `gap` 9, `radius` 4, `glyphs`) | `TITLEBAR.buttons`, `TRAFFIC_GLYPHS`, `TRAFFIC_*` |
| Titlebar colours | `titlebar-bg`, `titlebar-bg-inactive`, `titlebar-title`, `titlebar-title-inactive`, `titlebar-line` | `Palette.titlebar_bg`, `Palette.title`, `Palette.hairline` |
| Titlebar heights | `size.titlebar-h` 40 (own apps); `size.titlebar-ssd-h` 32 (the bar the compositor draws) | `TITLEBAR_CLIENT_HEIGHT`; `TITLEBAR.thickness` |
| Window corners | `components.window.corner`: `titlebar` → `radius.lg` (16), `compact` 20, `unified` 26, `utility` 15, `expansion` 1.52866, `exponent` 3.34 | `WINDOW_CORNER`; the names match the `set_bar` bars |
| Fonts | `font.families.sans` starts with Geist; `font.families.mono` with Geist Mono | `SANS_FAMILIES`; each `Font` takes the first family |
| Text styles | `font.styles` (size / line height / weight): `display` 26/32/700, `title-1` 22/26/650, `title-2` 15/20/600, `headline` 13/18/700, `body` 13/18/400, `callout` 12/16/400, `caption` 11/14/500, `mono` 13/16/400 | `LARGE_TITLE`, `TITLE`, `TITLE_2`, `HEADLINE`, `BODY`, `CALLOUT`, `CAPTION`, `MONO`; the window title uses `headline` (`TITLEBAR.font`) |
| Spacing | `space.1`–`space.4`: 4, 8, 12, 16 | `SPACE_1`–`SPACE_4` |
| Durations | `duration.fast` 120, `base` 220, `slow` 420, `wait-max` 150 (ms) | `DURATION_FAST_MS`, `DURATION_BASE`, `DURATION_SLOW_MS`, `THEME_FADE_WAIT_MS` |
| Shadows | `shadow.layers.{light,dark}` (ambient and contact layers); `shadow.unfocused` (blur 0.45, offset 0.4, opacity 0.55) | `SHADOW`, `SHADOW_DARK` |
| Controls | `components.<control>` | `BUTTON`, `SWITCH`, `SLIDER`, `TEXT_FIELD`… |
| Shell | `components.shell`: `menubar`, `dock`, `about`, `lock`, `launcher`, `menu` | `MENUBAR_*`, `DOCK_*`, `ABOUT_*`, `LOCK_*`, `LAUNCHER_*`, `MENU_*`, `SHELL_*` |
| Appearance factors | `appearance`: `dominant`, `tint-strength`, `modes`, `layer-factors` | `DOMINANT_*`, `TINT_STRENGTH`, `WINDOW_TINT_*`, `LAYER_FACTOR_*` |

## Appearance and wallpaper

`ctappearance` turns the constants into the resolved appearance with
`resolve(&load_appearance(), load_dominant())`.

1. `appearance.color_scheme` picks the theme. The old value `auto` reads as
   `light`.
2. `reduce_transparency` picks the `_OPAQUE` shell, menu and sidebar
   palettes.
3. `wallpaper_tinting` mixes the wallpaper behind each window into the
   parts of `tint_parts`, by `WALLPAPER_TINTING` times `tint_strength`
   ([4](../barrierkit/04-appearance-and-tokens.md#wallpaper-behind-a-window)).
4. The dominant colour and the wallpaper grid come from `wallpaper.state` in
   the data dir (`~/.local/share/malaysia/wallpaper.state`).
   `malaysia-wallpaper` writes it with `ctappearance::write_wallpaper_state`.
5. `ctappearance::backdrop::dominant_from_pixels` computes it: k-means in
   OKLab, drops the clusters with less chroma than `DOMINANT_MIN_CHROMA`,
   picks the one with the largest area × chroma and clamps its HSL lightness
   into `DOMINANT_LIGHTNESS`.

The kit, the compositor and the portal watch `appearance.domain` and
`wallpaper.state` and resolve again when either changes
([18. Platform](../barrierkit/18-platform.md#the-runner)).

## Checks

`scripts/check-design-tokens.sh` runs in `./dev.sh check` and in the flake's
`design-tokens` check. In the production code of `crates/*/src`, except
`ctdesign`, it rejects:

| Pattern | Rule |
|---|---|
| A hex colour (`0xRRGGBB`) or `Rgba::rgb8(…)` / `Rgba::rgba8(…)` with numbers | Colours live in `ctdesign` |
| A local `[u8; 4]` | Convert with `Rgba::to_rgba8` |
| `const NAME: i32/u32/f32 = <number>` in `password.rs`, `controls/`, `theme.rs` | Measurements live in `ctdesign` |

## Related

- [ADR 0011](../decisions/0011-design-values-as-rust-constants.md): the values are Rust constants.
- [ADR 0032](../decisions/0032-neptunia-as-design-source.md) and [ADR 0046](../decisions/0046-tengen-visual-language.md): Tengen as the design source.
- [`interface.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/architecture/interface.md): the full visual map.
- [`windows.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/architecture/windows.md): titlebar and traffic lights.
- [`wallpaper.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/architecture/wallpaper.md): the background component and its state.
