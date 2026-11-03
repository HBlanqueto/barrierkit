# 13. Platform

**Goal:** understand what the kit's runner does underneath the controls.
**Reader:** anyone who works inside the kit or debugs an app on another
desktop.

## The runner

`platform::run_window` opens a Wayland connection with smithay-client-toolkit,
creates an `xdg_toplevel` surface and runs a `calloop` loop. On each frame:

1. It handles the Wayland events (input, configure, frame callbacks).
2. It updates the Freya tree and computes the layout with `torin`.
3. It draws with Skia onto a Vulkan swapchain image and presents it.

The kit redraws when the state or the appearance changes, at the pace of the
compositor's frame callbacks.

## Vulkan

| Step | Detail |
|---|---|
| Instance | `VK_KHR_surface` and `VK_KHR_wayland_surface` |
| Device | The first one that can present on the display; Mesa's device-select layer puts the compositor's GPU first |
| Context | A Skia `DirectContext` shared by all windows, with a 128 MB cache |
| Swapchain | One per surface, `B8G8R8A8_UNORM`, premultiplied alpha; `MAILBOX` if available, otherwise `FIFO` |

With the `video` feature the device also enables the dmabuf import extensions
(chapter [12](12-fondos-y-medios.en.md)). The decision is in
[ADR 0056](../decisiones/0056-barrierkit-sobre-vulkan.md).

## Wayland protocols

| Protocol | Required? | What for |
|---|---|---|
| `wl_compositor`, `xdg_wm_base`, `wl_seat`, `wl_output` | Yes | Windows, input and outputs |
| `wp_fractional_scale_manager_v1` + `wp_viewporter` | No | Fractional scaling; without them, integer scales |
| `wp_cursor_shape_manager_v1` | No | Per-component cursors (`col-resize`, text…); without it, the default cursor |
| `zxdg_decoration_manager_v1` | No | Server-side decoration if the app asks for it |
| `xdg_activation_v1` | No | Bringing the window to the front when an instance is reactivated |
| `wl_data_device_manager` | No | Clipboard |
| `org_kde_kwin_appmenu_manager` | No | Showing the global menu in the menu bar |
| `infinity_focus_manager_v1` | No | Notices from the MalaysiaDreams compositor about the focus bar |

If an optional protocol is missing, the runner logs it with `tracing::info!`
and continues. If a required one is missing, `run_window` returns
`RunError::MissingGlobal`.

## Input, focus and accessibility

- The keyboard goes to the focused node; Tab enables keyboard navigation and
  the focus ring.
- The accessibility tree is published over AT-SPI with `accesskit_unix`: a
  screen reader can focus and press.
- `Cursor::set(CursorIcon::..)` changes the cursor if
  `wp_cursor_shape_manager_v1` is present.

## The shell part (`shell` feature)

Only `malaysia-shell` enables `barrier-kit/shell`. It adds:

| API | Protocol | For |
|---|---|---|
| `shell::run_layer(app, LayerSpec)` | `zwlr_layer_shell_v1` | Menu bar, Dock, background |
| `shell::run_lock(app)` | `ext_session_lock_v1` | Lock screen |
| `shell::Toplevels` | `ext_foreign_toplevel_list_v1`, `zwlr_foreign_toplevel_manager_v1` | Window list and control for the Dock |
| `shell::InfinityLayout`, `Toplevels::arrange` | `infinity_window_manager_v1` | Tiling windows |
| `shell::Activation` | `xdg_activation_v1` | Launching apps from the Dock |

The submodules of `shell/` are private and their API comes only from
`shell/mod.rs`. `platform/` knows nothing about the shell: the shell hooks in
through a single runner `Extension`
([ADR 0047](../decisiones/0047-barrierkit-en-capas.md) §4).

## Errors

`RunError` has the variants `NoDisplay`, `MissingGlobal`, `Vulkan`,
`EventLoop`, `Skia`, `Wayland`, `Registry` and `Io`.

## Known limitation: the first frame

The kit requests a frame callback and waits to receive it before drawing the
first frame. Weston and sway do not send frame callbacks to a surface that has
no content yet, so on them the window never appears (checked on 2026-11-03).
The [README](../../README.md) lists MalaysiaDreams and GNOME/Mutter as tested.
The screenshots in this documentation were made on sway with the first frame
drawn without waiting (see [`assets/README.en.md`](assets/README.en.md)).

Next: [14. Test apps](14-apps-de-prueba.en.md).
