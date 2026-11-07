# 0032 — Neptunia is the desktop's design source

Status: **accepted** (2026-10-22).

Renamed to Tengen in ADR 0046.

## Context

The desktop design lives in `crates/yuki-design/neptunia`
(`tokens.json`, `components.json`, `demo.slint` and the outputs in
`generated/`). Neptunia is a design system **as data**, not a runtime
toolkit: it defines colours (light, dark, dark-tint), radii, spacing,
typography, shadows and component metrics. Until now
`yuki-design` had values taken from somewm and a 32 px vertical titlebar,
which did not match that system.

The user has decided that **Neptunia is the desktop's default design**
while Yuki is being built.

## Decision

1. **Neptunia rules.** The default look is its `light` theme; `dark` and
   `dark-tint` are the other variants. Colours, radii, spacing,
   typography, shadows and window metrics come from its JSON.
2. **The Neptunia window**: titlebar **on top**, 12 px traffic lights with 8 px
   spacing and **radius 4** (rounded squares), window radius
   `radius.lg = 16` and the `macos` traffic-light style (`traffic-*` colours). The
   compositor uses **32 px** for third-party SSD windows; Neptunia defines
   44 px for own app windows. CSD apps (GTK, wezterm) do not
   receive the Yuki bar.
3. **It is implemented in Yuki, not in a foreign framework.** The chrome is
   compositor-side (ADR 0020); the titlebar text uses `cosmic-text` in the
   compositor. `celestial-kit` remains the toolkit of the shell and of the
   apps, and the compositor does not use it (ADR 0024): they share shaders, not context.
4. **The copy lives in `crates/yuki-design/neptunia` and `build.rs` generates the
   constants; there is no more manual syncing.** Changes are made in the
   JSON of that copy (and, if wanted, sent back to dotfiles by hand).
5. **Otto is a reference, not a dependency.** Its ideas are read (window, shell,
   launcher, settings, portal); **Skia is not adopted** because of its weight and because Yuki already
   has GLES + `grade-graphics` + `celestial-kit`. If MIT code from Otto is ever copied,
   its notice is added to `NOTICE` in the same commit.

## Consequences

- The titlebar, the menubar and the dock share Neptunia's light palette.
- The compositor titlebar is drawn at 32 px for SSD windows, with 12 px traffic lights and radius 16; its buttons
  close, minimise and maximise.
- `appearance.window_color` implements Neptunia's `neutral`/`accent`/`wallpaper`
  modes: it tints the titlebar with the accent or with the wallpaper's dominant
  colour and derives the traffic lights; `window_tint_scope` and the panel/dock tint
  are left to the shell.
- Real window rounding, the shadow, the traffic-light glyphs on
  hover and live theme switching are still pending.
- The launcher and own apps are designed first in Neptunia
  (`demo.slint`) and then implemented with `celestial-kit`.

## References

- `crates/yuki-design/neptunia/README.md`, `tokens.json`, `components.json`.
- [`0011`](0011-valores-de-diseno-como-constantes-rust.md) — the values
  live in `yuki-design`.
- [`0020`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0020-politica-de-diseno-y-unificacion.md) — compositor-side chrome.
- [`0024`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0024-stack-de-render-del-toolkit.md) — GLES/cosmic-text, no Skia.
- [`../arquitectura/interfaz.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/interfaz.md) — the full map.
- [`../arquitectura/ventanas.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/ventanas.md) — the titlebar.
