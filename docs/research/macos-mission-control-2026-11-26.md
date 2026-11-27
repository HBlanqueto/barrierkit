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
- **Menu bar:** replaced by the strip, which slides in from the top.
- **Dock:** shown even with auto-hide on. It slides up from the bottom edge,
  starting 50 to 77 ms after the key, and slides down on exit, finished at
  about 300 ms (low confidence, no fit).
- **Other Spaces:** their windows are shown only inside the strip thumbnails,
  which are live. An empty Desktop shows a pill with "No Available Windows" at
  the centre of the area.

## 4. Thumbnail decoration

- A thumbnail is the window image with its own shadow, scaled together. At
  scale 1 the region is pixel-identical to the window at rest, and a window
  that is not main keeps its weaker shadow.
- The shadow's extent scales with the thumbnail and its alpha does not: alpha
  .02 is reached at 39 at rest, 37 at .93 and 17 at .43.
- The corner radius scales the same way (the corner inset ends at row 11 at
  rest, 10 at .93, about 5 at .43; medium confidence).
- **Hover ring:** sRGB (58, 154, 252), 4 wide, starting right outside the
  window's outline, no gap. It has no fade: one frame, 18 to 51 ms after the
  pointer moves. Measured at scale .93 only.
- **Hover title:** a capsule centred on the thumbnail, not below it: 79×29 for
  "Probe 4", black text of about 14 pt medium (low confidence), 14 of side
  padding, a light material (251, 251, 89 over yellow). No app icon.

## 5. Animations

180 fps captures. N = 1, 4 and 9 windows, three runs each; fits with rms under
.02 only.

| Animation | Result |
|---|---|
| Entry and exit | Centre x, centre y, width and height of every window follow one critically damped spring: ω0 24 to 28 (median 25 to 26), ζ .97 to 1.07. t10 8 to 19, t50 53 to 69, t90 142 to 158 ms, last change 192 to 297 ms by travel. Exit is the same spring. Key to first changed frame 29 to 42 ms. |
| Pick (click on a thumbnail) | Fires on button release, motion starts 28 to 36 ms later. Same spring (ω0 25, ζ 1.00). The picked window is on top from the first frame. |
| Strip entry, 0 to 40 | t50 47 to 58, t90 136 to 142, last 169 to 175 ms; ω0 29 to 33, ζ 1.1 to 1.2. The content fills from the top: it slides down. |
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

In `ctdesign` now: `EXPANSION_OMEGA`, `EXPANSION_DAMPING`, `EXPANSION_MARGIN_SIDE`, `EXPANSION_MARGIN_TOP`, `EXPANSION_MARGIN_BOTTOM`, `EXPANSION_HOVER_RING_WIDTH` and `SPACES_STRIP_HEIGHT`, and the Spaces strip's `SPACES_STRIP_EXPANDED_HEIGHT`, `SPACES_STRIP_EXPAND_OMEGA`, `SPACES_STRIP_EXPAND_DAMPING`, `SPACES_STRIP_TINT`, `SPACES_STRIP_LINE`, `SPACES_STRIP_SHADOW`, `SPACES_THUMBNAIL_HEIGHT`, `SPACES_THUMBNAIL_TOP`, `SPACES_THUMBNAIL_GAP`, `SPACES_LABEL_*`, `SPACES_EXPANDED_LABEL_FONT`, `EXPANSION_TITLE_*`, `SPACES_RING_*`, `SPACES_ADD_*`, `SPACES_CLOSE_*`, `EXPANSION_CAPTION_COLOR` and `EXPANSION_GRID_GAP`; the rest are not. Each is added by the step of the compositor that
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
- The Dock's slide in Mission Control and in full screen, the motion of the
  strip's labels, Show Desktop, and the animation of App Exposé.
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
