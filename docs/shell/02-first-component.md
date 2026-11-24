# 2. Your first component

**Goal:** put a panel with a text along the top edge of the screen, and keep
windows out of its space.
**Scope:** from an empty crate to a top panel that runs under a compositor with
`zwlr_layer_shell_v1`. The fields of `LayerSpec` and the other layers are in
[3](03-layer-surfaces.md); running nested and the log are in
[10](10-run-and-debug.md).
**Reader:** whoever has written the app guide's
[first app](../barrierkit/02-first-app.md).
**Input:** the repository on Linux, `nix develop`, and a compositor with
`zwlr_layer_shell_v1`. **Output:** a panel that runs.

## Steps

1. Enter the dev shell: `nix develop`. `bk-shell` builds on Linux only
   ([1](01-what-is-the-shell-kit.md#what-it-needs)).
2. Create a binary crate (`cargo new hello-panel`) and add four dependencies:

   ```toml
   [dependencies]
   bk = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<commit on main>" }
   bk-shell = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<commit on main>" }
   ctdesign = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<commit on main>" }
   freya-core = "=0.5.3"
   ```

   `bk-shell` turns on `bk`'s `shell` feature itself. `freya-core` gives
   `AppComponent`; `ctdesign` gives the height token.
3. Replace `src/main.rs` with the program in [Hello, panel](#hello-panel).
4. Open a compositor that offers `zwlr_layer_shell_v1`, nested in your
   session: [10, Run nested](10-run-and-debug.md#run-nested).
5. Run `WAYLAND_DISPLAY=<its socket> cargo run`. A bar runs along the top edge
   of the output, with the text in its center. A maximized window stays below
   it.
6. Inside the repository the same program is an example:
   `cargo run -p bk-shell --example hello_panel`.

## Hello, panel

```rust
use bk::prelude::*;
use bk_shell::{Anchors, KeyboardInteractivity, Layer, LayerSpec, run_layer};
use ctdesign::MENUBAR_HEIGHT;
use freya_core::integration::AppComponent;

fn main() -> Result<(), RunError> {
    let panel = AppComponent::from(|| {
        rect()
            .expanded()
            .center()
            .background(color(Palette::current().window_background))
            .child(Text::new("Hello, panel"))
    });
    run_layer(
        panel,
        LayerSpec {
            namespace: "hello-panel".to_string(),
            layer: Layer::Top,
            anchors: Anchors::TOP,
            exclusive_zone: MENUBAR_HEIGHT,
            keyboard: KeyboardInteractivity::None,
            size: Some((0, MENUBAR_HEIGHT as u32)),
        },
    )
}
```

1. `use bk::prelude::*;` brings the kit's views and Freya's elements, as in an
   app. `bk_shell::Layer` hides Freya's own `Layer`.
2. `AppComponent::from(|| …)` wraps the root view. There is no `Application`:
   the closure is the view, and its hooks go at its top.
3. `rect().expanded().center()` fills the surface and centers the text. A
   layer surface starts transparent, so `.background(..)` paints the panel
   with the palette's `window_background`.
4. `LayerSpec` describes the surface. `"hello-panel"` is the namespace the
   compositor sees. `Layer::Top` puts it above windows. `Anchors::TOP` sticks
   it to the top, left and right edges.
5. `exclusive_zone: MENUBAR_HEIGHT` reserves the panel's height on the top
   edge, so the compositor keeps other windows out of it.
6. `size: Some((0, MENUBAR_HEIGHT as u32))`: a width of 0 lets the compositor
   give the whole edge, which needs both `left` and `right` anchors.
7. `run_layer(panel, spec)` connects, creates the surface and draws it. It
   returns `Ok` when the compositor closes the surface,
   `RunError::MissingGlobal("zwlr_layer_shell_v1")` without the layer shell,
   and `RunError::LayerSize` when a 0 lacks its two anchors.

The program installs no logger. To see the log, add `bk::init_tracing();` as
the first line of `main` ([10, Logs](10-run-and-debug.md#logs)).

## The ideas

1. **The view is written as in an app**
   ([2, The ideas](../barrierkit/02-first-app.md#the-ideas)): same prelude,
   same hooks, same tokens.
2. **The component describes its surface; the compositor places it.** It
   gives a layer, anchors, an exclusive zone and a size, never a position.
3. **The component paints everything that shows.** Nothing draws a
   background, a title bar or a frame for it.
4. **The program lives as long as its surface.** The compositor ends it by
   closing the surface.

Next: [3. Layer surfaces](03-layer-surfaces.md).
