# BarrierKit

BarrierKit is the application toolkit for [MalaysiaDreams](https://github.com/HBlanqueto/malaysia): the GTK + libadwaita to its GNOME, or the AppKit to its macOS. It provides the layers, controls, containers and patterns used to build Malaysia's own apps and its shell.

## Crates

- `barrier-kit` — the toolkit itself: platform runner, style primitives, controls, views and window patterns.
- `vow-design` — Tengen, the design-token system. `tokens.json` and `components.json` become typed Rust constants through `build.rs`.
- `vow-appearance` — resolves `appearance.domain` and the desktop wallpaper into a concrete palette, plus a file watcher.
- `infinity-protocol` — private Wayland protocol shared with the MalaysiaDreams compositor. Features `client` (kit) and `server` (compositor).

## Build

The repository must be built inside the Nix dev shell (ADR 0007):

```sh
nix develop
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

The Skia prebuild for `freya-skia-bindings` 0.98.0 lives in `nix/skia.nix`.

## Decisions

- ADR 0047 — [BarrierKit en capas](docs/decisiones/0047-barrierkit-en-capas.md)
- ADR 0048 — [BarrierKit en su propio repositorio](docs/decisiones/0048-barrierkit-repositorio-propio.md)

License: MIT
