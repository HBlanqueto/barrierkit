# UIKit animations (Mac Catalyst)

**Goal:** give one table of measured motion for the transitions of Apple's
UIKit apps on this Mac, so the kit can copy a duration and a curve instead of
guessing.
**Scope:** push and pop, sheet and its detents, popover, menu, alert, hover,
press, list insert and remove, plus the two Catalyst apps of decision T4 (Maps,
Weather) and the Malaysia comparison of
[macOS animations](macos-animations.md). It ends at the table and the proposed
tokens. It does not design the kit API (phase U2) and measures no iPhone or
iPad.
**Snapshot:** macOS 27.2.1, Xcode 27.2, Apple-silicon Mac, 2560x1440 display at
1x, system appearance Dark (the probe window is forced Light), Reduce Motion
off (`shellstate`), 2026-11-22.
**Reader:** the engineer who designs implicit animation (U2) and anyone who
changes a duration, spring or transition token.
**Input:** the Catalyst probe harness of phase U0 (`uikitmotion` in
`~/src/probes`), 60 fps recordings (`record.swift`), and the
[UIKit roadmap](uikit-roadmap.md) (T6, risks, E8).
**Output:** the [motion table](#2-motion-table), the
[Malaysia comparison](#3-malaysia-comparison), the proposed tokens and the list
of what is not measured.

## Steps

1. Read [Method and limits](#1-method-and-limits). Every value is **Mac
   Catalyst, not iOS**.
2. To copy a motion, take the row of [the table](#2-motion-table). Trust a row
   only if its reliability says measured.
3. To change a Malaysia duration, take its row in
   [the comparison](#3-malaysia-comparison).
4. To reproduce a row, run the commands in [Reproduce](#7-reproduce).

## 1. Method and limits

**Catalyst, not iOS.** Every row of the table is Mac Catalyst code on macOS, with
the Mac idiom (`UIDeviceFamily` 6 and the `UIUserInterfaceIdiomMac` keys that
Maps carries; `UIDevice.userInterfaceIdiom` reads 5). The scaled-iPad idiom of a
bare Catalyst bundle (idiom 1, window scaled to 77 %) is not measured. Whether
an iPhone or iPad uses the same timings is not verified.

**Two sources**, as decision T6 asks.

| Source | What it gives | What it cannot give |
|---|---|---|
| Own probe: a UIKit app on views we build. It reads every `CAAnimation` (class, duration, timing points, spring mass, stiffness, damping) and samples the presentation layer of every layer every 2 ms | The exact curve of position, size and opacity, and the public parameters when UIKit uses a `CASpringAnimation` or a `CABasicAnimation` | Anything drawn by AppKit outside the UIKit layers (popover window, menu, button press) |
| 60 fps recording (`record.swift`, [E8](uikit-roadmap.md#evidence)) of the probe, Maps, Weather and the desktop | The pixels the user sees, with the real frame times (`framedump.swift`) | Parameters; the frame step is 16.7 ms |

**Calibration of the probe.** A `CASpringAnimation` set by the probe itself
(mass 1 with stiffness 100 and damping 10, 333.33 and 36.515, 400 and 20)
matches the textbook spring within 0.003 of the travel. A spring with damping above critical (mass 3,
stiffness 1000, damping 500) moves exactly as a critically damped one: Core
Animation clamps it, as SwiftUI's `Spring(mass:stiffness:damping:)` does (it
reads `dampingRatio` 1.0). So every spring below is reported by natural
frequency `ω0 = sqrt(stiffness / mass)` in rad/s and damping ratio `ζ`.

**Words.** `t10`, `t50` and `t90` are the times after the first motion at which
the travel reaches 10 %, 50 % and 90 %. `settle` is the last time the value is
farther than 1 % from its end. A **fit** is a curve fitted to samples (a cubic
Bézier or a damped spring); it describes the motion and is not Apple's
parameter. A **read** is a public `CAAnimation` or `Spring` property. A
recording row is the blend progress of the frames (the projection of each frame
on the first-to-last difference), so a window with shadow or reflowing content
is approximate.

**Limits.**

- One run per row unless a range is shown. A first sheet or alert in a process
  stalls the main thread about 0.3 s before it moves; that latency is not part of
  the curve.
- A recording has 16.7 ms steps. The cross-check of push and pop against the
  recording agrees within 10 ms (t50 54 against 62 ms, t90 155 against 165 ms,
  settle 300 against 291 ms).
- The Hz of the display, the effect of Increase Contrast and the Reduce Motion
  setting on UIKit (it is off here) are not measured.
- The probe sources stay outside the repo; the files are named in
  [Reproduce](#7-reproduce).

## 2. Motion table

Spring rows give `ω0` and `ζ`. For `ζ = 1`, SwiftUI's
`Spring(duration: 2π / ω0, bounce: 0)` has the same `ω0` (checked with the
`springmap` probe: duration 0.344 gives ω0 18.27, duration 0.274 gives 22.93).
A critically damped spring with `ω0` 18.26 has t10 29 ms, t50 92 ms, t90 213 ms
and settle 364 ms; with 22.93, 23, 73, 170 and 290 ms.

| Transition | Platform | Motion | Source | Reliability |
|---|---|---|---|---|
| **Push** (`UINavigationController`) | Mac Catalyst, Mac idiom | The new view slides in from the right edge, 700 pt over its width; the old view moves 30 % of the width to the left (-210 of 700 pt) on the same curve; a dimming layer fades 0 to 1 and an edge shadow 1 to 0 on it. Curve: critically damped spring, fit `ζ` 1.10, `ω0` 25.3 rad/s (rmse 0.004); t10 12, t50 62, t90 165, settle 291 ms. Bézier fit (0.26, 0.76, 0.26, 1.0) over 368 ms. The coordinator reports `transitionDuration` 0.35 s | Own probe: presentation-layer samples; `CAAnimation` read (a linear pacing animation with speed 0, so the curve is the animator's, not read); recording cross-check | Measured (curve is a fit) |
| **Pop** | Mac Catalyst | The same layers in reverse, same 0.35 s: spring fit `ζ` 1.17, `ω0` 27.8 (rmse 0.008); t10 13, t50 60, t90 159, settle 290 ms; recording t50 64, t90 177, settle 283 ms | Own probe and recording | Measured (fit) |
| **Sheet present** (`pageSheet`) | Mac Catalyst | `CASpringAnimation` on the card position: mass 3, stiffness 1000, damping 500, initial velocity 0, duration 0.5 s, settling 0.6 s (read). Behaves as `ζ` 1, `ω0` 18.26; the samples fit `ζ` 1.00, `ω0` 18.2 (rmse 0.0002); SwiftUI equivalent duration 0.344, bounce 0. A dimming layer runs beside it | Own probe: `CAAnimation` read and samples | Measured |
| **Sheet dismiss** | Mac Catalyst | The same spring read (mass 3, stiffness 1000, damping 500); fit `ζ` 0.99, `ω0` 18.2 (rmse 0.0005). Motion began about 0.28 s after the call in this run (main-thread work, not understood) | Own probe | Measured; the latency is unexplained |
| **Sheet detent change** (medium to large and back, `animateChanges`) | Mac Catalyst | `CASpringAnimation` on the card, its grabber and its height: mass 1, stiffness 333.33, damping 36.515, initial velocity 0, duration 0.506 s, settling 0.6 s (read); fit `ζ` 1.00, `ω0` 18.3 (rmse 0.0001); t10 29, t50 92, t90 213, settle 359 ms; card height 295 to 468 pt. Detents animated in the first sheet of a run; a second sheet in the same process came up at its large size and was not investigated | Own probe | Measured (one process) |
| **Popover** | Mac Catalyst | A second window drawn by AppKit; no `CAAnimation` in the UIKit layers. Present: about 0.2 s with 1 % overshoot, t10 24, t50 74, t90 136, settle 267 ms; Bézier fit (0.25, 0.08, 0.49, 1.16) over 209 ms. Dismiss: a fade, t50 16, t90 89, settle 133 ms | Recording of the probe | Measured (one run each, window shadow included) |
| **Menu** (`UIButton.menu`, pull-down) | Mac Catalyst | Open: the menu appears between two consecutive 60 fps frames (at most 16.7 ms). Close: a fade, t10 8, t50 52, t90 132, settle 183 ms; Bézier fit (0, 0, 0.35, 1) over 200 ms (rmse 0.009) | Recording of the probe | Measured (one run) |
| **Alert** (`UIAlertController`) | Mac Catalyst | Present: alpha 0 to 1 and scale 1.2 to 1 on one curve, fit `ζ` 1.00, `ω0` 22.9 (rmse 0.0001); t10 22, t50 72, t90 169, settle 285 ms; SwiftUI equivalent duration 0.274, bounce 0. Dismiss: alpha 1 to 0, no scale; t50 73 to 86, t90 170 to 182, settle 287 to 296 ms in two runs; fit `ζ` 0.9 to 1.0, `ω0` 18 to 23 | Own probe: samples (the layers carry no readable spring) | Measured (fit) |
| **Press** (`UIButton` filled, highlight on and off) | Mac Catalyst | Both changes happen inside one 60 fps frame (at most 16.7 ms); the face goes from (253, 0, 0) to (224, 5, 0). The button is drawn by AppKit; no `CAAnimation` | Recording of the probe | Measured (one run) |
| **Hover** (pointer over a `UIButton`, `UIHoverGestureRecognizer` fires) | Mac Catalyst | No animation and no pixel change in about 2 s of hover | Recording of the probe | Measured (one run); the pointer effects of iOS are not shown on the Mac |
| **List insert and remove** (`UITableView`, styles `.automatic`, `.fade`, `.left`, `.top`) | Mac Catalyst | `CABasicAnimation`, 0.3 s, timing function cubic Bézier (0.42, 0, 0.58, 1) (ease in-out, points read). Rows below move 60 pt (one row) on `position`; `.fade` adds opacity 0 to 1 (remove: 1 to 0); `.left` adds a slide of the table width (700 pt) and opacity; `.top` moves the new row from above. Insert and remove use the same duration and curve; measured settle 264 to 275 ms | Own probe: `CAAnimation` read and samples | Measured |
| **Weather: sidebar collapse** (the toolbar button) | Mac Catalyst (Weather links UIKit, E2) | About 0.38 s: t10 10, t50 100, t90 242, settle 383 ms; Bézier fit (0, 0.13, 0.55, 1) over 354 ms (rmse 0.017). The content reflows in the same frames, so this mixes sidebar width and content. Re-opening was not captured | 60 fps recording of Weather | Measured (one run, approximate) |
| **Maps: card page open** (the Favourites guide, from the home card) | Mac Catalyst (Maps links UIKit, E2) | t10 31, t50 110, t90 374, settle 533 ms: a fast first 100 ms, then a long tail; fits are poor (rmse 0.015 to 0.025) | 60 fps recording of Maps | Measured (one run, approximate) |
| **Maps: card page close** (the X) | Mac Catalyst | t10 8, t50 80, t90 199, settle 350 ms; spring fit `ζ` 1.02, `ω0` 17.7 (rmse 0.025), the same family as the sheet spring | 60 fps recording of Maps | Measured (one run, approximate) |
| Sheet detent drag, interactive pop, rubber band, card drag of Maps | Mac Catalyst | Not measured: the pointer driver sends no drag | none | **Unmeasured** |
| Popover or alert with Increase Contrast or Reduce Motion on | Mac Catalyst | Not measured | none | **Unmeasured** |

Whether Maps and Weather draw these with `UIView` animation or with SwiftUI is
not verified (roadmap, Not verified). The recordings of Maps and Weather were
deleted after the numbers were taken.

## 3. Malaysia comparison

Every row of the comparison in [macOS animations](macos-animations.md#5-comparison-with-malaysia)
is here. The "Malaysia today" column is copied from that file; the tokens live
in the Malaysia repository and were not read again. The macOS values of the
desktop rows are macOS itself (not Catalyst), recorded at 60 fps, one run each.

| Element | Malaysia today | macOS value | Status |
|---|---|---|---|
| Menu open | No menu animation | At most 16.7 ms (also in the Catalyst menu above) | Measured; matches |
| Menu close | Not listed | Fade, settle 183 ms, Bézier fit (0, 0, 0.35, 1) over 200 ms | Measured; new row |
| Window default | `duration.base` 250 | `NSAnimationContext` 250 ms (earlier note); UIKit presentations are not 250 ms: popover settle 267, alert 285, sheet 364 | Matches for AppKit windows only |
| Dock magnify scale | 1.3 | about 1.3 (earlier note) | Matches |
| Dock magnify duration | 120 | In: t50 76, settle 117 ms. Out: t50 62, settle 133 ms (the pointer jumps onto the Dock and off it) | Measured; matches within a frame |
| Dock hide | 280 | none | Unverified (needs auto-hide, a system setting) |
| Dock minimise | 500 | none | Unverified (not attempted) |
| Dock slot | 400 | none | Unverified |
| Dock bounce | 620, 0.55 | none | Unverified |
| Notification in | 450, bounce 0.15 | none | Unverified: a banner posted with `osascript` did not appear in the capture |
| Notification out | 260 | none | Unverified (same) |
| Launcher open | 180 | Spotlight (`cmd+space`): t10 11, t50 50, t90 81 ms; its search content settles later (383 ms) | Measured; Malaysia is about twice as long. Launchpad and the apps view are not measured |
| Launcher close | 120 | Spotlight: t10 3, t50 14, t90 64, settle 83 ms | Measured; Malaysia is about 1.5 times as long |
| Space slide | 480 cubic Bézier | none | Unverified (not attempted) |
| Lock fade | 1100 | none | Unverified |
| Lock rise | 900, 120 offset | none | Unverified |
| Lock login | 600 | none | Unverified |
| Mission Control open and close (listed unknown in macOS animations, section 1) | not in the table | none | Unmeasured: `ctrl+up` through the key driver changed no pixel |

## 4. What the table says

1. **Every measured UIKit motion is critically damped or near it.** The sheet,
   the detents, the alert and (by fit) the push and pop are springs with
   `ζ` of 1 and `ω0` from 18 to 28 rad/s. Only the popover overshoots, by 1 %.
   None is a duration with a cubic ease.
2. **The kit's `Spring` is not that spring.** With `bounce` 0 it is
   `ease_out`, `1 - (1 - t)^3`, over `duration` ([animation.rs](../../crates/bk/src/animation.rs)).
   At 250 ms that reaches 10 %, 50 %, 90 % and 99 % at 9, 52, 134 and 196 ms; the
   critical spring with `ω0` 18.26 reaches them at 29, 92, 213 and 364 ms. The
   kit starts faster and stops harder.
3. **Velocity matters.** The detent spring has initial velocity 0 and no
   interruption was measured; whether UIKit carries velocity when a transition
   is reversed mid-way is not measured.
4. **Three transitions are not springs.** List rows use ease in-out, 300 ms; a
   press and a menu open take no time; a hover takes no time.
5. **Reduced motion** is not measured for UIKit. The kit already maps it to 0 ms
   (`use_reduced_motion`).

## 5. Proposed tokens

None is added to `crates/ctdesign/tengen/tokens.json`. The roadmap's option O3
(a), beside the existing `duration` tokens, fits, and nobody edits that file now.
But `ctdesign/src/lib.rs` and `components.json` are edited by another session at
the time of writing, `build.rs` generates the constants from the file, and the
spring model (O2) is the decision of phase U2, so a token of `ω0` or `ζ` would fix
the model before it is chosen. The proposal for U2 to enter, with the SwiftUI
equivalent of each spring (Mac Catalyst values, not iOS):

| Token | Value | Source row |
|---|---|---|
| `duration.sheet` spring | `ω0` 18.26, `ζ` 1 (SwiftUI duration 0.344, bounce 0) | Sheet, detent |
| `duration.alert` spring | `ω0` 22.9, `ζ` 1 (duration 0.274, bounce 0) | Alert present |
| `duration.push` spring | `ω0` 25, `ζ` 1 (duration 0.25, bounce 0); parallax 0.3 of the width | Push, pop (fit) |
| `duration.list-row` | 300 ms, cubic Bézier (0.42, 0, 0.58, 1) | List insert and remove |
| `duration.popover-in` | 210 ms, Bézier (0.25, 0.08, 0.49, 1.16) | Popover (fit) |
| `duration.popover-out` | 120 ms, fast ease out | Popover dismiss (fit) |
| `duration.menu-close` | 200 ms, Bézier (0, 0, 0.35, 1) | Menu |
| `duration.menu-open`, press, hover | 0 ms | Menu, Press, Hover |
| `duration.launcher-open`, `-close` | 100 ms and 80 ms (t90 81 and 64 ms) | Spotlight |

## 6. Scroll

**What was driven.** A Mac Catalyst `UIScrollView` (700 x 500 pt, Mac idiom) and an
`NSScrollView` of the same size, fed with continuous scroll events that carry a
gesture phase (began, changed, ended), posted with `CGEvent` at 8 to 10 ms steps
(`wheel` in `~/src/probes`). `UIScrollView` turns them into a pan: it calls
`willEndDragging` with a velocity and a target, then decelerates by itself. The
sample is `contentOffset` read every millisecond in the process
(`uikitscroll.swift`, `appscroll.swift`, `scrollfit.py`). A mouse drag does not
pan a Catalyst scroll view (the touches arrive as `indirectPointer` and the pan
ignores them), a finger path cannot be posted, and the system generates the
momentum of a real trackpad, so **the touch path of iOS and the momentum of a real
trackpad are not measured**. The recordings of Weather were not made.

**Deceleration** after a release at velocity `v` (points per millisecond, the
`velocity` of `willEndDragging`), four flicks:

| `v` (pt/ms) | 0.70 | 2.00 | 4.64 | 13.11 |
|---|---|---|---|---|
| Stop distance `D` (pt) | 97.0 | 396.6 | 1218.2 | 4868.5 |
| `D / v^(4/3)` | 157.5 | 157.5 | 157.5 | 157.5 |
| Duration `T` (ms) | 559 | 797 | 1050 | 1489 |
| `4 D / v` (ms) | 558 | 794 | 1051 | 1486 |

1. The target UIKit reports equals the stop (within 1 pt), and
   `D = 157.5 v^(4/3)` holds exactly: the distance is **not** linear in the
   velocity. An exponential with the documented rate 0.998 would give
   `D = 499 v`, 997 pt for the 2.00 flick instead of 397.
2. The path is a quartic ease-out, `y(t) = D (1 - (1 - t/T)^4)` with
   `T = 4 D / v`, so the speed at the release is exactly `v` and the motion stops
   at `T`. At 0.1, 0.2, 0.5 and 0.8 of `T` the three recorded runs read
   0.35, 0.60, 0.93 and 1.00 of `D` (the model: 0.34, 0.59, 0.94, 1.00); the
   rmse of the model is 0.5 to 1.8 % of `D`, under the 0.056 floor.
3. `decelerationRate` has **no effect** on this path: `.fast` gave
   `D / v^(4/3)` 157.5 as well (0.80 pt/ms to 116 pt, 2.58 to 557, 7.86 to 2463).
   The rate 0.998 is only reported by `willBeginDecelerating`.

**Rubber band** while the finger moves beyond an edge (`contentSize` 900 in a
500 pt view, the pan moves 1:1 inside): an input of 240, 600 and 1800 pt gives an
overscroll of 16, 39 and 103 pt on `UIScrollView`, 41 pt for 600 on
`NSScrollView`. The three UIKit points fit
`over = d (1 - 1 / (1 + c x / d))` with `d` the view height (500) and `c` 0.069,
0.071 and 0.072: **`c` = 0.07**, not the 0.55 often quoted for iOS touch. One view
size was measured, so `d` is the view length by assumption.

**Return** after a release beyond the edge (no velocity: the finger stopped):
a critically damped spring, `over(t) = o0 (1 + w t) e^(-w t)`, `w` 10.5 to 11.5
rad/s on UIKit and 12.0 on AppKit, rmse 0.02 to 0.05 of `o0`; a power ease-out
fits worse (0.05 to 0.075) and an exponential much worse (0.09 to 0.12). It
settles in 0.4 to 0.6 s.

**Edge hit while decelerating** (the content is 900 high, the release at 2.85
pt/ms, 112 pt from the end): the motion reaches the edge at about 2.5 pt/ms,
overshoots 23 pt at 75 ms and returns by `over(t) = u t e^(-w t)` with `w` 13 and
`u` 0.83 pt/ms (checked at 0.12, 0.25 and 0.30 s after the hit, within 1 pt): the
band takes **0.30 to 0.33 of the arrival speed**. Overshoot is `u / (w e)`.

**Pull to refresh** (`UIRefreshControl` on the Catalyst scroll view, 900 pt content,
rest offset -32): `valueChanged` fires while the finger is still down, when the
stretch reaches about 81 pt (offsets -104 and -113 straddle it); the adjusted top
inset grows by 60 (32 to 92) and the content settles on it, 60 pt down, in about
0.4 s as the return spring (from 96 pt at the release). With the control the pull
is looser than the band above: an input of 120 and 240 pt gave 63 and 72 pt, 600
pt gave 96 pt (the first 5 events jump to about 55 pt), and the stretch past the
held edge is the stiff band again. Three pulls only; the mapping is not fitted.

**Model for the kit** (`crates/bk/src/scroll_physics.rs`, [Scrolling](../barrierkit/05-controls.md#scrolling)):

| Constant | Value | Source |
|---|---|---|
| Stop distance | `157.5 v^(4/3)` pt, `v` in pt/ms | deceleration |
| Curve and duration | `D (1 - (1 - t/T)^4)`, `T = 4 D / v` | deceleration |
| Rubber band | `d (1 - 1 / (1 + 0.07 x / d))` | rubber band |
| Return | critically damped, `w` 12 rad/s | return, edge hit |
| Velocity into the band | 0.3 of the arrival speed | edge hit |
| Smallest fling | 0.05 pt/ms (50 pt/s, Freya's) | not measured |
| Refresh | trigger 80 pt, hold 60 pt | pull to refresh |
| Band of the pull to a refresh | 0.5 (the iOS touch constant; not fitted) | not measured |

The values are `scroll-physics` in `components.json` and
`ctdesign::SCROLL_PHYSICS`.

Not measured here: the loose band of the pull to refresh (three pulls only), the minimum velocity that starts a deceleration, the
velocity window that UIKit averages for the release, a horizontal axis, two
view sizes, `UIRefreshControl` on the Mac, and the same flick through a real
trackpad (the Mac adds its own momentum events, so on the kit's macOS runner a
release already follows the system's momentum).

## 7. Reproduce

The sources are outside the repo, in `~/src/probes` (each has a line in its
`README.txt`):

1. `uikitbuild.sh uikitmotion` builds the probe for `arm64-apple-ios27.0-macabi`.
2. `uikitrun.sh <push|sheet|popover|alert|insert|calibrate> <dir> [seconds]`
   runs one scenario as an app bundle with the Mac idiom and, with seconds, records it at 60 fps.
3. `uikitmouse.sh <press|hover|menu> <dir> <seconds> <steps>` drives the pointer
   and keys for the three scenarios that need them.
4. `python3 -I motionfit.py table <dir>/<scenario>-anim.json series` prints t10,
   t50, t90, settle and the two fits per moving layer.
5. `uikitscroll.sh <normal|fast|bounce> <dir> <dy> 30 8 1` (with `DRIVER=wheel`; a
   negative `dy` scrolls down) or `appscroll.sh` for `NSScrollView` writes
   `<dir>/<mode>-scroll.json`; `python3 -I scrollfit.py <files>` prints the release
   velocity, the stop and a fit.
6. `framedump <video> x y w h step out.bin`, then `python3 -I framecurve.py
   events <out.bin>` and `curve <out.bin> <start> <end>`, give the recording
   rows. `python3 -I motionfit.py test` and `framecurve.py test` run the checks.

## 8. Not measured and not verified

- Any iPhone or iPad value; the scaled-iPad idiom; Reduce Motion, Increase
  Contrast and a 120 Hz display.
- Interruption and velocity: a push reversed mid-way, a sheet dragged between
  detents, a pop by edge swipe.
- Whether a second sheet in one process drops its detents, and the 0.28 s
  latency of the sheet dismiss.
- Maps and Weather beyond the three transitions above, and which of them use
  SwiftUI inside.
- The touch path of `UIScrollView`, the system momentum of a trackpad, and Weather's
  scroll: [section 6](#6-scroll) lists what was measured through posted scroll events.
- Every Malaysia row marked unverified in [the comparison](#3-malaysia-comparison).
