# macOS motion: standard controls

**Goal:** record which animations the standard controls of SwiftUI and AppKit
play by default on macOS, with numbers the kit can reproduce.
**Scope:** buttons, switch, checkbox, radio, slider, stepper, segmented control,
menus and pickers, text fields and the focus ring, progress, gauge, disclosure,
SF Symbol effects and the SwiftUI default animation. It stops at what the
controls draw inside a window. Tooltips, menus opening, sheets and the desktop
shell are other families.
**Snapshot:** macOS 27.2.1 (26B131), Apple-silicon Mac, 2560x1440 display at 1x
and 180 Hz, Light appearance, 2026-11-25. System Reduce Motion is off; it was
never changed (see [Reduce Motion](#reduce-motion)).
**Reader:** anyone who changes a duration, spring or press behaviour of a kit
control.
**Input:** the in-process probes in `~/src/probes/controls` (not in the repo) and
the public SwiftUI interface in the SDK.
**Output:** the [animated rows](#2-animated), the [instant rows](#3-no-animation),
the kit comparison, what is not measured and the commands to reproduce.

## Steps

1. Read [Method](#1-method). Every value is one run unless a range is shown.
2. To copy a motion, take its row in [Animated](#2-animated). A value marked
   *fit* describes the motion; it is not Apple's parameter.
3. To decide that a control needs no animation, check [No animation](#3-no-animation).
4. To repeat a measurement, run the commands in [How to reproduce](#6-how-to-reproduce).

## 1. Method

One Swift process builds a borderless 320x120 pt window and, per case, places one
control (AppKit, or SwiftUI in `NSHostingView`). A step changes one state. The
probe then reads three things:

- **Pixels.** `CGWindowListCreateImage` of the window on every display-link tick.
  The link ticks at 180 Hz (5.6 ms). The progress `p` of a frame is its projection
  on the difference between the frame before the step and the last frame of the
  step; `t10`, `t50`, `t90` are the first times `p` reaches 0.1, 0.5 and 0.9,
  and `settle` is the last time `p` is farther than 0.01 from 1. A delay of
  6 ms or less is the capture latency: **instant** below means the change fits
  between two captures.
- **Layers.** The presentation layer of every layer under the control, to get
  one clean value (a knob position) instead of a blend.
- **Core Animation.** `CALayer add:forKey:` is hooked to read every
  `CAAnimation`. Almost no control uses one: AppKit steps its own values, so the
  pixels are the source for nearly every row.

Input is real: `CGEvent` mouse and key events posted from the shell into the
probe window (hover, press, drag, type), the window made key, another window
ordered front for "key window inactive". A case is repeated when the window
lost key status or the pointer was moved by another process.

**Limits.**

- Pointer events have 6 to 12 ms of latency; a press that reads "first 40 ms"
  is the control, not the event.
- A blend `p` mixes every moving part (knob, track, shadow). Fits of `p`
  are poor for multi-part motions (glass buttons); then only `t10`/`t50`/`t90`/`settle`
  are given. A fit is a delay plus a damped spring (`ω0`, `ζ`) or a delay plus a
  duration and a cubic Bézier; the one with the smaller rmse is shown.
- A click is a press at 0 ms and a release 60 to 80 ms later; rows say which.
- The window is 1x and not on a scrolling backdrop. Dark appearance, Increase
  Contrast and a second refresh rate were not measured.
- Reduce Motion is set per process by the launch argument `-reduceMotion 1`
  (`NSWorkspace.accessibilityDisplayShouldReduceMotion` reads `true`); the
  system setting is untouched. The argument was applied to a subset only.

## 2. Animated

`RM` is the behaviour with `-reduceMotion 1`; "same" means unchanged. Kit status
is checked against `crates/bk` and `crates/ctdesign`.

| Element | Trigger | Property | Duration and curve | Delay | Interruptible | RM | Source | Kit |
|---|---|---|---|---|---|---|---|---|
| **Switch** (`NSSwitch`, SwiftUI `.switch`; the same control) | Value changes: click release, `state =`, SwiftUI binding with or without `withAnimation` | Knob x (travels 18 pt: 19.9 to 1.9), track color, knob shadow | Knob: spring fit `ω0` 15.0, `ζ` 0.75 (rmse 0.010), 2.6 % overshoot; layer t10 58, t50 125, t90 214, settle 303 ms. Pixels, AppKit click release: t50 158, settle 252; SwiftUI on: t50 137, settle 415; off: t50 98, settle 304 | 26 to 42 ms | Yes: two clicks 80, 150 and 300 ms apart reverse from the shown value with no jump (peak change 12.6, 18.0, 22.6 of 22.5) | Same | measured, 3 runs | `SWITCH.transition_ms` = `duration.fast` 120 ms, ease out cubic: about 2.5 times shorter, no overshoot |
| Switch press | Mouse down | Knob grows and turns to glass | t10 70, t50 131, t90 182, settle 204 ms (SwiftUI: 55, 116, 183, 205); spring fit `ω0` 17.5, `ζ` 0.70 (rmse 0.03, poor) | 36 to 42 ms | Not measured | Gone: no lift (instant) | measured | `knob_pressed_scale` 1.65 on the 120 ms ease out |
| **Glass button** release (`.glass` bezel or `buttonStyle(.glass)`) | Mouse up after a press | Glass lift, shadow | t10 8 to 35, t50 68 to 74, t90 146 to 147, settle 369 to 370 ms; no single-curve fit (rmse 0.025) | 8 to 29 ms | Not measured | Gone: settle 6 to 17 ms | measured, 3 runs | Native glass: the kit hosts an interactive `NSGlassEffectView` (macOS 27) and adds nothing; macOS plays the lift. Without it (before macOS 26, Reduce Transparency) the fill changes only |
| Glass button, accent tint (AppKit `bezelColor`) release | Mouse up | Same | t50 72, t90 106, settle 122 ms; Bézier fit (0.69, 0.57, 0.43, 0.79) over 107 ms (rmse 0.013) | 22 ms | Not measured | Gone | measured | Native glass: the kit hosts an interactive `NSGlassEffectView` (macOS 27) and adds nothing; macOS plays the lift. Without it (before macOS 26, Reduce Transparency) the fill changes only |
| **Prominent glass** (`.glassProminent`) press | Mouse down | Glass squash | t10 15, t50 37 to 40, t90 78 to 81, settle 226 to 229 ms | 10 ms | Not measured | Gone | measured, 2 runs | Native glass: the kit hosts an interactive `NSGlassEffectView` (macOS 27) and adds nothing; macOS plays the lift. Without it (before macOS 26, Reduce Transparency) the fill changes only |
| Prominent glass release | Mouse up | Same | t10 23, t50 62, t90 95, settle 140 ms | 10 ms | Not measured | Gone | measured, 2 runs | Native glass: the kit hosts an interactive `NSGlassEffectView` (macOS 27) and adds nothing; macOS plays the lift. Without it (before macOS 26, Reduce Transparency) the fill changes only |
| **Slider** thumb press and release (`NSSlider`, SwiftUI `Slider`) | Mouse down, mouse up on the thumb | Thumb grows, then shrinks | Press t50 69 to 75, t90 119 to 131, settle 142 to 153 ms; release t50 21 to 64, t90 87 to 120, settle 138 to 153; Bézier fit (0, 0.13, 0.53, 1.11) over 143 ms (press, rmse 0.014) | 20 to 36 ms | Not measured | Not measured | measured | `SliderStyle::press_animation`: `Custom` with `slider_press`, the fitted Bézier, over `SLIDER.press_ms` 143 ms; matches |
| Slider click on the track | Click away from the thumb | Thumb and fill move to the click | t10 34 to 40, t50 106 to 112, t90 207 to 213, settle 324 to 335 ms; spring fit `ω0` 18, `ζ` 0.95 (rmse 0.012); Bézier (0.46, 0.92, 0.51, 0.91) over 311 ms | 5 to 23 ms | Not measured | Not measured | measured, 2 runs | `SliderStyle::animation`: `Spring` of `SLIDER.click_ms` 349 ms, bounce `SLIDER.click_bounce` 0.05 (ω0 18, ζ 0.95); a drag is 1:1; matches |
| Slider value set in code | `doubleValue =` (AppKit), binding change (SwiftUI) | Thumb, fill | Instant (see [No animation](#3-no-animation)). Only an explicit animation animates it: `NSAnimationContext` 0.25 s with `animator()`: settle 242 ms, Bézier (0.15, 0.22, 1, 0.91) over 241 ms (rmse 0.011); SwiftUI `withAnimation {}`: the default animation below | 5 to 34 ms | Not measured | Not measured | measured | instant: the kit animates only inside `with_animation`; key presses are not measured and stay instant; matches |
| **Progress bar**, determinate (`NSProgressIndicator`, SwiftUI `ProgressView(value:)`) | The value changes, with or without `withAnimation` | Fill width | t10 113, t50 253 to 292, t90 392 to 431, settle 453 to 498 ms; Bézier fit (0.51, 0.29, 0.56, 0.91) over 428 ms (SwiftUI, rmse 0.005), (0.57, 0.47, 0.45, 1.00) over 489 ms and (0.48, 0.25, 0.48, 0.74) over 406 ms (AppKit, rmse 0.005 to 0.008) | 45 to 90 ms | Not measured | Same | measured, 5 runs | `PROGRESS.fill_ms` 428 ms along `PROGRESS.fill_curve` (0.51, 0.29, 0.56, 0.91), on every value change and ignoring `with_animation`; the delay is not copied |
| **Progress bar**, indeterminate | The view is shown and animating | A blue segment about 40 % of the width travels left to right with a fading tail and wraps | Loop: 1900 ms (autocorrelation of the pixels, one run each for AppKit and SwiftUI) | none | n/a | Not measured | measured | `PROGRESS.indeterminate_ms` 1900 ms, one way, restarting; a segment of `PROGRESS.indeterminate_segment` (0.4) with a hard head and a tail that fades from transparent, growing in from the left and shrinking out at the right. Not copied: the head crossing in 25 % of the sweep with a slower tail (the tail speed is not measured) |
| **Spinner** (`NSProgressIndicator` spinning, SwiftUI `.circular` indeterminate) | The view is shown and animating | Sprite frame | `CAKeyframeAnimation` on `contentsRect`, 0.8 s, 24 stepped keyframes (33.3 ms each, read from the hook); the pixels repeat every 800 ms | none | n/a | Not measured | measured (read) | `PROGRESS.spinner_ms` 800 ms, 8 spokes, `spinner_frames` 24: the fade moves one third of a spoke per step |
| Level indicator | `animator().doubleValue` | Fill | t50 91, t90 214, settle 208 ms (fit poor, rmse 0.04) | 12 ms | Not measured | Not measured | measured | `LevelSlider` is a slider, not a level indicator, and follows the slider rows; the capacity indicators set the value instantly, as does `NSLevelIndicator` without `animator()`; not applied |
| **Outline view** row expand (`NSOutlineView`) | `animator().expandItem`, click on the triangle | Rows slide in (`NSTableRowsClipView.bounds` is animated), the triangle turns | `animator()` t10 52, t50 102, t90 230, settle 242 ms; a click: settle 270 to 287 | 12 ms | Not measured | Same (settle 242) | measured, 2 runs | `OutlineView`: expansion `ease_out_appkit` 200 ms, collapse and chevron instant; matches |
| **SwiftUI default animation** (`withAnimation {}`) | Any change inside `withAnimation` with no argument: a view appearing or leaving, text swap, a slider value | Opacity, content, fill | Critically damped spring fit `ω0` 13.5 to 16, `ζ` 1.05 to 1.20 (rmse 0.008 to 0.011): t10 47, t50 136 to 145, t90 314 to 324, settle 509 to 514 ms. Apple documents `spring(duration: 0.5, bounce: 0)` (`ω0` 12.57, settle 528 ms); the fit agrees within 10 to 25 % of `ω0` | 7 to 23 ms | Yes, SwiftUI retargets (not re-measured here) | Not applied: Reduce Motion does not shorten it (settle 509 ms) | measured, 6 runs; doc | `Spring` and `with_animation` ([animations](../barrierkit/14-animations.md#spring)); the kit's `SPRING_*_MS` differ |
| **`DisclosureGroup`** | Expanded state set inside `withAnimation` | Chevron turns 90 degrees, content appears or leaves | Same curve as the default animation: t50 145 to 153, t90 318 to 320, settle 546 to 548 ms (the content size drives the pixels) | 25 to 28 ms | Not measured | Not applied (settle 546 ms) | measured | `DisclosureGroup`: content and chevron on `ease_default`, 248 ms; matches |
| **SF Symbol swap** (`Image(systemName:)` changes) | Inside `withAnimation {}` | Cross-fade of the two glyphs | t10 46, t50 140, t90 252, settle 290 ms | 18 ms | Not measured | Not measured | measured | `Image::system` crossfades inside `with_animation` over the transaction (`ContentTransition::Opacity`), instant outside |
| Symbol `.contentTransition(.symbolEffect(.replace))` | The symbol name changes, no `withAnimation` | Replace effect | t10 28, t50 46 to 50, t90 69 to 73, settle 74 to 78 ms (7 intermediate frames) | 10 to 19 ms | Not measured | Not measured | measured, 2 runs | Not built: per-layer glyph rendering and blur (a 78 ms blur and scale of the glyph layers); not faked |
| Symbol with `variableValue` | The value changes, with or without `withAnimation` | Number of filled waves | t10 63 to 69, t50 163 to 169, t90 269 to 275, settle 302 to 342 ms | 18 to 36 ms | Not measured | Not measured | measured, 2 runs | Not built: per-layer glyph rendering and blur (one SVG per symbol has no waves) |
| Symbol `.symbolEffect(.bounce, value:)` | The value changes | Scale and offset, then back | A transient: the change from rest peaks about 219 ms after the trigger and ends at rest | n/a | Not measured | Not measured | measured, 2 runs | Not built: per-layer glyph rendering and blur (the glyph's layers cannot be scaled and offset apart) |
| Symbol `.symbolEffect(.pulse, isActive:)` | `isActive` true | Opacity loop | Period 2000 ms | n/a | n/a | Not measured | measured | Not built: nothing in the kit loops a symbol; a 2000 ms opacity loop would be the whole effect |
| **Gauge**, circular (`.accessoryCircular`) | The value changes inside `withAnimation` | Arc | t10 30, t50 64, t90 475, settle 598 ms; no single-curve fit | 24 ms | Not measured | Not measured | measured | `GaugeStyle::animation` is `None`: the marker of the open ring follows a new value only inside `with_animation`, with the app's animation (approximation: the measured curve has no single-curve fit) |
| **Text caret** (`NSTextInsertionIndicator`, read from the hook) | A text field has the focus | Opacity loop | `CAAnimationGroup` 1.0 s: opacity 0 to 1 in 5 steps over 0.65 s, then 1 to 0 in 5 steps over 0.35 s | none | n/a | Not measured | measured (read) | not checked |
| **Focus ring** (`NSTextField`, SwiftUI `TextField`) | Focus by a click | Ring opacity and size | A click: t10 125 to 156, t50 223 to 231, t90 296 to 303, settle 290 to 297 ms (the caret is masked out; one run each). SwiftUI `@FocusState` set in code: the ring is there within 35 ms, no fade (strip, one run). AppKit `makeFirstResponder`: the ring shows at 40 ms, drops out at about 200 ms, then a larger soft ring contracts to the tight ring from 260 to 460 ms; not reproduced in SwiftUI and not explained | 35 to 90 ms | Not measured | Not measured | measured, one run each | `FOCUS_RING` is drawn at once (no animation) |

## 3. No animation

Each of these changed between two captures (about 6 to 17 ms, the latency of the
probe); no intermediate frame was seen. They are findings: the kit must not add a
transition.

| Element | Trigger | Finding | Kit |
|---|---|---|---|
| Bordered, prominent, borderless, plain, link, toolbar, accessory-bar, circular, help, badge buttons (AppKit and SwiftUI) | Enabled to disabled and back | Instant fill and text change | Matches |
| The same buttons | Mouse down and up | Instant tint; no scale (`NSButton` has none) | `BUTTON.pressed_scale` 1, matches |
| The same buttons | Pointer enters and leaves | No visible change: standard push, bordered, borderless, link and SwiftUI buttons have no hover state. Only the glass buttons and `showsBorderOnlyWhileMouseInside` change, in one frame | Matches |
| Default button (`keyEquivalent` return, `.keyboardShortcut(.defaultAction)`) | 3 s idle | No pulse: the pixels did not change at all | Matches |
| All controls above | The window loses or regains key | Tint changes in one frame; no cross-fade (checked on buttons, default button, switch, segmented, slider, stepper, radio, checkbox, prominent and glass) | Matches |
| Checkbox and radio (`NSButton`, SwiftUI `.checkbox`, radio group) | Click, state set in code, mixed state | Mouse down tints, mouse up draws the mark in one frame; no check drawing animation | Matches |
| Toggle button style, push-on-push-off | Click | Instant | Matches |
| **Segmented control** (`NSSegmentedControl` automatic, capsule, rounded; SwiftUI `.segmented`) | Click a segment, set in code, with or without `withAnimation` | The selection jumps to the new segment in one frame: no slide. The pressed segment tints at mouse down | Matches |
| Stepper, `Menu`, pop-up and pull-down button, `ControlGroup`, `ColorPicker`, `DatePicker` field | Disable, enable, press, hover, arrow key | Instant | Matches |
| `NSOutlineView` | Programmatic expand and collapse; `animator()` collapse; a click on the triangle to collapse | Instant. Only expand animates | Matches |
| Slider, progress (circular determinate), level indicator, switch | Value set in code (slider, level, circular progress) | Instant; the switch is the exception and animates (see above) | Slider and level: matches |
| `Gauge` (linear capacity) | Value change, even inside `withAnimation` | Instant | Matches (`Gauge::capacity(true)` and `ProgressRing` never animate) |
| SwiftUI view appearing or text swapping | Change with no `withAnimation` | Instant | Matches |
| Symbol swap | Change with no `withAnimation` | Instant | Matches |
| Text field | Typing, deleting, placeholder | No fade of the placeholder seen; the clear button of a search field was behind the mask and is not measured | n/a |
| Text field focus lost | Blur, in code | The ring vanishes in one frame | Matches |
| `DisclosureGroup`, `NSButton` disclosure triangle | State set in code (no animation) | Instant. The triangle button turns instantly on mouse up (one frame) | Matches |

## 4. What the table says

1. **Controls are not animated by default.** Most state changes are one frame. The
   kit's press, hover, disabled and inactive handling already matches.
2. **Five things animate, and the kit has only some of them.** The switch (about 300
   ms, spring-like, with a lift), the glass button release (about 370 ms),
   the slider thumb, the determinate progress bar (about 450 ms) and the
   expanding outline row.
3. **The kit's switch is too fast and too stiff.** 120 ms with no overshoot
   against 300 ms with 2.6 % overshoot. The progress loops differ too: 1900 ms
   for the bar and 800 ms for the spinner, against 1200 and 1000.
4. **Reduce Motion is not uniform.** Glass press and release animations stop.
   The switch, the progress bar, the outline expansion and everything
   SwiftUI drives with `withAnimation` do not. The kit maps Reduce Motion to 0 ms
   for everything, which is stricter than AppKit and SwiftUI.
5. **The default SwiftUI animation is a critically damped spring of about 0.5 s.**
   It is what a bare `withAnimation {}` does to a symbol, a text, a slider fill
   and a disclosure group.

## 5. Not measured

- Increase Contrast, Dark appearance, other refresh rates.
- Reduce Motion for the slider, the focus ring, indeterminate loops, symbol
  effects and the gauge (the launch argument was applied to the buttons, switch,
  progress bar, outline, disclosure, segmented, view and text swaps only).
- Click on the SwiftUI `DisclosureGroup` (the click on its label did not toggle
  in the probe) and the chevron turn alone.
- Search field: expansion (needs a toolbar), the clear button fade (hidden by the
  mask), `SecureField` beyond the text-field cases.
- `NSColorWell`, `NSDatePicker` calendar and clock, `NSPathControl`, toolbar item
  states and `ControlGroup` in a toolbar, stepper hold-to-repeat.
- Velocity carry-over when a spring is interrupted (the switch shows no jump; the
  slope was not fitted).
- Whether `Animation.default` is exactly `spring(duration: 0.5)`: the SDK interface
  documents the spring defaults but not `default`.
- Tooltips (another family).

## 6. How to reproduce

The sources are in `~/src/probes/controls` and are not part of the repository
(`h.swift` harness, `m.swift` mouse helpers, `appkit_*.swift`, `swiftui*.swift`,
`extra.swift`, `an.py`, `fit.py`, `ser.py`, `period.py`). Each run takes the
shared screen lock `/tmp/barrierkit-screen.lock`.

```sh
cd ~/src/probes/controls
swiftc -swift-version 5 -O h.swift m.swift appkit_buttons.swift appkit_more.swift \
  swiftui.swift swiftui2.swift extra.swift main.swift -o "$TMPDIR/ctl"
./run.sh "$TMPDIR/ctl" ak-switch sw-toggle-switch > "$TMPDIR/run.log"   # case filters
./run.sh "$TMPDIR/ctl" -reduceMotion 1 ak-btn-glass > "$TMPDIR/rm.log"  # per-process Reduce Motion
python3 -I an.py "$TMPDIR/run.log"               # t10/t50/t90/settle per step
python3 -I fit.py "$TMPDIR/run.log" ak-switch animOff   # spring and Bézier fit
python3 -I period.py "$TMPDIR/run.log" ak-progress-indet2 observe
```

`STRIPS="case/step@t1,t2,...;..." PNGDIR=dir` saves a zoomed strip of frames at
those times. The binaries, logs and strips were deleted after the numbers were
taken.
