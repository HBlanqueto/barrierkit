# Limitless effects: one object per effect

**Goal:** replace the effect requests of `limitless_window_style_v1` with objects named as in AppKit and SwiftUI, so a new macOS effect adds an object, not a request with a `since` number.
**Reader:** whoever writes the compositor side or moves the kit to it.
**Status:** the XML is in `crates/limitless/protocols/limitless-effects-v1.xml` and the Linux backend of the kit sends it (`platform/linux/effects.rs`). No compositor serves it yet, and the Linux code was checked for types against the generated bindings, not built or run.

## Why

macOS has no `set_backdrop`. A view with a material is a layer in the tree, and WindowServer draws it. The style protocol grew the other way: one request per effect, each with arrays of rectangles and its own version (6, 8, 9, 10), and an effect is removed by sending an empty array. An object has a lifetime: destroying it removes the effect.

## What each request becomes

| Style request | Effects object |
|---|---|
| `set_backdrop` (v10) | `limitless_visual_effect_v1`: `set_material`, `set_blending_mode`, `set_shape` |
| `set_scroll_edge` (v10) | `limitless_scroll_edge_effect_v1`: `set_edge`, `set_style`, `set_depth` |
| `set_glass` (v6) | gone: the sidebar is a `behind_window` visual effect of the `sidebar` material ([ADR 0100](../decisions.md)) |
| `glass_regular`, `glass_clear` of `set_backdrop` | `limitless_glass_effect_v1` (`set_style`, `set_shape`, `set_container`) and `limitless_glass_container_v1` (`set_spacing`, the kit's `glass_spacing`) |

## What stays in `limitless_window_style_v1`

- `set_bar`, `set_popover`, `set_popover_shape`, `set_panel`: the window chrome, not an effect.
- `set_knobs` (v9) and `set_plates` (v8): their lens is drawn above the surface, or sized as a share of the plate. This draft carries neither; they move when the compositor measures them.
- `set_cards` (v4) and `set_desktop` (v5): shell requests. They belong with the shell in `limitless-shell`, and the kit does not send them.

## Not in version 1

Tint and interaction of a glass (`Glass.tint`, `.interactive`), `state`, `isEmphasized` and `maskImage` of `NSVisualEffectView`, and a glass' own lens. The kit's Linux backend sends none of them today, so the protocol does not carry them; a later version adds them with `since`.

## What changed in the kit

1. `limitless` has the `effects` module; `set_glass`, `set_backdrop` and `set_scroll_edge` left `limitless_window_style_v1`, now at version 9.
2. `platform/linux/effects.rs` binds the global and keeps one backdrop object and one scroll edge object per chrome surface (`Applied`), recreated when the material or the glass spacing changes. `platform/linux/glass.rs` keeps the sidebar's visual effect in `SurfaceWindow::sidebar`.
3. `GlassPanels` lost `base`, `solid` and `chroma`: only `set_glass` read them.

Blocked until a compositor serves the protocol: a run on Linux. Until then a window of this kit on Malaysia's current compositor draws its sidebar and chromes as flat fills.
