# Decisions

**Goal:** know why BarrierKit is built this way before you change it.
**Reader:** anyone who changes the kit's structure, render stack, look or public names.

Each entry gives the decision as it holds today, with its ADR number. When a
decision changes, edit its entry; when it no longer holds, delete it. Git keeps
the history. The compositor, session and shell decisions are in
[Malaysia's decisions](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisions.md).

## 0011 — Design values as Rust constants
Decision: every design value is a public compile-time constant of `ctdesign`, and no other crate defines one. A color that changes with window focus is a `FocusPair`.
Why: each value has one typed source; a missing token fails the build instead of hiding as a literal.
Code: `crates/ctdesign/build.rs`, `crates/ctdesign/src/lib.rs`, `scripts/check-design-tokens.sh`

## 0032 · 0046 — Tengen as the design source
Decision:
- The visual language is Tengen. Its source is `crates/ctdesign/tengen/tokens.json` and `components.json`.
- They hold colors, radii, spacing, typography, shadows and component metrics.
- `ctdesign/build.rs` generates `$OUT_DIR/tengen.rs` from them.
- The themes are `light` (default), `dark` and `dark-tint`.
- The kit draws the look itself; no alias keeps an old name or path.
Why: the kit, the shell and the compositor read the same values from one place.
Code: `crates/ctdesign/tengen/`, `crates/ctdesign/build.rs`, [Tengen](architecture/tengen.md)

## 0033 · 0034 · 0056 — Render stack
Decision:
- `bk` uses Freya's core crates pinned `=0.5.3`: `freya-core`, `freya-components` (no default features), `freya-edit`, `freya-animation`, `freya-engine`, `freya-clipboard`, `torin`, `ragnarok`.
- Never `freya`, `freya-winit` or any `otto-*` crate.
- Its own Wayland runner on smithay-client-toolkit and `calloop`.
- Skia runs only in clients. `ctdesign`, `ctappearance` and `infinity-protocol` pull no Freya or Skia, so the compositor never links them.
- The runner draws with Vulkan through `ash`, with no GL fallback. `freya-engine` builds Skia with only `vulkan`.
- One shared device and `DirectContext`, one swapchain per surface, `MAILBOX` when offered, else `FIFO`.
Why: winit would own the Wayland connection, and Freya's Vulkan driver lives in `freya-winit`. Frame callbacks already pace redraws, so `FIFO` would throttle twice.
Rejected: a kit from `otto-kit`; Skia on EGL/GLES.
Code: `crates/bk/src/platform/vulkan.rs`, `scripts/check-client-only-deps.sh`, [Platform](barrierkit/18-platform.md)

## 0035 — Own app identification and CSD
Decision: a window's `app_id` is `ctdesign::NATIVE_APP_ID_PREFIX` plus the app name, unless `WindowOptions::app_id` sets another. Every window requests client-side decoration; if the compositor answers server-side, the kit turns off its edge resizing.
Why: the prefix lets the compositor tell kit windows from the rest and leave their chrome to the kit.
Code: `crates/bk/src/platform/window.rs`

## 0039 — Infinity v1
Decision: `infinity_window_manager_v1` is a private Wayland protocol for the shell's window management. `infinity-protocol` generates the client side (feature `client`, used by `bk`) and the server side (feature `server`, used by the compositor). `bk` binds it only with the `shell` feature; the compositor decides which clients may bind it.
Why: the standard protocols do not cover what the Dock and the menu bar need from the compositor.
Code: `crates/infinity-protocol/protocols/`, `crates/bk/src/shell/infinity.rs`, [Wayland protocols](barrierkit/18-platform.md#wayland-protocols)

## 0041 · 0057 · 0065 · 0078 — Sidebar: island, backdrop and glass
Decision:
- The sidebar is island (default) or flat, by the user's `sidebar_style` in `ctappearance::AppearanceConfig`. Its geometry lives in `components.window.sidebar`; an app changes it through `SidebarStyle`.
- Over a `Backdrop` the app picks `SidebarBackdrop::Clear` (default) or `Tinted`, and sends no glass panel.
- Otherwise the sidebar is a glass panel: the kit leaves a hole and sends `set_glass` (Infinity window style, version 6) with the rectangle, corner, `base`, `solid` and `chroma`. There is no public API.
- The compositor fills the panel with `base` plus the blurred wallpaper's chroma and draws the rim; unfocused, it shows flat `solid`. Towards the edge that touches the content, `base` veils the fill, so the wallpaper's colours fade but stay.
- With no such compositor, or with reduce transparency or contrast, the kit paints the opaque sidebar itself.
- In full screen with a glass bar, the glass covers the whole top strip, not one column.
Why: only the compositor sees the wallpaper. The macOS sidebar keeps the app's lightness and takes only part of the wallpaper's chroma.
Rejected: a translucent fill over a plain blur region; it lets the wallpaper's lightness through and cannot say a corner.
Code: `crates/bk/src/sidebar.rs`, `crates/bk/src/glass_panel.rs`, [Sidebar](barrierkit/10-sidebar.md)

## 0047 — BarrierKit in layers
Decision: `bk` is one crate in four layers (platform, primitives, views, patterns) plus a cross-cutting `app`. A layer uses only the layers below it, and patterns compose only the public API of views. Code only `malaysia-shell` uses lives in `src/shell/`, behind the `shell` feature.
Why: the kit must build any desktop app, like AppKit or libadwaita, without each app rebuilding sidebars, search fields and controls.
Pending: nothing is built until an app needs it. Partial damage waits for Freya: 0.5.3 repaints the whole tree every frame.
Code: `crates/bk/src/lib.rs`, [Layers](barrierkit/03-layers.md)

## 0048 — BarrierKit in its own repository
Decision: `bk`, `ctdesign`, `ctappearance` and `infinity-protocol` live in `github.com/HBlanqueto/barrierkit`. malaysia pins all four by one git `rev` in `Cargo.toml` and takes barrierkit as a flake input, reusing its prebuilt Skia. `./dev.sh kit-local` in malaysia tests a local checkout before publishing.
Why: the toolkit carries its style and its reading of the appearance; keeping them in malaysia would make the two repos depend on each other.
Code: `flake.nix`

## 0061 · 0062 · 0063 — SwiftUI-shaped API
Decision:
- An app depends only on `bk` and writes `use bk::prelude::*;`.
- `Application::new(name).window(title, content).run()` opens a window with background and `Titlebar`; `run_window` stays for windows that draw their own bar.
- The crates are `bk`, `ctdesign` and `ctappearance`; prose still says BarrierKit.
- Every public view is a type built with `Type::new(..)` that forwards Freya's traits. Modifiers are methods of `ViewModifiers` (`.help`, `.context_menu`, `.focus_ring`).
- Low-level primitives stay functions, like `text(..)`, `rect()` and `label()`.
- Names follow SwiftUI, then AppKit, then plain English; the glossary lists each one.
Why: the common case needs one import and one expression, and the code reads like its SwiftUI version.
Code: `crates/bk/src/prelude.rs`, `crates/bk/src/app/scene.rs`, [Glossary](barrierkit/20-glossary.md)

## 0064 — Styles and themes
Decision:
- Each control has a style struct (`SliderStyle`) that implements `ControlStyle`. Its `tengen()` builds the default from `ctdesign`.
- An override is a closure from the inherited style to a new one.
- Levels, later wins: Tengen, `Theme` (`Application::theme`, `WindowOptions::theme`), `.control_style` on a container, `.style` on the instance.
- Paint is a `ShapeStyle`; `Role`, `Accent` and `Backdrop` follow the system, `Color` does not.
- The user's accessibility settings apply last: reduce motion, reduce transparency and the focus ring.
Why: an app changes any visual property, or builds a whole theme, without copying Tengen's numbers.
Code: `crates/bk/src/styles/theme.rs`, `crates/bk/src/styles.rs`, `crates/bk/src/modifiers.rs`
