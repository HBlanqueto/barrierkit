# Limitless effects: one object per effect

**Goal:** replace the effect requests of `limitless_window_style_v1` with objects named as in AppKit and SwiftUI, so a new macOS effect adds an object, not a request with a `since` number.
**Reader:** whoever writes the compositor side or moves the kit to it.
**Status:** design. The draft is [`limitless-effects-v1.xml`](limitless-effects-v1.xml); it generates client and server code with `wayland-scanner` 0.31. Nothing in the kit sends it yet.

## Why

macOS has no `set_backdrop`. A view with a material is a layer in the tree, and WindowServer draws it. The style protocol grew the other way: one request per effect, each with arrays of rectangles and its own version (6, 8, 9, 10), and an effect is removed by sending an empty array. An object has a lifetime: destroying it removes the effect.

## What each request becomes

| Style request | Effects object |
|---|---|
| `set_backdrop` (v10) | `limitless_visual_effect_v1`: `set_material`, `set_blending_mode`, `set_shape` |
| `set_scroll_edge` (v10) | `limitless_scroll_edge_effect_v1`: `set_edge`, `set_style`, `set_depth` |
| `set_glass` (v6) | gone: the kit sends `set_backdrop` with `sidebar` from version 10 ([ADR 0100](../decisions.md)) |
| pending v11: tint, interaction, group spacing | `limitless_glass_effect_v1` (`set_tint`, `set_interactive`, `set_container`) and `limitless_glass_container_v1` (`set_spacing`) |

## What stays in `limitless_window_style_v1`

- `set_bar`, `set_popover`, `set_popover_shape`, `set_panel`: the window chrome, not an effect.
- `set_knobs` (v9) and `set_plates` (v8): their lens is drawn above the surface, or sized as a share of the plate. This draft carries neither; they move when the compositor measures them.
- `set_cards` (v4) and `set_desktop` (v5): shell requests. They belong with the shell in `limitless-shell`, and the kit does not send them.

## Not in version 1

`state`, `isEmphasized` and `maskImage` of `NSVisualEffectView`, the hovered and pressed look of a glass, and a glass' own lens: the kit sends none of them today, and the compositor does not draw them.

## Steps to move the kit

1. The compositor serves `limitless_effects_v1` (Remake-Malaysia; the Malaysia in `~/src/malaysia` serves only style v10).
2. `limitless-protocol` gets the XML under `protocols/` and a `effects` module next to `focus` and `style`.
3. `platform/linux/glass.rs` and `platform/linux/chrome_surface.rs` create one object per sidebar and per chrome, and send the same rectangles and materials they send now. `SurfaceWindow` keeps the objects and destroys them with the surface.
4. `set_backdrop`, `set_scroll_edge` and `set_glass` leave the style XML, and the version of `limitless_window_style_v1` is renumbered.

Blocked: nothing serves the protocol, and `platform/linux/` does not compile on macOS, so step 3 needs a Linux build.
