# 0039 — Infinity v1: window management for the shell

Status: **accepted**, inherited from Malaysia. The full record is
[Malaysia ADR 0039](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0039-infinity-v1-gestion-de-ventanas-para-la-shell.en.md).

## What still holds in the kit

1. `infinity_window_manager_v1` is a private Wayland protocol, defined in
   `crates/infinity-protocol/protocols/infinity-window-manager-v1.xml` next to
   `infinity-focus-v1.xml` and `infinity-window-style-v1.xml`.
2. `infinity-protocol` generates the client side (feature `client`, used by
   `bk`) and the server side (feature `server`, used by the compositor).
3. Which clients may bind the global is the compositor's policy, in Malaysia.

The globals the kit binds and what it does without them are in
[18](../barrierkit/18-platform.md#wayland-protocols).
