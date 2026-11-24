# macOS materials, glass and vibrancy

**Goal:** turn the controlled-backdrop samples into a material model that
`ctappearance` can reproduce.
**Scope:** blur, tint, fill hierarchy and vibrancy on the menu panel, the Dock
and the notification card (sections 1 to 9), and the 14 materials of
`NSVisualEffectView` in windows (section 10). It excludes the dynamic grouping
behaviour, which has its own file.
**Reader:** anyone who changes `ctappearance::resolve`, `glass_soften`,
`dock_surface` or `menu_glass`.
**Input:** four controlled backdrops and the semantic colour dump.
**Output:** the measured material model, a Malaysia comparison and ordered
gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the complete default matrix in
   [`macos-defaults.md`](macos-defaults.md).
3. Pick a material by role, never by its apparent colour.
4. Check the fill hierarchy before adding a new token.
5. Read the per-element blur and lensing matrix in
   [`macos-blur.md`](macos-blur.md).
6. Do not copy Apple material images or SF Symbols.
7. To implement the blur of a window material in the compositor, read only
   [section 10.10](#1010-for-the-compositor): it holds the order per pixel, the
   units, the region, the Kawase verdict and the parity check in one place.

---

## 1. What a material is

A macOS material is not a flat colour. It combines:

- A **blur** of the content behind it.
- A **tint** that lifts or lowers the blurred result.
- A **fill hierarchy** for text and glyphs.
- A **rim** or edge treatment.
- Accessibility overrides that change the material itself.

The white, gray, black and checker backdrops separate these parts. The checker
shows the blur. The black and white results show the tone curve. The label
roles show the fill hierarchy. The rim is visible as a light/dark bevel.

## 2. Measured composites

The menu and Dock tables live in their area files, and the full matrix is in
[`macos-blur.md`](macos-blur.md). The shared result, measured at 2560x1440
under one wallpaper:

- The menu is **adaptive and scheme-dependent**: dark mode is a black veil
  capped bright (`#a3` over white, `#2b` over black); light mode is a white
  veil capped near white (`#f2` over white, `#75` over black). The ink follows
  the panel in both modes.
- The Dock is **scheme-biased**: dark glass in dark mode (`#be` over white,
  `#0d` over black), light glass in light mode (`#eb` over white, `#34` over
  black), still following the backdrop.
- Both blur strongly; the checker becomes a low-contrast texture or a single
  gray, depending on the surface.
- Neither is a single opacity. The bright and dark ends do not lie on one
  straight line.

The per-surface veil constants that reproduce these composites are in
[`macos-blur.md`](macos-blur.md#5-the-model-fit-one-veil-per-surface-per-mode).
The 2026-11-19 light values (`#eaeaea` Dock over white) belong to a darker
wallpaper state.

## 3. Measured fill hierarchy

Light appearance:

| Role | Value |
|---|---|
| `label` | `#000000` at 84.7% |
| `secondaryLabel` | `#000000` at 49.8% |
| `tertiaryLabel` | `#000000` at 25.9% |
| `quaternaryLabel` | `#000000` at 9.8% |
| `separator` | `#000000` at 10% |
| `systemFill` | `#000000` at 9.8% |
| `secondarySystemFill` | `#000000` at 7.8% |
| `tertiarySystemFill` | `#000000` at 4.7% |

Dark appearance:

| Role | Value |
|---|---|
| `label` | `#ffffff` at 84.7% |
| `secondaryLabel` | `#ffffff` at 54.9% |
| `tertiaryLabel` | `#ffffff` at 24.7% |
| `quaternaryLabel` | `#ffffff` at 9.8% |
| `separator` | `#ffffff` at 12% |
| `systemFill` | `#ffffff` at 9.8% |
| `secondarySystemFill` | `#ffffff` at 7.8% |
| `tertiarySystemFill` | `#ffffff` at 4.7% |

Vibrancy uses this hierarchy on top of the material. The menu text is not a
fixed white: it is a label role that resolves against the current material
and appearance.

## 4. Accessibility overrides

The public API exposes four settings that change the material:

- **Reduce Transparency** removes translucency and substitutes opaque or
  frosted surfaces.
- **Increase Contrast** strengthens text and separator contrast.
- **Differentiate Without Color** adds shapes or glyphs.
- **Reduce Motion** changes transitions, not the material.

The measured matrix includes Reduce Transparency. The Dock becomes a **scheme
solid**: `#03` to `#0d` in dark mode and `#f1` to `#89` in light mode, both
over white and black backdrops. The menu and Control Center stay unchanged.
A single solid token per scheme is therefore accurate for the Dock; the
backdrop response that remains is a few units, not a material.

## 5. Comparison with Malaysia

| Element | macOS default | Malaysia before | After |
|---|---|---|---|
| Light Dock glass | `#ebebeb` over white | `#ffffff66` | `#b5b5b548` |
| Light Dock opaque | `#f1f1f1` over white | `#757575` | `#f1f1f1` |
| Dark Dock glass | `#bebebe` over white, `#0d` over black | `#2727274e` | unchanged |
| Menu glass | Two veil sets by scheme | fixed scheme glass | dark veil set only |
| Menu ink | Follows panel | follows scheme | follows the panel |
| Menu blur | Strong | `menu_glass` soften | verify |
| Dock rim | 3-step light/dark bevel | `rim` and `tone` | model exists |
| Label hierarchy | 84.7 / 54.9 / 24.7 / 9.8 | `ink`, `ink-muted`, `ink-faint` | three levels; no bk consumer fakes a fourth, so none is added |
| Separator | `#000000` at 10% light, `#ffffff` at 12% dark | `line` at 4.7% light, 7.1% dark | `separator` token, measured |
| Fill hierarchy | 9.8 / 7.8 / 4.7 / 2.7 | `bg-control`, `line` factors | different scheme |
| Reduce Transparency | Dock scheme solid; menu and CC unchanged | solid fallback | `over_glass` keeps the menu clear; solid only with Increase Contrast |

## 6. What not to copy

- Apple material images or pre-rendered blur textures.
- SF Pro and SF Symbols.
- Apple's semantic names in public API. Use the glossary names.
- A photograph as the only backdrop for a glass test.

## 7. Prioritized changes

1. **Add the light menu veil set.** The dark tokens reproduce the dark
   matrix; a light scheme needs its own veil and cap (`#f2` over white,
   `#d1` over gray, `#75` over black). Targets in
   [`macos-blur.md`](macos-blur.md#5-the-model-fit-one-veil-per-surface-per-mode).
2. **Keep the corrected light Dock tokens.** `#b5b5b548` matches the measured
   light glass over white; verify it against `#ebebeb` and `#34` before
   changing it again.
3. **Keep the corrected light opaque token.** `#f1f1f1` is the measured light
   Reduce Transparency result over white; add the dark `#03` counterpart.
4. **Add four-backdrop tests for every glass token per scheme.** White, gray,
   black and checker, dark and light. A single wallpaper test cannot catch
   the adaptive menu.
5. **Add a quaternary label level.** macOS exposes four label levels; the
   current three-token set cannot express disabled or quaternary text.
6. **Verify the blur pass count and spread.** Use a checkerboard with a known
   square size and compare the edge energy.
7. **Test Reduce Transparency per scheme.** The fallback must replace the
   material with the scheme solid, not merely raise its opacity.

## 8. Open questions

- How many blur passes does each material use? The measured matrix is in
  [`macos-blur.md`](macos-blur.md); the pass count is still unknown. Answered for the
  materials of windows on 2026-11-22: one Gaussian of sigma 27.6 px, no
  passes ([section 10.2](#102-the-pipeline)).
- What is the rim width at 2x?
- Does the menu glass change with the wallpaper tint or only the window?
  [`macos-blur.md`](macos-blur.md) shows the menu is backdrop-driven and
  scheme-dependent; the wallpaper interaction is open.
- Do the notification and menu glass share the same composite? They do not:
  the menu and Control Center share one tone curve and Notification Center
  carries a lighter veil in both modes.
- Does light mode track a dark wallpaper and revert to the dark matrix?

## 9. Implementation prompt

```text
You are fixing the material model from measured values. Read this file and
macos-blur.md first. Work in ~/src/barrierkit and ~/src/malaysia, main
only: git pull --ff-only before editing and before each push, one commit
per session, English messages, no code comments.

Measured targets, dark / light:
- Menu: white #a3a3a3 / #f2f2f2, gray #92-94 / #d1d1d1, black #2b2b2b /
  #75-7b, checker #8b-8d / #bf-c6.
- Dock: white #bebebe / #ebebeb, gray #8b / #a6a6a6, black #0d-10 / #34,
  checker #6c / #91.
- RT: Dock scheme solid #03-0d / #f1-89; menu and CC unchanged.
- Label hierarchy: 84.7 / 54.9 / 24.7 / 9.8 per appearance.

Tasks:
1. barrierkit: add the light menu veil set; align the Dock light values
   with the targets; make the RT Dock resolve to the scheme solid.
2. barrierkit: four-backdrop tests per surface per scheme in
   ctappearance::resolve, asserting the targets within 8/255; keep the
   label hierarchy as the source for glass ink.
3. malaysia: after the rev bump, confirm the blur path and the RT
   early-out still hold.

Definition of done: ./dev.sh check passes in both repos; tests assert the
measured values; barrierkit main is pushed and malaysia pins the new rev.
Report changed tokens, test values and commits.
```

---

## 10. Materials in windows, measured 2026-11-22

**Goal:** give the compositor the numbers and the order of operations that make
a material of `NSVisualEffectView` look the same as on macOS 27.2.1, and a way
to check it.
**Scope:** the 14 materials of `NSVisualEffectView.Material` that the SDK does
not deprecate, in both blending modes, light and dark, active and inactive,
emphasized or not, with Reduce Transparency and Increase Contrast, at scale 1.
The SwiftUI materials, the glass tiles and the first real controls are
sections 11 to 13. It leaves out scroll edge effects and scale 2: section 10.8
says why.
**Reader:** whoever changes `ctappearance::material`, the `visual-effect`
tokens or the compositor's material pass.
**Input:** the probe `~/src/probes/materials.swift`, run by `materials.sh` and
`matbatch.sh`; the captures in `~/src/captures/materials` (raw PNG and JSON).
**Output:** the observed pipeline, one row per material and scheme, the fit
error against the captures, the states and a parity check.

### 10.1 Method

1. **The 14 materials.** `NSVisualEffectView.h` of the macOS 27.2 SDK
   (`/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/Frameworks/AppKit.framework/Versions/C/Headers/NSVisualEffectView.h`)
   lists `titlebar`, `selection`, `menu`, `popover`, `sidebar`, `headerView`,
   `sheet`, `windowBackground`, `hudWindow`, `fullScreenUI`, `toolTip`,
   `contentBackground`, `underWindowBackground` and `underPageBackground`. The
   five it deprecates (`appearanceBased`, `light`, `dark`, `mediumLight`,
   `ultraDark`) are not measured.
2. **Backdrops.** One borderless, non-opaque window holds one tile per
   material, state and emphasis. `withinWindow`: the window draws the backdrop
   itself. `behindWindow`: a second, opaque, full-screen window of the same
   process lies below it. The backdrops are a gray ramp from 0 to 255 in 256
   bands of 4 px, the red, green, blue, cyan, magenta and yellow ramps, and a
   white|black step at the middle of a 200 px tile (and, for the region edge,
   at the border of the tile). The ramp is smooth for a Gaussian of 27.6 px,
   so the tile shows the tone curve of the material at every level, but the tile
   is the region: the blur renormalises at both ends of the ramp and, within
   about 80 px (20 levels) of an end, the tile shows the shade of the blurred end
   and not of the level.
3. **Pixels.** `screencapture` of the whole screen, read as raw PNG bytes with
   PIL. The bytes are the display's device RGB (section 10.2); they are not
   converted to sRGB. The backdrop is read from the uncovered margin of the
   same capture. 2560x1440, scale 1.0.
4. **Observed, then measured.** The probe dumps the layer tree of each tile
   (public `CALayer` properties of its own process: class, filters, background
   colour, compositing filter, opacity). That is a runtime observation, not API,
   and may change. Every number below comes from that tree **and** is checked
   against the pixels: the tree gives the stack and its values, the pixels give
   the blur sigma, the order of operations, the space, the region edge and the
   fit error.
5. **Hygiene.** Each group of captures runs alone on the screen. A capture is refused if another window of layer 4 or more lies
   over the screen (a system dialog), and retried.
6. **Reproduce.** `~/src/probes/matbatch.sh core chroma a11y swiftui follows`
   captures; `matreport.py`, `matsigma.py`, `matdocs.py` and `mattokens.py`
   (Python of the venv of [`macos-observations.md`](macos-observations.md), run
   with `-I`) fit and print the tables.

### 10.2 The pipeline

Each blurred material is this stack of layers, bottom to top:

```text
backdrop of the region
  sdrNormalize
  gaussianBlur       inputRadius 30, inputNormalizeEdges 1
  colorSaturate      inputAmount s        (absent in titlebar and headerView)
veil layer           gray g, alpha a, normal blend
darken layer         gray c: per-channel minimum (light schemes)
  or lighten layer   gray f: per-channel maximum (dark schemes)
chameleon layer      opacity 0.05 (0.08 in the dark header view)
```

The shade of one backdrop pixel `b` (0 to 1 per channel) is, in the order the
stack has:

1. `y = 0.2126 r + 0.7152 g + 0.0722 b` and `b' = clamp(y + s (b - y), 0, 1)`.
2. `v = b' + (g - b') a`.
3. `shown = max(min(v, c), f)`, where the light schemes have no floor and the
   dark ones no cap.

What the pixels say about the parts the tree does not give:

| Question | Answer | Evidence |
|---|---|---|
| Kernel | Gaussian, sigma 27.6 px for `inputRadius` 30 (0.92 of it) | 36 fits of the white\|black step through the exact stack: sigma 27.5 to 27.8, mean 27.6, std 0.1 |
| Gaussian or triple box | Gaussian | rms of the step 0.27 to 0.53 against 0.30 to 0.56 for a triple box of the same variance; better in 9 of 9 materials |
| Space of the blur | Encoded, not linear light | the step fits at rms 0.2 to 1.2 in encoded values and at 1.8 to 12 in linear light |
| Region edge | The region sees only itself; weights are renormalised at its border (`inputNormalizeEdges`) | with the step at the border of the tile (black outside, white inside), the 40 px inside the border differ 0.0 levels from the plateau, in 36 tiles |
| Edge falloff | None: the material starts at the first pixel of the region | one pixel goes from the backdrop to the material, with no feather |
| Noise | None in `NSVisualEffectView`: standard deviation across a row 0.04 on average, 0.19 at most | every blurred tile of every ramp capture |
| Saturation | Rec. 709 luma weights, clamped to 0..255 before the veil | the variants below |
| Where the cap acts | Per channel, after the veil | the variants below |
| Resolution of the backdrop | one eighth: `CABackdropLayer.scale` 0.125 (SwiftUI 0.25, glass 0.5), read by key in a probe layer and confirmed by a layer built by hand | section 14.3 |
| Checker of 8 px | contrast 0.0 after the blur: a Gaussian of 27.6 px leaves about 1e-25 of a pattern of 16 px | the standard deviation of 0.0 of every blurred tile over an 8 px checker in [`blur-ownership.md`](blur-ownership.md) 1.2 (captured earlier, not repeated here) |
| Space of the mix | The display's device RGB (wide gamut here), not sRGB | the titlebar over pure red shows (255, 213, 207) in device bytes; mixing the same veil in sRGB would give (253, 203, 203) |

The variants of the stack against the gray, red, green and blue ramps of the
light `menu`, with the ramp blurred inside its region (rms / max, levels of 255):

| Variant | gray | red | green | blue |
|---|---|---|---|---|
| Rec. 709, clamp, cap per channel | 0.33 / 1.1 | 0.25 / 0.9 | 0.26 / 1.0 | 0.26 / 1.2 |
| no clamp before the veil | 0.33 / 1.1 | 4.07 / 10.2 | 6.43 / 16.6 | 0.51 / 1.3 |
| no cap | 2.61 / 10.1 | 3.42 / 11.5 | 2.51 / 11.5 | 4.61 / 11.5 |
| Rec. 601 luma | 0.33 / 1.1 | 1.15 / 3.4 | 1.96 / 4.9 | 1.24 / 3.6 |
| luma 0.3086, 0.6094, 0.0820 | 0.33 / 1.1 | 1.45 / 4.1 | 1.43 / 3.7 | 0.56 / 1.7 |
| luma of the display's primaries | 0.33 / 1.1 | 1.15 / 3.4 | 0.41 / 1.5 | 0.63 / 2.0 |

The mix happens in the display's device space because the panel's profile is
wide gamut (`Display`, red at x 0.7085): saturated sRGB colours read in it as,
for example, (208, 74, 35) for red. A conversion of the capture to sRGB turns
the mix into apparent chroma leaks of 10 to 20 levels. Gray is the same in both
spaces. A compositor draws the numbers of this section in its own output space:
on an sRGB output they act on sRGB values.

### 10.3 The materials, light and dark

Levels are of 255 and the gray of the veil is the same in the three channels.
`fit` is the worst rms and max over the gray, red, green, blue, cyan, magenta
and yellow ramps, in both blending modes. A `sheet`, `windowBackground`,
`contentBackground` or `underPageBackground` does not blur.

| Material | Scheme | Observed stack, levels of 255 | Chameleon | Fit rms / max |
|---|---|---|---|---|
| titlebar | light | blur r30, veil 253 @ 0.80 | 5% | 0.32 / 0.9 |
| titlebar | dark | blur r30, veil 60 @ 0.80 | 5% | 0.30 / 0.7 |
| selection | light | veil 0 @ 1.00 | - | see section 10.5 |
| selection | dark | veil 0 @ 1.00 | - | see section 10.5 |
| menu | light | blur r30, sat 2.1, veil 246 @ 0.72, cap 237.2 | 5% | 0.35 / 1.2 |
| menu | dark | blur r30, sat 2.2, veil 40 @ 0.70, floor 31.9 | 5% | 0.41 / 1.2 |
| popover | light | blur r30, sat 2, veil 246 @ 0.60, cap 241.0 | 5% | 0.43 / 1.7 |
| popover | dark | blur r30, sat 2, veil 40 @ 0.60, floor 28.1 | 5% | 0.40 / 1.4 |
| sidebar | light | blur r30, sat 2.2, veil 246 @ 0.84, cap 233.3 | 5% | 0.33 / 0.8 |
| sidebar | dark | blur r30, sat 2.4, veil 40 @ 0.80, floor 35.7 | 5% | 0.31 / 1.0 |
| headerView | light | blur r30, veil 255 @ 0.80 | - | 0.29 / 0.7 |
| headerView | dark | blur r30, veil 30 @ 0.80 | 8% | 0.28 / 0.7 |
| sheet | light | veil 255 @ 1.00 | - | 0.00 / 0.0 |
| sheet | dark | veil 30 @ 0.90 | 100% | 0.29 / 0.5 |
| windowBackground | light | veil 255 @ 1.00 | - | 0.00 / 0.0 |
| windowBackground | dark | veil 30 @ 1.00 | - | 0.00 / 0.0 |
| hudWindow | light | blur r30, sat 1.9, veil 246 @ 0.48, cap 244.8 | 5% | 0.55 / 1.5 |
| hudWindow | dark | blur r30, sat 1.6, veil 40 @ 0.40, floor 20.4 | 5% | 0.62 / 1.7 |
| fullScreenUI | light | blur r30, sat 1.9, veil 246 @ 0.48, cap 244.8 | 5% | 0.47 / 2.0 |
| fullScreenUI | dark | blur r30, sat 1.8, veil 40 @ 0.50, floor 24.2 | 5% | 0.57 / 1.8 |
| toolTip | light | blur r30, sat 2.2, veil 246 @ 0.84, cap 233.3 | 5% | 0.32 / 0.8 |
| toolTip | dark | blur r30, sat 2.4, veil 40 @ 0.80, floor 35.7 | 5% | 0.31 / 1.0 |
| contentBackground | light | veil 255 @ 1.00 | - | 0.00 / 0.0 |
| contentBackground | dark | veil 30 @ 1.00 | - | 0.00 / 0.0 |
| underWindowBackground | light | blur r30, sat 2.2, veil 246 @ 0.84, cap 233.3 | 5% | 0.33 / 0.8 |
| underWindowBackground | dark | blur r30, sat 2.4, veil 40 @ 0.80, floor 35.7 | 5% | 0.31 / 1.0 |
| underPageBackground | light | veil 246 @ 1.00 | - | 0.00 / 0.0 |
| underPageBackground | dark | veil 20 @ 1.00 | - | 0.00 / 0.0 |

The gray ramp of the active look, in `withinWindow`, over five backdrops (shown
levels):

| Material | Scheme | over 0 | over 64 | over 128 | over 192 | over 255 |
|---|---|---|---|---|---|---|
| titlebar | light | 203 | 215 | 228 | 241 | 252 |
| titlebar | dark | 49 | 61 | 74 | 86 | 98 |
| selection | light | 0 | 53 | 113 | 173 | 232 |
| selection | dark | 33 | 91 | 149 | 207 | 255 |
| menu | light | 178 | 195 | 213 | 230 | 237 |
| menu | dark | 32 | 47 | 66 | 85 | 103 |
| popover | light | 150 | 173 | 198 | 224 | 241 |
| popover | dark | 28 | 50 | 75 | 100 | 124 |
| sidebar | light | 207 | 217 | 227 | 233 | 233 |
| sidebar | dark | 36 | 45 | 58 | 70 | 82 |
| headerView | light | 205 | 217 | 230 | 242 | 254 |
| headerView | dark | 25 | 37 | 50 | 62 | 74 |
| sheet | light | 255 | 255 | 255 | 255 | 255 |
| sheet | dark | 27 | 33 | 40 | 46 | 53 |
| windowBackground | light | 255 | 255 | 255 | 255 | 255 |
| windowBackground | dark | 30 | 30 | 30 | 30 | 30 |
| hudWindow | light | 121 | 151 | 184 | 217 | 245 |
| hudWindow | dark | 20 | 54 | 92 | 131 | 165 |
| fullScreenUI | light | 121 | 151 | 184 | 218 | 245 |
| fullScreenUI | dark | 24 | 52 | 84 | 116 | 145 |
| toolTip | light | 207 | 217 | 227 | 233 | 233 |
| toolTip | dark | 36 | 45 | 58 | 70 | 82 |
| contentBackground | light | 255 | 255 | 255 | 255 | 255 |
| contentBackground | dark | 30 | 30 | 30 | 30 | 30 |
| underWindowBackground | light | 207 | 217 | 227 | 233 | 233 |
| underWindowBackground | dark | 36 | 45 | 58 | 70 | 82 |
| underPageBackground | light | 246 | 246 | 246 | 246 | 246 |
| underPageBackground | dark | 20 | 20 | 20 | 20 | 20 |

### 10.4 States and accessibility

| Question | Result | Evidence |
|---|---|---|
| `withinWindow` or `behindWindow` | The same tree and the same pixels | worst difference between the two gray captures 1.5 levels, rms 0.34, 28 tiles per scheme |
| `isEmphasized` | No change, except in `selection` | trees and gray ramps equal in 13 materials |
| Inactive (`state = .inactive`) | A solid: no blur, no saturation, no cap; table below | trees and ramps |
| Window not key with `followsWindowActiveState` | exactly the inactive look | 14 materials in 4 sets, difference 0.00 |
| Reduce Transparency (per-process flag) | Every blurred material becomes a solid; table below | tree without backdrop layer; pixels flat within 0.03 level |
| Increase Contrast alone | No blur changes; 52 of 56 trees equal those without the flag, the other 4 are the dark `windowBackground` and `contentBackground` (30 to 32) | trees |
| Reduce Transparency with Increase Contrast | A different solid in 11 of 13 materials | table below |

The solid of each state, in levels of 255 (one gray; the dark `sheet` stays a
veil of 30 at alpha 0.90):

| Material | Scheme | Inactive | Reduce Transparency | RT + Increase Contrast |
|---|---|---|---|---|
| titlebar | light | 240 | 255 | 253 |
| titlebar | dark | 40 | 38 | 56 |
| menu | light | 240 | 236 | 240 |
| menu | dark | 40 | 50 | 20 |
| popover | light | 240 | 234 | 230 |
| popover | dark | 40 | 44 | 20 |
| sidebar | light | 240 | 246 | 250 |
| sidebar | dark | 40 | 56 | 40 |
| headerView | light | 255 | 255 | 255 |
| headerView | dark | 30 | 30 | 30 |
| sheet | light | 255 | 255 | 230 |
| sheet | dark | 30 @ 0.90 | 30 | 45 |
| windowBackground | light | 255 | 255 | 255 |
| windowBackground | dark | 30 | 30 | 32 |
| hudWindow | light | 240 | 228 | 220 |
| hudWindow | dark | 40 | 38 | 10 |
| fullScreenUI | light | 240 | 228 | 220 |
| fullScreenUI | dark | 40 | 38 | 10 |
| toolTip | light | 240 | 246 | 250 |
| toolTip | dark | 40 | 56 | 40 |
| contentBackground | light | 255 | 255 | 255 |
| contentBackground | dark | 30 | 30 | 32 |
| underWindowBackground | light | 240 | 246 | 250 |
| underWindowBackground | dark | 40 | 56 | 40 |
| underPageBackground | light | 246 | 246 | 246 |
| underPageBackground | dark | 20 | 20 | 20 |

### 10.5 Selection

`selection` is not a blur. Emphasized, it is the accent colour as a solid
(`#007aff`, the system blue of this host). Not emphasized, its layer is a black
layer with the compositing filter `vibrantColorMatrixSourceOver`, and the pixels
show a per-channel affine map of what lies under it, `shown = k b + c` (c in
levels of 255), with no channel mixing:

| Scheme | Active | Inactive | Reduce Transparency | RT + Increase Contrast |
|---|---|---|---|---|
| light | x 0.932 -6.2 | x 0.968 -3.0 | x 0.933 -6.2 | x 0.760 -22.3 |
| dark | x 0.905 +32.8 | x 0.954 +16.0 | x 0.906 +32.8 | x 0.795 +71.2 |

The fit over the gray, red, green and blue ramps has rms 0.27 and max 0.6.

### 10.6 What the model leaves out

The tree has a fifth layer, `CAChameleonLayer`, at opacity 0.05 (0.08 in the
dark `headerView`). It is not modelled, and its share is under the error of
the fit: over the gray, red, green, blue, cyan, magenta and yellow ramps of
every material, with the ramp blurred inside its region, the worst error is 2.0
levels and the worst rms 0.62 (the table of 10.3). A ramp that is not blurred
at its ends shows residuals of +2.9 over black (light) and -3.3 over white
(dark): they come from the region edge, not from the chameleon layer. Two
captures of the same material in the two blending modes differ by at most 1.5
levels (rms 0.34): that is the noise of the measurement.

### 10.7 Parity check

Use it to compare the compositor with macOS over the same backdrop.

1. **Draw the layout.** The compositor, or a BarrierKit window on it, draws the
   layout of `materials.swift`: for `ramp`, tiles of 40 x 1024 px at x = 100 +
   44 i, y = 128 over 256 bands of 4 px (levels 0 to 255 from the top; 128 px
   of padding of the first and last level above and below); for `edge`, tiles
   of 200 x 200 px in a grid of 7 per row, 360 px apart, with the step at the
   middle of each tile (white left on even columns, black left on odd ones).
   `~/src/probes/materials.sh <out> <appearance> <within|behind> <ramp|edge>`
   gives the tile order and the JSON that the check reads.
2. **Capture without converting.** A screenshot or the framebuffer of the
   output, as PNG bytes, saved as `<prefix>-full.png` with the JSON of the same
   layout as `<prefix>.json`.
3. **Compare.** `matcompare.py ramp <macos-prefix> <candidate-prefix>` and
   `matcompare.py edge ...` match the tiles by material, state and emphasis and
   print the rms and the largest difference of each. They exit 1 if one tile
   fails.
4. **Tolerances**, in levels of 255, per channel: ramps, all 256 levels,
   `max <= 2.0` and `rms <= 0.8`; step profile, `max <= 2.0` and `rms <= 0.8`.
   They come from the model: the worst error of the observed stack against the
   captures is 2.0 and its worst rms 0.62 (section 10.6), and the same capture
   repeated differs by 1.5 and 0.34. A sigma off by 1 px moves the step by at most
   0.5 level at 51 levels of amplitude (titlebar) and 1.2 at 133 (HUD).
5. **Check the region edge** with a step at the border of the tile: the 40
   pixels inside differ from the plateau by 0.0 on macOS. A compositor that
   samples outside the region fails it.

`ctappearance::material` met the same on the samples: 286 ramp samples within
1.5 levels (the gray ramp is blurred inside its region first), 6 step profiles
within 2 and 26 solids within 1.

### 10.8 Not measured

| Cell | Why | Setting that would measure it |
|---|---|---|
| Scale 2 (Retina) of every cell above | the only display of this host is 1x; a HiDPI mode is a global system setting, which this study did not change (a message from another session is not the user's authorisation) | the panel in its 1280x720 HiDPI mode (2560x1440 px at 180 Hz), set by the user; the layouts need halving to fit 720 pt |
| The system switch of Reduce Transparency and Increase Contrast | the per-process flags (`-reduceTransparency 1`, `-increaseContrast 1`) were used; the tree and the pixels of a material change with them, so the system switch probably gives the same | the system switches set by the user (`com.apple.universalaccess reduceTransparency` and `increaseContrast`) and restored |
| The materials over an animated or live backdrop | out of scope | |
| The frequency response beyond 192 px periods and of a 8 px checker | the stripes of 48 to 192 px are in 14.4; the checker was captured by an earlier session | an impulse or a checker over a tile at scale 0.125 |
| Scroll edge effects: what `soft` and `hard` change | their layer trees are identical (section 16); the style set on a titlebar accessory builds the same stack | a scroll view pinned under a floating bar with `scrollEdgeEffectStyle` in SwiftUI |
| The title bar's scroll pocket and the private material 30 of the titled windows | their trees are listed in section 13; the pixels were not fitted | a ramp behind a window whose content is transparent |
| The coloured ramps of the SwiftUI materials | the model reproduces the gray path and not the saturated colours (section 11.3) | a probe that reads what the hosting layer does with colour |
| The alpha of a see-through backdrop (a clear area of the same window) | not captured | a tile over a transparent region and a known layer under the window |

### 10.9 Corrections

- [`blur-ownership.md`](blur-ownership.md) 1.4, **blur extent**: the 10 to 90 %
  width of 47 to 50 px (and 35 px for `sidebar`, `underWindowBackground` and
  `toolTip` in light) came from tiles of 92 px, the width of the step lying
  inside them, and from a cap that clips the white end of the profile. Every
  blurred material has the same Gaussian of sigma 27.6 px: its 10 to 90 % width is
  2.563 sigma, 70.7 px, in both blending modes and both appearances.
- [`blur-ownership.md`](blur-ownership.md) 1.4, **accessibility**: "a per-process
  flag does not change a material" is wrong on macOS 27.2.1. With
  `-reduceTransparency 1` every blurred material of `NSVisualEffectView` is a
  solid (section 10.4). `-increaseContrast 1` alone removes no blur; the two
  together change the solid of eleven of thirteen.
- Section 4 of this file speaks of the desktop (Dock, menu, Control Center) and
  stands. Its claim that the menu and Control Center do not change with
  Reduce Transparency is about the system panels, not about the `menu` material
  of a window, which becomes a solid of 236 in light and 50 in dark.

### 10.10 For the compositor

One place for an implementer. Everything here is measured or observed in the
sections it names; nothing else is needed to reproduce a window material.

1. **Pick the look.** Read `MATERIAL_<NAME>_LIGHT` or `_DARK` (dark tint takes the
   dark one) and take `active`; `inactive` when the window is not active (a
   window that is not key with `followsWindowActiveState` is inactive);
   `reduced` with Reduce Transparency; `reduced_contrast` with Reduce
   Transparency and Increase Contrast. Increase Contrast alone keeps `active`.
   The SwiftUI thicknesses are `MATERIAL_<THICKNESS>_LIGHT` and `_DARK`, active
   only (section 11). A real popover or menu is glass, not `MATERIAL_POPOVER_*` or
   `MATERIAL_MENU_*` (section 13).
2. **Units.** `sigma` is in pixels at scale 1 (27.6; SwiftUI 59.2); `saturation`
   is a factor; `veil` is straight RGBA with the alpha rounded to 1/255; `cap`,
   `floor` and `bias` are fractions of 1; `gain` is a factor. All colours are
   encoded values in the output's own space, not sRGB conversions of macOS's
   result. Scale 2 is not measured: the design keeps the radius in logical
   pixels ([ADR 0091](../decisions.md)).
3. **Order**, per pixel of the region:
   1. blur the backdrop of the region with the Gaussian of `sigma`;
   2. saturate about the luma `0.2126 r + 0.7152 g + 0.0722 b` (`luma + saturation
      * (c - luma)`) and clamp to 0..1;
   3. mix the veil: `c + (veil - c) * alpha`;
   4. take the per-channel minimum with `cap` and the maximum with `floor`;
   5. `gain * c + bias` (only `selection` has them).
   `ctappearance::material::over` is steps 2 to 5; `blur_region` is step 1.
4. **Region.** The blur reads only the pixels of the region and renormalises its
   weights at the region's border. Blurring a larger area and cropping it
   leaks the neighbours; clamping to the edge of a texture differs by up to 1.5
   levels on a ramp and more on content that changes across the border.
5. **Resolution.** macOS runs the blur on a backdrop at one eighth of the
   resolution (SwiftUI one quarter, glass one half; section 14.3). A box
   downsample by 8, a Gaussian of 3.44 low-resolution pixels and a linear upsample
   reproduce the Gaussian of 27.6 within 0.3 level.
6. **Solids.** A look with `sigma` 0 and a veil of alpha 1 is one colour: draw it
   without sampling the backdrop.
7. **Dual Kawase** (`blur_down.frag`, `blur_up.frag`; `kawase.py`,
   `kawasecheck.py`; section 14.2). Today's 3 passes at offset 3.0 give sigma 16.8:
   too small. At the offsets that fit sigma 27.6 (3 passes 4.70, 4 passes 2.24,
   5 passes 1.06) the largest difference to the Gaussian through the stacks is 2.0,
   1.5 and 1.3 levels, and the rms against the captured steps 0.77, 0.64 and
   0.52 against 0.36 for the Gaussian: inside the tolerance of step 8 only with 5
   passes.
8. **Parity check.** Draw the layout of `materials.swift` (ramp tiles of 40 x 1024
   px, 256 bands of 4 px; step tiles of 200 x 200 px), capture the output as raw
   PNG bytes, and run `matcompare.py ramp|edge <macos-prefix> <candidate-prefix>`.
   It passes with `max <= 2.0` and `rms <= 0.8` per channel over all 256 levels
   and the 200 px of the step, and with the 40 px inside a border-aligned step
   equal to the plateau (section 10.7).
9. **Not here.** Scale 2, the system accessibility switches, the lens of glass,
   the title bar's scroll pocket and the real sidebar are listed in 10.8 and
   section 13.

## 11. SwiftUI materials, measured 2026-11-22

**Goal:** give the compositor the six `Material` values of SwiftUI
(`.ultraThinMaterial` to `.ultraThickMaterial` and `.bar`) next to the
materials of AppKit.
**Scope:** a hosted `Rectangle().fill(material)` over the gray ramp and the
step, in `withinWindow` (the gray ramp also in `behindWindow`), light and dark,
active, in a window that is not key and with Reduce Transparency and Increase
Contrast by process flag, at scale 1; and the coloured ramps.
**Reader:** whoever reads `MATERIAL_<THICKNESS>_LIGHT` and `_DARK`.
**Input:** the probe set `set=swiftui` of `materials.swift`; the method of
[section 10.1](#101-method).
**Output:** the stack, sigma, the values and the fit error.

### 11.1 The stack

A SwiftUI material is the stack of section 10.2 with one layer less and a wider
blur. One `CABackdropLayer` carries `colorSaturate` and then `gaussianBlur`
(`inputRadius` 60, `inputDither` 1, `inputNormalizeEdges` 1), and its own
background colour is the veil (gray 246 in light, 40 in dark, at the alpha of
the thickness). A second layer is the cap (`darkenBlendMode`, light) or the
floor (`lightenBlendMode`, dark). `.bar` has no saturation and no cap: the blur
and a white (dark: 30) layer at opacity 0.8, as `titlebar` and `headerView` of
AppKit.

- **Sigma 59.2 px** at scale 1 for `inputRadius` 60: 12 fits of the 400 px step,
  58.1 to 62.2, 59.4 in light and 59.0 in dark, rms 0.23 to 0.52. That is 0.99
  of the radius; the AppKit materials have 0.92 of theirs, so sigma is not a
  fixed share of the radius. A triple box fits within 0.3 in rms, because the
  veil leaves 0.16 to 0.64 of the step.
- **Region edge**, **space** and **order** as in section 10.2; the ramp of the
  fit is blurred inside its region.
- **Dither.** The standard deviation across a row is 0.38 to 0.57 levels (0.15
  for `.bar`), against 0.04 on average for `NSVisualEffectView`: the
  `inputDither` of the blur. A compositor may add it or leave it: it is below
  the quantisation.
- **Same family as AppKit.** The values repeat those of the AppKit materials. In
  light `.thin` is `hudWindow` and `fullScreenUI`, `.regular` is `popover`,
  `.thick` is `menu` and `.ultraThick` is `sidebar`, `toolTip` and
  `underWindowBackground`; in dark `.ultraThin` is `hudWindow`, `.thin` is
  `fullScreenUI` and the three thicker ones are the same four as in light.
  `.ultraThin` in light (veil 0.36, cap 248.6, saturation 1.8) has no AppKit
  counterpart. Only sigma differs.

### 11.2 The values

Levels of 255; `fit` is over the gray ramp blurred inside its region with sigma
59.2.

| Material | Scheme | Saturate | Veil gray / alpha | Cap or floor | Noise | Fit rms / max |
|---|---|---|---|---|---|---|
| ultraThin | light | 1.8 | 246 / 0.36 | cap 248.6 | 0.53 | 0.38 / 0.9 |
| ultraThin | dark | 1.6 | 40 / 0.40 | floor 20.4 | 0.50 | 0.29 / 0.9 |
| thin | light | 1.9 | 246 / 0.48 | cap 244.8 | 0.57 | 0.32 / 1.1 |
| thin | dark | 1.8 | 40 / 0.50 | floor 24.2 | 0.51 | 0.37 / 0.8 |
| regular | light | 2.0 | 246 / 0.60 | cap 241.0 | 0.51 | 0.35 / 1.1 |
| regular | dark | 2.0 | 40 / 0.60 | floor 28.1 | 0.51 | 0.35 / 0.9 |
| thick | light | 2.1 | 246 / 0.72 | cap 237.2 | 0.55 | 0.29 / 0.9 |
| thick | dark | 2.2 | 40 / 0.70 | floor 31.9 | 0.52 | 0.28 / 0.8 |
| ultraThick | light | 2.2 | 246 / 0.84 | cap 233.3 | 0.38 | 0.30 / 0.8 |
| ultraThick | dark | 2.4 | 40 / 0.80 | floor 35.7 | 0.49 | 0.27 / 0.7 |
| bar | light | - | 255 / 0.80 | - | 0.15 | 0.20 / 0.4 |
| bar | dark | - | 30 / 0.80 | - | 0.15 | 0.20 / 0.4 |

The tokens are `MATERIAL_ULTRA_THIN_LIGHT` to `MATERIAL_BAR_DARK` (`MaterialSet`)
of `components.material.swiftui`.

### 11.3 States

| Question | Result | Evidence |
|---|---|---|
| `behindWindow` | the same tree and pixels as `withinWindow` | gray ramp, 6 materials, both schemes: error 0.2 to 0.4 rms |
| Window not key | the five blurred materials stay active, unlike AppKit; `.bar` becomes a solid, 255 in light and 30 in dark | difference 0.00 for five tiles, `.bar` 49 levels |
| Reduce Transparency (flag) | every thickness is a solid, light (dark): `.ultraThin` 222 (32), `.thin` 228 (38), `.regular` 234 (44), `.thick` 240 (50), `.ultraThick` 246 (56), `.bar` 255 (30) | flat within 0.01 level; `.thin` to `.ultraThick` are the values of the AppKit family of section 10.4 |
| Reduce Transparency with Increase Contrast | light (dark): `.ultraThin` 210 (0), `.thin` 220 (10), `.regular` 230 (20), `.thick` 240 (30), `.ultraThick` 250 (40), `.bar` 255 (30) | same |
| The coloured ramps | **not reproduced**: gray is exact (rms 0.38 at most) but the red, green and blue ramps give rms 7.7 and max 18.8 in light and rms 15.2 and max 64 in dark, in either order of saturation and blur and with any luma weights tried | the saturate-then-blur order of the tree, the clamp before and after, and six luma weights were tried; the cause is open |

---

## 12. Glass tiles, observed 2026-11-22

**Goal:** record what `NSGlassEffectView` and SwiftUI's `glassEffect` do to a
backdrop, so that the compositor knows what it does not yet reproduce.
**Scope:** `NSGlassEffectView` with `style` regular and clear and `cornerRadius`
0, and `Color.clear.glassEffect(.regular | .clear, in: Rectangle())`; 120 px
wide tiles over the gray ramp and 200 px tiles over the step, `withinWindow` and
`behindWindow`, light and dark, scale 1. The lens, the highlight and the shadow
are not modelled.
**Reader:** whoever builds the glass pass of the compositor.
**Input:** the probe set `set=glass`.
**Output:** the observed filter, the tone at the centre of a tile and the width
of the blur.

1. **One filter.** The tree of both views is a `CABackdropLayer` with the
   filter `glassBackground` and about 80 named inputs, a `CASDFLayer` shape and
   a `vibrantColorMatrix` layer for the content. The probe reads them as
   numbers. Regular: `inputBlurRadius` 5, `inputBlurFillBlurRadius` 8,
   `inputBlurFillLightenOpacity` 0.675, `inputBleedAmount` 42,
   `inputBleedBlurRadius` 42, `inputFaceColorMatrixBlack` 0.4,
   `inputFaceColorMatrixWhite` 1.03, `inputFaceColorMatrixSaturation` 1.2,
   `inputFaceColorMatrixMaxLumaSDR` 0.94, `inputOuterRefractionAmount` 30
   (height 24), `inputInnerRefractionAmount` -60 (height 20),
   `inputKeyFillHighlightAmount` 0.4, `inputRingShadowOpacity` 0.06 and
   `inputShadowOpacity` 0.25. Clear: blur radius 0, face black 0.125 and white
   1.08, saturation 1.06, outer refraction 24 (height 15), no ring shadow and
   shadow opacity 0.
2. **Same for AppKit and SwiftUI.** `NSGlassEffectView` and `glassEffect` have
   the same filter inputs and the same pixels (difference under 1 level).
3. **Tone at the centre of the tile** (gray ramp, levels of 255; `withinWindow`
   and `behindWindow` are equal):

| Tile | Scheme | over 0 | over 32 | over 64 | over 128 | over 192 | over 224 | over 255 |
|---|---|---|---|---|---|---|---|---|
| glassRegular | light | 113 | 124 | 144 | 185 | 222 | 239 | 252 |
| glassClear | light | 52 | 62 | 93 | 154 | 215 | 246 | 255 |
| glassRegular | dark | 42 | 59 | 84 | 123 | 144 | 147 | 151 |
| glassClear | dark | 52 | 62 | 93 | 154 | 215 | 246 | 255 |

   Regular follows the appearance and compresses the top in dark (from level
   192 to 255 it rises only from 144 to 151). Clear is the same in light and
   dark, lifts black to 52, dips to 47 over a backdrop of 16 and clips at 255
   from level 240.
4. **Width of the blur**, 10 to 90 % of the white|black step through the tone
   curve, 200 px tile, `withinWindow`: regular 25.5 px in light and 37.1 px in
   dark; clear 2.1 px in both. It is not one Gaussian: the filter has several
   blurs (radii 5, 8 and 42) and the tone is not linear. The earlier 13, 18, 8
   and 9 px of [`blur-ownership.md`](blur-ownership.md) came from tiles of 92
   px.

---

## 13. Real controls, 2026-11-22

**Goal:** tell which material a real control uses, against the tiles.
**Scope:** the HUD panel, an `NSPopover`, an `NSMenu` pop-up, a sheet, a
sidebar of an `NSSplitViewController` and a titled window with a toolbar, in
light and dark, 1000 px tall over the gray ramp, with the layer tree of their
windows (`matcontrols.swift`); the tooltip from the earlier captures over white
and black in `~/src/captures/tooltip`.
**Reader:** whoever maps a kit control to a material.
**Input:** `~/src/captures/materials/control-<control>-<appearance>` and
`matcontrol.py`.
**Output:** per control, the material its window holds and how its pixels
compare with the tile.

| Control | What its window holds | Against the tiles |
|---|---|---|
| HUD (`NSPanel` with `.hudWindow`) | an `NSVisualEffectView` of material 13 (`hudWindow`), `behindWindow`, active; the stack of the dark `hudWindow` (saturation 1.6, floor 0.08) in both appearances | the dark `hudWindow` tile within 1.0 level, levels 30 to 200, in light and in dark |
| Tooltip (the earlier probe, over white and black) | the `toolTip` material | exact: over white and black 233 and 207 in light, 83 and 36 in dark; with Reduce Transparency and Increase Contrast 250 and 40, the `reduced_contrast` look |
| Sheet | the sheet's `NSWindow` holds an `NSVisualEffectView` of material 11 (`sheet`), `behindWindow`, active; its parent holds `contentBackground` (18, `withinWindow`) and private views of material 30 in its titlebar zone | over an opaque parent window the sheet shows 255 in light and 30 in dark (the earlier captures of `~/src/captures/sheets`), as the `sheet` tile over the parent's `windowBackground` |
| `NSPopover` | `_NSPopoverWindow` with an `NSVisualEffectView` of material 0 (`appearanceBased`, deprecated) and glass layers (`glassBackground`) | no tile; table below |
| `NSMenu` pop-up | `NSPopupMenuWindow` and `_NSGlassEffectWindow` (`glassBackground`) | no tile; table below |
| Sidebar (`NSSplitViewItem(sidebarWithViewController:)`) | no `NSVisualEffectView` of material `sidebar`: the window holds `contentBackground` | the sidebar column shows 234 to 236 in light and 37 to 41 in dark over backdrop levels 0 to 192 (the `sidebar` tile shows 207 to 233 in light and 36 to 70 in dark over the same levels): a nearly flat fill, not the material |
| Title bar and toolbar (titled window, unified toolbar) | `contentBackground` (18); in the layer tree backdrop layers with `gaussianBlur` radius 6 and a `colorMatrix` at opacity 0.82, a layer blur of radius 20 and a portal with `destIn`: the scroll pocket of the title bar edge | light 255 like the content; dark: a band of 38 below a line of 60 at the top edge, over the content of 30; not modelled |

The gray ramp that the popover and the menu show in glass (levels of 255, over
backdrop levels 0, 64, 128 and 192):

| Control | Scheme | over 0 | over 64 | over 128 | over 192 |
|---|---|---|---|---|---|
| `NSPopover` | light | 104 | 146 | 188 | 226 |
| `NSPopover` | dark | 34 | 82 | 122 | 143 |
| `NSMenu` pop-up | light | 103 | 142 | 180 | 217 |
| `NSMenu` pop-up | dark | 32 | 89 | 134 | 166 |

On macOS 27.2.1 the popover and the menu are glass, not the `popover` and `menu`
materials: those two are tiles that these controls do not draw here. The
compositor must not use `MATERIAL_POPOVER_*` or `MATERIAL_MENU_*` for a popup
menu or a popover; it needs the glass of section 12, which is not reproduced.
The same holds for a sidebar of a titled window: `MATERIAL_SIDEBAR_*` is the
`NSVisualEffectView` material, not what this control draws.

---

## 14. The blur algorithm and malaysia's Dual Kawase, 2026-11-22

**Goal:** say which algorithm macOS runs for the blur of a window material, with
what error, and whether malaysia's Dual Kawase reproduces it.
**Scope:** the blur of the 14 AppKit materials and the six SwiftUI ones at scale
1, from the step and ramp captures of section 10. The kernel of the glass is not
here.
**Reader:** whoever writes or reviews the compositor's blur pass.
**Input:** the 10 white|black step profiles of `edge-center-<appearance>-behind`
(`titlebar`, `menu`, `popover`, `hudWindow`, `fullScreenUI`, light and dark), the
36 sigma fits of section 10.2, `~/src/probes/kawase.py` and `kawasecheck.py`.
**Output:** the best model and its error, the check of the Dual Kawase, an exact
alternative and what is still open.

### 14.1 Which kernel

Each candidate blurs the 200 px step inside its tile, the exact stack of section
10.2 shades it, and the rms against the captured profile is taken (levels of
255; mean of the 10 profiles; the best parameter of each candidate):

| Candidate | rms | Parameter |
|---|---|---|
| Gaussian, renormalised at the border | **0.36** (0.27 to 0.53) | sigma 27.6 px |
| Dual Kawase, 5 passes (malaysia's shaders) | 0.52 | offset 1.0 |
| Dual Kawase, 4 passes | 0.64 | offset 2.25 |
| Dual Kawase, 3 passes | 0.77 | offset 4.7 |
| Triple box of the same variance | 0.31 to 0.56 per material, worse than the Gaussian in 9 of 9 | width 53 to 55 |

The Gaussian is the best model and its rms is the quantisation floor (0.29 for
a rounded value). The Dual Kawase is worse in all 10 profiles, by a factor of 1.4
to 2.1 depending on the passes; the low-veil materials show it most (`hudWindow`
dark: 0.45 for the Gaussian, 1.17 / 0.97 / 0.79 for 3 / 4 / 5 passes). The
box and the Kawase are not what the render server runs.

The space is encoded (linear light fits at 1.8 to 12), the weights are
renormalised at the border of the region (0.0 levels of leak), the order is
saturate, veil, cap or floor, and nothing is added to the colour (no noise in
AppKit; SwiftUI dithers at 0.5 levels). The radius of the stack (`inputRadius`
30; 60 for SwiftUI) is 1.09 and 1.01 times sigma (27.6 and 59.2): sigma is the
measured value, the radius only names it.

### 14.2 Does malaysia's Dual Kawase reproduce it?

Not as it is, and only partly with the right parameters. `ctdesign` has
`BACKGROUND_BLUR_PASSES` 3 and `BACKGROUND_BLUR_OFFSET` 3.0: that is sigma 16.8 px
(`kawase.py table`), not 27.6. With the offset fitted to sigma 27.6, the
check of `kawasecheck.py` (largest difference to the exact Gaussian through the
stack of the nine blurred looks, light and dark, white|black step of 1024 px):

| Algorithm | Largest difference | Taps per full-resolution pixel |
|---|---|---|
| Dual Kawase, 3 passes, offset 4.70 | 2.0 levels (`hudWindow` dark) | 12.1 |
| Dual Kawase, 4 passes, offset 2.24 | 1.5 | 12.3 |
| Dual Kawase, 5 passes, offset 1.06 | 1.3 | 12.3 |
| Box downsample by 4, Gaussian sigma 6.9, linear upsample | 0.1 | 7.6 (3.8 with linear sampling) |
| Box downsample by 2, Gaussian sigma 13.8 | 0.0 | 51.5 |
| Box downsample by 8, Gaussian sigma 3.5 | 0.3 | 1.9 |
| Triple box, width 55 | 1.4 | 6 passes of a running sum |

The parity tolerance of section 10.7 is 2.0 levels and 0.8 rms. Against it:

1. A Dual Kawase of 3 passes sits at the limit of the maximum and over the rms
   for `hudWindow`, `fullScreenUI` and `popover` (0.97, 0.97 and 0.75 in light;
   1.17 and 0.94 in dark): **no**. 4 passes: `hudWindow` dark 0.97, over. 5 passes:
   0.79 at most, inside, at the cost of one more pass and a texture of 1/32 of the
   area (its stability on moving content is not measured).
2. **A downsampled Gaussian does** (0.1 level) at about the cost of the Dual
   Kawase. It also gives any sigma by changing the Gaussian, not the pass count:
   59.2 for SwiftUI is a downsample by 8 and a sigma of 7.4.
3. The kernel is not the whole difference. The compositor must blur only the
   region and renormalise at its border (section 10.2); a Kawase over a whole
   output texture clamps at the texture, not at the region, so it leaks the
   neighbours. A ramp blurred with clamp-to-edge instead of renormalised weights
   differs by up to 1.5 levels, and content that differs across the border of the
   region differs by more.
4. The radius is in logical pixels at scale 1 (28 px, sigma 27.6); scale 2 is
   not measured.

The compositor session decides; no compositor code is written here.

### 14.3 How the blur is configured: a throwaway probe with the private classes

`~/src/probes/cafilter.swift` builds a `CABackdropLayer` and a `CAFilter` by hand
(private classes of Core Animation, looked up by name at run time in a throwaway
probe; nothing of it enters the kit) over the white|black step, and
`cafit.py` fits sigma through the exact backdrop. Results, tile of 200 px
(400 px from radius 50), levels of 255:

| Sweep | Result |
|---|---|
| `inputRadius` 5, 10, 15, 20, 25, 30 | sigma 4.70, 9.85, 14.85, 19.65, 25.15, 29.70: 0.99 times the radius (rms 0.2 to 0.5). From radius 40 the 200 px tile truncates the fit; the 400 px tile gives 0.96 at 50 and 60, and is too small beyond |
| `inputNormalizeEdges` | with the step at the border of the tile (black outside): default and `true` leave the 40 px inside at 255.0; `false` shows 235 to 247: it samples outside the layer. The default is already the region-only behaviour |
| `inputHardEdges`, `inputLinear`, `inputDither` | same sigma (29.7 to 29.8), same border (`inputLinear` and `inputHardEdges` change nothing here; `inputDither` only adds noise) |
| `inputQuality` | `high`: sigma 29.0 and a border leak (202 to 239 inside); `low`: sigma 27.1 and no leak; unset or `default`: 29.7 to 29.8 |
| The backdrop layer's `scale` (`CABackdropLayer`, read by key) with radius 30 | 1: 29.8; 0.5: 29.6; 0.25: 28.6; **0.125: 27.9**; 0.0625: 31.2 and a staircase (rms 3.5). With radius 60: 58.0, 59.5, 58.8, 56.0, 53.6 |

The layers that macOS builds carry that `scale`: the backdrop layer of every
`NSVisualEffectView` has **`scale` 0.125**, `windowServerAware` 1 and
`allowsGroupBlending` 0 (1 for the `sidebar`; the `toolTip` and
`underWindowBackground` share its tree and were not read); the six SwiftUI materials have **0.25** with radius 60;
`NSGlassEffectView` and `glassEffect` have **0.5**. So the blur runs on a backdrop
at one eighth of the resolution (one quarter for SwiftUI, one half for glass).
That is why `inputRadius` 30 gives sigma 27.6 and not 29.7: a probe layer at
scale 0.125 with radius 30 gives 27.9, and with scale 0.25 and radius 60, 58.8
(the materials: 27.6 and 59.2). Both a plain Gaussian and a box downsample by 8,
a Gaussian of 3.44 low-resolution pixels and a linear upsample fit the profile
with the same rms (0.685), so the shape does not tell them apart; they agree
within 0.3 level through the stack (section 14.2). A compositor that blurs at
1/8 resolution with that Gaussian follows the structure macOS has, and costs
1/64 of the pixels.

| Square waves of 48, 64, 96, 128 and 192 px (3 periods per tile, `stripes` layout) | see 14.4 |

| Question | State |
|---|---|
| The filter used to downsample the backdrop (box, bilinear, mip) and to upsample it | not told apart by the step; an impulse or a checker over a tile at scale 0.125 would |
| The frequency response between the step and the checker | not captured (section 10.8) |
| The alpha of a backdrop that is transparent (a window behind a see-through window) | not measured |
| The glass kernel | another session measures it |

### 14.4 Square waves

Square waves of 48 to 192 px (`stripes-<appearance>-<blend>`; `matstripes.py`)
through `titlebar`, `sidebar` and `hudWindow`, `withinWindow` and
`behindWindow`, light and dark, give the transfer of the blur between the step
and the checker. Measured swing over the middle period, in levels: 1 to 4 at 48
and 64 px, 12 to 43 at 96, 16 to 129 at 192 (a Gaussian of 27.6 px passes 0.0013
of a 48 px fundamental and 0.665 of a 192 px one).

- A Gaussian that sees only its tile (weights renormalised) fits each tile with
  sigma 27.1 to 28.3 px for the periods of 96 to 192 (rms 0.4 to 2.8; the 48 and
  64 px tiles carry too little signal: sigma 22 to 27).
- With clamp-to-edge weights instead of renormalised ones the same Gaussian is
  4 times worse (rms 6.9 to 18.4, maximum 35 to 52 levels against 1.5 to 4.3 and 5
  to 13): for content that changes across the border of the region the region
  semantics weigh more than the kernel shape.
- A triple box (width 49 to 55) fits as well as the Gaussian (mean rms 1.3 to 1.4
  against 1.3 to 1.65); the 3-pass dual Kawase gives 2.2 to 2.5, the downsample by
  8 with a Gaussian 2.5 to 2.6 and the 5-pass dual Kawase did not simulate
  reliably. The stripes do not separate the Gaussian from a triple box, as the
  step did not; they confirm that the response follows a Gaussian of 27 to 28 px.
- The rms of 1.3 to 1.7 over the stripes is larger than the 0.36 of the steps and
  is open: the grid of the 1/8 backdrop may be involved.

---

## 15. Values read from the system and where each value comes from, 2026-11-22

**Goal:** record the material recipes the system ships, compare them with the
measurements and give every value its source.
**Scope:** `CoreMaterial.framework/Versions/A/Resources/*.materialrecipe` of macOS
27.2.1 (build 26B131), read as data. Nothing is copied: the numbers below carry
their resource and key. The recipes are not used by the AppKit and SwiftUI
materials measured here.
**Reader:** whoever compares a value of this study with Apple's own files.
**Input:** `plutil -p` of the recipes; the layer trees and captures of section 10.
**Output:** the recipes of the materials that look like the ones of sections 10
and 11, the differences and a table of provenance.

### 15.1 The recipes

Each `.materialrecipe` is a property list with `baseMaterial.materialFiltering`.
The keys are `blurRadius`, `backdropScale` (the blur runs on a backdrop scaled by
that factor), `blurAtEnd`, `saturation`, `brightness`, `luminanceAmount` and
`luminanceValues` (four values; a `luminanceColorMap.png` of 256 x 1 pixels sits
beside them). These are the recipes whose names resemble the SwiftUI
thicknesses and the Dock:

| Recipe (`CoreMaterial.framework/Resources/<name>.materialrecipe`) | blurRadius | backdropScale | saturation | brightness | luminanceAmount | luminanceValues |
|---|---|---|---|---|---|---|
| platformContentUltraThinLight | 22.5 | 0.25 | 1.1 | 0.12 | 0.5 | [0.45, 0.55, 0.65, 0.68] |
| platformContentThinLight | 30 | 0.25 | 1.35 | 0.12 | 0.6 | [0.725, 0.825, 0.76, 0.73] |
| platformContentLight | 30 | 0.25 | 1.5 | 0.1 | 0.75 | [0.9, 0.83, 0.925, 0.815] |
| platformContentThickLight | 45 | 0.25 | 1.5 | 0.045 | 0.88 | [0.99, 0.95, 0.98, 0.905] |
| platformContentUltraThinDark | 22.5 | 0.25 | 1.1 | 0 | 0.5 | [0.24, 0.24, 0.3, 0.39] |
| platformContentThinDark | 30 | 0.25 | 1.35 | 0 | 0.6 | [0.2, 0.21, 0.1, 0.15] |
| platformContentDark | 30 | 0.25 | 1.5 | 0 | 0.75 | [0.16, 0.26, 0.1, 0.1] |
| platformContentThickDark | 45 | 0.25 | 1.5 | 0 | 0.88 | [0.14, 0.16, 0.1, 0.03] |
| platformChromeLight | 22.5 | 0.25 | 1.1 | 0.1 | 0.75 | [0.8, 0.9, 1.1, 0.825] |
| platformChromeDark | 22.5 | 0.25 | 2.0 | -0.1 | 0.75 | [0.23, 0.52, 0.27, 0.255] |
| dockLight | 30 | 0.25 | 1.8 | 0.08 | 0.5 | [0.3, 0.5, 1.0, 0.77] |
| dockDark | 30 | 0.25 | 1.6 | - | 0.5 | [0.29, -0.2, 0.375, 0.65] |

### 15.2 Against the measurements

The recipes and the stacks of sections 10 and 11 differ, so the recipes are not
the source of the window materials:

- The layer trees of `NSVisualEffectView` and of SwiftUI's `Material` hold a
  `CABackdropLayer` with `gaussianBlur`, `colorSaturate`, a veil layer and a cap
  or floor layer. They hold no recipe, no luminance curve, no `brightness` and no
  `backdropScale`.
- The saturation of the recipes is 1.1 to 2.0; the stacks have 1.6 to 2.4 and none
  in the titlebar. The radius of the recipes is 22.5, 30 and 45; the stacks 30 and
  60.
- Where the recipes could still explain a surface (the Dock: radius 30,
  saturation 1.8 in light and 1.6 in dark, luminance 0.5), the capture decides:
  [`macos-blur.md`](macos-blur.md) holds the Dock measurements and this study has
  not cross-checked them.

### 15.3 Provenance

| Value | Source | Where |
|---|---|---|
| Stack and order of the layers, `inputRadius`, `colorSaturate`, veil colour and alpha, cap, floor, chameleon opacity | the layer tree of the probe's own process (public `CALayer` properties; runtime observation, not API) **and** the pixels | 10.2, 10.3 |
| Sigma 27.6 px and 59.2 px, Gaussian shape, space, region edge, noise | pixels (captures) | 10.2, 11.1, 14.1 |
| Fit error | pixels against the stack | 10.3, 10.6 |
| The 14 materials and their order | API: `NSVisualEffectView.h`, SDK 27.2 | 10.1 |
| Selection gain and bias, the solids of inactive, Reduce Transparency and Increase Contrast | pixels, and the tree for the solids | 10.4, 10.5 |
| Device space of the mix, the display profile | the ICC profile of the display read with `CGDisplayCopyColorSpace` and the pixels | 10.2 |
| The recipes of `CoreMaterial.framework` | read from the system resource; not used by these materials | 15.1 |
| The glass inputs | the layer tree (observed) and the pixels of the gray tone | 12 |

Where a read value and a capture differ the capture wins: the recipes lose to the
trees and the pixels in 15.2.

---

## 16. Scroll edge effects, observed 2026-11-22

**Goal:** record what AppKit builds for the scroll edge effect of a title bar.
**Scope:** `scrolledge.swift`: a titled, full-size-content window with a unified
toolbar and a scroll view over a checker; a bottom title bar accessory with
`preferredScrollEdgeEffectStyle` `automatic`, `soft`, `hard` or none; light and
dark, scale 1; the layer tree only, not the pixels.
**Reader:** whoever builds the compositor's scroll edge request.
**Input:** `~/src/captures/materials/scrolledge-<style>-<appearance>.json`.
**Output:** the stack, the values and what the styles do not change.

1. **The stack of the pocket**, for every style: a `CABackdropLayer` with
   `variableBlur` (`inputRadius` 2 with no accessory, 2.4737 with one,
   `inputNormalizeEdges` 1), `colorMatrix` and `sdrNormalize`; a
   `CABackdropLayer` with `gaussianBlur` (radius 6, `inputNormalizeEdges` 1), a
   `colorMatrix` and `sdrNormalize`, and a second one at opacity 0.82; a layer
   filter `gaussianBlur` of radius 20 on an `NSViewBackingLayer`
   (`inputNormalizeEdges` 0); `CAPortalLayer`s with `destIn` and a
   `CAGradientLayer`; and a SwiftUI `VisualEffectMaterialView`
   (`CAHostingLayer`) inside.
2. **Size.** The pocket is 52 pt tall without the accessory and 88 pt with it.
3. **The styles.** `automatic`, `soft` and `hard` give the same tree: 314 layers,
   0 differences in class, opacity, hidden flag, bounds or colour, light and
   dark. The earlier note of [`blur-ownership.md`](blur-ownership.md) that `soft`
   and `hard` looked the same is the tree too: the style set on the accessory
   does not change what AppKit builds in this probe.
4. **Not modelled.** The variable blur is 2 to 2.5 and the layer blur 20 (the
   edge falls off over the gradient). The pixels were fitted on 2026-11-22: a tone
   line per appearance, a blur of sigma about 3.4 and a hard edge
   ([Shader needs](shader-needs.md#not-verified)).
