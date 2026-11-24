# 3. Layer surfaces

**Goal:** put a shell component on an edge of the screen, reserve its space,
and change its place, size, pointer area and frames while it runs.
**Scope:** `run_layer`, `LayerSpec`, `Layer`, `Anchors`,
`KeyboardInteractivity`, `set_layer_geometry` and `LayerGeometry`,
`set_input_region` and `InputRegion`, `set_card_frames` and `CardFrames`,
`set_plate_frames` and `PlateFrames`, the popups and menus of a layer
surface, and the errors `RunError::LayerSize` and `MissingGlobal`. The first
panel is in [2. First component](02-first-component.md); the blur behind a
surface is in [7. Glass and blur](07-glass-and-blur.md); popups in general are
in [8. Windows](../barrierkit/08-windows.md#popups).
**Reader:** whoever writes a menu bar, a Dock, a panel, notification banners
or a background for Malaysia.
**Input:** a root component, as in [2](02-first-component.md).
**Output:** a layer surface that sits on its edge, reserves its space and
changes its geometry while it runs.

## Steps

1. Choose the [layer](#layers), the [anchors and the size](#anchors-and-the-size-rule),
   the [exclusive zone](#exclusive-zone) and the
   [keyboard interactivity](#keyboard-interactivity) of the surface, and put
   them in a `LayerSpec`.
2. Run the root component with `run_layer(app, spec)`. It returns when the
   compositor closes the surface ([`run_layer`](#run_layer-and-layerspec)).
3. While it runs, change its anchors, size, zone or keyboard mode with
   [`set_layer_geometry`](#changing-the-geometry-while-running).
4. If the surface draws less than its rectangle, limit where it takes the
   pointer with [`set_input_region`](#input-regions).
5. If it draws cards or icon plates, ask the compositor for their
   [frames](#card-and-plate-frames).
6. Open menus and popups from it as from a window
   ([Popups and menus](#popups-and-menus)).

This program is `crates/bk-shell/examples/side_panel.rs`. It holds the right
edge, reserves its width, and changes that width on a press:

```rust
use bk::{
    prelude::*,
    theme::{SurfaceKind, resolved, surface_background},
};
use bk_shell::{
    Anchors, KeyboardInteractivity, Layer, LayerGeometry, LayerSpec, run_layer,
    set_layer_geometry,
};
use freya_core::integration::AppComponent;

const RIGHT_EDGE: Anchors = Anchors {
    top: true,
    bottom: true,
    left: false,
    right: true,
};
const NARROW: u32 = 280;
const WIDE: u32 = 420;

fn side_panel() -> impl IntoElement {
    let mut width = use_state(|| NARROW);
    VStack::new()
        .expanded()
        .background(color(surface_background(
            &resolved(),
            SurfaceKind::LayerShell,
        )))
        .child(Text::new("Side panel").font(TITLE))
        .child(Button::new("Change width").on_press(move |_| {
            let next = if *width.peek() == NARROW { WIDE } else { NARROW };
            width.set(next);
            set_layer_geometry(LayerGeometry {
                anchors: RIGHT_EDGE,
                exclusive_zone: next as i32,
                size: (next, 0),
                keyboard: KeyboardInteractivity::OnDemand,
            });
        }))
}

fn main() -> Result<(), RunError> {
    run_layer(
        AppComponent::from(side_panel),
        LayerSpec {
            namespace: "side-panel".to_string(),
            layer: Layer::Top,
            anchors: RIGHT_EDGE,
            exclusive_zone: NARROW as i32,
            keyboard: KeyboardInteractivity::OnDemand,
            size: Some((NARROW, 0)),
        },
    )
}
```

How to run it under a compositor is in [10. Run and debug](10-run-and-debug.md).

## `run_layer` and `LayerSpec`

`run_layer(app: AppComponent, spec: LayerSpec) -> Result<(), RunError>` draws
`app` in one `zwlr_layer_surface_v1`. It runs these steps:

1. It checks the [size rule](#anchors-and-the-size-rule) and returns
   `RunError::LayerSize` before it connects.
2. It connects and binds `zwlr_layer_shell_v1`, versions 1 to 4, beside the
   runner's globals ([18. Platform](../barrierkit/18-platform.md#wayland-protocols)).
   Without it, it logs at `info` level and returns
   `RunError::MissingGlobal("zwlr_layer_shell_v1")`.
3. It creates the layer surface with no output, so the compositor picks one,
   in general the one the user used last.
4. It sets the anchors, the exclusive zone, the keyboard mode and, when `size`
   is `Some`, the size, and commits without a buffer.
5. On each `configure` it acknowledges it and draws at the size it got.
6. On `closed` it returns `Ok(())`.

| `LayerSpec` field | Type | Meaning |
|---|---|---|
| `namespace` | `String` | The purpose of the surface, as the compositor sees it |
| `layer` | `Layer` | [The layer](#layers); fixed for the surface's life |
| `anchors` | `Anchors` | The edges it sticks to |
| `exclusive_zone` | `i32` | [The space it reserves](#exclusive-zone), in logical pixels |
| `keyboard` | `KeyboardInteractivity` | [Whether it takes the keyboard](#keyboard-interactivity) |
| `size` | `Option<(u32, u32)>` | Logical size; a 0 lets the compositor choose that dimension; `None` sends no size, which is 0 × 0 |

The surface is cleared transparent. The component paints its own background:
`surface_background(&resolved(), SurfaceKind::LayerShell)` from `bk::theme`
gives the bar color ([4. Appearance and tokens](../barrierkit/04-appearance-and-tokens.md#theme-functions)).

Every component of the surface, and of its popups, can read these root
contexts:

| Context | Chapter |
|---|---|
| `Popups`, `Timers` | [8. Windows](../barrierkit/08-windows.md#running-a-window) |
| `Toplevels`, `Activation` | [4. Desktop windows](04-desktop-windows.md) |
| `use_screen_lock()` | [5. Lock screen](05-lock-screen.md#locking-notices-for-the-other-surfaces) |

There is no `WindowControls`: `WindowControls::get()` panics in a layer
surface. `SessionUnlock` exists only under `run_lock`.

## Layers

`Layer` orders the surfaces by depth, bottom first. Windows sit between
`Bottom` and `Top`; a full-screen window usually sits in `Top`.

| `Layer` | Sits | For example |
|---|---|---|
| `Background` | Under everything | A wallpaper |
| `Bottom` | Under the windows | Desktop widgets |
| `Top` (default) | Over the windows | A menu bar, a Dock, a panel |
| `Overlay` | Over everything, full-screen windows included | Banners that must show in full screen |

`set_layer_geometry` cannot move a surface to another layer.

## Anchors and the size rule

`Anchors` has four `bool` fields: `top`, `bottom`, `left` and `right`.
`Anchors::default()` has none set.

| Constant | Edges | For |
|---|---|---|
| `Anchors::TOP` | top, left, right | A bar along the top edge |
| `Anchors::BOTTOM` | bottom, left, right | A bar along the bottom edge |

Two edges that meet anchor the surface to their corner. One edge centers it on
that edge; no edge centers it on the output. A surface narrower than the space
between its anchors is centered there.

The size rule: a width of 0 needs `left` and `right`, and a height of 0 needs
`top` and `bottom`. The compositor then gives that dimension in its
`configure`. `size: None` is 0 × 0, so it needs all four edges.

```rust
// ✗ a width of 0 with only the top edge: RunError::LayerSize
anchors: Anchors { top: true, ..Anchors::default() },
size: Some((0, 30)),
// ✓ a width of 0 between the left and right edges
anchors: Anchors::TOP,
size: Some((0, 30)),
```

## Exclusive zone

`exclusive_zone` tells the compositor how to share the anchored edge:

| Value | Effect | For example |
|---|---|---|
| Positive | That distance from the edge is reserved: maximized windows stop there | A menu bar, a Dock that does not hide |
| `0` | The surface moves to avoid the zones of other surfaces | Notification banners |
| `-1` | The surface does not move and reaches the edges it is anchored to | A wallpaper |

A positive zone counts only when the surface is anchored to one edge, or to one
edge and the two edges beside it. Unanchored, or anchored to a corner, to two
opposite edges or to all four, it counts as `0`.

## Keyboard interactivity

| `KeyboardInteractivity` | The surface | For example |
|---|---|---|
| `None` (default) | Never gets the keyboard | A bar that only takes clicks |
| `Exclusive` | In `Top` and `Overlay`, the topmost such surface gets every key; in `Bottom` and `Background`, normal focus | A password prompt |
| `OnDemand` | Gets the keyboard as a window does, for example on a click, and the user can leave it | A panel with a text field |

`OnDemand` needs version 4 of `zwlr_layer_shell_v1`. The popups of a layer
surface inherit its mode (`zwlr_layer_surface_v1.get_popup`), so a menu that
takes keys may get none from a `None` surface.

## Changing the geometry while running

`set_layer_geometry(LayerGeometry { anchors, exclusive_zone, size, keyboard })`
changes the layer surface of the calling component:

1. Call it from a component of the surface or from one of its handlers. From
   a popup's content it does nothing: the popup is not a layer surface.
2. The runner checks the size rule. A geometry that breaks it is ignored with
   the warning `BarrierKit: ignored a layer geometry with an unanchored zero size`.
3. The runner sends the anchors, the size, the exclusive zone and the keyboard
   mode, and commits.
4. The compositor answers with a `configure`; the next frame has the new size.

| `LayerGeometry` field | Type | Meaning |
|---|---|---|
| `anchors` | `Anchors` | As in `LayerSpec` |
| `exclusive_zone` | `i32` | As in `LayerSpec` |
| `size` | `(u32, u32)` | As in `LayerSpec`, always sent |
| `keyboard` | `KeyboardInteractivity` | As in `LayerSpec` |

The namespace and the layer stay. Send every field, not only the one that
changes: a banner that grows with its content keeps its anchors and its zone.

## Input regions

`set_input_region(InputRegion { rects })` limits where the calling surface
takes the pointer and touch:

1. `rects` are `(x, y, width, height)` in surface-local logical pixels.
2. The runner adds them to one `wl_region`, sets it with
   `wl_surface.set_input_region` and commits.
3. Outside the rectangles, presses reach the windows below. The keyboard does
   not change.

An empty list takes no pointer anywhere. To take the whole surface again, send
its whole rectangle. Send the list again whenever the shape changes, for
example when a Dock with room above its bar for magnified icons takes the
pointer only on the bar, or on a strip at the screen edge while it hides.

## Card and plate frames

Both requests go through `infinity_window_style_v1`, which the runner binds
([18. Platform](../barrierkit/18-platform.md#wayland-protocols)). Without the
needed version nothing is sent and the compositor draws no frames. A later
call replaces the list; an empty list removes it.

`set_card_frames(CardFrames { radius, cards })` (`set_cards`, version 4) is for
a surface that draws cards, such as notification banners:

1. `cards` are `(x, y, width, height)` in surface-local logical pixels;
   `radius` is a `u32`.
2. The compositor draws the window shadow under the surface, and the inner and
   outer borders above it, around a continuous corner of `radius`.
3. The app draws each card with that corner,
   `bk::style::continuous_corner(radius as f32)`, so the shapes meet.

`set_plate_frames(PlateFrames { corner, plates })` (`set_plates`, version 8) is
for icons whose plate the compositor draws, such as the Dock's in the clear
icon style:

1. `plates` are `(x, y, width, height)` in surface-local logical pixels.
2. `corner` is the radius of each plate's continuous corner, in thousandths of
   its shorter side, so a plate that magnifies keeps its shape. The kit's plate
   is `(ICON_PLATE.corner * THOUSANDTHS) as u32` (`ctdesign`).
3. The compositor draws each plate as glass under the surface. The app draws
   only the logo, with `Image::glass_plate()`
   ([Images](../barrierkit/04-appearance-and-tokens.md#images-and-template-rendering)).

The compositor draws the plates of the Dock's layer surface only.

## Popups and menus

A layer surface opens popups as a window does
([8. Windows](../barrierkit/08-windows.md#popups)): `Popups::get().open`,
`.context_menu`, `pop_up_menu`, `.help` and `Popover` work the same.

1. `anchor_rect` is in the layer surface's logical pixels.
2. The runner creates the `xdg_popup` with no parent, gives it the layer
   surface as parent with `zwlr_layer_surface_v1.get_popup` before the popup's
   first commit, and commits the layer surface.
3. A popup opened from a popup is that popup's child, as in a window.
4. Its content gets the same root contexts as the layer surface.

A `MenuView` built with `MenuView::new` takes its live palette from
`SurfaceKind::LayerShell` (`bk::theme`): its background lifts from the bar
color, as `menu_palette` does. `.with_surface(SurfaceKind::Toplevel)` gives it
a window's palette. `.context_menu` and `pop_up_menu` always use the window's
popover palette ([7. Menus and popovers](../barrierkit/07-menus-and-popovers.md#menu-view)).

## Errors

`run_layer` returns the runner's `RunError`
([8. Windows](../barrierkit/08-windows.md#errors)). Two variants are its own:

| Variant | Raised when |
|---|---|
| `LayerSize` | The `LayerSpec` breaks the size rule; returned before it connects |
| `MissingGlobal("zwlr_layer_shell_v1")` | The compositor offers no layer shell |

Next: [4. Desktop windows](04-desktop-windows.md).
