# macOS blur and Liquid Glass per element

**Goal:** classify every macOS shell surface by its material and give measured
values for its blur, tint, border, shadow and lensing, so a compositor can tell
Liquid Glass from a general blur.
**Scope:** the menu bar, the application menu panel, the Dock, Control Center,
Notification Center, Spotlight, notification banners, Mission Control,
Launchpad and one window's chrome. It ends at the classification table.
**Reader:** anyone who changes `ctappearance` glass, the compositor blur path
or a Malaysia shell surface.
**Input:** the probes in `~/src/probes`, the four controlled backdrops plus a
grid backdrop, the Reduce Transparency switch and light and dark mode.
**Output:** this matrix, the Liquid Glass verdict and the corrections to the
earlier research.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the material model in
   [`macos-materials.md`](macos-materials.md) and the glass system in
   [`macos-liquid-glass.md`](macos-liquid-glass.md).
3. Use a value only with its backdrop; never lift one colour alone.
4. Treat an element as Liquid Glass only with lensing, specular or adaptive
   ink evidence; a blur plus a tint is general translucency.
5. Check the corrections section before trusting a value from an earlier
   research file.

---

## 1. Method

All values come from pixel samples of 1920x1080, scale 1.0 captures. The
backdrops were white, gray, black, checker (8 px cells) and `grid16` (16 px
cells with 2 px black lines). The `grid16` backdrop is the lens test: a
refracting surface shifts its lines; a blur only softens them.

The appearance was backed up to `~/src/captures/defaults-2026-11-19-blur`,
reset for the default Dock, measured, and restored. Reduce Transparency and
light mode were measured live; the Dock and menu hosts were restarted after
each switch. There is no 2x capture on this host, so one-pixel rims are not
comparable at Retina density. The lock screen was not captured because
locking the only session stops the host. This host has no desktop widgets.

This pass worked without image inspection: every number below is a pixel
sample or a window-server frame, not a visual estimate.

## 2. The material classes observed

| Class | Surfaces |
|---|---|
| Clear | menu bar, Spaces bar, Mission Control background |
| Adaptive glass | application menu panel, Control Center, Spotlight field, notification banner, Notification Center |
| Wallpaper-biased glass | Dock |
| Wallpaper blur | Launchpad background |
| Translucent chrome | window toolbar and sidebar |
| Opaque | window content |

## 3. The measured matrix

| Element | Class | White | Gray | Black | Checker | Border/rim | Shadow | Lens | Reduce Transparency | Ink |
|---|---|---|---|---|---|---|---|---|---|---|
| Menu bar | Clear | `#ffffff` | sharp | `#000000` | sharp 8 px cells | none | none | none | unchanged | scheme ink |
| Application menu | Adaptive glass | `#8d8d8d` | `#8d8d8d` | `#2b2b2b` | `#85` to `#88` with faint dither | 1 px light edge (`#af` over grid), dark at corners | about 30 px, 4% at 28 px on white | none; grid lines resume unshifted | unchanged: `#8d` over white, `#888` over checker | follows the panel: white on `#2b` even in light mode |
| Dock | Wallpaper-biased glass | `#bebebe` | `#8a8a8a` | `#0d0d0d` | `#7f7f7f` | 3-step bevel plus a 13 px bottom ramp ending `#a0` | none visible; flush with the screen edge | none | solid `#020202` to `#0a0a0a` | n/a |
| Control Center | Adaptive glass | `#8c8c8c` | not measured | not measured | `#85` to `#88` | 1 px edge, bottom light edge `#dd` to `#f5` | about 30 px on white | none; grid lines resume unshifted | unchanged: `#8c` over white, `#85` over checker | not measured |
| Notification Center | Adaptive glass, darker tone curve | `#e0e0e0` | not measured | not measured | `#1e1e1e` | 1 px dark edge zone | not measured | none observed | not measured | not measured |
| Spotlight | Adaptive glass | not measured | not measured | not measured | `#85` to `#8a` | 1 px dark outline `#0b` plus light edge | present, soft | none observed | not measured | not measured |
| Notification banner | Adaptive glass | not measured | not measured | not measured | `#85` to `#8a` | 1 px dark top edge | present, soft | none observed | not measured | not measured |
| Mission Control | No material | pixel-identical to the desktop | same | same | same | none | none | none | n/a | n/a |
| Spaces bar | Clear with glass thumbnails | thumbnails `#727272` over checker | not measured | not measured | bar itself stays sharp | none | none | none | not measured | not measured |
| Launchpad | Wallpaper blur | n/a | n/a | n/a | n/a | none | none | n/a | not measured | n/a |
| Window toolbar | Translucent dark glass | `#28` to `#36` over checker | not measured | not measured | checker fully flattened | none inside | window shadow | n/a | not measured | not measured |
| Window sidebar | Translucent dark glass | `#35` to `#3f` over checker | not measured | not measured | checker fully flattened | none inside | window shadow | n/a | not measured | not measured |
| Window content | Opaque | `#282828` | n/a | n/a | n/a | 1 px dark outline `#08` plus 1 px light bevel `#58` | about 10 to 15 px | n/a | n/a | n/a |

The menu bar over the checker stays a sharp 8 px pattern in dark and light
mode. It is clear, not blurred. The Mission Control background over the
wallpaper is pixel-identical to the desktop on this host: macOS 27.2 does not
dim or blur it, only the Spaces bar and the window overlays render.

## 4. Blur strength per element

The checker is 8 px cells. A surface that shows a flat colour has destroyed
that frequency; a surface that shows a faint dither has not.

- **Menu panel:** flat `#85` to `#88` with a residual dither of about 3/255
  over several pixels. The blur is strong but leaves a weak low-frequency
  remnant.
- **Control Center:** flat `#85` to `#88`, same family as the menu. The two
  share the composite.
- **Spotlight and banner:** flat `#85` to `#8a`, the same family.
- **Notification Center:** flattens the checker to `#1e1e1e`, a darker and
  flatter result than the menu. It is a different tone curve, not the same
  material with another tint.
- **Dock:** flattens the checker to `#7f7f7f`; over the grid it is flat
  `#78` to `#7a`. Strong, like the menu.
- **Window toolbar and sidebar:** checker fully flattened; the glass is
  darker than the Dock at the same backdrop.
- **Launchpad:** the wallpaper detail is gone and the luminance lifts about
  36/255 over the same region without Launchpad.

The panel interiors give no period from the grid16 lines, so a blur radius in
points cannot be read directly. The honest equivalent is "stronger than a
16 px blur of a 2 px line".

## 5. Borders and shadows

- The **menu panel** edge is 1 px: a light line over the grid and a dark line
  at the corners. Its shadow reaches about 30 px with a 4% depth at 28 px on
  a white cell.
- The **Dock** rim is not a line. Over the grid the bottom edge reads as a
  ramp from `#6c6c6c` up to `#a0a0a0` across 13 px, on top of the known
  3-step bevel. It has no visible shadow because it sits on the screen edge.
- The **Control Center** bottom edge is a 2 px light line (`#dd`, `#f5`) with
  a shadow that fades over about 30 px on white.
- A **window** has a 1 px dark outline plus a 1 px light bevel on the frame
  and a 10 to 15 px shadow.
- The **menu corner radius** re-measured from the grid capture stays about 6.

## 6. The edge zones: where the rim ends and the blur begins

The user-facing question is where the "reverse" edge effect starts: how far
from the border the material stops being an edge treatment and becomes a
plain blur. The measured answer is two parts.

### 6.1 The rim, the only edge-differentiated zone

At 1x the surfaces show an edge treatment only as a rim:

| Surface | Zone | Width |
|---|---|---|
| Dock | top edge | 1 px light line |
| Dock | bottom ramp | up to 13 px, only visible over a bright backdrop |
| Dock | left/right ends | 1 px light line |
| Menu panel | edge | 1 px light line, dark at the corners |
| Control Center | bottom edge | 2 px light line |
| Banner | top edge | 1 px dark line |
| Window frame | outline | 1 px dark plus 1 px light |

Everything inside the rim is the same blur. The panels do not show a wider
intermediate zone between the rim and the center.

### 6.2 The boundary test for refraction

A white/black boundary was placed at a known position and measured again
inside each panel. A refracting edge would move the boundary; a pure blur
only softens it.

| Surface | Boundary | Sample point | Midpoint | Displacement |
|---|---|---|---|---|
| Dock | horizontal at y=1030 | gaps at x=250, 316, 1200, 1500 | y=1030 | 0 px |
| Dock | horizontal at y=1030 | x=1730, 17 px from the right end | y=1030 | 0 px |
| Dock | vertical at x=250 | y=1000, 9 px below the top rim | x=250 | 0 px |
| Dock | vertical at x=250 | y=1060, 14 px above the bottom ramp | x=250 | 0 px |
| Notification banner | horizontal at y=80 | x=1700 | y=80 to 81 | within 1 px |
| Notification Center | horizontal at y=120 | x=1700 | y=120 | within 1 px |

The transition ramps are symmetric around the boundary in every sample. No
surface moved the boundary at 1x, neither in the center nor next to its
borders. The blur is the same material from the rim inward; the only
edge-specific rendering is the rim itself.

## 7. Liquid Glass or general blur

The grid and boundary tests are the verdict. No surface shifted the grid
lines or a white/black boundary near its edges: the lines stay in place
outside and vanish inside, and every boundary midpoint lands within 1 px of
its true position. There is no measurable lensing or refraction on the menu,
the Dock, Control Center, Notification Center or the banner at 1x.

What the surfaces do have is the rest of the Liquid Glass description:
a specular 1 px edge, a strong blur and adaptive ink, where the ink was
measured. The honest classification on this host is therefore **adaptive
blur and tint materials with specular edges, without proven lensing**. Do
not call any of them lensing glass in the code or the docs.

## 8. Corrections to the earlier research

These measurements replace parts of the light-mode and Reduce Transparency
tables in [`macos-defaults.md`](macos-defaults.md):

- **The Dock follows the wallpaper, not the appearance scheme.** With this
  host's dark wallpaper, light mode renders the same dark glass as dark mode:
  `#bebebe` over white and `#0d0d0d` over black. The light matrix in
  `macos-defaults.md` (`#eaeaea`, `#a5a5a5`, `#323232`) was recorded under a
  different wallpaper state. Verify the light Dock tokens with a light
  wallpaper before trusting them.
- **Reduce Transparency leaves the menu and Control Center unchanged.**
  The menu stays `#8d` over white and `#888` over checker; Control Center
  stays `#8c` over white. The earlier `#fefefe` light menu value was not
  reproducible. Only the Dock goes solid: `#020202` to `#0a0a0a`.
- **The menu panel is scheme-independent.** Over black it is `#2b2b2b` in
  light mode with white ink, and over white it is `#8d8d8d` in dark mode with
  dark ink. The earlier light-mode `#9a9a9a` over black was not reproducible.
- **Mission Control has no background material on this host.** The classic
  wallpaper blur is absent; the Spaces bar is clear with glass thumbnails.

## 9. Open questions

- Is the compact 340x265 Control Center panel the default or a host state?
- What are the rim and blur values at 2x? A sub-pixel refraction zone could
  exist at 1x without being measurable; Retina captures are needed.
- What Dock glass does a light wallpaper produce in light mode?
- Does Notification Center change under Reduce Transparency?
- Does the Launchpad lift depend on the wallpaper luminance?

## 10. Evidence

The captures are local and not committed:

- `~/src/captures/blur-base-c.png`, `blur-menu-checker.png`,
  `blur-apple-c.png`: menu and Dock over checker.
- `~/src/captures/blur-base-grid.png`, `blur-menu-grid.png`,
  `blur-dock-grid.png`, `blur-cc-grid2.png`: the grid lens tests.
- `~/src/captures/blur-dock-eh1030b.png`, `blur-dock-ev250.png`,
  `blur-banner-eh80.png`, `blur-nc-eh120b.png`: the boundary refraction
  tests.
- `~/src/captures/blur-cc-checker.png`, `blur-dark-cc-white2.png`,
  `blur-rt-cc-white2.png`, `blur-light-cc2.png`: Control Center matrix.
- `~/src/captures/blur-cc-1831.png`, `blur-cc-white.png`: Notification
  Center over checker and white.
- `~/src/captures/blur-spotlight-checker.png`, `blur-banner-checker.png`.
- `~/src/captures/blur-mc-wall.png`, `blur-mc-checker.png`: Mission Control.
- `~/src/captures/blur-lp3.png`, `blur-lp-base.png`: Launchpad.
- `~/src/captures/blur-finder-checker.png`: window chrome.
- `~/src/captures/blur-rt-*.png`, `blur-light*.png`: Reduce Transparency
  and light-mode passes.
- `~/src/captures/defaults-2026-11-19-blur/`: the backup before this pass.
