# Tengen: designs stored as data

This folder is the single source of nixdots' design. Claude's canvases
are where design happens and gets approved; here the approved design is stored in a format that
any project can read, present or future, without depending on a framework.

```
crates/vow-design/tengen/
├── tokens.json        colors (light, dark, tinted dark), radii, spacing, sizes,
│                      durations, typography and shadows
├── components.json    measurements and states of each approved component (32×18 switch,
│                      rectangular segmented control, toolbar tabs…)
├── default.nix        translates the JSON and composes the theme; builtins only, no nixpkgs
├── generated/         outputs already generated, for those who do not use Nix (SlintPad, another repo)
│   ├── tengen.slint   Theme and Metrics globals for Slint
│   ├── tengen.css     CSS variables (GTK4, web)
│   └── Tengen.qml     QtQuick singleton
└── demo.slint         gallery for tuning by eye with slint-viewer or SlintPad
```

## Golden rule

Nobody writes a color or a measurement by hand. If the design changes, change
`tokens.json` or `components.json` and regenerate:

```sh
cd crates/vow-design/tengen
nix eval --raw -f . slint > generated/tengen.slint
nix eval --raw -f . css   > generated/tengen.css
nix eval --raw -f . qml   > generated/Tengen.qml
```

## Desktop color modes

The `appearance` block of `components.json` describes (as an idea under study) the
window color: `neutral` (no tint), `accent` (tinted by the accent) and
`wallpaper` (tinted by the wallpaper's dominant color, across the whole window or across
the whole desktop). The same block picks the traffic-light style —`standard`
(fixed colors), `accent` or `wallpaper` (derived from its color)—; the `sys-*`
colors (warnings, error, success…) are never tinted and `layer-factors` sets how much each layer
lightens or darkens.

## How each project reads it

| Project | How it syncs |
| --- | --- |
| Psamathe and Triton apps (Slint) | `import { Theme, Metrics } from "…/tengen.slint";`. From Nix: `(import ./crates/vow-design/tengen).slint`; or copy `generated/tengen.slint`. Switching theme is `Theme.mode = TengenMode.dark;`. |
| GTK4 / web | `tengen.css` with `data-theme="dark"` or `"dark-tint"` on the root. |
| Home Manager | `xdg.configFile."tengen".source = (import ./crates/vow-design/tengen).files pkgs;` places the generated files and the JSON in `~/.config/tengen/`, and any app reads them from there. |
| Another future repo | As a flake input (`inputs.nixdots`) and `import "${inputs.nixdots}/kits/tengen"`, or reading `tokens.json` directly. Each project generates its own output its own way; the data is the same. |

## Tuning a screen without recompiling Rust

`slint-viewer --auto-reload` opens a standalone `.slint` and reloads it every time
you save. It is the fastest way to tune a single screen, for example «Tu
cuenta», without touching the rest:

```sh
slint-viewer --auto-reload crates/vow-design/tengen/demo.slint
slint-viewer --auto-reload setup-gui/ui/cuenta.slint   # one specific screen
```

`slint-lsp`, with the VS Code extension or with Neovim or Helix, gives
autocompletion and a live preview where you can change
properties. Both are installed from nixpkgs (`slint-lsp`, `slint-viewer`)
in the `devShell`.

## SlintPad

SlintPad (slintpad.com) is an in-browser editor for trying a component
without installing anything. Since `demo.slint` imports `generated/tengen.slint`, there are two
ways to open it:

- Paste the content of `generated/tengen.slint` first and below it the
  component you want to try, removing the `import` line.
- Or open it from GitHub with
  `https://slintpad.com/?load_url=<raw url of demo.slint>`. SlintPad usually
  resolves the relative import; if it does not, use the first way.

## Translations with `@tr()`

Mark each visible text with `@tr("Continuar")`, as `demo.slint` already does, and
use Spanish as the source language. Slint extracts those strings into `.po` files,
one per language, which are compiled into the binary. This way the assistant can
switch language live when it is chosen in step 1:

```sh
cargo install slint-tr-extractor
find setup-gui/ui -name '*.slint' | xargs slint-tr-extractor -o psamathe.pot
msginit -i psamathe.pot -l en -o lang/en/LC_MESSAGES/psamathe.po
```

```rust
// build.rs (recent Slint versions; check the API of the version you use)
slint_build::compile_with_config(
    "ui/app.slint",
    slint_build::CompilerConfiguration::new().with_bundled_translations("lang"),
).unwrap();
// when the language is chosen:
slint::select_bundled_translation("en").ok();
```

The animated greeting in 13 languages is not a translation: it is a fixed list of
words and stays as data.

## From Claude's canvases to this folder

1. Design happens and gets approved on the canvases (Tengen design system).
2. The approved design is dumped into `tokens.json` and `components.json`. Ask Claude
   «actualiza crates/vow-design/tengen» and it does it from the design system.
3. The outputs are regenerated and each app picks them up in its next build.
