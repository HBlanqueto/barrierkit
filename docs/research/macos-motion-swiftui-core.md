# macOS motion: the SwiftUI core

**Goal:** record the animations that every SwiftUI app on macOS plays by
default, with numbers the kit can reproduce.
**Scope:** `Animation` values, transitions, content transitions, layout
animation, SF Symbol effects, `ProgressView`, `TimelineView`, `PhaseAnimator`,
`KeyframeAnimator`, `scrollTransition`, programmatic scrolling and Reduce
Motion. It ends at the table of rows and the list of what is not measured. It
does not cover AppKit-only controls, windows, sheets or the desktop shell.
**Reader:** whoever changes `bk::animation`, `Animation`, `Transition` or a
duration token.
**Input:** macOS 27.2.1, Xcode 27.2 SDK (`swiftc` 6.4.2), a display that
reports 180 Hz (`NSScreen.maximumFramesPerSecond`), Reduce Motion off, 2026-11-25.
Four Swift probes and five Python scripts in `~/src/probes/swiftui-core`.
**Output:** the [table](#2-table), [Not measured](#3-not-measured) and
[How to reproduce](#4-how-to-reproduce).

## Steps

1. Read [Method and limits](#1-method-and-limits).
2. Find the element in the [table](#2-table). Trust a row by its source column.
3. Copy the kit column: it names the kit parameters that reproduce the fit.
4. To check a row, run the commands in [How to reproduce](#4-how-to-reproduce).

## 1. Method and limits

**Sources.** `doc` is the public SwiftUI interface in the SDK
(`SwiftUICore.swiftinterface`). `measured` is a probe run; the number after it
is the run count.

**Value curves** (`probe.swift`). An `Animatable` view records its animated
`Double` on every body evaluation (about 180 per second). The series is fitted
with a critically or under-damped spring (`ω0`, `ζ`, time shift) or a cubic
Bézier (duration, shift). `rmse` is in fractions of the travel.

**Layers** (`layers.swift`). A `CADisplayLink` samples `presentation()` of every
layer in the `NSHostingView` tree: opacity, bounds, position, transform, blur
radius and background color. SwiftUI drives these itself; the layer hook of the
brief sees no `CAAnimation` (`animationKeys()` is empty for every transition).

**Pixels** (`pixels.swift`). Text and symbols are drawn into one layer, so the
probe grabs its own window with `CGWindowListCreateImage` on every frame (about
175 per second, no permission needed for an own window) and reduces each frame
to ink (dark area), sharpness (sum of squared horizontal gradients; a blur lowers
it), extents and centroid. Only own-window, white-background captures; no clip
or frame is kept.

**Reduce Motion.** The probes never change a system setting. The argument
`-reduceMotion 1` makes `NSWorkspace.accessibilityDisplayShouldReduceMotion`
and SwiftUI's `\.accessibilityReduceMotion` read `true` in that one process
(`NSUserDefaults` argument domain). The host itself reads `false`.

**Limits.**

- The first visible change comes 8-28 ms after the call: one frame plus the
  commit. It is not a delay; the Delay column is 0 unless it says otherwise.
- The fit shift (0-7 ms) absorbs that frame; durations are fit values.
- The documented `Spring(duration: 0.5, bounce: b)` fits every spring row
  within `rmse` 0.0033. Free fits drift by up to 2 % in `ω0` because of frame
  quantization; use the documented parameters.
- Pixel rows are one run (two when Reduce Motion repeated them); scale is
  derived from ink as `sqrt(ink / ink0)`, which holds for a filled symbol
  (bounding box 39 to 49 pt for ink 1.00 to 1.56).
- Retarget fits use finite differences; they say "carried or reset", not the
  exact velocity.

## 2. Table

Kit parameters: `Spring(d)` is `Animation::Spring { duration_ms: d, bounce }`
and `Spring::new(from, to, Duration::from_millis(d), bounce)`; `period = 2π/ω0`
and `bounce = 1 − ζ` (ch. 14). `ζ` is 1 for every `bounce` 0 row.

### 2.1 Animation values

| Element | Trigger, property | Duration and curve | Delay | Interruptible | Reduce Motion | Source | Kit |
|---|---|---|---|---|---|---|---|
| `Animation.default`, `withAnimation {}` | any animatable change | Spring, `ω0` 12.48 `ζ` 1.000 (rmse 0.0024); documented 0.5 s, bounce 0 is `ω0` 12.57. t10 53, t50 142, t90 321, settle 1 % 540 ms | 0 | Yes, velocity kept | Same curve | measured 4 | No `default`. Use `Animation::Spring { duration_ms: 500, bounce: 0.0 }` |
| `.spring()`, `.smooth`, `.interpolatingSpring()` | same | All three equal `default`: fits 12.53, 12.48-12.54 and 12.57 (rmse 0.0026, 0.0026, 0.0000) | 0 | Yes | Same | measured 1, 3, 1; doc (`Spring.smooth` is `Spring(duration: 0.5, bounce: 0)`) | `Spring { 500, 0.0 }` |
| `.snappy` | same | Spring, `ω0` 12.80 `ζ` 0.866 (rmse 0.0024); documented bounce 0.15: `ζ` 0.85 (rmse 0.0032). Peak 1.004, settle 1 % 367 ms | 0 | Yes | Same | measured 3; doc | `Spring { 500, 0.15 }` |
| `.bouncy` | same | Spring, `ω0` 12.76 `ζ` 0.711 (rmse 0.0022); documented bounce 0.3: `ζ` 0.70 (rmse 0.0030). Peak 1.042, settle 1 % 520 ms | 0 | Yes | Same | measured 2; doc | `Spring { 500, 0.3 }` |
| `.bouncy(duration: 0.8)` | same | `ω0` 7.93 `ζ` 0.707 (rmse 0.0003) | 0 | Yes | Same | measured 1 | `Spring { 800, 0.3 }` |
| `.interactiveSpring()` | same | Spring, `ω0` 44.8 `ζ` 0.906 (rmse 0.0061); documented response 0.15, damping 0.86 is `ω0` 41.9 `ζ` 0.86 (rmse 0.0074). Settle 1 % 121 ms | 0 | Yes | Same | measured 1; doc | `Spring { 140, 0.09 }` (fit) or `{ 150, 0.14 }` (doc) |
| `.easeIn`, `.easeOut`, `.easeInOut`, `.linear` | same | 350 ms each; Bézier (0.42, 0, 1, 1), (0, 0, 0.58, 1), (0.42, 0, 0.58, 1), linear (rmse 0.0000-0.0001) | 0 | Yes, additive (2.2) | Same | measured 1 each | `Animation::Custom { 350, curve }`. `ease_in_ease_out` is exact for `easeInOut`; `Animation::Ease` is a cubic `1 − (1 − p)³`, not the (0, 0, 0.58, 1) Bézier; no named `easeIn`, `easeOut`, `linear` curve |
| `.linear(duration: 1)`, `.easeInOut(duration: 1)`, `.timingCurve(0.2, 0.1, 0.3, 1, duration: 1)` | same | Exactly 1000 ms; Bézier as given (rmse 0.00001) | 0 | Yes | Same | measured 1 each | `Custom { 1000, curve }` |
| `.animation(_, value:)` | the value changes | Same curve as `withAnimation` (`default:implicit` `ω0` 11.93 `ζ` 0.985, rmse 0.0038; `smooth:implicit` 11.80) | 0 | Yes | Same | measured 2 | `use_animated(target, animation)` |

### 2.2 Retarget

| Element | Trigger, property | Duration and curve | Delay | Interruptible | Reduce Motion | Source | Kit |
|---|---|---|---|---|---|---|---|
| Spring retargeted midway (`smooth`, `snappy`, `bouncy`, explicit and implicit) | target changes at 40-126 ms | New spring starts from the shown value and velocity: fit with the velocity carried rmse 0.0020-0.0041, with it reset 0.0074-0.0202 (6 runs); same direction keeps the old speed (4.16 against 4.27 per s) | 0 | Yes | Same | measured 6 | Matches: `use_animated` keeps spring velocity |
| Timed curve retargeted midway (`easeInOut`, 120 ms in, back to 0) | target changes | Animations add: the old curve runs to its end while the new one subtracts. The value rises to 0.591 114 ms after the retarget, then ends at 0 after 320 ms. Additive model rmse 0.018; restart from rest 0.258 | 0 | Yes | Same | measured 1 | Gap: `Ease` and `Custom` restart from rest. Sum `old(t) + (new − old target) × new(t − t1)` |

### 2.3 Transitions

`p` is the animation's progress (0 to 1). Every row moves in step with `p`
(linear under `.linear(duration: 1)`: t10 113, t50 513, t90 914 ms). The view
is 60 by 40 pt in a 200 by 120 pt container.

| Element | Trigger, property | Duration and curve | Delay | Interruptible | Reduce Motion | Source | Kit |
|---|---|---|---|---|---|---|---|
| `if`/`else` with no `.transition` | insert, remove: opacity | `p` in and `1 − p` out. With `default`, t50 145 ms | 0 | Yes | Same | measured 2 (insert, remove) | `Transition::OPACITY`, the default of `Presence` |
| `.transition(.opacity)` | same | Same as the default | 0 | Yes | Same | measured 2 | `Transition::OPACITY` |
| `.scale` | scale 0 to 1 about the center; opacity stays 1 | `p` | 0 | Yes | Opacity only (0 to 1) | measured 2; Reduce Motion 2 | `Transition::SCALE` |
| `.scale(scale: 0.5)` | scale 0.5 to 1 | `p` | 0 | Yes | Opacity only | measured 1 | `Transition::scale(0.5)` |
| `.slide` | x from `−width` to 0 in; 0 to `+width` out; no opacity | `p` | 0 | Yes | Opacity only | measured 2; Reduce Motion 2 | Not built. `Transition::asymmetric(move_edge(Leading), move_edge(Trailing))` is exact |
| `.move(edge: .leading)` | x from `−width` to 0 in; 0 to `−width` out; no opacity | `p` | 0 | Yes | Opacity only | measured 2; Reduce Motion 1 | `Transition::move_edge(Edge::Leading)` |
| `.push(from: .trailing)` | in: x `+width` to 0 and opacity 0 to 1; out: x 0 to `−width` and opacity 1 to 0 | `p` | 0 | Yes | Opacity only | measured 2; Reduce Motion 2 | Not built. `asymmetric(move_edge(Trailing).combined(OPACITY), move_edge(Leading).combined(OPACITY))` is exact |
| `.blurReplace` | opacity 0 to 1, scale 0.90 to 1, blur radius 7 to 0 (out is the reverse) | `p` | 0 | Yes | Opacity only | measured 2; Reduce Motion 1 | Not built; the kit has no blur. `scale(0.9).combined(OPACITY)` drops the blur |
| `.identity` | none | The view appears and goes at once | 0 | n/a | Same | measured 2 | `Transition::IDENTITY` |
| `.asymmetric(insertion: .scale, removal: .opacity)` | in as `.scale`, out as `.opacity` | `p` | 0 | Yes | Opacity only both ways | measured 2; Reduce Motion 2 | `Transition::asymmetric` |
| `.offset(x: 30)` | x from +30 to 0 on insertion | `p` | 0 | Yes | Opacity only | measured 1; Reduce Motion 1 | `Transition::offset(30.0, 0.0)` |
| `matchedGeometryEffect`, one view that moves between two places | x 10 to 150 | `p` (the layout moves it) | 0 | Yes | Same | measured 2 | Not built (ch. 14) |
| `matchedGeometryEffect`, two different views with the same id | both layers move by (+30, +60) pt while opacity crosses 1 to 0 and 0 to 1; sizes do not scale | `p` | 0 | Yes | Same | measured 3 | Not built |
| `matchedTransitionSource` | only with `.navigationTransition(.zoom)` | `ZoomNavigationTransition` is `@available(macOS, unavailable)`: nothing plays on macOS | n/a | n/a | n/a | doc | Not needed |

### 2.4 Layout and values

| Element | Trigger, property | Duration and curve | Delay | Interruptible | Reduce Motion | Source | Kit |
|---|---|---|---|---|---|---|---|
| Stack reflow when a child appears | siblings' x (55 to 30, 105 to 130 pt) | `p`; the new child fades in at its final place (opacity 0 to 1) | 0 | Yes | Same | measured 2 | Missing: an absent `Presence` takes no room, so siblings jump |
| `.frame` and `.padding` change | width 40 to 120, x 80 to 40 | `p` | 0 | Yes | Same | measured 2 | App-level: `use_animated` on the size |
| `.opacity`, `.scaleEffect`, `.rotationEffect`, `.blur` | opacity 1 to 0.2; scale 1 to 2; angle 0 to 90 degrees; blur radius 0 to 10 | `p`, linear in the value | 0 | Yes | Same | measured 1 each | `use_animated`; kit draws no blur |
| `Color`, `fill`, `foregroundStyle` change | fill color | `p` in Oklab: rmse 0.0043 against 0.104 in sRGB and 0.055 in linear light (blue to red, 40 samples) | 0 | Yes | Same | measured 1 | Gap: mix colors in Oklab, not sRGB |

### 2.5 Content transitions and symbols

Pixel rows use a 60 pt font. Times are after the change; "ink" is dark area.

| Element | Trigger, property | Duration and curve | Delay | Interruptible | Reduce Motion | Source | Kit |
|---|---|---|---|---|---|---|---|
| `Text` change, no modifier | text string | Glyphs interpolate (ink stays 600 to 680, sharpness dips 71 to 40 at mid-way); follows the transaction: ends +511 ms with `default`, +1006 ms with `linear(1)`. Same stats as `.contentTransition(.interpolate)` | 0 | Yes | Same | measured 5 | Missing (`ContentTransition::Opacity` is a true crossfade) |
| `.contentTransition(.opacity)` | text string | Crossfade: ink dips 600 to 506 at +447 ms of `linear(1)`, sharpness min 29 | 0 | Yes | Same | measured 1 | `ContentTransition::Opacity` |
| `.contentTransition(.numericText())`, `numericText(value:)` | digit 5 to 6 | Own timing, **not** the transaction's: identical under `default` and `linear(1)`. Motion +25 to +395 ms. Ink falls to 51 % of the start at +80 to +105 ms and sharpness to 0.7 (fully blurred); of the final ink it is 72 % at +193, 86 % at +246, 94 % at +302, 99 % at +420 ms; of the final sharpness 37 % at +246, 92 % at +302. The ink centroid moves within 58.5 to 63.1 pt (about ±2.3 pt, up first) | 0 | Yes | Same | measured 4 (two with Reduce Motion) | Missing (`numericText`, per-glyph) |
| `numericText` 9 to 10 | width grows | Motion +25 to +560 ms; ink min 32 % of the start at +136 ms; of the final ink 54 % at +247, 98 % at +468; width 31 to 68 to 63 pt | 0 | Yes | Same | measured 1 | Missing |
| `numericText(countsDown: true)`, 5 to 6 | digit | Same timing (+23 to +402 ms), rolls down (centroid up to 65.1) | 0 | Yes | Same | measured 1 | Missing |
| `Image(systemName:)` swap, no modifier | symbol | Crossfade on the transaction: ink 787 to 2257 in 330 ms with `default`; settles 1 % at +815 ms | 0 | Yes | Same | measured 2 | Built: `Image::system` crossfades inside `with_animation` by default (`ContentTransition::Opacity` over the transaction's spring) |
| `.contentTransition(.symbolEffect(.replace))`, `.replace.magic` | symbol | About 100 ms and independent of the transaction: ink 787 to 2257 between +35 and +100 ms; sharpness min 42 at +56 ms. `.magic` the same for heart to heart.fill | 0 | Yes | Same | measured 3 | Not built: a blur and scale of the glyph layers, which the kit's one-SVG symbol cannot draw |
| `.symbolRenderingMode` change | rendering mode | One frame (+13 ms), no animation under `default` or `linear(1)` | 0 | n/a | Same | measured 2 | Equal (no animation) |
| `Image(systemName:variableValue:)` change | variable value | Follows the transaction: t90 about 290 ms with `default` | 0 | Yes | Same | measured 1 | Not built: the symbol has no waves to fill |
| `.symbolEffect(.bounce)`, `.bounce.up` | trigger value | Scale 1 to 1.25 at +232 ms, 0.91 at +455 ms, 1 at +650 ms; starts +64 ms | 0 | Not tested | Same | measured 2 each | Not built: scale and offset of glyph layers |
| `.symbolEffect(.pulse)` (discrete) | trigger value | Opacity 1 to 0.30 at +1011 ms, back to 1 at +1940 ms; starts +91 ms | 0 | Not tested | Same | measured 2 | Missing |
| `.symbolEffect(.wiggle)` | trigger value | x offset +2.4, −1.6, +1.2, −1.0 pt at +181, +348, +515, +683 ms (period about 334 ms, decaying); ends +880 ms | 0 | Not tested | Same | measured 2 | Missing |
| `.symbolEffect(.rotate)` | trigger value | One turn lasting about 1.85 s (starts +69 ms, ends +1845 ms); the angle was not extracted | 0 | Not tested | Same | measured 2 | Missing |
| `.symbolEffect(.scale.up)` (active) | `isActive` | Scale 1.056 at +92, 1.186 at +181, 1.251 at +270 ms; holds 1.25 | 0 | Not tested | Same | measured 2 | Missing |
| `.symbolEffect(.appear)`, `.disappear` | `isActive` | Appear: ink 0 to 1 over +42 to +275 ms with scale 0.8 to 1 (ink 0.14 at +88, 0.67 at +177). Disappear: +29 to +176 ms, ink 0.75 at +65, 0.17 at +130 | 0 | Not tested | Same | measured 2 each | Missing |
| `.symbolEffect(.breathe)`, `.pulse` (active), `.variableColor.iterative` | `isActive` | Looping; first change +155 ms (breathe), +92 ms (pulse), +347 ms (variable color). Breathe: ink 0.92 at +460, 0.46 at +1350 ms with the box growing 39 to 47 pt; periods not resolved in a 2 s window. Active pulse: one 2 s dip (ink 0.30 at +1005 ms) in the window | 0 | Not tested | Same | measured 1 each | Missing |

### 2.6 Controls, timelines and scrolling

| Element | Trigger, property | Duration and curve | Delay | Interruptible | Reduce Motion | Source | Kit |
|---|---|---|---|---|---|---|---|
| `ProgressView(value:)` linear, value 0 to 1 | value | AppKit-driven: ignores the transaction (equal under `default` and `linear(1)`). Cubic Bézier (0.42, 0, 0.58, 1) over 531 ms, rmse 0.0043; visible +51 to +490 ms | 0 | Not tested | Same | measured 2 | `Animation::Custom { 530, ease_in_ease_out }` |
| `ProgressView()` linear, indeterminate | continuous | A segment grows from one end, crosses the track and shrinks away at the other: 0.95 s each way, cycle 1.9 s, it holds about 0.1 s at each end; peak ink at mid-way | 0 | n/a | Not tested | measured 1 | `PROGRESS.indeterminate`: 1200 ms (segment 0.3). Gap: measured 950 ms per way |
| `ProgressView()` circular (spinner) | continuous | One revolution in 0.78 s (repeat at 0.766 and 0.803 s); 24 distinct images per revolution (about 32.5 ms each) | 0 | n/a | Not tested | measured 1 | `PROGRESS.spinner`: 1000 ms, 12 spokes. Gap: 780 ms |
| `TimelineView(.animation)` | frame ticks | 123 ticks per s (mean 8.3 ms, min 5.4, max 11.4) while animated bodies run at the 180 Hz display rate; `.animation(minimumInterval: 1/30)` gives 33.4 ms, `.periodic(by: 0.1)` 100.1 ms | 0 | n/a | Same | measured 3 | Missing |
| `PhaseAnimator`, default `animation` | trigger | Each phase uses `Animation.default`: 0 to 1 `ω0` 12.47 `ζ` 1.000 (rmse 0.0034), 1 to 0 `ω0` 12.32. The next phase starts at +504 ms; with `.linear(duration: 0.4)` it starts at +407 ms | 0 | Not tested | Same | measured 2; doc (default is `{ _ in .default }`) | Missing |
| `KeyframeAnimator`, `SpringKeyframe(1)` | trigger | `Spring(duration: 0.5, bounce: 0)`: `ω0` 12.57 `ζ` 1.000 (rmse 0.0001) | 0 | Not tested | Same | measured 1 | `Spring { 500, 0.0 }` |
| `CubicKeyframe(1, duration: 1)`, `LinearKeyframe` | trigger | Cubic: Bézier (1/3, 0, 2/3, 1), 1000 ms (rmse 0.00003). Linear: linear | 0 | Not tested | Same | measured 1 each | `Custom { 1000, curve }` |
| `scrollPosition(id:)` set in `withAnimation(.default)` | programmatic scroll, 240 pt | Ignores the spring: Bézier (0.42, 0, 0.58, 1) over 213 ms (rmse 0.0022); t10 51, t50 112, t90 168 ms. With `.linear(duration: 1)` or no animation the page did not scroll in 2.2 s (seen twice, not explained) | +17 ms | Not tested | Same | measured 2 | Missing |
| `.scrollTransition(.animated)` | page leaves the identity phase | Starts 67 ms after the scroll starts; uses the given animation (`linear(1)`: linear, t50 579 ms). Default is `.default` (doc). Probe effect: opacity 1 to 0.2, scale 1 to 0.5 | 0 (waits for the phase change) | Not tested | Same | measured 2; doc | Missing |

### 2.7 Reduce Motion

| Element | Trigger, property | Duration and curve | Delay | Interruptible | Reduce Motion | Source | Kit |
|---|---|---|---|---|---|---|---|
| `Animation` values (every curve of 2.1) | any change | Unchanged: the same springs and Béziers (4 runs) | 0 | Yes | n/a | measured 4 | The kit sets 0 ms instead (ch. 14): a gap |
| Built-in transitions (`.scale`, `.slide`, `.move`, `.push`, `.blurReplace`, `.offset`, `.asymmetric`) | insert, remove | Geometry and blur are dropped; opacity 0 to 1 and 1 to 0 stay, on the same animation (t50 519 ms of `linear(1)`) | 0 | Yes | Opacity crossfade | measured 10 | Gap: reduced motion should map each `Transition` to `OPACITY`, not to 0 ms |
| Layout, frame, matched geometry, user `scrollTransition`, `numericText`, symbol effects, `replace`, `ProgressView(value:)` | as above | Unchanged in all measured rows (`env accessibilityReduceMotion` read `true` in the probe) | 0 | Yes | Same | measured 18 | Gap: the system keeps these; the kit stops them |

The `-reduceMotion 1` override is process-local; it reaches SwiftUI's
environment but not necessarily every private check of the real system setting.

## 3. Not measured

- The real system Reduce Motion setting (only the process-local override); a
  change of system settings is out of bounds.
- Paging snap and deceleration after a user fling, and `.scrollTransition(.interactive)`
  (no input was posted; documented curve is `.easeInOut` over the scroll distance).
- Why `scrollPosition(id:)` did not scroll under `.linear(duration: 1)` and under no animation.
- `.contentTransition(.identity)`; the angle of `.symbolEffect(.rotate)`; the
  periods of `.breathe`, `.pulse` (active) and `.variableColor`; the
  interruption of a symbol effect; `.symbolEffect(.replace)` directions.
- A glyph morph for `.interpolate`: its ink and sharpness equal the default
  text change, so a difference was not detected.
- `blendDuration` of `.spring(duration:bounce:blendDuration:)`, `repeatForever`,
  `Transaction.disablesAnimations`.
- `numericText` blur radius and roll distance in points (only the ink and
  centroid curves).
- `ProgressView` with other control sizes, and Increase Contrast.
- Navigation, sheets, popovers, menus and toolbar items (other families).

## 4. How to reproduce

All commands run in a scratch directory; build, run and delete the binary.

```sh
P=~/src/probes/swiftui-core
for f in probe layers pixels anim2; do swiftc -O $P/$f.swift -o ./$f; done
./probe c.json default smooth snappy bouncy easeInOut linear1 default:implicit
./probe r.json smooth:explicit:retarget120:0 easeInOut:explicit:retarget120:0
./layers l.json insert:slide remove:push insert:blur reflow fill matched2 scrollpage@default
./pixels p.json numeric:5\>6@default symbol:replace effect:bounce pbar@default spin
./anim2 a.json phase keyspring keycubic timeline
./probe rm.json smooth -reduceMotion 1
python3 -I $P/fit.py c.json; python3 -I $P/exact.py c.json; python3 -I $P/retarget.py r.json
python3 -I $P/summary.py l.json; python3 -I $P/pix.py p.json
rm -f probe layers pixels anim2 *.json
```

A scenario is `<view>[:<arg>][@<animation>]`; the Swift files list the views.
Each probe opens a 100 to 200 pt borderless window at the screen corner for 2
to 4 s per scenario and needs no input. A scenario takes about 3 to 4 s.
