# Compare: measure an artifact against the kit

**Goal:** tell, control by control, how far an HTML artifact is from what
BarrierKit really draws.
**Scope:** from an artifact made with [`artifacts.md`](artifacts.md) and an
image the kit rendered to a report in numbers and a diff image. It measures
pixels; it does not change the artifact or the kit.
**Reader:** an AI agent or a person who iterates on an artifact or an app.
**Input:** an HTML file and a reference PNG. **Output:** a table on stdout and
`artifact.png`, `diff.png` and `report.json` in the output folder.

## Reference images

| Images | Made by | Scene |
|---|---|---|
| `crates/bk/tests/golden/<name>-{light,dark}.png` | `cargo test -p bk --test kit` | One control per image, 360 × 240, centered with 24 px padding, on the test runner's white page |
| `docs/barrierkit/assets/states/*.png` | `doc_shots` ([assets README](../docs/barrierkit/assets/README.md)) | Whole windows |

The scenes are the functions in `crates/bk/tests/kit/controls.rs` and
`crates/bk/src/platform/doc_shots.rs`. An app of your own gets reference
images the same way: render its view with `freya_testing::TestingRunner` and
`render_to_file`.

## Steps

1. Draw the twin of one scene: the same views, texts, order and spacing. A
   twin of a golden image has a white page (`background: white`), like the
   test runner.
2. Run it, outside or inside `nix develop`. It needs Node and Playwright with
   Chromium:

   ```sh
   node scripts/compare.cjs agents/twins/button.html crates/bk/tests/golden/button-light.png
   ```

   The viewport takes the reference's size. A name with `-dark` turns on
   `prefers-color-scheme: dark`. A third argument sets the output folder;
   without it, the files go to `barrierkit-compare` in the temp dir.
3. Read the table:

   | Column | Meaning |
   |---|---|
   | `% differ` | Pixels whose channels differ by more than 3, the tolerance of the golden tests |
   | `Δx`, `Δy`, `Δwidth`, `Δheight` | Artifact minus reference, in px, of the box of non-background pixels |
   | `Δfill` | Largest channel difference of the most frequent color, the control's fill |

   The `(whole)` row covers the page; each other row is one `data-bk`
   element, grown 4 px on each side.
4. Fix the larger differences first: fill, then size, then position. Open
   `diff.png`: differing pixels are red over a pale copy of the reference.
5. Run it again until the table is as close as the limits allow.

## Limits

- Text is never pixel-identical: Skia and Chromium shape and smooth glyphs
  differently. Judge text by `Δwidth` and `Δheight`, not by `% differ`.
- A control's box is measured inside the artifact's box plus 4 px. A reference
  control more than 4 px larger is cut at that edge.
- The box is the bounding box of pixels unlike the corner of the area, so a
  shadow or a focus ring is part of it.
- A difference can come from the kit, not the artifact.
