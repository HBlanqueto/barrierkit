# BarrierKit

![BarrierKit, the application toolkit of Malaysia Compositor](docs/barrierkit/assets/brand/barrierkit-banner.svg)

BarrierKit is a Rust UI toolkit with the names and look of SwiftUI and AppKit. Its priority is macOS on Apple silicon: build Mac apps that look native, and port SwiftUI apps visually. It is also the toolkit of [Malaysia](https://github.com/HBlanqueto/malaysia) on Linux.

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
