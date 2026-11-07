# 14. Platform

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
| Context | A Skia `DirectContext` shared by all windows, with a 128 MB cache; every 10 s it releases what went 10 s unused |
| Swapchain | One per surface, `B8G8R8A8_UNORM`, premultiplied alpha; `MAILBOX` if available, otherwise `FIFO`; one `SkSurface` per image. With `VK_EXT_swapchain_maintenance1`, each present carries a fence and a resize retires the old swapchain without waiting for the GPU; it is destroyed once its last present signals |

With the `video` feature the device also enables the dmabuf import extensions
(chapter [13](13-fondos-y-medios.md)). The decision is in
[ADR 0056](../decisiones/0056-barrierkit-sobre-vulkan.md).

## Wayland protocols

| Protocol | Required? | What for |
|---|---|---|
| `wl_compositor`, `xdg_wm_base`, `wl_seat`, `wl_output` | Yes | Windows, input and outputs |
| `wp_fractional_scale_manager_v1` + `wp_viewporter` | No | Fractional scaling; without them, integer scales |
| `wp_cursor_shape_manager_v1` | No | Per-component cursors (`col-resize`, text…); without it, the default cursor |
| `zxdg_decoration_manager_v1` | No | Server-side decoration if the app asks for it |
| `xdg_activation_v1` | No | Bringing the window to the front when an instance is reactivated |
| `wl_data_device_manager` | No | Clipboard and drag and drop |
| `zwp_text_input_manager_v3` | No | Input method: composition and committed text |
| `zwp_primary_selection_device_manager_v1` | No | Primary selection, the one the middle button pastes |
| `org_kde_kwin_appmenu_manager` | No | Showing the global menu in the menu bar |
| `infinity_focus_manager_v1` | No | Notices from the Malaysia compositor about the focus bar |
| `infinity_window_style_v1` | No | The `Titlebar` sends its bar with `set_bar`: `titlebar` when `close_only`, `unified` with an island sidebar and `compact` otherwise (Malaysia ADR 0059). From version 3, each utility window declares itself a panel with `set_panel` and the options of its `PanelBehavior` |

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

## Desktop services

- `notifications::post(Notice { .. }, |token| ..)` posts a notification
  through `org.freedesktop.Notifications`; when it is clicked, the function
  gets the xdg-activation token the server sends. `play_event_sound(name)`
  plays a sound from the freedesktop sound theme.
- `secrets::Secrets::open()` opens the Secret Service default collection
  (`org.freedesktop.secrets`), the Linux keyring: `search`, `secret`,
  `create`, `set_secret` and `delete`. Secrets come back in `Zeroizing`.
  With the keyring locked it returns `Error::Locked` and the service shows
  its own prompt.

## The shell part (`shell` feature)

Only `malaysia-shell` enables `barrier-kit/shell`. It adds:

| API | Protocol | For |
|---|---|---|
| `shell::run_layer(app, LayerSpec)` | `zwlr_layer_shell_v1` | Menu bar, Dock, background |
| `shell::run_lock(app)` | `ext_session_lock_v1` | Lock screen |
| `shell::Toplevels` | `ext_foreign_toplevel_list_v1`, `zwlr_foreign_toplevel_manager_v1` | Window list and control for the Dock. Each `ToplevelInfo` carries a stable `id`; `activate`, `close`, `minimize` and the rest act by that `id` |
| `shell::InfinityLayout`, `Toplevels::arrange` | `infinity_window_manager_v1` | Tiling windows |
| `shell::Activation` | `xdg_activation_v1` | Launching apps from the Dock |
| `shell::use_status_items()` | [StatusNotifierWatcher](https://specifications.freedesktop.org/status-notifier-item/latest/) and dbusmenu | App status icons in the menu bar: acts as the watcher (or follows the one already running), reads the title, icon (`IconName`, KDE's `IconThemePath` or `IconPixmap`, the attention one when the status is `NeedsAttention`) and menu, and reads them again on every signal. `StatusHost` sends `Activate`, `SecondaryActivate` and menu clicks; `pixmap_image` paints an `IconPixmap` |

The submodules of `shell/` are private and their API comes only from
`shell/mod.rs`. `platform/` does not import `shell/`: the shell hooks in
through a single runner `Extension`, and the `cfg(feature = "shell")` blocks
in `platform/` only compile that hook (extension surfaces, their popups and
the globals list)
([ADR 0047](../decisiones/0047-barrierkit-en-capas.md) §4).

## Errors

`RunError` has the variants `NoDisplay`, `MissingGlobal`, `Vulkan`,
`DeviceLost`, `SurfaceLost`, `LayerSize`, `EventLoop`, `Skia`, `Wayland`,
`Registry` and `Io`.

## The first frame

The kit draws the first frame of a window or a layer when its first configure
arrives, without waiting for a frame callback. `xdg_surface` and
`zwlr_layer_surface_v1` ask for this, and a compositor may not send frame
callbacks to a surface without content. Later frames wait for their frame
callback. Checked on 2026-11-03 on Weston 16 (window) and on 2026-11-04 on
sway 1.12 (layer), both headless.

Next: [16. Frequently asked questions](16-preguntas.md).
