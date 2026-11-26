# macOS motion: windows

**Goal:** record every animation an app's own window plays on macOS by
default, with numbers the kit can recreate.
**Scope:** `NSWindow` and SwiftUI `WindowGroup` windows: open, close,
minimize, zoom, `setFrame`, full screen, traffic lights, key state, tabs,
toolbar, title, tiling and the sheet's effect on its window. The Dock, Mission
Control and Spaces are out of scope; the sheet itself belongs to the
presentation family.
**Reader:** anyone who sets a window duration, spring or transition token.
**Input:** probes in `~/src/probes/windows` and the public `NSWindow` API.
**Output:** one table row per animation, then what was not measured and how to
rerun it.

## Steps

1. Read the host and the method below.
2. Find the row of the animation; read its curve, duration and source.
3. Treat "none" rows as findings: the system draws no animation there.
4. Rerun a row with the commands in "How to reproduce".

## Host and method

macOS 27.2.1, display 2560x1440 at 180 Hz and 1x, Reduce Motion off
(`NSWorkspace.accessibilityDisplayShouldReduceMotion` is false; no setting was
changed), Dock `mineffect` unset. Four sensors, no video is stored:

- **WindowServer:** `CGWindowListCopyWindowInfo` of the probe's own windows
  every 1 ms. It gives bounds (integer points) and `kCGWindowAlpha`; it sees
  open, close, minimize, zoom and `setFrame`, not the full-screen transition.
- **`NSWindow.frame`** on a `CADisplayLink`.
- **Pixels:** a ScreenCaptureKit stream of a rect at 120 fps. It keeps only
  numbers per frame: centre pixel, bounding box of the probe colour, mean
  luma. Frames arrive only when pixels change.
- **Core Animation hook:** a swizzle of `CALayer.addAnimation:forKey:` logs
  the public `CASpringAnimation` and `CABasicAnimation` fields.

Time 0 is the first changed sample minus 4 ms (half the poll interval); times
carry about 5 ms of error. Bounds are integer points, so a 2 % scale on a
500 pt window moves in 1 pt steps; shapes were fitted on an 1800 pt window.
Fits are pure Python (`fit.py`). "Spring" is a unit-mass damped spring from
rest, `omega0` in rad/s; "sine" is `p = (1 - cos(pi t / T)) / 2`, which is
`cubic-bezier(0.37, 0, 0.63, 1)`. "rms" is the residual in progress units
(0 to 1). Every row has Reduce Motion "not measured".

## Table

Kit column: on macOS the kit asks AppKit (`makeKeyAndOrderFront`, `zoom`,
`miniaturize`, `toggleFullScreen`), so these rows play without kit code; on
Linux the compositor owns them. Kit tokens today: `duration.fast` 120,
`duration.base` 250, `duration.slow` 420 and `spring.*`
([`tokens.json`](../../crates/ctdesign/tengen/tokens.json)); none is named for
a window.

| # | Element and trigger | Property | Duration | Curve | Delay | Interruptible | Source | Kit |
|---|---|---|---|---|---|---|---|---|
| 1 | Open: `orderFront` on a titled window, `animationBehavior` default (= `documentWindow`) | Scale 0.98 to 1 about the centre. Alpha stays 1 from the first frame | Last step at 165 to 170 ms (1800 pt window) | Spring omega0 25 to 29, zeta 0.77 to 0.82, overshoot 0.1 %, rms 0.02 | 8 to 20 ms after the call | No: `orderOut` at 40 ms snaps the scale to 1, then fades | Measured, 15 runs, 1 pixel run | AppKit; no token |
| 2 | Close: `orderOut` or `close()` on the same window | Alpha 1 to 0 and scale 1 to 0.98 | Alpha under 0.01 at 150 ms, 0 at about 230 ms; scale 1800 to 1764 pt in about 130 ms (t90 85 ms) | Alpha: critically damped spring omega0 44 (42.9 to 46.0), rms 0.004 to 0.015; scale follows the same curve | 7 to 14 ms | No: `orderFront` at 70 ms shows the window at once while the old image fades on | Measured, 12 runs | AppKit; no token |
| 3 | `animationBehavior = .documentWindow` | Same as rows 1 and 2 | Same | Same | Same | Same | Measured, 6 runs | AppKit |
| 4 | `animationBehavior = .utilityWindow`: open | None (1 sample) | 0 | none | none | n/a | Measured, 6 runs | AppKit |
| 5 | `.utilityWindow`: close | Alpha only, no scale | 0.97 to 0.02 in about 140 ms | Near linear with a slight ease-in: `cubic-bezier(0.5, 0.5, 0.9, 0.9)`, T 145 ms, rms 0.009 | 9 to 12 ms | Not measured | Measured, 6 runs | AppKit |
| 6 | `.alertPanel`: open | Scale 0.108 to 1.2 to 1 about the centre, alpha 0.18 to 1 | About 330 ms | Phase 1 (0 to 168 ms) ease-in `cubic-bezier(0.32, 0.10, 0.62, 0.32)`; phase 2 (168 to 330 ms) ease-out `cubic-bezier(0.25, 0.35, 0.65, 0.95)`, each rms under 0.01; alpha linear, T 328 ms, rms 0.004 | 8 to 10 ms | Not measured | Measured, 4 runs | AppKit |
| 7 | `.alertPanel`: close | None | 0 | none | none | n/a | Measured, 2 runs | AppKit |
| 8 | `.none`: open and close | None | 0 | none | none | n/a | Measured, 6 runs | AppKit |
| 9 | `NSPanel` with default behavior: HUD and utility style | Open and close: none | 0 | none | none | n/a | Measured, 2 runs each | AppKit |
| 10 | `NSPanel` titled, default behavior | Open: scale 0.98 to 1 as row 1. Close: none | Open about 80 ms (1 pt steps) | As row 1 (not fitted) | none | Not measured | Measured, 2 runs | AppKit |
| 11 | `NSApp.hide` and `unhide` | None: the window leaves and returns in one sample | 0 | none | none | n/a | Measured, 3 runs | AppKit |
| 12 | Minimize: `miniaturize` (default effect) | The window warps into its Dock tile; the bounding box widens to 2.17x, then converges to 56x47 | 498 to 499 ms first to last change; `willMiniaturize` to `didMiniaturize` 529 to 536 ms | Warp, not a scale; the bounds are not a curve. Width peaks at 270 ms | Width from 14 ms, position from 190 to 207 ms | Not measured | Measured, 2 runs | AppKit |
| 13 | Deminiaturize | Reverse warp from the Dock tile | 507 to 510 ms; `didDeminiaturize` 520 ms after the call | As row 12 | 13 to 33 ms | Not measured | Measured, 2 runs | AppKit |
| 14 | Zoom: `zoom(nil)`, green button, title-bar double click, `ToggleMaximize` | Frame to the visible frame (here `0,30 2560x1410`) and back | 339 to 350 ms for a 2000 pt move (the 0.35 s cap, row 16) | Sine: T 339 to 349 ms, rms 0.002 to 0.006; same both ways | 10 to 22 ms | No: synchronous, a second call waits for the first | Measured, 6 runs | AppKit |
| 15 | `setFrame(_:display:animate: true)` | Frame | `animationResizeTime`: 44 ms at 20 pt, 98 at 50, 167 at 100, 265 at 200, 350 from 326 pt | Sine, T within 6 ms of row 16 in 15 of 16 runs (one run stalled), rms 0.002 to 0.006 above 20 pt | 9 to 20 ms | No: it blocks the main thread | Measured, 16 runs | AppKit |
| 16 | `NSWindow.animationResizeTime(_:)` | The duration rule of rows 14 and 15 | `T = min(0.35, 0.24133 * ln(1 + d / 100))` s, `d = max(abs(dx), abs(dy), abs(dw), abs(dh))` in pt | n/a | n/a | n/a | Measured: 10 points and 18 combinations, fit residual 0.0000 | None |
| 17 | `setFrame(animate: false)` and a live resize step | None: one WindowServer frame, content laid out in the same frame | 0 | none | none | n/a | Measured, 1 run | AppKit |
| 18 | `window.animator().setFrame` in a default `NSAnimationContext` (`duration` 0.25, `timingFunction` nil) | Frame | t10 24, t50 70, t90 150, last step 220 ms (0.5 s context: 49, 143, 306, 450 ms) | Spring scaled by the duration: omega0 x T = 5.6, zeta 0.89 (omega0 21 to 24 at 0.25 s, 11.2 at 0.5 s), rms 0.005 | 9 to 12 ms | Not measured | Public default plus measured, 6 runs | AppKit; `duration.base` 250 matches the context |
| 19 | Full screen enter and exit: `toggleFullScreen`, green button | Window grows from its frame to the screen (700x450 to 2560x1440); a Space transition | `willEnter` to `didEnter` 538 to 546 ms (5 runs); exit 524 to 544 ms | Width: `cubic-bezier(0.24, 0.02, 0, 1)`, T 500 ms, rms 0.0009 (2 runs); exit `cubic-bezier(0.25, -0.03, 0, 1)`, T 506 ms, rms 0.002. Spring alternative omega0 25, zeta 1.4, rms 0.009 | 15 to 45 ms | No: a second toggle at +200 ms is ignored; the window stays in full screen | Measured, 6 runs, 3 pixel runs | AppKit; kit Plain windows use the same call |
| 20 | Full screen: title strip after enter | Strip waits at `y -32`, alpha 0 | 0 | none | none | n/a | Measured, 3 runs | AppKit |
| 21 | Full screen: strip reveal at the top edge (pointer at `y 2`) | Strip slides down with the menu bar | Dwell about 150 ms, slide about 140 ms; hide: about 70 ms, slide about 165 ms | Near linear in luma (rms 0.025); finer shape not resolved | about 150 ms in, 70 ms out | Not measured | Measured, 1 run each, strip luma | AppKit; see [full-screen title bars](fullscreen-titlebars.md) |
| 22 | Traffic lights: pointer enters the group | Glyphs appear on the three lights (luma of the close light) | 90 % in 40 ms, settled in 100 ms | Fast ease-out, exponential tau about 18 ms (spring fit rms 0.016) | none after the pointer arrives | Not measured | Measured, 1 run | AppKit |
| 23 | Traffic lights: pointer leaves | Glyphs fade | 150 to 170 ms | Spring omega0 70, zeta 1.1, rms 0.0065 | none | Not measured | Measured, 2 runs | AppKit |
| 24 | Traffic lights: press | The pressed light jumps to its pressed tone in the first frame, then relaxes | Relaxes in 134 ms | `cubic-bezier(0.21, 0.92, 0.17, 0.99)`, T 132 ms, rms 0.003 | 0 | Not measured | Measured, 1 run | AppKit |
| 25 | Traffic lights: release outside the button | Pressed tone relaxes | About 500 ms | `cubic-bezier(0.39, 1.37, 0.24, 0.80)`, T 493 ms, rms 0.004 | 8 ms | Not measured | Measured, 1 run | AppKit |
| 26 | Key state: window becomes inactive or active | Lights go colour to gray, shadow shrinks (luma 189 to 240 below the window), title bar fill unchanged (luma change 0.01) | One captured frame (5.5 to 11 ms), no fade | none | none | n/a | Measured, 3 runs, 6 transitions each (shadow on a white backdrop, 5 seen) | AppKit |
| 27 | Tabs: second tab added | Tab bar appears inside the same frame; window frame unchanged | One or two frames | none | none | n/a | Measured, 1 run | AppKit |
| 28 | Tabs: third tab added | Tab buttons resize (285 to 193 pt); the new button grows from 8 to 193 pt | Span 483 to 489 ms | Critically damped spring omega0 21.5, zeta 1.04, rms 0.0025 (3 buttons); the layer holds a `CASpringAnimation` with mass 1, stiffness 400 (omega0 = 20) | about 0 | Not measured | Measured, 1 run | Missing |
| 29 | Tabs: a tab closes | Remaining buttons widen (192 to 290 pt) | Span 504 ms | Critically damped spring omega0 20.0, zeta 1.00, rms 0.0004 | 5 ms | Not measured | Measured, 1 run | Missing |
| 30 | Tabs: select, last extra tab closes, `toggleTabBar` | Content or tab bar swaps | One or two frames | none | none | n/a | Measured, 1 run each | AppKit |
| 31 | Toolbar: `toggleToolbarShown` hide and show | Toolbar row collapses, content moves | 122 ms hide, 123 ms show (12 to 15 frames) | Not fitted (luma) | 17 to 23 ms | Not measured | Measured, 1 run each | Missing |
| 32 | Toolbar: `insertItem`, `removeItem` | Item appears or disappears in place; no slide | One frame | none | none | n/a | Measured, 1 run each | Missing |
| 33 | Toolbar: window narrows so an item no longer fits | Item opacity 1 to 0 in place; positions do not move | 200 ms | Spring omega0 28, zeta 0.89, rms 0.004 (bezier `cubic-bezier(0.23, 0.09, 0.24, 1.01)`, T 199 ms) | 23 ms | Not measured | Measured, 4 items, 1 run | Missing |
| 34 | Toolbar: window widens again | Item is back at alpha 1 in the first sample | One frame | none | none | n/a | Measured, 1 run | Missing |
| 35 | Toolbar style `unified` to `unifiedCompact` and back | Bar height and items jump | One frame | none | none | n/a | Measured, 1 run each | AppKit |
| 36 | Title and subtitle change | Text swaps; no crossfade, no `CAAnimation` of the text | One frame | none | none | n/a | Measured, 3 runs | AppKit |
| 37 | Toolbar customization palette opens and closes | Sheet slides over the window | About 220 ms in, 260 ms out | Not fitted | 100 ms in | Not measured | Measured, 1 run (presentation family) | n/a |
| 38 | Sheet `beginSheet`, `endSheet` | The parent frame never changes; the sheet window moves 32 pt | About 230 ms each way | Not fitted | 25 to 30 ms | Not measured | Measured, 4 runs | n/a |
| 39 | Tiling through Window > Fill and Window > Move & Resize > Left | Frame to the tile | t10 30, t50 60, t90 140, t99 240 ms (both) | Overdamped, dominant rate 19.2 per s (tau 52 ms): spring omega0 45, zeta 1.38, rms 0.011 (Left); omega0 151, zeta 4.0, rms 0.016 (Fill). Not the sine of row 14 | 3 ms | Not measured | Measured, 2 runs | AppKit; kit uses the same menu |
| 40 | Reduce Motion for every row above | Not changed on the host | Not measured | Not measured | Not measured | Not measured | Not measured | None |

Row 3 equals rows 1 and 2 because a titled `NSWindow` resolves `default` to
`documentWindow`. Rows 12 and 13 were seen as the bounds of the warp; the
genie shape needs pixels of the Dock tile and was not captured.

**Re-read 2026-11-26.** The scale of row 1 was not seen: the window is at its
final size in its first frame, and the cause is unknown. The close of row 2
fits a spring omega0 55 to 68, zeta 1.2 to 1.4 in that pass
([2026-11-26 measurements, section 3](macos-measurements-2026-11-26.md#3-animations)).

## Surprises

- Open does not fade. It is a 2 % scale spring with an alpha of 1 from the
  first frame; close is the fade (alpha 1 to 0 over 230 ms).
- Key state is instant: lights, shadow and title bar change inside one
  captured frame. There is no fade to copy.
- `animationResizeTime` is a logarithm of the largest edge delta, capped at
  0.35 s. Zoom and animated `setFrame` use it with a sine curve; tiling uses
  a 240 ms overdamped curve instead.

## Not measured

- Reduce Motion: the host setting stays off and was never changed.
- The genie shape and the `scale` minimize effect (`mineffect` stays default).
- Drag-to-edge snap, the green-button hover menu and Option-drag tiling:
  only the Window menu route was driven.
- Live resize by dragging an edge, window move by dragging, title-bar drag.
- Tab merge (`mergeAllWindows`), tab drag out and reorder, tab bar toggle
  effect on content height.
- `NSPanel.hidesOnDeactivate` fade (the app cannot be deactivated from the
  probe without the shell).
- The overflow chevron and its menu, and the customization palette contents.
- Interruption of close, minimize, tiling and the toolbar fades.

## How to reproduce

Run each probe while no other agent holds `/tmp/barrierkit-screen.lock`
(`lock.sh` takes it).

```sh
cd ~/src/probes/windows
swiftc -O wprobe.swift -o wprobe && swiftc -O sck.swift -o sck
./lock.sh ./wprobe open default:big > open.log      # rows 1 and 2
python3 -I an.py open.log --win 900                  # summary per mark
python3 -I series.py open.log hide alpha 230 --pool  # fit of one series
./lock.sh ./wprobe setframe > sf.log                 # rows 15 and 16
./wprobe rt | grep ' rt '                            # row 16, no window shown
./lock.sh ./wprobe zoom > zoom.log                   # row 14
SCK_STRIDE=3 SECS=14 ./lock.sh sh cap.sh fs 0 0 2560 1440 fs   # row 19
python3 -I bbox.py fs.sck fs.log fs-enter 800 540
```

Rows 22 to 36 use `batch2.sh`, `batch3.sh` and `batch4.sh` (`lock.sh sh
batchN.sh <dir>`) and `mx.py`, `lt.py`, `lf.py` and `tv.py`.
