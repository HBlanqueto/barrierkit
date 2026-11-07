# Documentation images

**Goal:** know where each image comes from and how to regenerate it.
**Reader:** anyone who updates the documentation after a visual change.

| Folder | Content | Origin |
|---|---|---|
| `componentes/` | A single control or view, in light and dark | Copy of `crates/barrier-kit/tests/golden/` |
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
