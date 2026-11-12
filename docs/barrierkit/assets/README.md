# Documentation images

**Goal:** know where each image of the BarrierKit chapters comes from, and
regenerate it after a visual change.
**Scope:** the two folders under `docs/barrierkit/assets/` and the golden
images in `crates/bk/tests/golden/`. Which chapter
shows each image is in that chapter.
**Reader:** whoever updates the documentation after changing how a control,
the titlebar or the sidebar looks.
**Input:** the dev shell (`nix develop`). **Output:** images that are real
renders of the current code, never retouched.

## Folders

| Folder | Holds | Produced by |
|---|---|---|
| `crates/bk/tests/golden/` | One control or view per image, `<name>-light.png` and `<name>-dark.png`; the chapters link them directly | The golden tests in `crates/bk/tests/kit/` |
| `states/` | The titlebar and the sidebar in each window state ([9. Titlebars](../09-titlebars.md)) | The `doc_shots` test in `crates/bk/src/platform/doc_shots.rs` |
| `brand/` | The BarrierKit icon and banner, as SVG | Drawn by hand; the repository has no script for them |

Both tests render with `freya-testing` on the CPU at scale 1, without a
compositor or a GPU. They set the appearance in code, so the host's
`appearance.domain` and wallpaper do not change the result.

## Language and fonts

The images follow the language of the process and its fonts, so every step
below runs with the same environment:

1. The dev shell: its `FONTCONFIG_FILE` gives Geist and Inter. Outside it the
   text falls back to another font and the images differ.
2. Spanish, the language of the test texts. `.cargo/config.toml` points
   `MALAYSIA_CONFIG_DIR` at `crates/bk/tests/config`, whose `language.domain`
   says `es`, so the host's language and settings do not change the images.

## Steps to regenerate the golden images

1. Enter the dev shell: `nix develop`.
2. Rewrite the golden images:

   ```sh
   UPDATE_GOLDEN=1 cargo test -p bk --test kit
   ```

3. Check the new images with `git diff --stat crates/bk/tests/golden/`
   and look at each changed one.

Which tests write golden images:

| Test (`tests/kit/`) | Images | Size |
|---|---|---|
| `controls.rs`, `controls_match_their_golden_images` | Controls (`button`, `switch`, `slider`, `form`…) in light and dark | 360 × 240 |
| The same test, window cases | `preferences`, `about`, `form-long-text`, `backdrop` in light and dark | 900 × 560 |
| `controls.rs`, `keyboard_focus_ring_matches_its_golden_image` | `focus-ring-light`, after a Tab press (light only) | The test case's size |
| `menu.rs` | `menu-view-*` and `menu-view-accents-*` in light and dark | The menu's preferred size |

Without `UPDATE_GOLDEN`, the tests compare. A channel that differs by more
than 3 fails the test, and the actual image is written to the temp dir
(`malaysia-control-<name>-<theme>.png`).
A golden image that does not exist fails the test.

## Steps to regenerate `states/`

1. Enter the dev shell: `nix develop`.
2. Render the scenes into the folder:

   ```sh
   BARRIER_KIT_SHOTS=$PWD/docs/barrierkit/assets/states cargo test -p bk --lib doc_shots
   ```

Without `BARRIER_KIT_SHOTS`, the test returns without rendering. Each scene
sets:

- the window status (active, inactive, maximized, full screen);
- the chrome style (`Top` or `Rail` layout, unified rail);
- focus mode: overlay titlebar and the bar down;
- the `FullscreenBar` and `fullscreen_push`;
- the sidebar style (island or flat).

The black band at the top of the focus-mode scenes stands for the compositor
bar, which Malaysia draws over a real window.
