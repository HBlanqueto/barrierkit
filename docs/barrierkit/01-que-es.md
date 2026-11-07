# 1. What BarrierKit is

**Goal:** know what the kit is, what it ships and what it needs to run.
**Reader:** anyone starting with the kit.

## In one sentence

BarrierKit is to Malaysia what AppKit is to macOS or GTK + libadwaita is
to GNOME: the set of pieces that builds the native apps and the shell. An app
does not draw buttons or sidebars: it asks the kit for them, and they all look
and behave the same.

## Equivalents

| Concept | AppKit / SwiftUI | libadwaita | BarrierKit |
|---|---|---|---|
| API style | Declarative SwiftUI | GtkBuilder + signals | Declarative, on top of Freya |
| Button | `NSButton` | `GtkButton` | `controls::Button` |
| Sidebar + detail | `NSSplitView` | `AdwNavigationSplitView` | `views::split::SplitView` |
| Source list | `NSOutlineView` (source list) | `GtkListBox` `.navigation-sidebar` | `sidebar::Sidebar` |
| Form | `Form` (SwiftUI) | `AdwPreferencesGroup` | `views::form::Form` |
| Preferences window | `Settings` | `AdwPreferencesWindow` | `patterns::preferences::PreferencesWindow` |
| Actions and shortcuts | `NSMenuItem` + responder chain | `GAction` | `app::Action` |
| App preferences | `UserDefaults` | `GSettings` | `app::load_defaults` / `save_defaults` |
| System colors | `NSColor.labelColor`… | CSS variables | `style::Palette` |

## The crates

The [`barrierkit`](https://github.com/HBlanqueto/barrierkit) repository has
four library crates:

| Crate | What it is |
|---|---|
| `barrier-kit` | The kit: Wayland runner, style, controls, views, patterns and app infrastructure. |
| `vow-design` | **Tengen**, the design system. It turns `tengen/tokens.json` and `tengen/components.json` into Rust constants. |
| `vow-appearance` | Reads `appearance.domain` and the wallpaper and resolves them into a concrete palette. It watches for changes. |
| `infinity-protocol` | Private Wayland protocol between the apps and the Malaysia compositor. |

The kit builds on [Freya](https://freyaui.dev) `=0.4.3`, but only on its core
crates: `freya-core`, `freya-components` (no features), `freya-animation`,
`freya-engine`, `torin` and `ragnarok`. Never on `freya` or `freya-winit`: the
kit has its own Wayland runner ([ADR 0034](../decisiones/0034-celestialkit-sobre-freya.md)).

## Does it use the GPU or the CPU?

| Work | Where it runs |
|---|---|
| Layout with `torin` | CPU |
| Drawing with Skia | **GPU, through Vulkan** ([ADR 0056](../decisiones/0056-barrierkit-sobre-vulkan.md)) |
| Presentation | Vulkan swapchain on `VK_KHR_wayland_surface`, `MAILBOX` if available, otherwise `FIFO` |
| Video (`video` feature) | GPU with VA-API and dmabuf import; if missing, it downloads to the CPU or decodes on the CPU |

There is no EGL or `wl_shm` path. The kit picks the first Vulkan device that
can present on the display. That device can be a real GPU or a software driver
such as lavapipe (`llvmpipe`): the screenshots in this documentation were made
that way, without a GPU. If there is no Vulkan device, the window does not open
and `run_window` returns `RunError::Vulkan`.

Freya 0.4.3 repaints the whole tree on every frame; there is no partial damage
yet ([ADR 0047](../decisiones/0047-barrierkit-en-capas.md), section «Pendiente» ("Pending")).

## What it needs

- A Wayland compositor with `wl_compositor`, `xdg_wm_base` and `wl_seat`. The
  other protocols are optional (chapter [14](14-plataforma.md)).
- `libvulkan` and a Vulkan driver (ICD).
- Building inside `nix develop` (Malaysia ADR 0007): the dev shell ships
  Skia, the fonts (Geist) and the RADV driver.

Next: [2. Your first app](02-primera-app.md).
