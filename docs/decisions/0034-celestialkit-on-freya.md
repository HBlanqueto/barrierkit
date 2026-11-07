# 0034 — CelestialKit on Freya

Status: **accepted** (2026-10-22). Supersedes [ADR 0033](0033-skia-in-clients-and-celestialkit-from-otto-kit.md)
and amends [ADR 0024](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0024-stack-de-render-del-toolkit.md) for the client
processes.

Renamed to BarrierKit, `vow-design` and `vow-appearance` in [ADR 0043](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0043-rebranding-a-malaysia.md).

## Context

ADR 0033 put Skia in the clients and proposed basing CelestialKit on `otto-kit`
with `laye-rs` as the animation engine. The code review showed two
problems: `laye-rs` has no license and `otto-kit` is not a toolkit you can
build on without dragging in other people's decisions. Freya (MIT) already offers a
reactive UI core, layout, accessibility and Skia rendering, with crates separate
from the window backend.

## Decision

1. **CelestialKit is built on Freya's core crates**, pinned
   exactly to `=0.4.3`: `freya-core`, `freya-components` (with
   `default-features = false`), `freya-animation`, `freya-engine`, `torin` and
   `ragnarok`. The umbrella crate `freya` and `freya-winit` are **never** used: the
   umbrella enables `winit` by default and `freya-winit` drags in `winit`, `gtk`,
   `rfd`, `tray-icon`, `ash` and `glutin`.
2. **The kit has its own Wayland runner** (SCTK + calloop + EGL), because
   `winit` does not support layer-shell or `ext-session-lock-v1`. The runner covers
   `zwlr_layer_shell_v1`, `ext-session-lock-v1` and, later, `xdg_toplevel`.
3. **Skia only in client processes.** The compositor never links `freya-*` crates
   or Skia: it stays with GLES, `grade-graphics` and `cosmic-text`.
4. **Grade Animation is a module of `celestial-kit` on `freya-animation`**,
   with its own spring (duration + bounce) written from scratch. From `laye-rs`
   only the idea is taken, not the code.
5. **Tokens still come from `yuki-design` and `yuki-appearance`.** The
   `freya-components` theme is generated from them. No `material-design`.
6. **Infinity stays the same**: optional in the kit until the compositor
   advertises it.
7. **`otto-kit` stops being a source of code.** It remains only as a reference for
   logic (DBusMenu, menu states). If MIT code from Otto is ever copied,
   its notice goes to `NOTICE` in the same commit.
8. **Move to Freya 0.5 when it is stable**, in its own commit, regenerating
   Skia.

## Drawing split

The split follows macOS (Dock, SystemUIServer and loginwindow versus
WindowServer), not a division by app type:

1. **The shell** (wallpaper, menubar, dock, lock) **and the own apps draw
   all their UI with CelestialKit**, each component in its own process. There is no
   chrome drawn by the compositor inside the shell.
2. **The own apps draw their titlebar and their traffic lights with the kit** (CSD,
   `xdg-decoration` in client-side mode) and with the `yuki-design` tokens.
3. **The compositor still composites and applies the effects** (shadows,
   corners, blur and window animations) in GLES with
   `grade-graphics`, like WindowServer. Later, the kit will request them through
   Infinity.
4. **Third-party apps that request SSD still receive the compositor's
   titlebar.** `chrome.rs` and the compositor's render path do not change
   because of this ADR.

## Consequences

- `celestial-kit` gains its own Wayland runner; the kit stops being a
  drawing module and becomes a complete UI library for the shell.
- The compositor does not change its render path or gain Skia,
  Freya or `winit` dependencies.
- The Nix derivation builds Skia for `freya-skia-bindings` 0.98.0, not for
  `skia-safe` 0.93. Its version and features are checked against the real
  Cargo tree.
- `grade-animation` no longer exists as a crate: it is `celestial-kit::animation`.
- The MIT notices of `otto-kit` and `laye-rs` disappear from `NOTICE`; none of their
  code remains.

## References

- [`0024`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0024-stack-de-render-del-toolkit.md) — render stack, amended
  by this ADR for the clients.
- [`0020`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0020-politica-de-diseno-y-unificacion.md) — chrome and effects.
- [`0011`](0011-design-values-as-rust-constants.md) — tokens in
  `yuki-design`.
- [`0023`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0023-los-componentes-del-shell-corren-como-procesos-separados.md) —
  one process per component.
- [`0025`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0025-la-shell-es-nativa.md) — native shell `yuki-shell`.
- [`0032`](0032-neptunia-as-design-source.md) — Neptunia as the source.
- Freya — MIT, <https://github.com/marc2332/freya>.
