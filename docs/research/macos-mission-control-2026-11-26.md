# macOS Mission Control, full screen and Split View, 2026-11-26

**Goal:** the numbers and the rules needed to recreate Mission Control (in
malaysia: *Expansion*), the full-screen Space and Split View.
**Scope:** macOS 27.2.1, Apple M4, one 2560×1440 display at scale 1 and 180 Hz,
dark mode, Dock auto-hide on, tile 59, two Desktops, wallpaper Aerial,
`expose-group-apps` and `mru-spaces` unset.
**Reader:** whoever writes the workspaces code of the compositor or sets a
token for it.
**Input:** probes written for this pass (flat-colour windows on a Desktop of
their own, `screencapture -x` read as raw bytes, ScreenCaptureKit at 180 fps,
keys and pointer posted from the same clock), Apple's documentation and the
open-source code that calls the private Spaces API.
**Output:** measured tables, the behaviour taken from sources, the proposed
token names, and what is still not measured.

Sizes are points; at this scale a point is a pixel. Tone values are encoded
sRGB device values, as in [macos-blur.md](macos-blur.md). The method and the
conventions are those of
[macos-measurements-2026-11-26.md](macos-measurements-2026-11-26.md). The
Accessibility tree does not expose Mission Control, so every number here comes
from pixels.

The proposal built on these numbers is in malaysia,
`crates/malaysia-compositor/src/workspaces/`.

**Update of 2026-11-27, for the session that builds Expansion on Linux.** A
second pass on the Mac measured the strip's line and the animations of the
strip. What changed, and what the compositor may need to follow:

- `SPACES_STRIP_LINE` is now black at alpha .224 (`#00000039`); white was a
  placeholder. macOS draws it on the first row below the strip's body (y 40
  collapsed, 146 expanded) and the shadow starts on the row after it;
  `effects/strip.rs` draws it on the strip's own last row (section 2).
- Adding and removing a Desktop are animated, on the strip's spring
  (`SPACES_STRIP_EXPAND_*`, ω0 16, ζ 1), so no new token is needed: the other
  thumbnails move half a pitch to stay centred, a new one slides in from the
  right edge at full size and full opacity, a removed one vanishes in one
  frame, labels move with their thumbnails and the "+" stays (section 5). The
  probe rested the pointer on "+" for 300 ms before the click, and the new
  thumbnail starts with its centre at the display's edge, which fits the
  docking of the preview that `SPACES_ADD_PREVIEW_REACH` stands for; that
  preview itself is still not measured.
- The close button shows about 1.0 s after the pointer enters a thumbnail, not
  within 300 ms, with no fade, and dims to 229 while pressed (section 2).
- A click on another Desktop's thumbnail closes Mission Control into it, with
  no slide of the row (section 5, low confidence).
- Control+Up, a third pass. The strip comes in as one rigid piece from −41 to
  0: its labels, pill and "+" ride with it with no fade, and its shadow is
  there from the first frame. On exit the labels go before the pill and the
  "+" because they sit lower, and the line and the shadow go on the first
  frame. The windows all start on one frame, 14 to 28 ms before the strip,
  with their corners and shadows scaled on every frame, no dim and no title.
  The empty Desktop's pill fades in on the windows' spring and goes in one
  frame. The Dock has a spring now (section 5).
- Hover and drag, a fourth pass (sections 4 and 5). The ring keeps its width
  at every scale and its radius is the window's plus its offset; the title
  shows one pointer event after the ring, at a fixed size. A window held in
  Mission Control expands the strip at once, jumps to its slot size and
  shrinks linearly with the pointer's y down to a scale of about .074 at the
  strip's bottom edge, keeping its grab point. Over a full-screen Space's
  thumbnail the thumbnail splits at once into the app's share and a "+"
  placeholder with the dragged window's share of the width, on the side of
  the pointer; that is the preview of the Split View the drop makes.

## 1. Window layout

116 arrangements of 1 to 9 probe windows, 312 window rows, in
`scripts/macos-probes/mission-control/layout-results.txt` (original frame,
thumbnail box, scale), measured in two passes. Reruns matched pixel for
pixel: the layout is deterministic.

| Rule | Value | Confidence |
|---|---|---|
| Scale | One scale for every window of the view, never above 1. A small window is never enlarged. | high |
| A window that fits | Stays where it is, at scale 1 (`n1a`, `n1tiny`, `tiny4`). | high |
| Area, two or more windows | x 20 to 2540, y 80 to 1341: 20 from each side, 40 below the strip (which is 40 high), 15 above the Dock drawn in Mission Control (y 1356 to 1434). The menu bar does not count: the strip replaces it. | high |
| Area, strip expanded | The top moves to about 184 (strip 146 plus about 38) and the layout is computed again (`n4`: scale .928 to .847). The bottom stays. | high |
| Area, one window | y about 97 to 1324. A window that must shrink is capped at a height of about 1220 and centred at (1280, 711): `n1full` .866, `n1big` .911, `n1tall` .867. | medium |
| Least movement | A window outside the area is moved inside without scaling. Windows that do not overlap keep their place and their gaps. | high |
| Overlap | An overlapping pair is pushed apart along the line between the two centres until it is clear on one axis, whichever comes first. Both windows move the same distance whatever their sizes, so the pair's centroid stays (`u_nest`: a 600×400 and a 300×200 window both move 157.5). Offsets of (100, 50), (50, 100), (200, 20), (20, 200) and (300, 300) all end on the original line, to 1. | high |
| Same centre | The pair splits at exactly 45°: the window behind goes up and left, the one in front down and right. The order is the stacking order, not the order of creation (`z1`, `z3`). | high |
| Only what overlaps | Windows that overlap nothing do not move (`n4one`). A pair closer than 12 is pushed (`bh605`); one 12 apart is left (`bh612`). | high |
| Gap | 12 to 15 between the edges of a pair that was pushed, at every scale. It varies with the window size as a sawtooth of about 2.5 with a period of about 20, the trace of an iteration and not a constant. Cascades leave more, 30 to 116. | high |
| Walls | A window that is only moved inside stops exactly at the area. One driven into a wall by a push ends 2 past it sideways and 3 to 4 past it up or down. | high |
| Scale | It drops below 1 only when a window is held by two walls at once. A pair in a corner is scaled although there is room: .921 with no offset, .943, .963, .979, .993 and 1 as the pair starts 40, 80, 120, 160 and 200 from the corner (`k_*`). A window held by one wall slides along it and nothing is scaled. "The largest scale that packs" is ruled out. | high |
| Apps | Windows of two apps are placed like windows of one (`g1`, `g2`); `expose-group-apps` is off by default. | high |

Example scales: four windows that nearly fill the screen .928, a cascade of
four 1200×800 windows .770, of six .574, of nine .434.

The fitted algorithm is in section 8.

## 2. Spaces strip

Measured over a flat backdrop window that stays under Mission Control.

| Part | Collapsed | Expanded |
|---|---|---|
| Height | 40 | 146 |
| Trigger | shown on entry | the pointer enters the strip (y < 40; not at y 60); it then stays expanded until Mission Control closes |
| Desktop entry | label only, white 255, ink 70×11 for "Escritorio 1" (about 14 pt regular; low confidence), in a cell 95 wide; cells touch and the row is centred | thumbnail 160×90 (1/16 of the display), top at y 29, 32 apart, row centred; label under it, ink 7 below the thumbnail, white 254, about 12 pt (low confidence) |
| Current Desktop | pill 95×24 at y 7 to 30, radius about 8, 35 levels lighter than the strip on every backdrop (53 on black, 130 on gray 128, 204 on white) | ring 3 wide, 1 outside the thumbnail, sRGB (58, 154, 252), opaque, outer radius about 4 (low confidence); thumbnail corners about square |
| Add button | "+" with arms of 20 and a stroke of 2, gray 229, centre 26.5 from the right edge, y 19.5 | centre moves to y 77.5 |
| Close button | none | on hover of any thumbnail, the current one too: a white circle of 22 with a gray (152) cross, centred on the thumbnail's top-left corner. It shows in one frame, with no fade, about 1.0 s (1014 to 1021 ms, 4 runs) after the pointer first moves inside the thumbnail, whether the pointer stops at 650 or 990 ms; a hover of 15 px in 450 ms never showed it. It hides in one frame within 12 ms of leaving. Pressed, the circle dims from 255 to 229 |

Material, the same in both states: output 18, 95 and 169 over gray 0, 128 and
255, which is a gray of 44 at alpha .41 over the blurred backdrop; blur σ 27.5
(error-function fit on a 0|255 step, rms .58).

The line under the strip is black at alpha .224, one row high, on the first row
below the body: y 40 collapsed, y 146 expanded, the same in both states. It
darkens the backdrop by the same factor on every backdrop and every channel,
so it is a line of fixed colour and not a row that lets the strip through.
Raw device values at x 1500 and x 2200 (both columns equal), body, line and the
rows below:

| Backdrop | Body | Line | Below (41, 42, 43) | Backdrop × .776 |
|---|---|---|---|---|
| 0, 0, 0 | 18 | 0 | 0, 0, 0 | 0 |
| 128 gray | 95 | 99 | 111, 112, 112 | 99.3 |
| 255 gray | 169 | 198 | 221, 223, 224 | 197.9 |
| 59, 28, 242 | 57, 27, 169 | 46, 22, 188 | 51, 24, 210 | 45.8, 21.7, 187.8 |

A row that let the strip through would mix body and backdrop with one
coverage on every backdrop; the coverage it would need differs on every
backdrop, and the red channel of the colour backdrop (46) is below both the
body (57) and the row under it (51), which no mix gives. A light line would
show over black, and it does not.

Below the line a shadow: alpha .133 on the first row, zero at about 44,
alpha(d) = .137 · (1 − d/48)^1.8 (rms .0010).

## 3. Backdrop and chrome

- **Wallpaper:** identical to the Desktop, pixel for pixel. No dim, no blur,
  no scale.
- **Desktop icons** are hidden.
- **Menu bar:** replaced by the strip, which slides in from the top. The menu
  bar fades out on entry and back in on exit, over about 150 to 300 ms (low
  confidence).
- **Dock:** shown even with auto-hide on. It slides up from the bottom edge
  and down on exit (section 5). At rest its top row is 1356 (a light rim) and
  its bottom 1434, 6 above the edge.
- **Other Spaces:** their windows are shown only inside the strip thumbnails,
  which are live. An empty Desktop shows a pill with "No Available Windows"
  ("No hay ventanas disponibles" here): 216×29 at x 1172 to 1387, y 696 to
  724, with a light line on the row above (y 695), so about 30 high, centre
  1279.5, 710, a capsule of radius about 15 (medium confidence). Its fill reads
  168 over gray 128 and its text 17.

## 4. Thumbnail decoration

- A thumbnail is the window image with its own shadow, scaled together. At
  scale 1 the region is pixel-identical to the window at rest, and a window
  that is not main keeps its weaker shadow.
- The shadow's extent scales with the thumbnail and its alpha does not: alpha
  .02 is reached at 39 at rest, 37 at .93 and 17 at .43.
- The corner radius scales the same way (the corner inset ends at row 11 at
  rest, 10 at .93, about 5 at .43; medium confidence).
- **Hover ring:** sRGB (58, 154, 252), starting right outside the window's
  outline, no gap. It has no fade: one frame, 17 to 51 ms after the pointer
  moves into the window. It is 4 full pixels plus anti-aliased edges (summed
  coverage 4.7 to 5.0) and its outer edge is 7 outside the window's content,
  the same at scales 1, .90 and .46: the width does not scale. Its outer
  radius is the window's plus 7.5 to 7.75 (24.0, 20.75, 15.25 against 16.5,
  13.25, 7.5; medium confidence), so it is concentric with the window.
- **Hover title:** a capsule centred on the thumbnail, not below it: 76 to
  79×29 to 30 for "Probe 4", the same size at scales 1, .90 and .46, black
  text of about 14 pt medium (low confidence), 14 of side padding, a light
  material (251, 251, 89 over yellow). No app icon. It does not come with the
  ring: it shows 8 to 13 ms after the next pointer event inside the window
  (a move of zero distance is enough, at any gap from 10 to 730 ms), in one
  frame with no fade; after a single move and 1.6 s of stillness it never
  showed. Ring and title leave together, 16 to 23 ms after the pointer leaves.
  6 runs, high confidence.
- **While Mission Control opens**, with the pointer still: nothing shows
  until the windows settle; ring and title then appear together, 56 ms after
  the last window pixel changed (1 run). With the pointer moving: the ring
  shows from the first frame of the motion, scaled with the window (about 10
  wide and 14 out near scale 1, 5 and 7 at the end) and following it on every
  frame; the title shows at full size and opacity but placed at the window's
  top-left plus its final centre offset, and is centred only once the window
  settles (3 runs).

## 5. Animations

180 fps captures. N = 1, 4 and 9 windows, three runs each; fits with rms under
.02 only.

| Animation | Result |
|---|---|
| Entry and exit | Centre x, centre y, width and height of every window follow one critically damped spring: ω0 24 to 28 (median 25 to 26), ζ .97 to 1.07. t10 8 to 19, t50 53 to 69, t90 142 to 158 ms, last change 192 to 297 ms by travel. Exit is the same spring. Key to first changed frame 29 to 42 ms. |
| Pick (click on a thumbnail) | Fires on button release, motion starts 28 to 36 ms later. Same spring (ω0 25, ζ 1.00). The picked window is on top from the first frame. |
| Strip entry, 0 to 40 | t50 47 to 58, t90 136 to 142, last 169 to 175 ms; ω0 29 to 33, ζ 1.1 to 1.2. The whole strip moves from −41 to 0 as one piece. Its shadow is drawn from the first frame, under the bar still off screen. The pill and the "+" keep their rest offset from the strip's lower edge (10 above it) and the label ink 15 to 16, on every frame, with no fade (pill 130 over gray 128 and "+" 229 on their first visible row). They show 22 to 30 ms (pill, "+") and 33 to 40 ms (label) after the strip's first frame, only because they enter from above. 4 entries, high confidence. An earlier note that the content fills from the top was wrong. |
| Strip content on exit | Rigid with the strip, no fade. The labels leave first (91 to 101 ms from the key), then the pill and the "+" together (112 to 118), only because they sit lower. The strip is gone at 175 to 206 ms. The line and the shadow go on the first exit frame. Escape and Control+Up give the same timing. 6 exits, high confidence. |
| Windows during entry and exit | All windows change on the same frame (6 windows, unbroken 5.6 ms frames): 14 to 28 ms before the strip on entry, on the strip's frame on exit. The corner radius and the shadow scale with the window on every frame: corner insets of rows 1 to 4 go 15.0, 10.6, 8.4, 6.9 at rest to 5.5, 3.4, 2.2, 1.6 at .49, and the shadow's reach of 13 to 6 or 7, its alpha at d 2 from .148 to about .10, as the scaled profile gives. No dim (window content stays exactly 255 and 0), no title, no ring, the stacking order kept. Last pixel change 403 to 545 ms for travels up to about 800. High confidence, stacking medium. |
| Dock | Entry starts 0 to 11 ms after the strip's first frame (48 to 117 ms from the key): ω0 18 to 24 (median 19), ζ 1.10 to 1.32 (median 1.11), t50 75 to 92, t90 214 to 220, last 370 to 387 ms from the first change, 5 runs. Exit starts 5 to 17 ms after the strip (34 to 73 from the key): ω0 15 to 16, ζ .82 to .87, t50 75 to 92, t90 192 to 203, last 248 to 264 ms, 7 runs. Fit rms .009 to .014, the size of one row of the 84 travel: medium confidence. Bezier curves fit as well but scatter between runs. |
| "No Available Windows" pill | Entry: fades in at full size and in place from the windows' first frame (5 to 11 ms before the strip), its alpha on the windows' spring: ω0 26, ζ 1.00 to 1.02, t50 53 to 58, t90 136 to 142, last 242 to 253 ms (3 runs; one outlier at ω0 35, ζ 1.25). Exit: gone in one frame, on the strip's first exit frame (4 runs). High confidence. |
| First open after idle | 50 to 60 ms later than a warm open: windows start 81 to 88 ms after the key instead of 24 to 35. |
| Drag a window, start | Probe windows of 2400×1350 at scales .465 and .903. The strip expands on the first drag frame (43 ms after the first drag event), with the pointer still near y 840, on its hover spring (t50 about 90 ms), and the other windows lay out again for it at the same moment. There is no drag with the strip collapsed. The held window jumps in one frame to its slot size in the new layout, keeping its top-left corner. 4 runs, high confidence. |
| Drag a window, scale | From then on the pointer keeps its fractional place in the window (fx .223 to .226, fy .209 to .217 in one run; .203 to .206, .887 to .888 in another). The width is linear in the pointer's y, from the slot size at the drag's start y to a scale of about .074 at y 146, the strip's bottom edge, and constant above it: 1020 wide from y 840 gives 972, 728, 486, 242 and 178 at y 800, 600, 400, 200 and 146 (slope 1.2146, predicted 1.2147); 1980 from y 1090 gives 1906, 1236, 472, 282 at 1050, 700, 300, 200. At and inside the strip a 2400×1350 window is 177×100 and a 1600×900 one 117 to 118×66: a scale floor, not a box, 1.11 times a thumbnail's width. A pure function of y: on a drag of 550 in about 80 ms the size follows within a frame and holds still. 3 runs, high confidence. |
| Drag a window, release | Over the background: it returns to its slot in the expanded-strip layout and Mission Control stays open; size and y on ω0 16 to 17, ζ .98 to 1.04, first change 18 to 26 ms after mouse-up (2 runs; centre x ω0 22 to 25, low confidence). On another Desktop's thumbnail: the thumbnail does not highlight while hovered; on release the window flies to its miniature in that thumbnail (its frame divided by 16) on ω0 16.5 to 17, ζ .96 to 1.12, from 20 to 22 ms, done at about 430 ms; its old miniature leaves the source thumbnail in 3 frames at 472 to 494 ms (1 run, medium confidence). |
| Drag onto a full-screen Space's thumbnail, preview | The thumbnail splits when the pointer enters the strip in that thumbnail's column, already at the strip's bottom edge (y 142 to 146), below the thumbnail. The pointer's x on the thumbnail chooses the side: left half puts the placeholder on the left, right half (centre included) on the right; crossing the centre switches it in one frame, 28 ms after the move. Split and un-split take one frame each, with no animation. In a 160×89 thumbnail the app keeps 60 and the placeholder takes 100, with no divider: (2560 − 1600) to 1600, the dragged window's share of the width, as in the Split View the drop then makes (956, 12, 1592). The placeholder is a gray gradient from 70, 70, 86 on the left to 78, 78, 99 on the right with a white "+" at its centre. The label turns at once into both names ("<app> y <app>"). The dragged window does not change over it: it stays at the floor scale and its grab point, above the thumbnail with its shadow. Leaving without a drop restores the thumbnail in one frame 22 to 44 ms later; the label returns at once while a ghost of the joined label fades out from 66 to 255 ms (τ about 60 ms). High confidence for the preview, medium for leaving (1 run). |
| Drag onto a full-screen Space's thumbnail, drop | One run, medium confidence. The window flies into the placeholder and takes its shape, 118×64 to 99×86 by 375 ms: centre ω0 17, ζ 1.02; width ω0 15, ζ .82; height ω0 20, ζ 1.58, from 22 to 28 ms after mouse-up. From 386 to 470 ms the thumbnail turns into the live Space (the placeholder flickers back for a few frames while the real tiles resize); at 475 ms it is the app 60, a black divider 2, the window 98. The window's old miniature fades out of its Desktop's thumbnail from 425 to about 770 ms (τ about 65 ms). The strip and the other thumbnails do not move, and Mission Control stays open. |
| Strip expansion, 40 to 146 | Starts 48 to 75 ms after the pointer enters. ω0 16, ζ 1.00 (rms .003, high confidence): t50 92, t90 225 to 236, last 353 to 386 ms. |
| Strip exit, 146 to 0 | t50 64 to 69, t90 142 to 147, last 192 to 197 ms; ω0 22 to 24, ζ .85 to .91. |
| Add a Desktop ("+", strip expanded), 1→2, 2→3, 3→4 | One run each. The existing thumbnails and their labels move 96 left (half the pitch of 192, so the row stays centred) on the strip's spring: ω0 16 to 17, ζ .99 to 1.03, t50 92 to 97, t90 225 to 231, last 420 to 437 ms. Mouse-up to first change 36 to 48 ms; the fitted curve starts 6 to 14 ms earlier. The new thumbnail slides in from the right at full size and full opacity on the same spring, its left edge starting at about x 2480 (its centre at the display's edge), and passes over the "+" for about 30 ms. Its label moves with it and does not fade in. The "+" does not move. |
| Add a Desktop, strip still collapsed | Pointer at the "+" at 0, press at 10, release at 45 ms. A Desktop is added and the strip grows anyway, because the pointer stays inside it: 39 to 145 from 40 ms after the move, ω0 20, ζ 1.13, t50 92, t90 237 (one run, medium confidence). The "+" goes down with the strip; the thumbnails grow and shift at once, so their shift was not fitted alone. |
| Remove a Desktop (close button), 2→1, 5→4, 4→3 | Three runs, always the last Desktop. The removed thumbnail and its label vanish in one frame 14 to 31 ms after mouse-up, with no shrink, fade or slide. The others move 96 to recentre with their labels: ω0 16, ζ .99 to 1.00, t50 92 to 97, t90 225 to 231, last 376 to 381 ms, first change 34 to 45 ms after mouse-up. |
| Click on another Desktop's thumbnail | Two runs, from line captures only (low confidence). No row slide: Mission Control closes into that Desktop. The strip goes up from about 90 to 200 ms; all is still at 250 to 300 ms; first change 35 to 55 ms after mouse-up. |
| Control+Left and Right inside Mission Control | The thumbnails of the area slide sideways with the Space switch curve (t10 22 to 29, t50 98 to 109, t90 304 to 320 ms). The strip stays. |
| App Exposé (Control+Down), still only | No strip. Four windows in a strict 2×2 grid at .768, each with its title always shown below it in white. |

## 6. Settings read on this Mac

Read only; nothing was written.

- `com.apple.dock`: `wvous-tl-corner` 2 (Mission Control), `autohide` 1,
  `tilesize` 59, `magnification` 1, `largesize` 128. Unset, so the defaults
  apply: `mru-spaces`, `expose-group-apps`, `expose-animation-duration`,
  `workspaces-auto-swoosh`, the `show*GestureEnabled` keys.
- Trackpad: the three- and four-finger swipes and the pinch are 2 (on).
- `com.apple.spaces`: `SpacesDisplayConfiguration` → `Management Data` →
  `Monitors[]`, each with `Current Space`, `Spaces[]` (type 0 is a Desktop)
  and `Display Identifier`. The `Current Space` in the file lags behind the
  screen.

## 7. Full screen and Split View

Probes in `scripts/macos-probes/full-screen/`. The order of the Spaces was
read from the private call behind `SLSCopyManagedDisplaySpaces` (`sp.swift`,
read only). Captures at 180 fps had single dropped frames in 13 to 15 % of the
intervals and no longer gap inside a transition. The enter and exit times of
[fullscreen-titlebars.md](fullscreen-titlebars.md) (about 550 ms) stand.

### The Space

| Case | Result | Runs |
|---|---|---|
| Enter from Desktop 2 | order Desktop 1, Desktop 2, full screen | 3 |
| Then enter another from Desktop 1 | order Desktop 1, full screen b, Desktop 2, full screen a | 1 |
| Exit while its Space is current | back on the Desktop it came from, at its exact frame; the Space is removed and that Desktop becomes current | 3 |
| Exit while another Desktop is current | the window lands on Desktop 1, also when it came from Desktop 2; frame kept, no switch, no animation (29 ms) | 2 |
| Close the window while its Space is current | the Space is removed and the Desktop to its left, which was also its origin, becomes current about 530 ms later | 1 |
| Close while another Space is current | the Space is removed, the current Space stays | 1 |

The new Space goes right after the Desktop the window came from, not after the
last Desktop. That Desktop was also the current one in every run, so "after
its origin" and "after the current Desktop" are not told apart. In the strip
its label is the name of the app, not the title of the window, and its
thumbnail is the window's content alone. With "rearrange Spaces by most recent
use" at its default (on), a full-screen Space moved once to sit after the
Desktop that was used last.

### The transition

Three runs each way, from the call to `toggleFullScreen`.

| | Enter | Exit |
|---|---|---|
| First changed frame | 60 to 87 ms | 55 to 62 ms |
| Active Space changes | 546 to 553 ms | 540 ms |
| Done (`didEnter`, `didExit`) | 574 to 579 ms | 546 to 560 ms |
| Window width, from its first change | t10 36 to 42, t50 97, t90 258 to 264, still at 442 ms | t10 36 to 42, t50 92 to 97, t90 258 to 264, still at 419 to 425 ms |
| Spring on the width | ω0 22 to 26, ζ 1.25 to 1.44 (rms .01) | ω0 25 to 26, ζ 1.39 to 1.44 (rms .009) |

- The window grows in place, from its frame to the display, while the Spaces
  slide. It does not travel with its Space, and it keeps its shadow and its
  title bar while it grows. Its left and top edges run a little ahead of its
  width (t50 69 to 92 ms).
- On enter the Desktop left behind slides to the left with its wallpaper, its
  menu bar and its other windows, and what comes in from the right is black.
  Displacement: 0 at 73 ms, 76 at 100, 304 at 150, 628 at 200, 1012 at 250,
  1420 at 300. The growing window then covers it, so the end of the slide is
  not measured (low confidence).
- On exit a black gap of about 60 (consistent with the 64 between Spaces)
  moves to the right: its left edge is at 1320 at 100 ms, 1608 at 200, 2004 at
  300, 2368 at 400 and 2556 at 500. From 100 ms on it is close to a
  cubic-bezier (0.2, 0, 0.8, 1) over about 380 ms (rms 12.8). It is not the
  2624 spring of the Space switch started at the call (medium confidence on
  the shape, low on the start).
- Control+Left and Right between a Desktop and a full-screen Space is the
  same slide as between Desktops: travel 2624 to 2640, ω0 21 to 23.5, ζ 1.60
  to 1.75, t50 109 to 113, t90 327 to 334 ms (2 runs each way, high
  confidence).

### Inside the Space

- A full-size window is 0, 0, 2560, 1440 with square corners and no shadow.
- A window with `maxFullScreenContentSize` 1600×900 is centred exactly
  (480, 270) on pure black, with square corners and no shadow.
- **Menu bar reveal:** the pointer at y 0 to 4 shows it; y 5 does not. After
  175 to 183 ms it slides 62 (the menu bar, 30, and the title strip, 32) at a
  near constant speed in 97 to 108 ms. It hides when the pointer reaches y 98,
  after 45 to 56 ms, in 103 to 114 ms (2 runs each, medium confidence).
- **Dock:** it shows at the bottom edge when the pointer moves onto it.

### Split View

Made with Window > Full Screen Tile > Left of Screen on one probe, then a
click on the other.

| Part | Value |
|---|---|
| Tiles at start | left 0 to 1278, divider 12, right 1290 to 2560 (2 runs) |
| Divider | 12 wide, pure black. Handle 4×64 with round ends, gray 217 when the left app is active and 140 when the right one is |
| Margins and corners | none; square |
| Smallest tile | 87 on either side (2 runs a side) |
| Snapping | none; the width ends within 15 of where the divider is dropped |
| Double-click on the divider | equal halves: 1274, 12, 1274 (2 runs) |
| Label in the strip | both app names, left one first, joined by "and" in the system language; the thumbnail shows both and the divider |
| One window leaves full screen | it returns to its Desktop and frame and that Desktop becomes current; the other stays full screen alone, 2560×1440 |
| One window closes | the other stays full screen alone in the same Space, which stays current, and takes the focus |
| Focus | the clicked tile's app is active. The reveal at the top shows both title strips; only the active tile's buttons are coloured |

### A new window while a full-screen Space is current

| Who opens it | Where it goes |
|---|---|
| The same app, tabs allowed | a tab of the full-screen window (a tab bar of 36) |
| The same app, tabs off | in the full-screen Space, floating at its own frame; no switch |
| Another running app | on Desktop 1; no switch |
| An app just launched | on Desktop 1, and the system switches there |

## 8. The layout as an algorithm

The rule for a few overlapping windows at scale 1 is found: it predicted
fresh arrangements to 1 to 3 before they were measured. The rule for the
scale and for dense arrangements is not. The best algorithm,
`scripts/macos-probes/mission-control/layoutfit2.py`, is within 3 on 78 of
the 116 arrangements; `layoutfit2-results.txt` lists every one. The first
attempt, `layoutfit.py`, is kept for comparison.

The area is that of section 1. Windows are taken back to front.

1. **One window:** s = min(1, 2520/w, 1220/h). Scale it about its own centre,
   then move it inside the one-window area; if it is still larger, centre it.
2. **Start:** s = min(1, area width / widest window, area height / tallest
   window). Each window gets the size s·w by s·h about its own centre, is
   moved inside the area, and then window i is moved by .25·i on both axes,
   which breaks ties in stacking order.
3. **Sweep:** for every pair i before j that is closer than 12.5 on both axes,
   take the unit vector u from the centre of i to the centre of j, and let t
   be the smallest, over the axes where u is not zero, of ((size i + size
   j)/2 + 15.5 − distance on that axis) / |u on that axis|. Window i collects
   −.375·t·u and window j +.375·t·u. Every pair is read from the positions at
   the start of the sweep. Then each window moves by the mean of what it
   collected and is kept inside the area grown by 2 sideways and 3 up and
   down.
4. **Scale:** if a window was stopped on both axes in that sweep, s becomes
   .995·s, with the sizes about the centres.
5. **Stop** at the first sweep with no pair closer than 12.5, or after 400.

| Largest edge error | Arrangements |
|---|---|
| 3 or less | 78: every window alone, every arrangement without overlap, every free pair (`a2_600`, `a2_900`, `bh*`, `bv*`, `d_*`, `s1_*`, `u_*`, `z1`, `g1`), three windows (`t3`, `t3c`, `t3cr`), pairs on one wall (`e3_*`, `c_left`, `c_top`), `n2same`, `n4o20`, `n4one` |
| 4 to 10 | 3: `n4sym`, `k_200`, `ky_1` |
| 11 to 33 | 13: the pairs in a corner (`k_0` to `k_160`, `c_tl`), `n2stack` (scale .990 against .996), `a2_1250`, `kx_1`, four and five windows on one frame (`t4`, `t4c`, `t5`) |
| 34 to 100 | 7: `n3` (the first attempt was 6 off), `c_br`, `hc4`, `g2`, `mixed` |
| over 100 | 15: `n4` and its mirrors (200), the cascades (`n4cascade` 107, `n6cascade` 701, `n9cascade` 1192) |

Predictions made before measuring:

| Arrangement | Predicted | Measured | Error |
|---|---|---|---|
| `p1`, two unequal windows, diagonal offset | (621, 143), (979, 657), scale 1 | (621, 143), (979, 658), scale 1 | 1 |
| `p2`, a chain of three | (287, 468), (800, 520), (1314, 572) | (287, 469), (800, 520), (1314, 572) | 1 |
| `p3`, a pair on the left wall | (18, 119), (411, 532) | (16, 118), (412, 533) | 3 |
| `p4`, a pair on one frame near a corner | scale .980 | scale .995 | 12 |
| `p5`, two overlapping pairs | scale .951 | scale .985 | 85 |

Where it fails:

- **The scale.** The .995 per sweep is a stand-in. Measured, 1 − s grows as
  about the 1.6 power of the travel the cornered window lost, which neither a
  constant per sweep nor a shrink in proportion to the lost travel gives.
- **The step.** No single value fits the four pairs on a wall: they ask for
  .60, .70, .77 and .80 of the needed separation per sweep.
- **Three or more windows against a wall:** 25 to 54 off.
- **`n4`:** two rows that cannot separate up and down. The sideways part of
  the tie grows with each sweep; in the symmetric case the rows shear, the top
  one left and the bottom one right by 117 each, and in the others the
  stacking order decides which row packs.
- **Cascades:** hundreds off.

Ruled out by the data: slots or a grid; a push along the axis of least
overlap; a split weighted by size; the order of creation; grouping by app; a
final fit of the bounding box; positions scaled about a common point; the
largest scale that packs.

New rows were read at a colour tolerance of 40, because the shadow of the
front window clips the fill of the one behind, and stored in the convention
of the first pass. Positions are good to 1 and the scale to about .001.

## 9. Behaviour, from sources

A is documented by Apple, O is seen in open-source code that calls the private
API, F is a forum or press hint.

1. **Enter:** swipe up with three or four fingers, the Mission Control key or
   Control+Up, a hot corner, a double tap with two fingers on a Magic Mouse,
   and, as an option, dragging a window to the top of the screen (A [1], [5]).
2. **Leave:** the same key or gesture, Escape, Control+Down (F [9]); a click
   on the empty background (A [1]); a click on a window, which comes to the
   front (A [1]); a click on a Space, which becomes current. Option-click on a
   Space switches and stays in Mission Control (F [9], [10]).
3. **Which windows:** those of the current Space of the display where it was
   opened (A [1]). Minimized windows and hidden apps are left out (F [12]).
   Windows marked `.transient` hide and `.stationary` ones stay in place
   (A [7]).
4. **Space bar on a hovered window** enlarges it and Space again shrinks it
   (F [9], [13]).
5. **Spaces strip:** Desktops, full-screen apps and Split View pairs (A [1]).
   Collapsed to names since OS X 10.11; a hot corner at the top opens it
   expanded (F [14], [15]). Desktops are named by position. The "+" is at the
   top right, or top left with the Dock on the right (F [10]). A new Desktop
   goes after the Desktops and before the full-screen Spaces (F [15]). The
   limit is 16 (A [3]).
6. **Close a Desktop:** hover and click its close button, or hold Option to
   show them all (A [3], F [10]). Its windows move to another Space (A [3]),
   by a press report the one before (F [9]). The last Desktop of a display
   cannot be removed (O [6]).
7. **Drag:** a window onto a Space thumbnail moves it there (A [3]); onto "+"
   makes a Desktop for it (F [9], [10]); onto a full-screen thumbnail makes a
   Split View (A [1]). Thumbnails are reordered by dragging (F [9]). Outside
   Mission Control, a window held against a screen edge goes to the next
   Space (A [3]).
8. **Switch inside Mission Control:** Control+Left and Right and Control+digit,
   with no wrap (F [10]).
9. **Settings** (A [5]): rearrange Spaces by most recent use (on by default),
   switch to a Space with the app's windows when the app is activated, group
   windows by application, displays have separate Spaces, drag to the top of
   the screen. Dock "Assign To": all Desktops, this Desktop or none (A [3]).
10. **App Exposé:** Control+Down. The windows of the front app, minimized ones
    too (A [1], F [12]). **Show Desktop:** Command+Mission Control or the
    spread gesture (A [1]).
11. **Gesture:** Apple does not document whether the swipe follows the
    fingers; one forum post says it does (F [17]). gnome-shell, read for
    behaviour only: progress is the distance over 300 (400 sideways), a
    release slower than .6 goes to the nearest end, a faster one is projected,
    and the animation takes 100 to 400 ms [18].
12. **Accessibility:** Reduce Motion turns the zoom and the Space slide into
    fades (F [20]; Apple only says it reduces motion when switching Desktops,
    A [21]).
13. **Several displays:** each display shows its own windows and Spaces
    (A [1]) and, with "Displays have separate Spaces", has its own list and
    its own current Space (O [4]).

## 10. How macOS builds it

- **Owner:** Dock.app. It holds the one window server connection allowed to
  change other apps' windows and Spaces (O [11]).
- **Windows are transformed, not copied:** the private calls
  `SLSSetWindowTransform` and `SLSTransactionSetWindowTransform` let the
  window server scale the real windows, and `SLSHWCaptureWindowList` gives
  snapshots (O [4]). The pixels agree: a thumbnail carries the window's own
  shadow and corners, scaled.
- **Spaces:** `SLSCopyManagedDisplaySpaces` lists them per display and
  `SLSSpaceGetType` gives 0 for a Desktop, 2 for a system Space and 4 for a
  full-screen one; a Split View pair has the full-screen type (O [4], [6]).
- **No public API:** Apple calls `com.apple.spaces` an implementation detail
  [24].
- **Published numbers:** none for sizes. The only published timing is the
  `expose-animation-duration` key, which a forum says is ignored since 10.11
  (F [27]).

Defaults keys, from [25], [26] and [28]:

| Key | Meaning | Default |
|---|---|---|
| `com.apple.dock mru-spaces` | rearrange Spaces by most recent use | true |
| `com.apple.dock expose-group-apps` | group windows by application | false |
| `NSGlobalDomain AppleSpacesSwitchOnActivate` | switch to the app's Space | true |
| `com.apple.spaces spans-displays` | Spaces span all displays | false |
| `com.apple.dock wvous-{tl,tr,bl,br}-corner` | hot corner: 1 off, 2 Mission Control, 3 app windows, 4 Desktop | — |
| `com.apple.dock showMissionControlGestureEnabled` and the two like it | gesture switches | — |

## 11. Proposed tokens

In `ctdesign` now: `EXPANSION_OMEGA`, `EXPANSION_DAMPING`, `EXPANSION_MARGIN_SIDE`, `EXPANSION_MARGIN_TOP`, `EXPANSION_MARGIN_BOTTOM`, `EXPANSION_HOVER_RING_WIDTH` and `SPACES_STRIP_HEIGHT`, and the Spaces strip's `SPACES_STRIP_EXPANDED_HEIGHT`, `SPACES_STRIP_EXPAND_OMEGA`, `SPACES_STRIP_EXPAND_DAMPING`, `SPACES_STRIP_TINT`, `SPACES_STRIP_LINE`, `SPACES_STRIP_SHADOW`, `SPACES_THUMBNAIL_HEIGHT`, `SPACES_THUMBNAIL_TOP`, `SPACES_THUMBNAIL_GAP`, `SPACES_LABEL_*`, `SPACES_EXPANDED_LABEL_FONT`, `EXPANSION_TITLE_*`, `SPACES_RING_*`, `SPACES_ADD_*`, `SPACES_CLOSE_*` (with `SPACES_CLOSE_DELAY_MS` and `SPACES_CLOSE_PRESSED_COLOR`), `EXPANSION_CAPTION_COLOR` and `EXPANSION_GRID_GAP`; the rest are not. Each is added by the step of the compositor that
first reads it, as `DESKTOP_SWITCH_*` was.

The times of sections 5 and 7 count from the first frame that changes. A
spring started from rest with the values below is half way and at 90 % at
67 and 156 ms (25, 1), 105 and 243 ms (16, 1) and 84 and 238 ms (25, 1.4).

| Token | Value | From |
|---|---|---|
| `EXPANSION_OMEGA`, `EXPANSION_DAMPING` | 25, 1 | section 5: windows in, out and on a pick |
| `SPACES_STRIP_EXPAND_OMEGA` | 16 (ζ 1) | section 5 |
| `EXPANSION_MARGIN_SIDE`, `EXPANSION_MARGIN_TOP`, `EXPANSION_MARGIN_BOTTOM` | 20, 40 below the strip, 15 above the Dock | section 1 |
| `EXPANSION_GAP` | 13 | section 1 |
| `SPACES_STRIP_HEIGHT`, `SPACES_STRIP_EXPANDED_HEIGHT` | 40, 146 | section 2 |
| `SPACES_THUMBNAIL_DIVISOR`, `SPACES_THUMBNAIL_TOP`, `SPACES_THUMBNAIL_GAP` | 16, 29, 32 | section 2 |
| `SPACES_LABEL_CELL`, `SPACES_LABEL_PILL_HEIGHT`, `SPACES_LABEL_PILL_RADIUS`, `SPACES_LABEL_PILL_LIFT` | 95, 24, 8, 35 levels | section 2 |
| `SPACES_RING_WIDTH`, `SPACES_RING_GAP`, `EXPANSION_HOVER_RING_WIDTH` | 3, 1, 4; the colour is the accent | sections 2 and 4 |
| `SPACES_ADD_SIZE`, `SPACES_ADD_STROKE`, `SPACES_ADD_INSET` | 20, 2, 26.5 to its centre | section 2 |
| `SPACES_CLOSE_SIZE` | 22 | section 2 |
| `SPACES_STRIP_BLUR`, `SPACES_STRIP_SATURATION`, `SPACES_STRIP_TINT`, `SPACES_STRIP_CAP`, `SPACES_STRIP_LINE` | σ 27.5, saturation 1.6, gray 46 at .40, a cap of 169, a black line of alpha .224 on the row below the body (dark). The veil with the cap is the fit of [macos-blur.md](macos-blur.md) over 21 samples; it gives 18, 95 and 169 over the three grays of section 2, where gray 44 at .41 gives 18, 94 and 169 | section 2 |
| `SPACES_STRIP_SHADOW` | alpha .137, reach 48, power 1.8 | section 2 |
| `FULL_SCREEN_OMEGA`, `FULL_SCREEN_DAMPING` | 25, 1.4: the window growing to the display and back | section 7 |
| `FULL_SCREEN_REVEAL_EDGE`, `FULL_SCREEN_REVEAL_DELAY_MS`, `FULL_SCREEN_REVEAL_MS`, `FULL_SCREEN_HIDE_BELOW` | 4, 180, 100, 36 below the revealed bar | section 7 |
| `SPLIT_DIVIDER_WIDTH`, `SPLIT_HANDLE_WIDTH`, `SPLIT_HANDLE_HEIGHT`, `SPLIT_MIN_TILE` | 12, 4, 64, 87 | section 7 |

## 12. Corrections to older documents

- `macos-measurements-2026-11-26.md`, section 3, row "Mission Control Spaces
  strip": the low-confidence figures are replaced by section 5 here. The
  final height of 40 stands.
- The same section's "Mission Control entry" row gave a scale of .649 for one
  arrangement. The scale depends on the arrangement (section 1).

## 13. Not measured

- Whether the swipe follows the fingers, its threshold and its velocity rule:
  a probe cannot post a trackpad gesture.
- The rule for the scale and for dense arrangements (section 8), the margins
  on another display size, with the Dock on a side or hidden, and with windows of two
  apps.
- Fonts: sizes come from the width of the ink only, with no weight. The radii
  of the Space ring and of the label pill. The hover ring at a small scale.
- Light mode: every tone here is dark.
- The arm and stroke of the close button's cross (`SPACES_CLOSE_CROSS_SIZE` 8, `SPACES_CLOSE_CROSS_STROKE` 1.5, placeholders).
- Removing a Desktop from the middle of the row; whether "+" fires on press or
  release; the zoom of a click on another Desktop's thumbnail beyond the strip.
- The Dock's slide in full screen and finer than one row in Mission Control,
  Show Desktop, and the animation of App Exposé.
- A second run of a drop onto a full-screen Space and onto a Desktop's
  thumbnail; how far to the sides of a full-screen thumbnail the split is
  triggered; the drag's scale floor for a window whose aspect ratio is not the
  display's.
- The preview of a new Desktop as the pointer nears "+" and its docking on a
  click (`SPACES_ADD_PREVIEW_REACH` is a placeholder), the Space bar preview,
  and App Exposé with other than four windows.
- Whether the wallpaper moves on Control+arrows inside Mission Control: both
  Desktops had the same one.
- The key for full screen (the probe has no View menu, so Control+Command+F
  did nothing): times are from the call. The second half of the enter slide
  and the first 100 ms of the exit slide, which the window hides.
- Whether a full-screen Space goes after its origin or after the current
  Desktop, which needs a third Desktop.
- Reduce Motion, Reduce Transparency, Increase Contrast, more than one
  display, grouping by application.

The probes and the small result files are in
`scripts/macos-probes/mission-control/` and `scripts/macos-probes/full-screen/`.
The shell probes read the folder of the built Swift tools from `BIN`. The raw
captures are not kept.

## Sources

- [1] https://support.apple.com/guide/mac-help/mh35798/mac
- [3] https://support.apple.com/guide/mac-help/mh14112/mac
- [4] https://github.com/koekeishiya/yabai (`src/misc/extern.h`, `src/space.c`, `src/mission_control.c`)
- [5] https://support.apple.com/guide/mac-help/mchlp1119/mac
- [6] https://www.hammerspoon.org/docs/hs.spaces.html
- [7] https://developer.apple.com/documentation/appkit/nswindow/collectionbehavior-swift.struct
- [9] https://macmost.com/31-mission-control-tips.html
- [10] https://www.macworld.com/article/217775/inside-the-mysteries-of-mission-control.html
- [11] https://github.com/koekeishiya/yabai/wiki/Disabling-System-Integrity-Protection
- [12] https://www.oreilly.com/library/view/switching-to-the/9781449372927/ch04s10.html
- [13] https://osxdaily.com/2017/07/03/see-large-preview-mission-control-thumbnail-mac/
- [14] https://discussions.apple.com/thread/7256998
- [15] https://www.macworld.com/article/226448/hands-on-with-spaces-mission-control-and-split-view-in-el-capitan.html
- [17] https://forums.macrumors.com/threads/mission-control-in-macos-sierra.1978053
- [18] https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/main/js/ui/swipeTracker.js
- [20] https://osxdaily.com/2018/12/17/how-reduce-motion-mac-disable-animations/
- [21] https://support.apple.com/guide/mac-help/mchlc03f57a1
- [24] https://developer.apple.com/forums/thread/832884
- [25] https://macos-defaults.com/mission-control/
- [26] https://github.com/nix-darwin/nix-darwin/blob/master/modules/system/defaults/dock.nix
- [27] https://discussions.apple.com/thread/7889628
- [28] https://github.com/nix-darwin/nix-darwin/blob/master/modules/system/defaults/trackpad.nix
