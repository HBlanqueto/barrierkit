# 0039 — Infinity v1: window management for the shell

Status: **accepted**.

Yuki is now Malaysia and CelestialKit (`celestial-kit`) is BarrierKit (`barrier-kit`) since [ADR 0043](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0043-rebranding-a-malaysia.md).

## Context

Yuki's menubar (`yuki-shell menubar`) shows the **Ventana** ("Window") menu with
entries to place the active window in halves, quarters, layouts,
fill, center, restore and full screen. Today those entries appear greyed out
because there is no channel through which the shell can ask the compositor for those
geometry operations.

Infinity (see [glossary](https://github.com/HBlanqueto/malaysia/blob/main/docs/glossary.md)) is Yuki's private
app↔compositor protocol. The first piece implemented is window
management for the shell components, starting with the menubar.

## Decision

1. **Private protocol `infinity_window_manager_v1`.** It is defined in
   `crates/infinity-protocol/protocols/infinity-window-manager-v1.xml`. It is a server-side Wayland
   extension generated with `wayland-scanner` in the compositor and with the
   client generation in `celestial-kit`.
2. **Only the `yuki-shell` components can see the global.** Smithay's
   visibility filter (`GlobalDispatch::can_view`) allows it only when
   the client's executable (`/proc/<pid>/exe`) matches one of the
   trusted binaries:
   - `yuki-shell` in the same directory as the compositor, resolved from
     `std::env::current_exe()`.
   - `.yuki-shell-wrapped` in that same directory, for the Nix
     wrappers that rename the real binary.
   - The exact path given by the environment variable `YUKI_SHELL_EXE`, which
     a wrapper can export when the previous heuristic is not enough.
   - `YUKI_DEV=1` in the compositor's environment disables the filter for
     development.

   It is not strong security: any process that can pass itself off as
   `yuki-shell` or run with `YUKI_DEV=1` could use the protocol. The
   decision not to encrypt or authenticate with a token comes from ADR 0035:
   Yuki trusts its own shell processes by binary identity and
   development environment, not by a cryptographic mechanism.
3. **The requests act on an existing `zwlr_foreign_toplevel_handle_v1`.**
   The shell already handles those handles for the dock and the app menus. Reusing
   the same identifier avoids adding a second window namespace.
4. **Requests:**
   - `arrange(handle, layout)`: layout = `fill`, `center`, `left`, `right`,
     `top`, `bottom`, `top_left`, `top_right`, `bottom_left`, `bottom_right`.
   - `arrange_pair(layout)`: `side_by_side`, `side_by_side_reversed`,
     `stacked`, `stacked_reversed`. The active window takes the first half and
     the previous one in focus order takes the second.
   - `arrange_quarters`: the four most recent visible windows in
     quarters.
   - `restore(handle)`: returns to the rectangle before the first `arrange` and
     deletes the stored rectangle.
   - `set_fullscreen(handle, bool)`: enters or leaves real full screen.
5. **Differences from Maximize and from Full screen.** Maximize (green
   button, double click) keeps Yuki's frame and fills the work area.
   Full screen (`set_fullscreen`) has no frame, fills the whole output and
   is reflected in the `Fullscreen` state of `zwlr_foreign_toplevel_handle_v1`.
   Focus mode stays outside this protocol; it will be handled through another channel.
6. **Event `restorable(handle, bool)`.** It is sent when it changes whether a window
   has a stored restore rectangle. The shell uses it to enable or
   disable the entry "Regresar al tamaño anterior" ("Return to previous size"). A protocol event
   was chosen instead of reading it from `ToplevelInfo` because the
     restore state is not part of the toplevel state that
     `zwlr_foreign_toplevel_management_v1` publishes; it is internal to Infinity. Centralising
     the signal in the same protocol that produces the state keeps the shell
     from having to guess or poll.
7. **Geometries.** The calculations use the compositor's `workspace()` (output minus
   the exclusive zones of the bar and the dock) with a margin of 8 px around
   and 8 px between windows. `fill` takes the available area; `center` keeps the
   current size and centers the window.

## Versions

| Version | Adds |
|---|---|
| 1 | `arrange`, `arrange_pair`, `arrange_quarters`, `restore`, `set_fullscreen`, `restorable` and `destroy` |
| 2 | The `app_menu` event |
| 3 | The `gtk_menus` event |
| 4 | The `force_quit` request |
| 5 | The `space` event and the `focus_bar` enum |
| 6 | The `send_shortcut(handle, keys)` request, which types a chord into the window (malaysia ADR 0061) |

Opcodes follow the XML order. Version 4 declared `force_quit` before
`destroy`, which moved from opcode 5 to 6: the `destroy` of a v1–v3 client
reaches a v4 or later compositor as `force_quit`. New requests go after `destroy`.

## Consequences

- `yuki-shell menubar` can enable the geometry entries when
  Infinity is available and call the requests on the active window.
- The compositor gains one more server-side protocol, generated with
  `wayland-scanner`.
- `celestial-kit` includes the client generation of the protocol as an optional
  capability: if the compositor does not advertise the global, the Infinity functions
  in `Toplevels` simply do nothing.
- Any external client that tries to bind
  `infinity_window_manager_v1` will not see the global.

## References

- [`0034`](0034-celestialkit-sobre-freya.md) — CelestialKit on Freya.
- [`0035`](0035-identificacion-de-apps-propias-y-csd.md) — own apps and CSD.
- [`0036`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0036-menu-global.md) — global menu.
- [`0037`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0037-marco-compositor-dibuja-ventanas.md) — window
  frame drawn by the compositor.
- [`glossary.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/glossary.md) — definition of Infinity.
- `crates/infinity-protocol/protocols/infinity-window-manager-v1.xml`.
