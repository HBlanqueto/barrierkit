# Tengen in malaysia: data, generation and checks

Tengen is the desktop design system (ADR 0032). In malaysia it is no longer
read from `~/dotfiles`: it lives **inside the repo**, in
`crates/vow-design/tengen/`, and `build.rs` turns its JSON into the Rust
constants of `vow-design`. They are still compile-time constants
(ADR 0011); what changes is where they come from. `vow-appearance` applies those
constants to `appearance.domain` and `wallpaper.state` for the compositor, the
shell and the portal.

## Source of truth

```
crates/vow-design/tengen/
├── tokens.json        colors per theme, radii, spacing, sizes, duration,
│                      typography, shadows and accents
├── components.json    component sizes and states, shell and ssd-titlebar
├── default.nix        Slint/CSS/QML generator (builtins only, no nixpkgs)
├── generated/         tengen.slint, tengen.css, Tengen.qml
├── demo.slint         gallery for slint-viewer/SlintPad
├── SOURCE.md          provenance, copy rule and how to regenerate
└── README.md          value catalog and usage guide
```

`SOURCE.md` records the dotfiles commit the folder was copied from. **This
copy is the source for malaysia**: changes are made here and, if wanted,
they are sent back to dotfiles by hand. Neither Cargo nor Nix reads `~/dotfiles` or any other repo.

`tools/sync-tengen.sh <ruta>` brings changes from an external copy in one
direction only, shows the diff and updates `SOURCE.md`. It does not run in the build or
in CI; it is a manual tool.

## Constant generation

`crates/vow-design/build.rs`, with one module per family in `build/` (`json`,
`color`, `controls`, `window`, `appearance`, `views`, `sidebar`, `menu`,
`shell`, `settings`):

1. Reads `tengen/tokens.json` and `tengen/components.json` with `serde_json`
   (only `build-dependencies`).
2. Writes `$OUT_DIR/tengen.rs` with palettes, radii, typography, shadows,
   traffic lights, tint factors, shell metrics and `ssd-titlebar`.
3. Emits `cargo:rerun-if-changed` for both JSON files.
4. If a field that `vow-design` needs is missing, it fails and names the field and the
   file.

`crates/vow-design/src/lib.rs` keeps the types (`Rgba`, `Palette`,
`Titlebar`, `ShellPalette`, `LockPalette`, …) and the public API, and ends with:

```rust
include!(concat!(env!("OUT_DIR"), "/tengen.rs"));
```

Chrome, the shell and `barrier-kit` keep reading the same names. The
literals that used to be copied by hand no longer live in `src/`.

## Values that Tengen owns

- **Themes**: `light`, `dark` and `dark-tint`, with `LIGHT`, `DARK` and `DARK_TINT`.
- **Traffic lights**: `traffic-close/min/zoom` are `#ff5f57`, `#febc2e` and `#28c840`
  in all three themes; `traffic-inactive`, `traffic-rim` and
  `traffic-rim-inactive` (1 px inner ring, composited over the fill) and
  `traffic-glyph` are in `tokens.json`.
- **Titlebar**: `titlebar-bg`/`titlebar-bg-inactive` (`bg-window`/`bg-sunken`),
  `titlebar-title`/`titlebar-title-inactive` (`ink-strong`/`ink-muted`) and
  `titlebar-line` (`line`).
- **Heights**: `size.titlebar-h = 44` for own apps; `size.titlebar-ssd-h =
  32` for the bar that the compositor draws in third-party apps with server-side
  decoration. The `ssd-titlebar` component describes the centred title,
  symmetric reserve, truncation with `…`, a single line and `radius.lg` corners.
- **Typography**: `font.families.sans` starts with Geist; `titlebar-title` is
  13/17/600; the weights in use are 400, 500, 600, 650 and 700.
- **Shadows**: `shadow.layers.{light,dark,dark-tint}` hold the two
  ambient/contact layers of each theme, with their colour and opacity; `shadow.unfocused`
  sets the inactive-window factors (blur ×0.45, offset ×0.4, opacity
  ×0.55). The CSS strings still exist for whoever consumes them.
- **Shell**: `components.shell` defines menubar, dock and lock; `shell-bar` is the
  translucent background (`#rrggbbaa` in JSON/Slint/CSS, `#aarrggbb` in QML).

## Shell and wallpaper

- `ShellPalette` (`SHELL_LIGHT`, `SHELL_DARK`, `SHELL_DARK_TINT`) is chosen by
  `appearance.color_scheme`; `auto` resolves to `light` with
  `ColorScheme::resolved()`, the same source that the portal publishes.
- `window_tint_scope = "desktop"` mixes bar and dock with the wallpaper's
  dominant colour (20 % light / 26 % dark), reading `wallpaper.state`.
- `malaysia-shell wallpaper` decodes PNG/JPEG, computes the dominant colour (grid
  ~256×256, 4-bit histogram weighted by chroma, greys discarded, HSL
  45–65 %) and writes `~/.local/share/malaysia/wallpaper.state`
  (`dominant = [r, g, b]`). The compositor no longer decodes images: it reads that
  file and watches its mtime together with `appearance.domain` to rebuild
  `ChromeTheme` live.
- The portal publishes that same dominant colour as `accent-color` in wallpaper mode,
  and watches `appearance.domain` and `wallpaper.state`.

## Checks

In `./dev.sh check`:

- `scripts/check-design-tokens.sh`: no UI colour or metric outside
  `vow-design` in `crates/malaysia-compositor`, `crates/malaysia-shell` and `barrier-kit`; it
  skips test modules and allows buffer formats.
- `scripts/check-tengen-generated.sh`: compares `generated/tengen.slint`,
  `tengen.css` and `Tengen.qml` with what `default.nix` produces.
- `demo.slint` is compiled with `slint-compiler` if it is in the devShell; if not,
  it warns and skips it.

In the flake:

- `checks.<system>.tengen-generated`: the three generated outputs.
- `checks.<system>.design-tokens`: the grep above.
- `checks.<system>.slint-demo`: renders `demo.slint` with `slint-viewer` under
  Xvfb (it compiles the whole file).

## Changing a value

1. Edit `crates/vow-design/tengen/tokens.json` or `components.json`.
2. Regenerate the outputs if needed:

   ```sh
   cd crates/vow-design/tengen
   nix eval --raw -f . slint > generated/tengen.slint
   nix eval --raw -f . css   > generated/tengen.css
   nix eval --raw -f . qml   > generated/Tengen.qml
   ```

3. Run `./dev.sh check` (and, when relevant, `nix build .#checks…`).
4. Do not edit `crates/vow-design/src/` to change a value.

## Related

- [`interfaz.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/interfaz.md) — the full visual map.
- [`ventanas.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/ventanas.md) — titlebar and traffic lights.
- [`wallpaper.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/wallpaper.md) — the background component and its state.
- ADR 0011 — the values are Rust constants.
- ADR 0032 — Tengen as the design source.
- `crates/vow-design/tengen/README.md` — catalogue and per-project usage.
