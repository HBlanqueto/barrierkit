# Comparing the kit with AppKit

**Goal:** measure, control by control, how far a BarrierKit scene is from the
same controls drawn by AppKit, in light, dark and Increase Contrast.
**Scope:** from a probe capture on macOS to a table of numbers and a
side-by-side image. It compares fills, geometry, ink and text positions. It
does not compare glass, blur, shadows or window shapes: the compositor draws
them ([observations](macos-observations.md), section 5).
**Reader:** anyone who changes a control style or a token in `ctdesign` and
wants to check it against macOS.
**Input:** a macOS host with the probes in `~/src/probes`, and a Linux
`nix develop` shell with this repository.
**Output:** `target/appkit/kit*-{light,dark}-{white,black}.png`, a difference
table on stdout and `target/appkit/sheet-*.png`.

## Steps

1. Build the probes once. Each one lives next to its binary:

   ```sh
   cd ~/src/probes
   for probe in controls views alerts menu toolbar tabs sheets gauges; do swiftc -O $probe.swift -o $probe; done
   ```

2. Capture on the Mac. Each run writes `<prefix>.json` (frames) and
   `<prefix>@4x.png` (the control view at 4x, not color managed). Capture over
   white and over black, so the alpha of translucent fills can be solved:

   ```sh
   mkdir -p ~/src/captures/controls
   for look in aqua darkAqua; do for back in white black; do
       ~/src/probes/controls $look $back ~/src/captures/controls/$look-$back
   done; done
   ```

   For Increase Contrast, name the high-contrast appearances and override the
   flags for that process only; no system setting changes:

   ```sh
   ~/src/probes/controls aquaHC white ~/src/captures/controls/hc-aquaHC-white -increaseContrast 1 -reduceTransparency 1
   ```

   `menu` takes `MENU_ITEMS="Title|key|flags;..."` (flags `sub`, `on`, `mixed`,
   `off`) to draw marks and submenu arrows, and `sheets` captures a sheet over
   a black, white or striped parent (`SHEET_BACK=black|white|stripes`).

   `gauges` draws rings, bars, capacity, rating and relevancy indicators at several values and takes the same arguments (`~/src/captures/gauges`).

   `views` (table, source list, tabs, scroll views) takes the same
   arguments and writes to `~/src/captures/views`; `VIEWS_INACTIVE=1` renders
   the window as inactive. `alerts`, `menu`, `toolbar`, `tabs`, `sheets` and `gauges` dump a view
   tree or screenshot and have their own usage line in the source.

3. Convert any `screencapture` image to sRGB before sampling a color. The
   display profile turns blue into indigo:

   ```sh
   sips -m "/System/Library/ColorSync/Profiles/sRGB Profile.icc" in.png --out out.png
   ```

   `cacheDisplay` images (`@4x.png`) are already sRGB but do not draw
   scrollers, glass or SwiftUI-hosted views.

4. Write the scene layout from the probe frames, in the dev shell or
   anywhere with Python, NumPy and Pillow:

   ```sh
   scripts/appkit-compare.py layout ~/src/captures/controls/aqua-white.json target/appkit/layout-controls.tsv
   scripts/appkit-compare.py layout ~/src/captures/views/aqua-white.json target/appkit/layout-views.tsv
   ```

5. Render the kit scene at 4x in the dev shell. The scene is
   `crates/bk/tests/kit/appkit.rs`; each entry has the name of a probe item.

   ```sh
   scripts/appkit-render.sh controls        # kit-{light,dark}-{white,black}.png
   scripts/appkit-render.sh contrast        # kit-hc-*.png
   scripts/appkit-render.sh views           # kit-views-*.png
   scripts/appkit-render.sh views-contrast  # kit-views-hc-*.png
   scripts/appkit-render.sh menu            # kit-menu-{,marks-}{light,dark}.png
   ```

   `APPKIT_BG=1e1e1e` sets the backdrop of the non-black variant, for views
   that paint no background of their own.

6. Compare. The table prints, per control, the share of pixels that differ by
   more than 3, the box and the dominant color of each side:

   ```sh
   scripts/appkit-compare.py controls
   scripts/appkit-compare.py controls --tag hc
   scripts/appkit-compare.py sheet controls light white push-regular,switch-on-regular
   scripts/appkit-compare.py table dark
   ```

   To size an internal glyph, `appkit_glyph_lab` renders a symbol over a grid of sizes and
   strokes (`APPKIT_LAB_STEP` sets the size step) and the glyph's ink box and mass in the
   AppKit capture pick the cell:

   ```sh
   cargo test -p bk --test kit --features bk/shell appkit_glyph_lab -- --ignored
   ```

   Fix the larger differences first: fill, size, position. Then regenerate
   the goldens and run `./dev.sh check`.

## Limits

- Captures live outside the repository, as the other evidence does
  ([observations](macos-observations.md), section 4).
- The kit draws Inter or Adwaita Sans and AppKit draws SF Pro with macOS font
  smoothing, so glyph weight differs by about 40% of the ink and text widths
  differ. Judge text by its position and baseline.
- `APPKIT_CAPTURES` and `APPKIT_OUT` change the capture and output folders.
- The measured values are in [`macos-control-metrics.md`](macos-control-metrics.md).
