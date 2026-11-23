# BarrierKit

![BarrierKit, the application toolkit of Malaysia Compositor](docs/barrierkit/assets/brand/barrierkit-banner.svg)

BarrierKit is a Rust UI toolkit with the names and look of SwiftUI and AppKit. Its priority is macOS on Apple silicon: build Mac apps that look native, and port SwiftUI apps visually. It is also the toolkit of [Malaysia](https://github.com/HBlanqueto/malaysia) on Linux.

The idea behind it: as on macOS, the compositor draws every blur and glass of a window, never the app. An app declares a material and where it goes: the sidebar, the toolbar, a popover, a control. On macOS, WindowServer composes it through AppKit's own views. On Linux, Malaysia composes it through a Wayland protocol. Every app then stays in step with the desktop: the same blur, wallpaper tint and accessibility settings, with no lag behind the content. An app may still add its own shaders as extras.

**Where is my Liquid Glass?** On macOS it is the real one. The kit calls AppKit from Rust (`NSGlassEffectView`, `NSVisualEffectView`, `NSPopover`) and has no Swift code. The kit draws only the content and the colors over the glass, measured against SwiftUI. What AppKit cannot reproduce is listed, not faked: see [ADR 0118](docs/decisions.md#0118--on-macos-appkit-composes-every-blur-and-glass-and-swiftui-is-the-reference) and [Backdrops and media](docs/barrierkit/15-backdrops-and-media.md#on-macos).

## How to install

### macOS

You need Xcode's command line tools and `rustup`. From the repository root:

```sh
cargo run -p bk --example hello_world
./dev.sh check
```

Details: [build and check on macOS](docs/barrierkit/18-platform.md#build-and-check-on-macos).

### Linux

> [!NOTE]
> macOS is the priority for now, so Linux has not been tested against the latest changes. This section will be rewritten later.

The repository builds inside the Nix dev shell:

```sh
nix develop
./dev.sh check
```

## Crates

- `bk` — the toolkit: platform runners, controls, views and window patterns.
- `ctdesign` — Tengen, the design tokens, as typed Rust constants.
- `ctappearance` — resolves the appearance settings and the wallpaper into a palette.
- `infinity-protocol` — the private Wayland protocol shared with Malaysia.

## Documentation

- [The kit, chapter by chapter](docs/barrierkit/README.md), starting with [your first app](docs/barrierkit/02-first-app.md).
- [Glossary](docs/barrierkit/20-glossary.md): every SwiftUI and AppKit name and its BarrierKit name.
- [Porting a SwiftUI app](scripts/swift-to-bk/README.md) with `swift-to-bk`.
- [Decisions](docs/decisions.md) and the [prompt for AI agents](agents/prompt.md).

License: MIT
