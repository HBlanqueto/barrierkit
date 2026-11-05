# Documentation images

**Goal:** know where each image comes from and how to regenerate it.
**Reader:** anyone who updates the documentation after a visual change.

| Folder | Content | Origin |
|---|---|---|
| `componentes/` | A single control or view, in light and dark | Copy of `crates/barrier-kit/tests/golden/` |
| `apps/` | Complete windows of the test apps | Screenshots of the running apps |
| `ejemplos/` | The examples in `crates/barrier-kit/examples/` | Screenshots of the running examples |
| `estados/` | The titlebar and the sidebar in each window state (chapter [7](../07-titlebars.en.md)) | Kit scenes rendered headless by `platform/doc_shots.rs` |
| `marca/` | BarrierKit icon and banner | Logo as SVG and its PNG version, in Spanish and English |

All of them are real renders of the code. None is retouched.

## Regenerating `componentes/`

1. Enter the dev shell: `nix develop`.
2. Regenerate the references: `UPDATE_GOLDEN=1 cargo test -p barrier-kit --test kit`.
3. Copy them: `cp crates/barrier-kit/tests/golden/*.png docs/barrierkit/assets/componentes/`.

## Regenerating `estados/`

1. Enter the dev shell: `nix develop`.
2. Render the scenes:
   `BARRIER_KIT_SHOTS=$PWD/docs/barrierkit/assets/estados cargo test -p barrier-kit --lib doc_shots`.

Each scene sets the window state (focus, maximized, full screen), the
compositor bar, `FullscreenBar` and the titlebar layout. The black band at the
top of the focus-mode scenes is the compositor bar, which Malaysia draws in a
real window.

## Regenerating `apps/` and `ejemplos/`

`apps/` was captured on 2026-11-04 and `ejemplos/` on 2026-11-03, on headless sway, with lavapipe as the Vulkan
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
   - `MALAYSIA_CONFIG_DIR/tengen-app-show.domain` with `layout = "Top"`, `"Rail"`…
3. Launch the app with `VK_DRIVER_FILES` pointing to lavapipe's `lvp_icd` and
   wait a few seconds. `nix develop` sets `VK_DRIVER_FILES` to every Mesa ICD,
   and it wins over `VK_ICD_FILENAMES`: override it with `lvp_icd` alone.
4. Crop the window with `grim -g "x,y ancho×alto"`, taking the geometry from
   `swaymsg -t get_tree`.

On 2026-11-04 Marea stopped requesting frames halfway through its fade-in,
also with the 2026-11-03 code. For its screenshot, alternate the width with
`swaymsg "[pid=…] resize set …"` every 0.4 s for about 6 s: each configure
paints a frame.

Sway does not honor the CSD request in the first configure. For the
screenshots, the binary was built with one local change that is not on `main`,
in `platform/window.rs`: treating the configure as `DecorationMode::Client`.

See [14, "The first frame"](../14-plataforma.en.md#the-first-frame).
