# macOS desktop animations

**Goal:** record which macOS animations were observed, which timings are
reliable, and which Malaysia durations are still unverified.
**Scope:** menu open, Dock magnification, notification motion, space slides,
window open and close, and launcher transitions.
**Reader:** anyone who changes a duration, spring or transition token.
**Input:** 60 fps recordings, frame extraction and the public
`NSAnimationContext` default.
**Output:** a reliability table, a Malaysia comparison and a measurement
plan.

## Steps

1. Read the recorder limits in
   [`macos-observations.md`](macos-observations.md).
2. Trust a duration only if it is marked measured.
3. Use 60 fps frames, not the 40.43 fps `screencapture` video.
4. Compare a duration and a curve together; a duration alone is not a
   specification.
5. Do not copy Apple's animation code or private curves.

---

## 1. What is reliable

| Animation | Observation | Reliability |
|---|---|---|
| Menu panel open | Appears between two consecutive 60 fps frames | Measured; effectively instant |
| Menu panel close | Fade of about 183 ms in a Catalyst menu ([UIKit animations](uikit-animations.md#2-motion-table)) | Measured, one run |
| Window open and close | `NSAnimationContext` default 250 ms | Measured default; curve not measured |
| Dock magnification | About 1.3x at rest under the pointer | Measured size; timing not reliable |
| Notification enter | Not measured | Unknown |
| Notification exit | Not measured | Unknown |
| Mission Control open | Not measured | Unknown |
| Space slide | Not measured | Malaysia only |
| Launcher open and close | Not measured | Malaysia only |
| Lock fade and rise | Not measured | Malaysia only |

The menu panel is the clean result: it does **not** animate. A 60 fps
recording showed the panel in the first frame and absent in the previous one.
The earlier note about `0.53 s` is the time from the recorder start to the
action, not the animation duration.

## 2. Window default

`NSAnimationContext` reports a default duration of **250 ms** on this host.
That is the value AppKit uses for window open, close and many implicit
animations unless an app overrides it.

No frame-by-frame window recording was captured, so the curve is unknown. Do
not assume a spring. AppKit's default is not the same as a SwiftUI spring.

## 3. Dock magnification

The Dock measured about **1.3x** under the pointer; the icon geometry lives
in
[macos-dock.md](macos-dock.md#4-icons-magnification-and-label). The scale
matches Malaysia's `DOCK_MAGNIFY_MIN = 1.3`, so this file only needs the
timing.

The timing is not reliable:

- `record.swift` starts about 1.5 s after the command.
- The cursor was moved after the recording started in some runs.
- One recording showed the icon at 88x102 at 0.5 s and 87x102 at 1.167 s.
  That is not enough to fit a curve.
- Another run showed no magnification at all because the cursor had moved
  back before the action frame.

Conclusion: the **scale floor** is known; the **duration, curve and
overshoot** are not.

## 4. Other surfaces

- **Notification:** the banner was captured still. The slide, bounce and
  fade were not recorded.
- **Mission Control:** the overview was captured still. The open and close
  transitions were not recorded.
- **Spaces:** the slide duration and easing are Malaysia values, not
  measured macOS values.
- **Launcher:** Spotlight and Launchpad open and close were not recorded.
- **Lock:** not captured.

## 5. Comparison with Malaysia

| Element | macOS | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Menu open | Instant | No menu animation | `menubar/menu.rs` | Matches |
| Window default | 250 ms | `duration.base` 250 | `tokens.duration` | Matches |
| Dock magnify scale | about 1.3 | 1.3 | `DOCK_MAGNIFY_MIN` | Matches |
| Dock magnify duration | settle 117 ms in, 133 ms out | 120 | `DOCK_MAGNIFY_MS` | Matches ([UIKit animations](uikit-animations.md#3-malaysia-comparison)) |
| Dock hide | Not measured | 280 | `DOCK_HIDE_MS` | Unverified |
| Dock minimise | Not measured | 500 | `DOCK_MINIMIZE_MS` | Unverified |
| Dock slot | Not measured | 400 | `DOCK_SLOT_MS` | Unverified |
| Dock bounce | Not measured | 620, 0.55 | `DOCK_BOUNCE_*` | Unverified |
| Notification in | Not measured | 450, bounce 0.15 | `NOTIFICATION_SLIDE_IN_MS` | Unverified |
| Notification out | Not measured | 260 | `NOTIFICATION_SLIDE_OUT_MS` | Unverified |
| Launcher open | Spotlight t90 81 ms | 180 | `LAUNCHER_OPEN_MS` | Malaysia about twice as long |
| Launcher close | Spotlight t90 64 ms, settle 83 ms | 120 | `LAUNCHER_CLOSE_MS` | Malaysia about 1.5 times as long |
| Space slide | Not measured | 480 cubic bezier | `SPACE_SLIDE_MS` | Unverified |
| Lock fade | Not measured | 1100 | `LOCK_FADE_MS` | Unverified |
| Lock rise | Not measured | 900, 120 offset | `LOCK_RISE_*` | Unverified |
| Lock login | Not measured | 600 | `LOCK_LOGIN_MS` | Unverified |

Five rows are grounded: menu open is instant, the window default is 250 ms,
the Dock magnification floor is about 1.3, its duration is about 120 ms, and
Spotlight opens and closes in about 100 ms and 80 ms (one run each, in
[UIKit animations](uikit-animations.md#3-malaysia-comparison), which also
measures the menu close). Everything else in the Malaysia column is a project
choice that still needs a macOS reference.

## 6. What not to copy

- Apple's private animation implementations.
- Apple's exact spring constants without measuring the public result.
- A duration without its curve and its reduced-motion fallback.
- A capture rate below 60 fps for a short transition.

## 7. Measurement plan

1. Start `record` and wait at least 2 s before the action.
2. Keep the cursor still for Dock magnification.
3. Use a fixed action sequence: open, hold, close.
4. Extract frames at 60 fps.
5. Measure the first and last frame of the change, not the action time.
6. Fit a curve to at least five points.
7. Repeat with Reduce Motion on.

## 8. Prioritized changes

1. **Keep the menu instant.** Do not add an open animation to match a
   generic duration.
2. **Measure Dock magnification next.** It is visible on every hover and the
   scale floor is already known.
3. **Measure notification enter and exit.** They are frequent and currently
   unverified.
4. **Measure Mission Control.** It is a new mode and needs a transition
   specification.
5. **Add Reduce Motion tests for every duration.** A long fade that ignores
   Reduce Motion is a regression.

## 9. Open questions

- Is the macOS window open a spring or an ease?
- Does the Dock magnification overshoot?
- Does the notification banner use a spring or a cubic curve?
- Does Mission Control scale the windows from their desktop rectangles?
- Which transitions change under Reduce Motion?
