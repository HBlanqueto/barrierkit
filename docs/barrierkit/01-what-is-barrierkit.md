# 1. What BarrierKit is

**Goal:** know what the kit is, what it needs to run and which level of it to
start from.
**Scope:** from the crates in the repository to the choice of a level. The
first program is in [2](02-first-app.md); the modules of each level are in
[3](03-layers.md).
**Reader:** anyone who starts with the kit.
**Input:** nothing. **Output:** the crates to depend on and the level to build
with.

## Steps

1. Check that the machine has what the kit needs: [What it needs](#what-it-needs).
2. Depend on `bk`. Add `ctdesign` only to read tokens that the
   prelude does not re-export, and `ctappearance` only to read or change the
   user's appearance ([The crates](#the-crates)).
3. Pick the level of the kit with [Which level to use](#which-level-to-use).
4. Open the first window: [2. Your first app](02-first-app.md).
5. Find the modules of that level: [3. Layers](03-layers.md).

## In one sentence

BarrierKit is to Malaysia what AppKit is to macOS and GTK with libadwaita is to
GNOME: the toolkit for the shell and for Malaysia's own apps. An app asks the
kit for a button or a sidebar; it does not draw one. So every app looks and
behaves the same.

The public API follows SwiftUI and AppKit: their names, their defaults and their
shape ([ADR 0061](../decisions/0061-swiftui-style-entry-api.md)). The
[glossary](20-glossary.md) gives the Apple equivalent of each name.

## Equivalents

| Concept | SwiftUI / AppKit | libadwaita | BarrierKit |
|---|---|---|---|
| App and window | `App`, `WindowGroup` / `NSApplication` | `AdwApplication` | `app::Application`, `app::Scene` |
| Text | `Text` / `NSTextField` | `GtkLabel` | `elements::Text` |
| Stacks | `VStack`, `HStack` / `NSStackView` | `GtkBox` | `elements::VStack`, `elements::HStack` |
| Button | `Button` / `NSButton` | `GtkButton` | `controls::Button` |
| Sidebar and detail | `NavigationSplitView` / `NSSplitView` | `AdwNavigationSplitView` | `views::split::SplitView` |
| Source list | `List` with `.sidebar` style / `NSOutlineView` | `GtkListBox` `.navigation-sidebar` | `sidebar::Sidebar` |
| Form | `Form` | `AdwPreferencesGroup` | `views::form::Form` |
| Settings window | `Settings` | `AdwPreferencesWindow` | `patterns::settings::SettingsWindow` |
| Actions and shortcuts | `Commands` / `NSMenuItem` | `GAction` | `app::Action` |
| App preferences | `UserDefaults` | `GSettings` | `app::load_defaults`, `app::save_defaults` |
| System colors | `Color` / `NSColor.labelColor`… | CSS variables | `style::Palette` |

## The crates

The [`barrierkit`](https://github.com/HBlanqueto/barrierkit) repository is a
Cargo workspace with four library crates. They need Rust 1.98 (edition 2024).

| Crate | What it is | Who depends on it |
|---|---|---|
| `bk` | **BarrierKit**, the kit: Wayland runner, style, controls, views, patterns and app services. | Apps and the shell. |
| `ctdesign` | **CTDesign** (CT = Curse Techniques): **Tengen**, the design system. `build.rs` turns `tengen/tokens.json` and `tengen/components.json` into Rust constants ([4](04-appearance-and-tokens.md#tokens-ctdesign)). | The kit, the compositor, the shell and the portal. |
| `ctappearance` | **CTAppearance**: reads the user's appearance and the wallpaper color and resolves them into concrete colors ([4](04-appearance-and-tokens.md#user-appearance-ctappearance)). | The kit, the compositor, the shell and the portal. |
| `infinity-protocol` | Private Wayland protocol between the apps and the Malaysia compositor ([18](18-platform.md)). | The kit and the compositor. |

`bk` has six Cargo features. Only `a11y` is on by default; an app lists the
others it needs: `bk = { …, features = ["charts", "print"] }`.

| Feature | Default | What it adds |
|---|---|---|
| `a11y` | on | Publishes each window's accessibility tree over AT-SPI with `accesskit_unix` ([18](18-platform.md#accessibility)). |
| `charts` | off | The `charts` module and `ChartStyle` ([16](16-charts.md)). |
| `print` | off | The `print` module, `portal::print` and `PrintPreviewStyle` ([6](06-text-and-editing.md#printing)). |
| `shell` | off | The `shell` module: layer-shell surfaces, the lock screen, the list of toplevels and the status tray host ([18](18-platform.md)). Only `malaysia-shell` enables it. |
| `video` | off | The `video` module, a GStreamer player ([15](15-backdrops-and-media.md)). |
| `tracy` | off | Sends the traces of `init_tracing` to Tracy ([19](19-profiling.md)). |

### On top of Freya

The kit builds on [Freya](https://freyaui.dev) `=0.5.0`, but only on its core
crates: `freya-core`, `freya-components` (without default features),
`freya-edit`, `freya-animation`, `freya-engine`, `freya-clipboard`, `torin` and
`ragnarok`. It never uses `freya` or `freya-winit`: the kit has its own Wayland
runner ([ADR 0034](../decisions/0034-celestialkit-on-freya.md)). Freya's
elements and hooks (`rect`, `label`, `use_state`…) come with the prelude.

## Does it use the GPU or the CPU?

| Work | Where it runs |
|---|---|
| Layout with `torin` | CPU |
| Drawing with Skia | GPU, through Vulkan ([ADR 0056](../decisions/0056-barrierkit-on-vulkan.md)) |
| Presentation | Vulkan swapchain on `VK_KHR_wayland_surface`: `MAILBOX` when the driver offers it, otherwise `FIFO` |
| Video (`video` feature) | VA-API decoding with zero-copy dmabuf import; without dmabuf the frames go through the CPU; without VA-API GStreamer decodes on the CPU |

There is no EGL and no `wl_shm` path. The kit takes the first Vulkan device with
a graphics queue that can present on the Wayland display. That device can be a
GPU or a software driver such as lavapipe. If there is none, the window does not
open and `run_window` returns `RunError::Vulkan`.

Freya 0.5.0 repaints the whole tree on every frame; partial damage waits for
Freya ([ADR 0047](../decisions/0047-barrierkit-in-layers.md), «Pending»).

## What it needs

| Need | Why |
|---|---|
| A Wayland compositor with `wl_compositor` and `xdg_wm_base` | Without them `run_window` returns `RunError::MissingGlobal`. The other protocols are optional ([18](18-platform.md)). |
| `libvulkan` and a Vulkan driver (ICD) | Drawing and presentation. |
| `nix develop` | The dev shell ships the Skia binaries, the Geist and Inter fonts, the Mesa Vulkan drivers and the GStreamer plugins. `./dev.sh` refuses to run outside it. |

## Which level to use

The kit has four levels; each one builds on the one below
([3](03-layers.md#the-map)). Start from the highest level that fits.

| If you want… | Use | Level |
|---|---|---|
| A settings app with a sidebar, search field and panes | `SettingsWindow` | 3 · Patterns |
| An app with its own navigation (a player, a file browser) | `SplitView`, `Sidebar` and `Titlebar` | 2 · Views |
| A small window or a dialog | `Application`, `VStack` and the controls | 1 · Primitives |
| A component that does not exist | Freya's `rect` and `label`, `use_interaction` and the `Palette`. If other apps need it, it goes into the kit. | 1 · Primitives |
| A window that draws its own titlebar | `run_window` and `WindowOptions` ([8](08-windows.md)) | 0 · Platform |

Next: [2. Your first app](02-first-app.md).
