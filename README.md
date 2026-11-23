# BarrierKit

![BarrierKit, the application toolkit of Malaysia Compositor](docs/barrierkit/assets/brand/barrierkit-banner.svg)

BarrierKit is a Rust UI toolkit with the names and look of SwiftUI and AppKit. It began as the toolkit of [Malaysia](https://github.com/HBlanqueto/malaysia), the GTK + libadwaita to its GNOME. Today its priority is macOS on Apple silicon: build Mac apps that look and behave like native ones, and port existing SwiftUI apps visually.

## macOS on Apple silicon

The macOS runner (`crates/bk/src/platform/macos/`) draws with Skia on Metal inside a real `NSWindow`. AppKit gives the frame, the traffic lights, the menu bar, the sheets, the file panels, the Keychain and the notifications. On macOS 26 and later, the toolbar and `glass_effect` use `NSGlassEffectView`. What works and what is missing: [the macOS runner](docs/barrierkit/18-platform.md#the-macos-runner).

### Build and run

You need Xcode's command line tools and `rustup`. Nix is not needed on a Mac.

1. Run an example from the repository root:

   ```sh
   cargo run -p bk --example hello_world
   ```

2. Run the checks. `rust-toolchain.toml` fetches `rustfmt` and `clippy` on the first run:

   ```sh
   ./dev.sh check
   ```

3. Optional: open every recipe as an app and check its window:

   ```sh
   ./dev.sh macos-smoke
   ```

Details: [build and check on macOS](docs/barrierkit/18-platform.md#build-and-check-on-macos).

### Port a SwiftUI app

`scripts/swift-to-bk/` turns a SwiftUI file into a BarrierKit program. It needs `swiftc` and Python 3:

```sh
python3 -I scripts/swift-to-bk/translate.py scripts/swift-to-bk/tests/counter/Counter.swift
```

Each construct the kit or the tool lacks becomes a `TODO` line in the summary. Usage: [swift-to-bk](scripts/swift-to-bk/README.md). The [glossary](docs/barrierkit/20-glossary.md) maps every SwiftUI and AppKit name to its BarrierKit name.

### Limits compared to Swift

BarrierKit is a toolkit that imitates SwiftUI and AppKit; it does not run them. A port reaches the look, not every behavior:

- **SwiftUI only, one file.** The translator reads views, `@State`, `@Binding`, `ForEach` and `if`. Model code (`@Observable`, `@AppStorage`, `@Environment`) stays a `TODO` ([limits of the tool](docs/research/swift-app-translation.md#limits-of-the-tool)).
- **No AppKit code.** The kit has no responder chain, delegates, Auto Layout or key-value observing (ADR 0085, [what the kit does not do](docs/barrierkit/16-swiftui-to-the-kit.md#what-the-kit-does-not-do)).
- **Runner gaps.** Not run at 2x yet; no file drops, window tabs or glass `tint`. The full list is in the [runner table](docs/barrierkit/18-platform.md#the-macos-runner).

Design and milestones of the port: [macOS arm64](docs/research/macos-arm-port.md).

## Linux

> [!NOTE]
> macOS is the priority for now, so the Linux side has not been tested against the latest changes.

On Linux, `bk` is a Wayland client. It runs on any compositor with `wl_compositor`, `xdg_shell` and `wl_seat`. When an optional global is missing (`infinity_window_manager_v1`, `ext_foreign_toplevel_list_v1` / `zwlr_foreign_toplevel_manager_v1`, `wp_cursor_shape_manager_v1`, `org_kde_kwin_appmenu_manager`), the runner logs it with `tracing::info!` and carries on.

The repository builds inside the Nix dev shell (ADR 0007):

```sh
nix develop
./dev.sh check
```

CI runs `./dev.sh check` on Linux for every push to `main` ([`.github/workflows/check.yml`](.github/workflows/check.yml)). The Skia prebuild for `freya-skia-bindings` 0.104.0 lives in `nix/skia.nix`.

## Crates

- `bk` — BarrierKit, the toolkit itself: platform runners, style primitives, controls, views and window patterns.
- `ctdesign` — CTDesign (CT = Curse Techniques): Tengen, the design-token system. `tokens.json` and `components.json` become typed Rust constants through `build.rs`.
- `ctappearance` — CTAppearance: resolves `appearance.domain` and the desktop wallpaper into a concrete palette, and watches the file.
- `infinity-protocol` — private Wayland protocol shared with the Malaysia compositor. Features `client` (kit) and `server` (compositor).

## Documentation

[`docs/barrierkit/`](docs/barrierkit/README.md) explains the kit from the first window to the runners, with real screenshots. Start with [your first app](docs/barrierkit/02-first-app.md).

AI agents start from one prompt: [`agents/prompt.md`](agents/prompt.md).

[`docs/decisions.md`](docs/decisions.md) gives the reason behind each decision that still holds.

License: MIT
