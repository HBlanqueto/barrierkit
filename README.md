# BarrierKit

![BarrierKit, the application toolkit of Malaysia Compositor](docs/barrierkit/assets/brand/barrierkit-banner.png)

BarrierKit is the application toolkit for [Malaysia](https://github.com/HBlanqueto/malaysia): the GTK + libadwaita to its GNOME, or the AppKit to its macOS. It provides the layers, controls, containers and patterns used to build Malaysia's own apps and its shell.

## Crates

- `bk` — BarrierKit, the toolkit itself: platform runner, style primitives, controls, views and window patterns.
- `ctdesign` — CTDesign (CT = Curse Techniques): Tengen, the design-token system. `tokens.json` and `components.json` become typed Rust constants through `build.rs`.
- `ctappearance` — CTAppearance: resolves `appearance.domain` and the desktop wallpaper into a concrete palette, and watches the file.
- `infinity-protocol` — private Wayland protocol shared with the Malaysia compositor. Features `client` (kit) and `server` (compositor).

## Build

The repository is built inside the Nix dev shell (ADR 0007):

```sh
nix develop
./dev.sh check
```

`./dev.sh check` runs fmt, clippy, the kit checks and the tests; its list is in [`dev.sh`](dev.sh).

The Skia prebuild for `freya-skia-bindings` 0.98.0 lives in `nix/skia.nix`.

## Try it on any desktop

`bk` can run as a Wayland client on any compositor, not only on Malaysia. An app starts with the basic protocol (`wl_compositor`, `xdg_shell`, `wl_seat`) and carries on when the optional globals are missing.

When `infinity_window_manager_v1`, `ext_foreign_toplevel_list_v1` / `zwlr_foreign_toplevel_manager_v1`, `wp_cursor_shape_manager_v1` or `org_kde_kwin_appmenu_manager` is missing, the runner logs it with `tracing::info!` and carries on.

## Documentation

[`docs/barrierkit/`](docs/barrierkit/README.md) explains the kit from the first window to the Wayland runner, with real screenshots. Start with [your first app](docs/barrierkit/02-first-app.md). The [glossary](docs/barrierkit/20-glossary.md) maps every SwiftUI and AppKit name to its BarrierKit name.

AI agents start from one prompt: [`docs/ai/prompt.md`](docs/ai/prompt.md).

## Decisions

- ADR 0047 — [BarrierKit in layers](docs/decisions/0047-barrierkit-in-layers.md)
- ADR 0048 — [BarrierKit in its own repository](docs/decisions/0048-barrierkit-own-repository.md)
- ADR 0062 — [Short crate names: `bk`, `ctdesign` and `ctappearance`](docs/decisions/0062-short-crate-names.md)
- ADR 0061 — [A SwiftUI-shaped entry API and a glossary of names](docs/decisions/0061-swiftui-style-entry-api.md)
- ADR 0057 — [The full-screen glass covers the whole window; the sidebar over a tinted backdrop](docs/decisions/0057-glass-and-sidebar-over-backdrop.md)

License: MIT
