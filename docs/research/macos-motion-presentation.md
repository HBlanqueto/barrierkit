# macOS motion: sheets, alerts, popovers, menus and drag

**Goal:** record how long each transient or presented element of a SwiftUI or
AppKit app moves by default on macOS, and with which curve, so the kit can copy
it.
**Scope:** sheet, alert and confirmation dialog, popover, menus (pop-up button,
context menu, `Menu`, submenus), combo box, tooltip, find bar, drag and drop,
spring loading, Quick Look, the completion list and the color and date
pickers. The desktop shell (Dock, Mission Control, notifications) is out of
scope, and so is any system setting change.
**Snapshot:** macOS 27.2.1, Xcode 27.2, Apple-silicon Mac, 2560x1440 at 180 Hz,
Dark appearance, Reduce Motion off (`NSWorkspace.accessibilityDisplayShouldReduceMotion`),
2026-11-25.
**Reader:** whoever sets a duration, spring or delay token of a kit sheet,
alert, popover, menu or tooltip.
**Input:** the probe in `~/src/probes/presentation` (outside the repo) and the
notes in [macOS animations](macos-animations.md) and
[UIKit animations](uikit-animations.md).
**Output:** [the table](#2-motion-table), the [kit comparison](#3-kit-comparison),
[what is not measured](#4-not-measured) and [how to reproduce](#5-how-to-reproduce).

## Steps

1. Read [Method](#1-method). Every number is a fit of samples, not an Apple
   parameter.
2. Take the row of the element in [the table](#2-motion-table). Trust it by
   its source column.
3. To change a kit token, take its row in [the kit comparison](#3-kit-comparison).
4. To repeat a row, run the commands in [How to reproduce](#5-how-to-reproduce).

## 1. Method

**Window server, not video.** The probe is one AppKit app (`pres.swift`; the
popover, menu, alert and sheet are real AppKit windows of that process; the
SwiftUI rows host a `View` in an `NSHostingView`). A background thread polls
`CGWindowListCopyWindowInfo` for the windows of the process about every 5 ms and
logs every change of bounds, alpha and visibility. That sees animations the
`NSWindow` model hides: AppKit sets the final frame at once and the window
server plays the motion. The kit's `NSWindow.frame` stays constant during a
sheet; the polled bounds do not.

**Pixels where the window does not change.** For highlights, the find bar and
the drag image, a second thread captures a rect of the screen with
`CGWindowListCreateImage` at about 65 frames per second (16 ms steps) and a
script reads pixels. A `CAAnimation` hook on `CALayer.add(_:forKey:)` saw no animation that
carries a window's motion: it is in the window server.

**Pointer and keys** are posted by a second binary (`mouse.swift`) with
`CGEvent`. The probe logs the pointer position every millisecond, so delays are
counted from the pointer entering a view, not from the tool's start.

**Words.** `t10`, `t50` and `t90` are the times after the first change at which
the value has travelled 10 %, 50 % and 90 % of its way. `settle` is the last
time it is farther than 1 % from its end. `cb(x1,y1,x2,y2)` is a cubic Bézier
fitted to the samples, with its length; `spring` is a fitted damped spring
(`ω0` in rad/s, `ζ`); a SwiftUI `Spring(duration:bounce:)` has `duration` 2π/ω0
and `bounce` 1 − ζ. A fit describes the motion; it is not Apple's parameter.
`alpha` is the window alpha of the window server. 1 frame is at most 16 ms.

**Limits.**

- Window bounds are whole points, so a 32 pt travel has 32 levels. Alpha and
  scale are continuous.
- The window server starts a motion about 13 ms after the call.
- One screen, one pointer, shared with other sessions: the probe held the
  `/tmp/barrierkit-screen.lock` lock for every run.
- Reduce Motion, Increase Contrast and Reduce Transparency were not changed (the
  brief forbids it), so no row has a reduced-motion value.
- `beginSheet` and `beginSheetModal` keep the main thread busy until the present
  animation ends (a 120 ms timer fired 280 ms late), so an app cannot interrupt
  a sheet or sheet alert while it appears.

## 2. Motion table

Window-server rows give the animated property of the window itself. "RM" is the
behavior with Reduce Motion; no row was measured with it on.
Source: **M(n)** measured in n runs, **doc** public header or documentation,
**NM** not measured.

| Element | Trigger | Animated property | Duration | Curve | Delay | Interruptible | RM | Source | Kit |
|---|---|---|---|---|---|---|---|---|---|
| Sheet present | `beginSheet`, SwiftUI `.sheet` | window alpha 0 to 1; window y -32 pt to 0 (slides down); parent window dims | alpha 116 ms; y 215 ms; whole 265 to 285 ms | alpha almost linear (t10 5, t50 54, t90 104); y ease in-out `cb(0.35,0.2,0.6,0.8)` over 215 (t10 27, t50 104, t90 181) | first change +13 ms; parent dim runs the same 270 ms | No: the call blocks the main thread until the end | NM | M(4) | Missing: `Sheet` has no motion |
| Sheet dismiss | `endSheet`, `isPresented = false` | alpha 1 to 0; window y 0 to -32 pt (slides up); dim recedes | 250 to 260 ms | alpha almost linear `cb(0,0,0.95,1)` over 259; y `cb(0.5,0.2,0.55,0.8)` over 235 | +5 ms; the window leaves the screen about 10 ms after the last frame | No (same block) | NM | M(4) | Missing |
| Sheet parent dim | with present and dismiss | parent content darkens | 270 ms | follows the sheet y | with the sheet | No | NM | M(1): content 240 to 106 over a green fill, 30 to 24 over a dark one: a dark-gray veil of alpha about 0.6 | `Sheet` scrim exists, static (`styles/sheet.rs`) |
| Sheet content resize | `setFrame(_, animate: true)` | sheet window height, 201 to 320 pt; it grows about its middle (top moves up 60 pt for +120 pt) | 178 ms | ease in-out `cb(0.4,0.1,0.6,0.9)` over 177 | 0 | NM | NM | M(1) | Missing |
| Sheet content resize | `animate: false`, SwiftUI content grows (80 to 217 pt) | sheet window height | 1 frame | none: jumps | 0 | n/a | NM | M(2) | Matches (no motion) |
| Alert present (sheet style) | `beginSheetModal`, SwiftUI `.alert`, `.confirmationDialog` | alpha 0 to 1; scale 0.90 to 1.017 to 1 (410 pt wide alert) | 245 ms | alpha almost linear, 115 ms (`cb(0,0.1,0.05,0.2)` over 118). Scale starts slow, peaks +1.7 % at 165 ms and settles: 0.900 at +21, 0.920 at +76, 0.951 at +118, 0.998 at +150, 1.017 at +161, 1.010 at +194, 1.000 at +244. A spring fit is poor (rmse 0.1) | first change +20 ms | No (main thread blocked) | NM | M(4) | Missing: tokens `SPRING_ALERT_MS` 274 (Catalyst) unused |
| Alert dismiss (sheet style) | end of the sheet | scale 1 to 0.90 (410 to 369 pt); alpha 1 to 0; a shadow window shrinks with it | 241 to 269 ms | alpha `cb(0.3,0.3,0.95,1)` over 250 (t10 24, t50 121, t90 220); scale linear-ish `cb(0.1,0.1,0.95,1)` | +5 ms | No | NM | M(3) | Missing |
| Alert present (`runModal`) | `NSAlert.runModal` | alpha 0 to 1; scale 0.03 to 1.19 to 1 about the screen centre | 430 ms | alpha linear over 420. Scale accelerates, overshoots and relaxes: 0.03 at +46, 0.13 at +101, 0.34 at +158, 0.71 at +212, 1.19 at +268 (peak), 1.14 at +296, 1.05 at +351, 1.005 at +410 | first change +16 ms | Not tried | NM | M(3) | Missing |
| Alert dismiss (`runModal`) | Return key, `orderOut` | window disappears | 1 frame | none | 0 | n/a | NM | M(2) | Matches (no motion) |
| Alert shake or beep | invalid input, default button | none | none | none | none | n/a | NM | doc: no shake API in the AppKit headers (`shake` only names the pointer locator); beep not recorded | n/a |
| Popover present | `NSPopover.show`, SwiftUI `.popover` | whole window scales from the anchor tip (a 2 pt window) to full size; alpha follows the scale (alpha = scale, capped at 1) | settle 260 ms; the last 1 pt change at 410 ms | spring `ω0` 21.3, `ζ` 0.85 (rmse 0.007): `Spring(duration: 0.295, bounce: 0.15)`; t10 23, t50 74, t90 150; overshoot +1.5 % (272 for 268 pt) | first change +13 ms (AppKit); +330 ms for the first SwiftUI popover | Reverse: a `close` at 65 % of the present makes the window jump to full size and then dismiss from there | NM | M(9) | `Popover` is an `NSPopover` on macOS (native motion). Tokens `DURATION_POPOVER_IN_MS` 210 unused |
| Popover dismiss | `close`, `performClose`, `isPresented = false` | scale to the anchor tip, alpha follows | 240 ms | spring `ω0` 20, `ζ` 0.90 (rmse 0.007); t10 24, t50 79, t90 170 | +10 ms | `show` during a dismiss starts a full present about 250 ms after the dismiss ends | NM | M(7) | `DURATION_POPOVER_OUT_MS` 120 is half of this |
| Popover `animates = false` | `NSPopover.animates` | none | 1 frame | none | 0 | n/a | NM | M(2) | n/a |
| Popover `.transient` and `.semitransient` | behavior | same motion as above | same | same | same | same | NM | M(2) | n/a |
| Popover content size | `contentSize` 166 to 286 pt tall | window height | 44 ms (7 samples) | fast ease out | +10 ms | NM | NM | M(4) | NM |
| Popover anchor moved | `positioningRect` | window x 147 to 197 | 1 frame | none: jumps | 0 | n/a | NM | M(3) | NM |
| Menu open | click on a pop-up button, right click, SwiftUI `Menu`, `.contextMenu` | menu and shadow windows appear at their final place | 1 frame | none | 0 | n/a | NM | M(9) | Matches ([macOS animations](macos-animations.md#1-what-is-reliable)) |
| Menu highlight move | pointer to another item | the accent fill jumps to the item (24 pt rows); colour fixed at (92,121,245) | 1 frame | none; no fade, no slide | 20 ms after the pointer | n/a | NM | M(4 runs, 12 moves) | Not checked |
| Submenu open | pointer rests on an item with a submenu | submenu window appears at its final place, no animation | 1 frame | none | 212 to 233 ms after the pointer arrives (3 runs: 233, 215, 210) | n/a | NM | M(4) | `MENU_SUBMENU_INTENT_MS` 300: 90 ms longer |
| Submenu close | pointer jumps to another item or out | submenu window disappears, no fade | 1 frame | none | 13 to 20 ms | n/a | NM | M(3) | NM |
| Menu close after a choice | click on a leaf item | the chosen row blinks (off 47 ms after the mouse up, off for 77 ms, on again), then the whole menu fades alpha 1 to 0 | blink about 125 ms, then 232 to 239 ms | fade `cb(0.25,0.2,0.3,0.9)` over 205 (t10 19, t50 63, t90 140, settle 195); spring `ω0` 26, `ζ` 0.95 | fade starts about 30 ms after the blink | NM | NM | M(4) | `DURATION_MENU_CLOSE_MS` 200 exists, unused |
| Menu close without a choice | Escape key, click outside | the menu fades alpha 1 to 0 (submenu with it); no blink | 224 to 230 ms | `cb(0.25,0.3,0.3,0.9)` over 200 (t50 57, t90 134) | +74 ms after the key | NM | NM | M(4) | same token |
| Combo box drop-down | click on the arrow | list appears in 1 frame; highlight moves in 1 frame; a choice blinks (off for 45 ms) and the list vanishes about 70 ms after it is back on, with no fade | open 1 frame; close about 120 ms | none | 0 | n/a | NM | M(1) | Not checked |
| Tooltip show (AppKit `toolTip`) | pointer rests in the view | window appears at once, alpha 1, 19 pt below the pointer at the pointer x | 1 frame, no fade | none | 1.52 s after the pointer enters (1520, 1521, 1527, 1519, 1518 ms: five hovers); no shorter delay when it re-enters 1.2 s after a hide; leaving after 0.45 s shows nothing | n/a | NM | M(5) | `TOOLTIP.delay_ms` 1500 and `pointer_gap` 19 match |
| Tooltip hide (AppKit) | pointer leaves | window disappears at once | 1 frame | none | 7 to 13 ms | n/a | NM | M(4) | Matches (no fade) |
| Tooltip (SwiftUI `.help`) | pointer rests on the view | window appears at once | 1 frame | none | 1.77 to 1.80 s after the pointer enters (2 runs) | n/a | NM | M(2) | `TOOLTIP.delay_ms` 1500 is 0.3 s shorter |
| Tooltip hide (SwiftUI `.help`) | pointer leaves | one run: no hide for 3.5 s; the other: alpha 0.9 to 0.1 in 10 steps over 450 ms starting 160 ms after leaving | 450 ms | steps of 0.1 about every 45 ms | 160 ms | NM | NM | M(2), unclear | NM |
| Find bar | `showFindInterface` on an `NSTextView` in an `NSScrollView` | the content moves down 30 pt | 1 frame (16 ms) | none: no slide | at most 41 ms after the call | n/a | NM | M(1) | No find bar |
| Drag image lift | drag starts | the 100x50 pt image appears at the source at full size and opacity; the pointer moves it 1:1 | 1 frame | none: no scale, no fade | `beginDraggingSession` 77 ms after the mouse down | n/a | NM | M(2) | `DragSource` exists, no image motion |
| Drop accepted | release on an accepting view | the image disappears | 1 frame | none | 0 | n/a | NM | M(2) | NM |
| Drop refused (snap-back) | release where nothing accepts | the image slides back to the source | 225 ms | `cb(0.05,0,0.3,0.5)` over 222: almost constant speed, then a short ease out (t10 17, t50 94, t90 188); the source gets `drag ended` 300 ms later | starts within about 50 ms of the release | NM | NM | M(1) | Missing |
| Spring loading | drag rests over an `NSSpringLoadingDestination` | `springLoadingEntered` at 0; highlight 1 (standard) at +19 ms; highlight 2 (emphasized) at +534 ms; then 1, 2, 1 every 100 ms; `springLoadingActivated` at +902 ms | 902 ms | steps | 0.5 s before the blink (`com.apple.springing.delay` = 0.5 on this host) | n/a | NM | M(1) + doc (the default key) | Missing |
| Quick Look open | `QLPreviewPanel.makeKeyAndOrderFront` | panel frame grows from the source frame to its final size, alpha constant 1 | 235 ms (settle), t50 29, t90 103 | decelerating `cb(0.15,1.3,0.9,0.9)` over 246 | the first panel needed 510 ms to appear; then the panel jumps to the image size (810x543 to 370x223 pt) in 60 ms | NM | NM | M(1) | Missing |
| Quick Look close | `close` | panel frame shrinks to the source frame, alpha constant 1 | 175 ms (settle 172) | `cb(0.35,0.1,0.6,0.9)` over 181 (t10 31, t50 86, t90 146) | +7 ms | NM | NM | M(1) | Missing |
| Completion list | `NSTextView.complete` | list window appears, alpha 0.88 | 1 frame (8 ms) | none | 0 | n/a | NM | M(1); the close was not seen | Missing |
| Color panel | `ColorPicker` click | the color panel appears | 1 frame | none | 153 ms after the click | n/a | NM | M(1) | `color_picker.rs` exists, NM |
| Date picker pop-up | compact `DatePicker` click | a 139x172 pt window appears | 1 frame | none: no scale | 160 ms after the click starts | n/a | NM | M(1); its close was not seen | `date_picker.rs` exists, NM |
| Validation bubble, banner or undo toast | invalid text, undo | no such element in AppKit or SwiftUI | none | none | none | n/a | NM | doc: `control(_:didFailToFormatString:errorDescription:)` only lets the app respond; no bubble in the headers | n/a |

## 3. Kit comparison

| Token or code | Kit today | macOS here | Gap |
|---|---|---|---|
| `TOOLTIP.delay_ms` | 1500 | 1520 (AppKit), 1790 (SwiftUI `.help`) | None for AppKit |
| `TOOLTIP.pointer_gap` | 19 | 19 below the pointer, same x | None |
| `MENU_SUBMENU_INTENT_MS` | 300 | 212 to 233 ms to open; 13 to 20 ms to close | Open 90 ms late; a diagonal path to a submenu was not measured, so the close side may keep it |
| `DURATION_MENU_CLOSE_MS` | 200, unused | fade 224 to 239 ms, settle 195 | Matches; wire it to every menu close |
| Menu open and highlight | instant | instant | None |
| Menu blink before the fade | none | off 45 ms, off for 77 ms, on, fade | Missing |
| `DURATION_POPOVER_IN_MS` and `_OUT_MS` | 210 and 120, unused | spring `ω0` 21.3 and 20, `ζ` 0.85 and 0.90, scale from the anchor tip | Out is half of macOS; the curve is a scale from the tip, not a fade |
| `SPRING_SHEET_MS` | 344, a Catalyst card | AppKit sheet: ease in-out slide of 32 pt in 215 ms plus a 116 ms fade | Different motion; the kit `Sheet` has none |
| `SPRING_ALERT_MS` | 274 (Catalyst) | AppKit alert: 245 ms fade and a 0.9 scale with 1.7 % overshoot (sheet style) or 430 ms and 1.19 overshoot (`runModal`) | Different motion; the kit alert has none |
| Drag image and snap-back | image only | 225 ms near-linear slide back | Missing |
| Spring loading, Quick Look, find bar slide | none | 0.5 s delay then blink; 235 ms zoom; instant | Missing or nothing to copy |

## 4. Not measured

- **Reduce Motion, Increase Contrast, Reduce Transparency** on every row. The
  flag reads off and the settings were not changed.
- **Inline rename** (`editColumn` in a table): one run, no pixel changed in the
  captured rect, so the field editor and focus ring motion are not known.
- **Search suggestions** (SwiftUI `.searchable`): the Command-F and typing keys
  opened no window in the one run.
- **Sheet or alert interruption** and a sheet or alert with the Return key: the
  `beginSheet` call blocks the main thread; the default button press flash was
  not isolated.
- **Menu with a diagonal pointer path** to a submenu (the intent triangle),
  slow pointer moves, and the highlight of a menu opened with the keyboard.
- **Quick Look alpha**, the completion list close, the date picker close, and
  whether the 139x172 pt date picker window is a popover.
- **Beep sounds** of an invalid default or a refused input (audio, not motion).
- **Toast, banner and undo toast** of an app: AppKit and SwiftUI have none by
  default (see the last row).
- **TipKit popovers**, `NSAlert` button press highlight, the pop-up button and
  combo box button press tint.

## 5. How to reproduce

The sources are `pres.swift`, `mouse.swift`, the Python readers and the step
files in `~/src/probes/presentation`. They stay outside the repo. Build into a
scratch folder, run under the screen lock and delete the captures after:

```sh
P=~/src/probes/presentation; B=$TMPDIR/pres; mkdir -p $B/out
swiftc -O $P/pres.swift -o $B/pres; swiftc -O $P/mouse.swift -o $B/mouse
cd $P; SCRATCH=$B ./hold.sh ./run.sh $B/out sheet sheetresize alert alertmodal popover swiftpopover
SCRATCH=$B DUR=14 ./hold.sh ./drive.sh $B/out popup steps/popup3.txt   # menu, submenu
python3 -I episodes.py $B/out/alert.jsonl      # one line per motion of each window
python3 -I fs.py $B/out/alert.jsonl <windowid> 4 <t0> <t1> name   # fit alpha (3: height)
rm -rf $B
```

Modes (`pres <mode> <out.jsonl>`): `sheet`, `sheetresize`, `swiftsheet`, `alert`,
`alertmodal`, `alertkey`, `swiftalert`, `swiftdialog`, `popover`, `popoversemi`,
`popovernoanim`, `popint`, `swiftpopover`, `findbar`, `quicklook`, `drag`,
`popup` (pop-up button), `tooltip`, `swiftui` (menu, help, date and color pickers,
context menu). `CAP="x y w h" CAPFILE=f` streams screen frames, `WATCH="x y w h"`
logs windows of other processes, `DUR` sets the length in seconds. `steps/*.txt`
list the pointer moves (`mouse move|glide|click|rclick|drag|key`, CG
coordinates); `a_hl.py` and `a_drag.py` read highlights and the drag image from
a capture.
