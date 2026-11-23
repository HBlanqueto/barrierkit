# UIKit in BarrierKit: roadmap

**Goal:** fix what UIKit means for BarrierKit, record the owner's decisions of
2026-11-22, and order the work that follows from them.
**Scope:** from the decision that UIKit is a measured reference to the first
three sessions that can start. It covers the animation work (phase 1) in
detail and the later phases as an order. It does not cover an iOS or iPadOS
runner, touch on Linux, a UIKit API in the kit or a UIKit translator: the owner
closed them ([Decisions](#decisions-taken)).
**Snapshot:** `main` at `3512410a`, 2026-11-22. Xcode 27.2, Swift 6.4.2,
macOS 27.2.1 on an Apple-silicon Mac.
**Reader:** the owner, and the engineer who starts one of the sessions.
**Input:** a debate with the owner (four rounds), [macOS arm64](macos-arm-port.md),
[Swift app translation](swift-app-translation.md), [SwiftUI API gap](swiftui-api-gap.md),
[AppKit gap](appkit-gap.md), [macOS animations](macos-animations.md),
[Shader needs](shader-needs.md), [Platform](../barrierkit/18-platform.md),
ADR 0033 and 0085, and eight checks on this Mac ([Evidence](#evidence)).
**Output:** the decisions, the open decisions, the problem list with options,
seven phases with checks, the risks and the first three sessions.

## Steps

1. Read the [verdict](#verdict) and the [decisions taken](#decisions-taken).
2. To pick work, take the first unfinished row of the [phases](#phases).
3. To start a session, copy its brief from [First three sessions](#first-three-sessions).
4. Before a change that the [open decisions](#open-decisions) block, ask the owner.

## Verdict

UIKit is not a platform or an API for this kit. It is a source of measurements.
Malaysia is a desktop, and the owner wants no iOS, no phone and no touch for now.
What UIKit still gives is the look and motion of Apple's own apps, and 14 of
them on this Mac are built on UIKit (Mac Catalyst), so they can be measured here
without a simulator. The first thing to measure is motion, because the owner
names it as the largest visible difference from the Apple apps.

## Evidence

| # | Check | Result |
|---|---|---|
| E1 | `swiftc -typecheck -target arm64-apple-ios27.0-macabi -sdk $(xcrun --show-sdk-path) -F <sdk>/System/iOSSupport/System/Library/Frameworks` on a `UIViewController` subclass | compiles; UIKit builds for Mac Catalyst without a simulator |
| E2 | `otool -L` of each `/System/Applications/*.app` binary, looking for `iOSSupport/.../UIKit` | 14 link UIKit: Books, Clock, FaceTime, Find My, Home, Journal, Maps, Messages, News, Phone, Podcasts, Stocks, Voice Memos, Weather |
| E3 | `otool -L` of Notes | AppKit and SwiftUI, no UIKit: Notes on macOS is not a UIKit app |
| E4 | `xcrun simctl list devices available` and `rustup target list --installed` | no simulator listed; only `aarch64-apple-darwin` and `x86_64-unknown-linux-gnu` installed |
| E5 | `freya-core-0.5.3/src/style/shader.rs`; `grep RuntimeEffect crates/bk/src` | Freya has `ShaderFill` (SkSL); the kit has two SkSL shaders (`glass_container.rs`, `app_icon.rs`) |
| E6 | `crates/bk/src/animation.rs`; [14. Animations](../barrierkit/14-animations.md) | the kit's `Spring` is a duration plus a `bounce` in 0 to 1 with a fixed `ctdesign::SPRING_DAMPING`; no mass, stiffness or velocity; `use_animated` is private |
| E7 | [macOS animations](macos-animations.md) sections 1 and 5 | only three timings are grounded (menu open is instant, window default 250 ms, Dock scale 1.3); every other Malaysia duration is marked unverified |
| E8 | [macOS observations](macos-observations.md): recorders | `screencapture` video runs at about 40.43 fps; an AVFoundation recorder (`~/src/probes/record.swift`) gives 60 fps |

## Decisions taken

All by the owner on 2026-11-22.

| # | Decision | Alternatives the owner passed over |
|---|---|---|
| T1 | **UIKit is a measured reference.** Probes on this Mac (Mac Catalyst) plus a document of idioms and kit gaps. | A UIKit translator; kit types with UIKit names; leaving UIKit out |
| T2 | **Closed: no iOS or iPadOS runner, no phone, no touch, no UIKit API in the kit.** Nothing reopens it by itself; the owner reopens it. | Reopen on touch screens under Malaysia, on an app that asks, or on an iOS release |
| T3 | **The glossary gets a UIKit column.** Same kit types; each row lists its UIKit equivalent. Zero new API. | No column; aliases with UIKit names in code |
| T4 | **Clone targets: Maps and Weather (UIKit, Catalyst) and Notes of macOS (AppKit and SwiftUI).** Notes of iOS is not measured. | Notes of iOS in a simulator |
| T5 | **Phase 1 is animations and transitions**, by two routes: measured tokens, and per-pixel effects with SkSL shaders. | Only tokens; shaders first |
| T6 | **Measure with our own probes plus 60 fps recordings of Maps and Weather.** | Recordings only; probes only |
| T7 | **Freya is worked around with kit-owned `ElementExt` elements**, not forked and not accepted as is. | A patched fork; accepting the limit; deciding per case |
| T8 | **Phase 1 is done when** there is a table of measured tokens, one clone transition matches the real one frame by frame, and the kit has implicit animation. | |
| T9 | **No calendar.** Each milestone is a session of a few hours. | A date; low priority |

## What the owner reported

| Pain | Source |
|---|---|
| Inner animations of the apps are incomplete; transitions are far less smooth than Apple's | owner, round 3 |
| Scroll has no bounce and no inertia | owner, round 3; [AppKit gap](appkit-gap.md) row 20 |
| No gesture layer | owner, round 3; [AppKit gap](appkit-gap.md) row 15 |
| Navigation and presentation | owner, round 3; [SwiftUI API gap](swiftui-api-gap.md) row 8 |
| State: no `Binding`, no responder chain | owner, round 3; [ADR 0085](../decisions.md) |
| Details to take from UIKit: the blur values apps apply, animations, how apps are designed; an app structure like AppKit and Swift | owner, round 2 |

## Problems and options

| # | Problem | Cause | Options | Choice |
|---|---|---|---|---|
| 1 | Transitions are unmeasured and rougher than Apple's | Only three timings are grounded (E7); the kit has one fixed spring (E6); Freya 0.5.3 offers fixed easing functions (`freya-animation-0.5.3/src/easing.rs`, `Function`) | (a) tokens only: cheap, no new API, does not make state changes animate; (b) tokens and implicit animation (problem 2): changes how apps are written, M; (c) b plus shaders | (c), phases U1 to U3 (T5) |
| 2 | No `withAnimation`, `Transaction` or `Transition` | Per-value hooks only ([AppKit gap](appkit-gap.md) row 19; [SwiftUI API gap](swiftui-api-gap.md) row 16) | (a) a SwiftUI-named `Animation` and `with_animation` over `use_animation_transition`; (b) per-control animation only, as today | (a). The semantics of nested and interrupted transactions are unmeasured, so U1 comes first |
| 3 | The spring cannot copy Apple's | `Spring` has `duration` and `bounce`, no velocity (E6). Which parameters Apple's apps use is not verified | (a) keep duration and bounce and carry velocity; (b) add a physical model beside it | Open: O2 |
| 4 | Effects per pixel are limited to fills | Freya has no hook around a subtree; shaders run only as `ShaderFill` (E5, [Shader needs](shader-needs.md) finding 4) | (a) P1, the subtree hook, as an `ElementExt` of the kit; (b) only fills until Freya changes | (a), phase U3 (T7) |
| 5 | Scroll has no overscroll | Freya clamps scroll ([AppKit gap](appkit-gap.md) row 20) | (a) a kit scroll element that owns offset, bounce and deceleration; (b) accept | (a), phase U4. Measured on the Mac path of `UIScrollView`: a quartic ease-out and a stop distance of `157.5 v^(4/3)`, not the documented exponential ([UIKit animations, section 6](uikit-animations.md#6-scroll), [ADR 0104](../decisions.md)) |
| 6 | No gesture layer | Freya has one handler per event; the runner binds no pointer-gestures ([AppKit gap](appkit-gap.md) row 15, D5) | (a) a SwiftUI-named `Gesture` over raw events; (b) raw events | Open: O5; phase U6 |
| 7 | No navigation transitions or `isPresented` | [SwiftUI API gap](swiftui-api-gap.md) row 8 | (a) after implicit animation; (b) before | (a): push and pop are animation first, phase U5 |
| 8 | No `Binding`, no responder chain | ADR 0085 made both non-goals until an app asks | (a) keep; (b) reopen when a clone needs it | Open: O6; the clones of U5 decide |
| 9 | Apple's motion is hard to read from outside | `screencapture` video is 40.43 fps (E8); an app's private curves cannot be read from outside | (a) 60 fps recordings; (b) own probes read `CAAnimation` parameters of views we build; (c) both (T6) | (c). Do not copy private curves ([macOS animations](macos-animations.md) section 6) |
| 10 | The glossary has no UIKit column | [Glossary](../barrierkit/20-glossary.md) lists SwiftUI and AppKit only | (a) add the column (T3); (b) none | (a). `scripts/swift-to-bk/make-table.py` reads the chapter as `c[5]` (line 53): a new column moves it, so the script changes in the same commit |

## Open decisions

| # | Decision | Options | Blocks |
|---|---|---|---|
| O1 | Margin of parity for a transition against the clone | (a) a frame count; (b) a curve error against the recording; (c) the owner's eye | U1 check |
| O2 | The spring model | see problem 3 | U2 |
| O3 | Where the measured motion tokens live | (a) `ctdesign` beside the existing duration tokens, as AGENTS.md asks for new metrics; (b) in `bk` | U1 |
| O4 | Name of the implicit animation API | (a) SwiftUI names: `Animation`, `with_animation`, `Transition`; (b) AppKit: `NSAnimationContext`. The glossary rule says SwiftUI first | U2 |
| O5 | Gesture layer (D5) | [AppKit gap](appkit-gap.md) D5 | U6 |
| O6 | Whether ADR 0085 changes | [Swift app translation](swift-app-translation.md) open decision 2 | U5 |
| O7 | Whether an ADR records T2 | (a) an ADR; (b) this document is the record | first sessions |

## Phases

| U | Output | Check |
|---|---|---|
| U0 | A Catalyst probe harness in `~/src/probes`: a `UIViewController` app built with the macabi target (E1) that runs a transition and records it at 60 fps (E8) | One probe window pushes a view controller and its frames extract with `frames.swift` |
| U1 | `docs/research/uikit-animations.md`: a table of motion per transition (push and pop, sheet and its detents, popover, menu, alert, hover, press, list insert and remove) with duration, curve or spring, source and reliability, from own probes and from Maps and Weather. Tokens in `ctdesign` (O3) | Each row says measured or not; every Malaysia value in the [comparison](macos-animations.md) gets a macOS value or stays marked unverified |
| U2 | Implicit animation in the kit: `Animation`, `with_animation`, `Transition`, interruptible, with velocity (O2, O4); the controls move to it; an ADR | A transition of a probe app matches the table of U1 in duration and curve; reduced motion still gives 0 ms (`use_reduced_motion`) |
| U3 | Effects: P1 and P2 of [Shader needs](shader-needs.md) as kit elements; the first effects driven by a time uniform (edge of a scroll, a glass morph) | An SkSL effect of the kit equals a recorded frame of the real one within the margin of O1 |
| U4 | Scroll physics: a kit scroll element with overscroll, deceleration and, after it, pull to refresh | Scroll of Weather and of a kit clone, recorded at 60 fps, give the same stop distance for the same flick |
| U5 | Navigation and presentation: push and pop with transitions, `isPresented` and detents; the decision on ADR 0085 (O6) | One clone (Maps or Weather) navigates with the measured transition |
| U6 | Gestures: the layer of D5, long press, pan arbitration with scroll | A clone's drag and long press behave as the real one |

The glossary column (problem 10) goes first, in parallel with U0. U3 and U4
start after U1.

## U5 design: navigation and presentation

Decided 2026-11-22 with O6 (ADR 0085 stays; a clone that needs `Binding` or a
close veto reopens it with its own ADR). What is measured is in
[UIKit animations](uikit-animations.md#2-motion-table), Mac Catalyst values.

| Piece | Design | Motion | Status |
|---|---|---|---|
| `NavigationPath<R>` | A `Vec<R>` the app owns in a `State`: `push`, `pop`, `pop_to_root`, `truncate`, `len`. `History` stays for browser-like panes | none | pure, tests first |
| `NavigationStack` | Renders the root and the last route through the destination closure; provides the path by context (`use_navigation_path::<R>()`) so a deep child pushes without a callback chain | A push moves the new page in from the right edge, 100 % of the width, and the old page 30 % of the width to the left, dimmed to a black of 0.13 (one reading) with an edge shadow; both on one critically damped spring of `SPRING_PUSH_MS` (a fit, 250 ms, `bounce` 0). Pop is the same reversed. Reduce Motion: none. A path edited mid-way carries its velocity (`use_animated`) | measured, fit |
| `NavigationLink` | A button that pushes a value | none | after the stack |
| `isPresented` on `Sheet`, `Alert`, `Popover` | A `State<bool>` that opens and closes the component; closing from the window (button, Esc) writes `false` | the compositor or AppKit draws them (a window); the kit animates nothing. The measured sheet (344 ms), alert (274 ms) and popover (210 ms in, 120 ms out) are tokens for Malaysia | sugar over what exists |
| Detents | Not built: a desktop sheet is a window, it has no medium or large. UIKit detents (spring of `SPRING_SHEET_MS`) wait for a touch target (T2 closed) | n/a | deferred |
| `NavigationSplitView` columns | A `SplitView` with a stack per column; the sidebar width and visibility animate as today | `DURATION_BASE` today; the Weather sidebar measured 380 ms, t50 100 ms | not in U5 |

Rejected: a page stack that keeps every lower page mounted (Freya has no
hidden but mounted node; the page below a push is drawn while it moves and
dropped when the push settles, so its state is rebuilt on pop: a limit, not a
choice); transitions for `Sheet` and `Popover` inside the kit; edge-swipe pop
(the gesture layer is U6).

Not measured: the push on macOS SwiftUI (`NavigationStack` in an AppKit window)
and the dimming color beyond one reading.

## U6 design: the gesture layer

Decided 2026-11-22 with O5 (a): a SwiftUI-named layer over Freya's raw pointer
and touch events, in the kit, no fork (T7). Gap 15 and D5 of
[AppKit gap](appkit-gap.md) are its source. Values here are SwiftUI's documented
defaults; none is measured on this Mac yet.

**What Freya gives.** `PointerEventData` is a mouse or a touch; a touch has a
`finger_id` and a phase, so two fingers are two ids. A mouse drag is a pointer
with the button down. There is no pinch, rotate or swipe event: a trackpad pinch
needs `pointer-gestures-unstable-v1` (not bound) or the macOS magnify event
(the runner does not map it). Freya has one handler per event per node, a fixed
500 ms and 5 px for a double click (`EventsCombos`), and no arena: the kit
builds one.

| Piece | Design | Default (SwiftUI) |
|---|---|---|
| Input | `Input { id, phase (down, move, up, cancel), at, time }`, one per pointer or finger | |
| State | `State { Possible, Began, Changed, Ended, Failed, Cancelled }`, the names of `UIGestureRecognizer.State` | |
| `TapGesture` | `count` taps, each up within the slop of its down and `count` taps within the double-click interval | 1 tap, 10 pt, 500 ms |
| `LongPressGesture` | Fires once the pointer has stayed within the slop for the duration; the recognizer reports a `deadline` and the element arms a timer | 0.5 s, 10 pt |
| `DragGesture` | Begins when the pointer has moved the minimum distance; value: start, location, translation, velocity (pt/ms, the window of the scroll tracker) | 10 pt |
| `MagnifyGesture` | Two fingers: the ratio of their distance to the first distance, and its velocity | 0.01 |
| `RotateGesture` | Two fingers: the angle of the line through them, from the first | 1 degree |
| `@GestureState` | `use_gesture_state(initial)`: a `State` that goes back to `initial` when the gesture ends or fails | |
| Modifiers | `.gesture(g)`, `.simultaneous_gesture(g)`, `.high_priority_gesture(g)` on `ViewModifiers`, with `on_changed`, `on_ended`, `updating` on each gesture | |

**Arbitration (pure).** Every gesture on a node competes with the gestures of its
ancestors for the same pointer. The rules are SwiftUI's: the deepest gesture
wins; a `high_priority_gesture` of an ancestor beats its descendants; a
`simultaneous_gesture` never fails another. When a recognizer reaches `Began`,
the others that are not simultaneous with it go to `Failed` (a tap that has
begun to drag is no tap). A recognizer that can still become `Began` keeps
the others from ending: a double tap makes a single tap wait the interval, as
`exclusively(before:)` does. `exclusivity` is data of the arena, a function of
the list of contenders, so it is tested without a window.

**What the scroll element needs (U4).** Its touch pan is a `DragGesture` of the
lowest priority on the scroll view: a child's `TapGesture` or `LongPressGesture`
fails once the pan has moved 10 pt, and the pan's release velocity feeds
`Glide`. UIKit also delays a touch to a child by 150 ms
(`delaysContentTouches`); that is not measured and the kit starts with no delay.
A mouse never pans a scroll view (measured on a Catalyst scroll view: the mouse
drag is ignored by the pan); only a wheel and a touch do. The band of a touch
pan is not the Mac wheel's 0.07: UIKit's documented 0.55 is a hypothesis.

**Touch against pointer.** A gesture takes both. `DragGesture` with a mouse needs
the button; hover is not a gesture (`.on_hover`). Two-finger gestures take only
touches until a runner maps the trackpad pinch (macOS `magnify` and `rotate`,
Wayland `pinch`); the recognizers take their values from `Input`, so a runner
feeds them without a change here.

**Order.** 1. The pure recognizers and the arena, tests first (a table of
inputs and states). 2. The element: a hook that attaches the handlers, arms the
long-press timer and calls the arena; the modifiers. 3. The scroll pan on top of it.
4. A trackpad pinch in each runner. Not measured: the thresholds above on SwiftUI
and UIKit, the 150 ms delay, the touch band.

## Risks

| Risk | Why | Mitigation |
|---|---|---|
| Apple's curves are private | A recording gives samples, not parameters ([macOS animations](macos-animations.md) section 6) | Fit a curve to frames and call it a fit; never copy code |
| Catalyst differs from iOS | Mac Catalyst may scale and restyle controls, so a measure can differ from an iPhone; not verified | Say so in each row; do not claim iOS values |
| Own probes miss Apple's apps | A probe reads its own `CAAnimation`s, not Maps' | Pair every probe row with a recording row (T6) |
| Freya's frame pacing | The loop redraws everything each frame ([Shader needs](shader-needs.md), cost per frame) | Measure a kit transition against a 60 fps recording before tuning a duration |
| Several sessions share the checkout | The sessions on the macOS runner, CSD and the translator edit the same tree | Commit by temporary index; the first sessions write docs and probes, not `crates/` |
| The glossary column breaks the translator | `make-table.py` reads fixed columns and its tests fail while the table is stale | Change both in one commit; run `scripts/swift-to-bk/check.sh` |

## First three sessions

1. **Catalyst motion probes (U0 and U1 start).** Write the harness in
   `~/src/probes` (E1), measure push and pop, sheet, popover, alert and hover,
   record Maps and Weather at 60 fps with `record.swift`, and write
   `docs/research/uikit-animations.md`. Touch no code in `crates/`. Done when
   every row of the table has a measured or an unmeasured mark.
2. **Glossary UIKit column (problem 10).** Add the column to
   `docs/barrierkit/20-glossary.md`, fill it from the iOSSupport headers of the
   SDK (E1), and fix `scripts/swift-to-bk/make-table.py` for the new column.
   Done when `scripts/swift-to-bk/check.sh` passes and `./dev.sh check` is clean.
3. **Spring and implicit animation design (U2, design only).** Decide O2 and
   O4 from the table of session 1 and write the ADR with a sketch of
   `Animation` and `with_animation`. No code. It starts after the first rows of
   session 1.

Each session is its own task; two sessions never edit the same files.

## Not verified

- Whether Maps and Weather draw their transitions with `UIView` animation or
  with SwiftUI inside; only their link to UIKit is checked (E2).
- The deceleration of `UIScrollView` on the touch path and of a real trackpad;
  only the Mac path with posted scroll events is measured. The spring parameters
  of a push are a fit, not a read.
- Whether Mac Catalyst timings equal iOS timings.
- Whether a 60 fps recording is enough to fit a spring; the frame step is
  16.7 ms.
- Whether `make-table.py` reads glossary columns elsewhere than the lines
  checked (`GLOSSARY` and `c[5]`).
