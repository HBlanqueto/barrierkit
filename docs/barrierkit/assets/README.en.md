# Documentation images

**Goal:** know where each image comes from and how to regenerate it.
**Reader:** anyone who updates the documentation after a visual change.

| Folder | Content | Origin |
|---|---|---|
| `componentes/` | A single control or view, in light and dark | Copy of `crates/barrier-kit/tests/golden/` |
| `apps/` | Complete windows of the test apps | Screenshots of the running apps |
| `ejemplos/` | The examples in `crates/barrier-kit/examples/` | Screenshots of the running examples |

All of them are real renders of the code. None is retouched.

## Regenerating `componentes/`

1. Enter the dev shell: `nix develop`.
2. Regenerate the references: `UPDATE_GOLDEN=1 cargo test -p barrier-kit --test kit`.
3. Copy them: `cp crates/barrier-kit/tests/golden/*.png docs/barrierkit/assets/componentes/`.

## Regenerating `apps/` and `ejemplos/`

They were captured on 2026-11-03 on headless sway, with lavapipe as the Vulkan
driver and the Geist font installed:

1. Start headless sway with floating windows and CSD:

   ```sh
   WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 sway -c sway.cfg
   ```

   ```
   output HEADLESS-1 mode 1400x900 bg #3a4256 solid_color
   default_border none
   for_window [app_id=".*"] floating enable, border csd
   ```

2. Prepare the appearance and the pane in dedicated folders:
   - `MALAYSIA_CONFIG_DIR/appearance.domain` with `color_scheme` (and
     `sidebar_style` for the flat sidebar);
   - `MALAYSIA_DATA_DIR/io.github.hblanqueto.malaysia.<app>.state` with
     `pane = "<id>"` to open a pane;
   - `MALAYSIA_CONFIG_DIR/rotulo.domain` with `layout = "Top"`, `"Rail"`…
3. Launch the app with `VK_ICD_FILENAMES` pointing to lavapipe and wait a few
   seconds.
4. Crop the window with `grim -g "x,y ancho×alto"`, taking the geometry from
   `swaymsg -t get_tree`.

Sway does not send frame callbacks to a surface without content and does not
honor the CSD request in the first configure. For the screenshots, the binary
was built with two local changes that are not on `main`, in
`platform/window.rs`:

- drawing the first frame when the configure arrives;
- treating the configure as `DecorationMode::Client`.

See [13, "Known limitation"](../13-plataforma.en.md#known-limitation-the-first-frame).
