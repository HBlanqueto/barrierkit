# Tengen: from the JSON files to the vow-design constants

Tengen is the desktop design system (ADR 0032, renamed in ADR 0046). This
document covers how a design value goes from the Tengen JSON files to the
Rust constants of `vow-design`, and how `vow-appearance` applies them. It
starts when you edit a JSON file and ends when the kit, the compositor, the
shell and the portal read the constant.

**Reader:** whoever changes a colour, a metric or a design threshold.

## Inputs and outputs

- **Inputs:** `crates/vow-design/tengen/`.
  - `tokens.json`: colours per theme, radii, spacing, sizes, typography and
    shadows.
  - `components.json`: component metrics and states, the shell, the titlebar
    and the appearance factors.
- **Output:** `$OUT_DIR/tengen.rs`, which `crates/vow-design/src/lib.rs`
  includes with `include!`. They are compile-time constants (ADR 0011).
- **Consumers:** `barrier-kit`, the compositor, the shell and the portal read
  the constants. `vow-appearance` combines them with `appearance.domain` and
  `wallpaper.state` to give the resolved appearance.

No other format comes out of these JSON files. Nothing but `build.rs` reads
them.

## Steps to change a value

1. Edit `tokens.json` or `components.json`. A component value can be a
   number or a `group.key` reference to `tokens.json`.
2. Build. `crates/vow-design/build.rs`, with one module per family in
   `build/`, reads both JSON files with `serde_json` (only in
   `build-dependencies`) and writes `tengen.rs`. `cargo:rerun-if-changed`
   watches both files.
3. If a field that `vow-design` needs is missing, the build fails and names
   the field and the file.
4. Run `./dev.sh check`.

Do not edit `crates/vow-design/src/` to change a value: it holds the types
(`Rgba`, `Palette`, `Titlebar`, `ShellPalette`…) and the API.

## Values that Tengen owns

- **Themes:** `light`, `dark` and `dark-tint`, with `LIGHT`, `DARK` and
  `DARK_TINT`.
- **Traffic lights:** `traffic-close/min/zoom` are `#ff5f57`, `#febc2e` and
  `#28c840` in all three themes. `traffic-inactive`, `traffic-rim`,
  `traffic-rim-inactive` and `traffic-glyph` are in `tokens.json`.
- **Titlebar:** `titlebar-bg`/`titlebar-bg-inactive`,
  `titlebar-title`/`titlebar-title-inactive` and `titlebar-line`.
- **Heights:** `size.titlebar-h = 44` in own apps and
  `size.titlebar-ssd-h = 32` in the bar the compositor draws.
- **Typography:** `font.families.sans` starts with Geist; `titlebar-title` is
  13/17/600.
- **Shadows:** `shadow.layers.{light,dark}` hold the ambient and contact
  layers; `shadow.unfocused` sets the inactive-window factors.
- **Shell:** `components.shell` defines menubar, dock and lock.

## Appearance and wallpaper

- `ShellPalette` (`SHELL_LIGHT`, `SHELL_DARK`, `SHELL_DARK_TINT`) is chosen by
  `appearance.color_scheme`; `auto` resolves to `light` with
  `ColorScheme::resolved()`.
- `window_tint_scope = "desktop"` mixes bar and dock with the wallpaper's
  dominant colour, read from `wallpaper.state`.
- `malaysia-wallpaper` (ADR 0049) decodes the image and leaves the dominant
  colour in `wallpaper_state_path()`
  (`~/.local/share/malaysia/wallpaper.state`, `dominant = [r, g, b]`).
- `vow_appearance::backdrop::dominant_from_pixels` computes it: k-means in
  OKLab, it drops the clusters with less chroma than
  `appearance.dominant.min-chroma`, picks the one with the most area times
  chroma and brings its HSL lightness into `appearance.dominant.lightness`.
  `vow_appearance::write_dominant` writes or removes the file.
- The compositor, the kit and the portal read that file and watch it together
  with `appearance.domain`.

## Checks

- `scripts/check-design-tokens.sh`, in `./dev.sh check` and in the flake's
  `design-tokens` check: no UI colour or metric literal outside `vow-design`
  in `crates/barrier-kit/src`.

## Related

- [`interfaz.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/interfaz.md): the full visual map.
- [`ventanas.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/ventanas.md): titlebar and traffic lights.
- [`wallpaper.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/wallpaper.md): the background component and its state.
- ADR 0011: the values are Rust constants.
- ADR 0032 and ADR 0046: Tengen as the design source.
