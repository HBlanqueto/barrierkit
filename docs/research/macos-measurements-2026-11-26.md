# macOS measurements, 2026-11-26

**Goal:** fill the cells the earlier macOS research left empty: window
shadows, unmeasured animations, wallpaper tint, the shell in light mode and the
dependence on display scale.
**Scope:** macOS 27.2.1 (26B131), Apple M4, one 2560×1440 display at scale 1 and
180 Hz, Liquid Glass slider at 1 (Tinted), wallpaper tinting on, wallpaper
Aerial, dark system mode unless the row says light.
**Reader:** anyone who sets a shadow, a duration, a spring or a tint token.
**Input:** probes written for this pass (flat-colour windows, ScreenCaptureKit at
180 fps, accessibility tree, `screencapture -x -m` read as raw device bytes).
**Output:** tables, then the corrections to older documents, then what is still
not measured.

Sizes are points unless a row says px. Alpha and tone values are encoded sRGB
device values, as in `macos-blur.md`. Captures come out in BT.2020 primaries with
the sRGB curve: grays are read raw, colours are converted first.

## 1. Window shadows

Method as the menu shadow in `macos-blur.md`: alpha(d) = 1 − px/bg over flat
white, gray 128 and black. The shadow is black composited in encoded sRGB, with
no light glow, and linear in the backdrop (gray 128 gives the same alpha as
white within 0.002). Every window has a 1 px dark outline on the first row
outside its frame; d = 1 is the row after it, so d = 0 is the centre of the
outline row, 0.5 px outside the frame.

One blurred, offset box fits all four sides (rms 0.0004 to 0.0012):
alpha(d) = A · ½ · erfc((d − spread ∓ dy) / (σ√2)). Spread grows (+) or
shrinks (−) the box in px. Dy pushes the shadow down: upper sign at the bottom
edge, lower sign at the top edge.

| Case | σ | dy | A aqua | A dark |
|---|---|---|---|---|
| Titled window, main | 19.5 | 18 | .393 | .689 |
| Titled window not main, and sheet | 7.75 (sheet 7.5) | 7 | .210 | .431 (sheet .401) |
| Utility panel | 7.5 | 4 | .210 (key .241) | .289 |
| Borderless window | 7.5 aqua, 8.5 dark | 4 | .207 | .394 |
| Popover (top and sides) | 7.0 | about 4 | .188 | .226 |
| Alert, modal / sheet | 16.5 | 10 | .205 / .176 | .224 / .205 |

- Only the borderless profiles are kept whole
  (`scripts/macos-probes/shadows/data/profiles-meta-borderless.txt`). Their
  fit gives spread 0 in aqua and −3 in dark.
- The titled rows keep 11 points per side in `data/summary-shadows.txt`.
  They fit with spread 0, so their box is the frame grown by 0.5 px
  (`WINDOW_SHADOW_SPREAD` counts from the frame). The not-main σ is 7.75:
  `fit.py` steps σ by 0.5 and returns 7.5, but in steps of 0.25 7.75 fits
  better (rms .0006 aqua and .0010 dark, against .0007 and .0014), matches
  section 6 at 1x and gives the extents below, where 7.5 gives 8 at the top.
- The profiles of the other rows were not kept, so their spread cannot be
  recovered; read their σ as fitted with an unknown spread.
- The strong shadow follows the **main** window, not the key window.
- Dark is 1.75 to 2 times stronger than aqua at the same σ and dy.
- The alert is small (260×176), so its σ is biased low.
- The outline is black with alpha: about .14 to .15 in aqua and .85 (not main)
  to .95 (main) in dark. A light bevel one row inside the frame exists only in
  dark (48 to 60 against a fill of 30 to 38, brightest on top).
- Behind a sheet the parent is veiled: gray 255 reads 204 in aqua and 112 in
  dark (a veil of about 61 % of gray 171, or 20).
- Extent where alpha < .005: titled main 26 (top), 62 (bottom), 44 (side);
  not main 9, 23, 16.

## 2. Dock bounce

`requestUserAttention(.criticalRequest)`: a cycle of two hops every 2.01 s until
cancelled. Big hop: peak 48, airborne .42 to .46 s, peak at about .21 s. After
.09 s on the ground a small hop: peak 16, airborne about .23 s. Then a 1.5 s
pause. The informational request plays one cycle. The curve is a sine arch,
h = H · sin(πu) (rms 1.2 px; a parabola gives 2.2). The first hop starts .38 s
after the call (critical) or .05 s (informational). Time resolution was 16.7 ms.

## 3. Animations

Capture: ScreenCaptureKit with `minimumFrameInterval` 1/180 and `pixelFormat`
BGRA; flat saturated colour windows so the bounding box and boundary columns
read exactly; keys and pointer posted from the same host clock. Time 0 is the
first changed frame.

| Animation | Result |
|---|---|
| Space switch, ctrl+Right/Left (6 runs, ±5 ms) | Constant 64 gap between Spaces, travel 2624, no overshoot. Overdamped spring ω0 23 to 24, ζ 1.72 to 1.75. t10 19 to 31, t50 103 to 119, t90 319 to 331 ms, last pixel at 1008 to 1019 ms. Same both ways. A cubic-bezier (0.2, 1, 0.2, 1) fits worse (rms .010 against .001 to .003). |
| Mission Control entry | Thumbnails scale to .649. Critically damped spring ω0 25 to 26. t50 58 to 69, t90 153 to 158, last change 247 to 275 ms. Exit the same (ω0 25 to 27). |
| Mission Control Spaces strip | Low confidence (rms .06 to .09): appears about 60 ms after the key, half height at about 45 ms, settles at about 245 ms; final height 40. |
| Dock magnification in/out | 59 to 128 in 108 to 119 ms, delay 28 to 33, t50 64 to 69, nearly linear (bezier (0.5, 0.4, 0.5, 0.7), T 125 ms). Out is the same. |
| Dock magnification profile | size = 59 + 69 · (1 − (d/207)^2.1)^1.6, rms .37; d is the pointer distance from the tile's rest centre. Radius of influence 207 (3.3 tiles). Table: d 0 → 128; 63 → 118.5; 126 → 93; 189 → 63.5; 215+ → 59. Symmetric. |
| Dock auto-hide | Starts 248 to 259 ms after the pointer reaches the edge (`autohide-delay` unset). Show (74 travel): spring ω0 14 to 18, ζ .70 to .85, t50 75 to 86, last 169 to 186. Hide: ease-in, starts 35 to 39 ms after the pointer leaves, t50 114, last 175 to 192. |
| Minimize, genie (default) | 458 to 464 ms (restore 486 to 497). The bottom row narrows first (t50 about 164), the top row holds about 150 ms then narrows (t50 about 280); the bbox widens only to 1.04× here. Centre path (1280,616) → (1313,694) at 153 ms → (1475,977) at 242 → (1722,1238) at 342 → (1796,1349) at 436. |
| Minimize, scale | Uniform, straight path to the Dock slot. Ease-in-out (0.4, 0.1, 0.6, 0.9), t50 86 to 103, last 169 to 186, the same each way. |
| Window close | Width 1801 → 1771 in 31 to 42 ms, then a fade: t50 31, t90 86 to 97 ms. Spring ω0 55 to 68, ζ 1.2 to 1.4. |
| Window open | **No scale animation seen.** The window is at its final size in its first frame, 15 to 68 ms after the call (ScreenCaptureKit and 1 ms `CGWindowList` polling; new and reused windows; `makeKeyAndOrderFront` and `orderFront`). Cause unknown. |

Not measured: drag-to-edge snap, title-bar drag and live resize (posted mouse
events never moved a window); the notification banner (none appeared).

## 4. Wallpaper tint

`AppleReduceDesktopTinting` is **0 (tinting on)** on this Mac. A per-process
`-AppleReduceDesktopTinting YES` argument switches it off exactly like the
global key.

Model, in encoded sRGB: out = neutral · (1 − k) + k · wallpaper (per channel).
Nine solid wallpapers (red, green, blue, white, black, gray 128, orange, navy,
ice); residuals are rms / max encoded levels.

| Surface | Look | k (R G B) | Neutral | rms / max | Switch off |
|---|---|---|---|---|---|
| windowBackground, contentBackground, plain NSWindow, split-view content | dark | .046 .047 .044 | 31.6 30.9 31.2 | 1.15 / 2.9 | flat 30 |
| same | aqua | 0 | 255 | 0 | 255 |
| AppKit sidebar | dark, main | .049 .032 .078 | 38.4 38.2 37.4 | 5.0 / 15.7 | flat 40 |
| same | dark, not main | .041 .032 .068 | 36.6 36.1 35.6 | 4.2 / 13.4 | flat 37 |
| same | aqua | under .01 | 236 / 243 | 0.7 / 1.6 | flat 235 / 242 |
| underWindowBackground | aqua / dark, main | .045 | 227 / 59.6 | 1.2 / 3.1 | same as on |
| menu, popover, hudWindow | main | about .045 | aqua 214 / 200 / 186, dark 68 / 77 / 95 | 1.1 to 1.3 / 2.6 to 3.9 | same as on |
| sidebar, underWindowBackground, menu, popover | not main | aqua .044, dark .09 | aqua 240, dark 44 | 1.3 to 2.0 / 3.9 to 5.7 | same as on |
| headerView | dark | .065 to .071 main, .086 to .091 not main | 53 / 34 | 1.8 to 2.1 / 4.4 to 5.7 | same as on |
| headerView | aqua | 0 | 230 main, 255 not main | 0 | same |
| NSGlassEffectView | both | under .006 | 224 aqua / 75 dark | 0.3 / 1.0 | same |

- The switch changes only the **dark non-blurring window backgrounds** and the
  **dark AppKit sidebar**. In aqua neither is ever tinted.
- Blurring materials always take about 4.5 % of the wallpaper (the
  `CAChameleonLayer` at opacity .05, .08 for the dark header view), switch on or
  off. Not-main behind-window materials ignore the windows behind them.
- The dark sidebar tint is mostly colour: `out ≈ 40.3 + 0.017 · Y709 + 0.086 ·
  (w − Y709)` (rms 2.9, max 9.1). Saturated wallpapers overshoot (R 43 over red
  against 41 over white); residuals of 1 to 3 levels over a range of 14 to 16
  remain.
- The system light or dark mode changes nothing; the window's own appearance
  decides.
- **One flat colour per window.** Twenty samples 50 apart across a 1000×300
  window agree within 1 level. It is the average of the wallpaper picture under
  the window frame. On a half red, half blue wallpaper the colour starts to
  change when the right edge crosses the boundary and is blue when the left edge
  passes it: a near-linear ramp about one window width long. Windows between the
  tinted window and the wallpaper do not matter.
- Timing: no fade. A moved window shows the new tint in its first frame; a
  wallpaper change shows it one frame (about 6 ms) after the wallpaper pixels.
- `NSGlassEffectView` is not tinted by the wallpaper; it shows its backdrop.

## 5. Shell surfaces, light and dark (slider at 1)

Raw output for inputs 0 16 32 48 64 96 128 160 192 224 240 255:

| Surface | Dark | Light |
|---|---|---|
| Dock | 12 26 39 51 64 87 108 128 145 161 169 176 | 38 50 62 74 86 110 134 158 182 206 218 229 (38 + .749 · in) |
| App menu = Spotlight = Apps (equal to 0 levels) | 32 39 46 52 58 68 76 82 85 86 86 86 | 185 190 195 200 205 215 224 233 241 248 252 255 |
| Control Center tile | 32 41 51 61 71 90 110 129 149 168 178 186 | identical |
| Popover | 25 36 47 59 70 93 115 138 160 183 194 205 (25 + .706 · in) | 71 82 92 103 114 135 157 178 199 221 232 242 (71 + .67 · in) |
| Control Center backing panel | black veil, alpha .094 to .098 | identical |
| Finder sidebar | 38 38 42 flat | 237 flat |
| Probe NSSplitView sidebar | 38 flat | 242 flat |

Red (208,74,34) and blue (59,28,243), raw device values: Dock dark (213,60,14)
(64,27,255), light (248,81,31) (87,48,255); menu family dark (137,53,28), light
(255,197,168) (207,186,255); popover dark (197,71,33), light (235,115,79)
(112,85,255); Control Center tile the same in both.

Single-Gaussian σ (px = pt at 1x), steps 0|255 / 0|64 and 1 px lines
(white / black): Dock 16.0 / 15.9, lines 15.7 / 19.1 in dark, 16.0 / 16.0, lines
15.5 / 19.0 in light. Control Center tile 16.7 / 16.4, 14.5 in dark; 16.5 / 16.5,
14.5 in light. Popover 15.2 / 15.3 dark, 15.5 / 15.0 light. App menu, Spotlight
and Apps: dark 33, 28, 27.5 (rms 5.6, 2.5, 1.7: probably core plus halo that one
Gaussian cannot separate); light 16, 15, 15.5 (rms 1.0, .8, .5).

Rims: Dock top is an inner gradient then a light line, 179 → 189 dark, 236 → 255
light (over white); the Dock left end is one dark pixel (104/50/0 dark,
161/64/0 light). Dark panels have an inner light line (menu bottom 121/111/73,
Apps top 134/126/91) and most edges an outer near-black pixel (8 to 30 over
white, 0 over black). Light panels have an inner light line (255/246/215) and an
outer gray pixel (122 to 195 over white, 55 to 91 over gray).

Shadow A · exp(−d/τ), over white (white and gray agree within .01):

| Edge | Dark A / τ | Light A / τ |
|---|---|---|
| Dock | 0 | 0 |
| Menu, bottom | .201 / 17.5 | .171 / 17.0 |
| Menu, right | .129 / 13.0 | .113 / 13.0 |
| Apps, right | .381 / 15 | .219 / 15 |
| Apps, top | .129 / 10.5 | .073 / 11.0 |
| Spotlight, bottom | .50 / 27.5 | .375 / 23 |
| Popover, bottom | .233 / 7.5 | .171 / 7.5 |

In light the amplitude falls to .55 to .9 of dark and τ stays. Radii (px, dark /
light): Dock 21.5 / 21.9 (0.27 of its height 79), app menu 11.1 / 11.6,
Spotlight bar 25.1 / 26.8, Apps 25.7 / 27.6, popover 20.0 / 20.1; the modes are
equal within the fit error.

Menu bar: Clear in light too (rows 4 to 26 equal the rows 32 below to 0 levels
over 14 backdrops). Its text and icons are **white in light**, and Control
Center keeps its dark look in light mode; the cause is unknown (guess: the dark
Aerial wallpaper, untested).

## 6. Display scale

Everything above is in **points**. At 1280×720 at scale 2 every px value doubles
and the pt value is unchanged; 1920×1080 at scale 1 gives the 1440 numbers to the
digit. Baselines taken before and after match to the digit.

| Item | 1440@1x | 720@2x px (pt) | Verdict |
|---|---|---|---|
| Dock list | 1116×79 | 2232×158 (1116×79) | pt |
| Dock glass height | 79 | 158 (79) | pt (width 3 pt narrower at 2x) |
| Reserved bottom, menu bar | 85, 30 | 85, 30 pt | pt |
| Dock blur σ | 16.0 | 32.0 to 32.5 (16.0 to 16.25) | pt |
| 1 px line under Dock glass | σ 16.5 | 30.5 (15.25) | pt: no px-based softening |
| Dock corner radius | 21.5 | 43.1 (21.55) | pt |
| Window shadow, main dark σ / dy / A | 19.5 / 18 / .690 | 39 / 36 / .690 (19.5 / 18) | pt |
| Window shadow, not main aqua | 7.75 / 7 / .210 | 16 / 14 / .220 (8 / 7) | pt |
| Window outline | 1 px | 2 px | pt |
| Window corner radius (circle fit) | 15.3 | 30.7 to 30.9 (15.35 to 15.45) | pt |
| Popover radius / σ | 19.3 / 25.5 | 38.4 / 50 (19.2 / 25.0) | pt |
| App menu radius / right shadow | 11.1 / .109·e^(−d/12.5) | 22.3 / τ 25 (11.15 / 12.5) | pt |
| Sidebar material blur σ | 27.0 / 26.5 | 55 / 54.5 (27.5 / 27.25) | pt |
| Tones (Dock, menu, popover, sidebar) | as in section 5 | same | independent of scale |

The Dock rim at 2x is not a plain copy of the 1x profile (156 136 118 115 against
145 116 113). The Dock shrinks when 1116 does not fit (960×540 at 2x: 66 high,
reserved 72, width screen − 72); that mode was not compared.

## 7. What is public, for contrast

Apple publishes no blur, saturation, tint or refraction numbers; the only one is
the 35 % dark dimming layer for Clear glass over bright content (HIG, Materials).
Third-party reverse engineering (Groth's MaterialView, Vaidyam's backdrop write-up)
gives the chain colorSaturate, gaussianBlur, colorBrightness at backdrop scale
.25, plus a tint layer composited lighten (dark) or darken (light); its example
values (radius 30, saturation 1.8, brightness .02) are set by the authors, not
read from Apple. WWDC18 session 210 describes the wallpaper tint as the average
of the wallpaper behind the window mixed with gray, updated as the window moves;
section 4 confirms the average, the update and the mix in encoded sRGB, and gives
the ratio. KWin blurs with a dual Kawase chain (15 strengths) plus noise and a
Rec. 709 saturation matrix; mutter's `ClutterBlur` is a separable Gaussian with
σ = radius / 2.

## 8. Corrections to older documents

- `macos-settings.md` lines 204 to 207 and 435: `AppleReduceDesktopTinting` is 0
  (tinting on), not 1.
- `macos-blur.md` and `macos-liquid-glass.md` §13.6: the Dock and the Control
  Center tile blur is about 16 pt at slider 1, not 2.0 and 3.5 px. Those were read
  at slider 0; the slider's effect on these kernels is not separated from a
  change of build.
- `macos-liquid-glass.md` §14.2: Spotlight and Apps follow the slider like the app
  menu (identical tone at slider 1).
- `macos-menu-bar.md`: the bar's ink is white in light mode too.
- `macos-blur.md`: the dark Spotlight shadow at d = 1 is .53, about twice the
  earlier pill value (.263).
- `macos-motion-windows.md` rows 1 and 2: the window-open scale was not seen in
  this pass; the close fit is ω0 55 to 68, ζ 1.2 to 1.4 here against ω0 44.
- `macos-corners.md`: a circle fit of the window corner gives 15.3 here, against
  16 and 16.8 continuous; the menu shadow fit is .109 · exp(−d/12.5) against
  .129 · exp(−d/13). Both differ by fit method; the older values stay.
- Light sidebar reads 237, against 234 to 236.

## 9. Not measured

- Window shadow: main borderless, key dark panel, popover bottom, the corners.
- Dock bounce at more than 60 Hz.
- Notification Center and banners (nothing opened or appeared on this host);
  Control Center backing-panel kernel, tile radius and expanded modules; controls
  on glass over a backdrop.
- Glass kernels at slider 0 (the slider was not changed); Reduce Transparency,
  Increase Contrast and Reduce Motion as system switches.
- Mouse-driven drag, snap and live resize; textured wallpapers; more than one
  display; 960×540 at 2x.
- The menu, popover and Spotlight σ in dark from full steps (the tone flattens
  above input 192); the sidebar σ from the 0|64 step.
- Why the window open shows no scale animation, and why the menu bar ink and
  Control Center ignore the light scheme.

The probes and the small result files of this pass are in
`scripts/macos-probes/` (`shadows`, `anim`, `tint`, `light`, `scale` and the
loose display and pacing probes). The screenshots and raw captures (about
1.6 GB) are not kept. Each shell and Python probe works in its own folder. Each Swift
probe reads its work directory from the `PROBE_DIR` environment variable: the
folder of its own sources.
