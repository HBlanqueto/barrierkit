# 0041 — Island sidebar

Status: **accepted** (2026-10-26).

Yuki is now Malaysia and CelestialKit (`celestial-kit`) is BarrierKit (`barrier-kit`) since [ADR 0043](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0043-rebranding-a-malaysia.md).

## Context

`celestial-kit` provides a reusable sidebar (`Sidebar`, `SidebarItem`,
`SidebarLayout`) used by `yuki-settings` and intended for future own apps.
The Neptunia theme defines two ways to draw it:

- **Island**: a floating card with rounded corners, a subtle border, an
  inner line, a shadow and a background blur.
- **Flat**: an embedded list that blends with the window background.

The sidebar must be able to switch mode through configuration without touching the code of
the apps.

## Decision

1. **The sidebar style is a design token.**
   `components.window.sidebar.default-style` in
   `crates/yuki-design/neptunia/components.json` is `"island"`. Flat mode
   is `"flat"`.

2. **The concrete values live in `yuki-design`, never in the apps.**
   Neptunia sets the inset, radius, colors (`sidebar-island-bg`,
   `sidebar-island-rim`, `sidebar-island-inner`, ...), shadow (`shadow.island`)
   and blur (`30`) in `components.json` and `tokens.json`. `build.rs`
   exports them as Rust constants.

3. **`yuki-appearance` resolves the style at runtime.**
   `AppearanceConfig::sidebar_style` (`SidebarStyle::Island | Flat`) is read from
   `appearance.domain`; by default it is `Island`. `resolve()` returns a
   `SidebarSurface` with the colors and the shadow already blended for the current
   scheme.

4. **`celestial-kit` draws the sidebar from `SidebarSurface`.** The app
   only mounts `Sidebar::new()` with its children; the kit applies the background, the radius,
   the shadow and the traffic lights according to the resolved style.

5. **In light mode the island uses no border or shadow.** In `tokens.json` the
   border color and the shadow are transparent in light; in dark/dark tint they
   are enabled to separate the island from the window background.

## Consequences

- The own apps get the Neptunia style automatically.
- Changing the default style only requires editing `components.json`;
  rebuilding regenerates the constants.
- Flat mode exists but is not exposed in the Ajustes ("Settings") UI yet.
- The `yuki-settings` sidebar shows the integrated traffic lights thanks to
  `.traffic_lights(true)`; their position also comes from the tokens.

## References

- [`0032`](0032-neptunia-como-fuente-de-diseno.md) — Neptunia
  as the design source.
- [`0034`](0034-celestialkit-sobre-freya.md) — CelestialKit on Freya.
- `crates/yuki-design/neptunia/components.json` — definition of `sidebar`.
- `crates/yuki-design/neptunia/tokens.json` — island colors.
- `crates/yuki-appearance/src/lib.rs` — `SidebarStyle` and `SidebarSurface`.
- `crates/yuki-settings/src/main.rs` — use of `SidebarLayout`.
- `crates/celestial-kit/src/sidebar.rs` — kit component.
