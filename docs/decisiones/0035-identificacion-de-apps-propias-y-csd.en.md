# 0035 — Identifying own apps and CSD

Status: **accepted** (2026-10-23).

Yuki is now Malaysia and CelestialKit (`celestial-kit`) is BarrierKit (`barrier-kit`) since [ADR 0043](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0043-rebranding-a-malaysia.en.md).

## Context

Yuki's own apps are drawn with CelestialKit and carry their own
titlebar and traffic lights (CSD). The compositor can also add
server-side chrome (`window.decorations`, ADR 0019/0020). With `force_server` that
policy would paint a second bar over an own app.

A stable way is needed for the compositor to recognise a window as
own and never add its titlebar to it. It cannot rely on the title or on the
decoration protocol alone: a third-party app can also request CSD, and
an own app could forget to request it.

## Decision

1. **App-id prefix.** A CelestialKit app uses an `app_id` that starts
   with `NATIVE_APP_ID_PREFIX = "io.github.hblanqueto.yuki."`, a constant of
   `yuki-design`.
2. **Explicit CSD.** Before its first commit, the app requests
   `zxdg_toplevel_decoration_v1.set_mode(ClientSide)`, as `otto-kit` does with
   `WindowDecorations::RequestClient`.
3. **Precedence.** ~~For a window with the prefix, the effective mode is
   always `Client`, even with `force_server`.~~ **Replaced by the section
   "App choice"**: an own app chooses `Client` or `Server` and Yuki
   respects that choice.
4. **Order does not matter.** The compositor stores the *requested* mode per window and
   recomputes the effective one when `app_id_changed` arrives and before the initial
   configure. A KDE request that arrives before the `xdg_toplevel` is stored as
   requested, not as effective.
5. **Warning.** If an own app explicitly requested `Server`, the compositor
   warns once per window («app propia que no pidió CSD» ("own app that did not request CSD")). It still gets no bar.
6. **It is not security.** The prefix only decides whether a bar is drawn. Any
   client can use that app-id; it authenticates and isolates nothing. Infinity will
   replace it when it exists.

## Consequences

- The compositor shares the pure rule `effective_decoration(policy, requested,
  native)`; the handlers only translate protocol into requested mode.
- CelestialKit exposes `TrafficLights` for the own apps. The runner does not
  create `xdg_toplevel` yet; the contract for doing so is noted as
  `TODO(ADR 0035)` in `platform/mod.rs`.
- An own app without CSD does not receive the compositor's bar: it shows without chrome,
  which is the visible failure the warning aims to detect.
- `yuki_appearance::traffic` stays pure and without Skia; the CelestialKit
  component is the only one that draws.

## App choice (amendment 2026-10-23)

Replaces the rule "native → Client always" of point 3.

- Each CelestialKit window requests `Client` and draws its unified 40 px bar
  (`titlebar-h`), with `TrafficLights` and room for a toolbar. The kit does
  not offer a `Server` request.
- The kit **always** requests the mode explicitly with
  `xdg-decoration.set_mode` before the first commit, together with
  `set_app_id(NATIVE_APP_ID_PREFIX + nombre)`.
- For an own window, Yuki respects the requested mode and does not apply
  `window.decorations`: `force_server` and `force_client` only affect
  third-party apps.
- `requested` is `None` until `request_mode` arrives; creating the
  decoration object or calling `unset_mode` do not count. An own app without a mode negotiates
  `Client` and the compositor warns once per window («app propia sin modo de
  decoración» ("own app without a decoration mode")).
- If the compositor does not advertise `xdg-decoration`, the kit still draws
  its 44 px bar.

## Infinity replaces the prefix (amendment 2026-11-07)

Replaces point 1 and fulfils point 6.

- The compositor recognizes a kit app because its client binds
  `infinity_window_style_v1`. The kit always binds it at startup
  (`platform/runtime.rs`).
- The `app_id` no longer counts. An app built with the kit by another author
  (`org.curseterm.CurseTerm`) gets the same treatment as one of our own: its
  decoration mode is respected and its popups carry the kit frame.
- `NATIVE_APP_ID_PREFIX` remains only as the default `app_id` when the app does
  not request another one with `Window::app_id`.
- It is still not security: any client can bind the protocol.
