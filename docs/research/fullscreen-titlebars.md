# Full screen: title bars, toolbars, traffic lights and animation

**Goal:** record which title bar, toolbar and traffic-light designs macOS lets
an app keep in full screen (the green button), whether the kit reproduces them,
and whether kit windows get the system maximize and full-screen animations.
**Scope:** SSD (system title bar), hybrid (hidden title bar, native lights, the
app's own bar) and CSD (the app draws everything) windows on a single display.
It excludes tabs, split view in Mission Control, multiple displays and the
Stage Manager.
**Snapshot:** macOS 27.2.1, swiftc 6.4, 2560x1440 display, Freya `=0.5.3`,
repo HEAD `0750a21b` plus uncommitted work of other sessions.
**Reader:** anyone who changes `crates/bk/src/platform/macos`, `titlebar.rs`,
`chrome.rs` or the full-screen part of [09-titlebars.md](../barrierkit/09-titlebars.md).
**Input:** 46 probe windows (14 SwiftUI, 29 AppKit, 3 kit), driven through the
accessibility tree and the real pointer, 60 fps screen recordings, CGWindowList
traces every ~4 ms. Companion to [macos-windows.md](macos-windows.md), which
excluded full screen.
**Output:** a designs-by-observations matrix, the kit comparison, the animation
table and ranked fixes. No `crates/` code was changed.

## How it was measured

Each probe prints one log (`<variant>.log`): window frame, `contentLayoutRect`,
`contentView.safeAreaInsets`, presentation options, every window of the process
(the system adds `NSToolbarFullScreenWindow`), the three buttons, and the AX
tree. The harness takes four screenshots per window: windowed, full screen with
the pointer mid-screen, pointer at the top edge (after ~1 s), pointer back
mid-screen. Colour bands in the probes make the content shift visible. In full
screen the AX tree only lists the traffic lights while the pointer is at the top
edge. Screenshots and logs: `~/Desktop/barrierkit-capturas/sesion-9-pantalla-completa/`
(`<variant>-N-*.png`, `sheets/<variant>.png` for the four states in one image,
`animacion/` for recordings and traces, `frames-*.png` for 60 fps grids). The
probe sources and harness live in `~/src/probes/fs/` (outside the repo);
`scripts/macos-fullscreen.sh` is the in-repo check and still prints `0 failed`.

## What Swift lets a window do in full screen

"Shift" is how far the content moves down when full screen starts (measured as
the content frame change, with the pointer mid-screen). "Edge" is what the
pointer at the top edge reveals.

| # | Design | Mid-screen | Top edge | Shift | Source |
| - | ------ | ---------- | -------- | ----- | ------ |
| 1 | Default `WindowGroup`, no toolbar (SSD) | nothing | menu bar plus a 32 px title strip (lights, title) slide down 30 px | 0 | `swift-default`, `appkit-std` |
| 2 | `.windowStyle(.hiddenTitleBar)`, no toolbar | nothing | same as 1 | 0 | `swift-hidden` |
| 3 | `.toolbar{}` (unified, default) | toolbar pinned, lights hidden | menu bar, toolbar slides down 30 px, lights appear in it | 52 | `swift-toolbar`, `appkit-toolbar` |
| 4 | `.hiddenTitleBar` plus `.toolbar{}` | same as 3 | same as 3 | 52 | `swift-hidden_toolbar` |
| 5 | `.windowToolbarStyle(.unifiedCompact)` | toolbar pinned | as 3 | 40 | `swift-compact`, `appkit-toolbar-compact` |
| 6 | `.windowToolbarStyle(.expanded)` | toolbar plus title row pinned (68 px strip) | as 3 | 52 | `swift-expanded`, `appkit-toolbar-expanded` |
| 7 | `.toolbarBackgroundVisibility(.hidden, for: .windowToolbar)` | geometry identical to 3 | identical | 52 | `swift-toolbar_bg_hidden` |
| 8 | `ToolbarCommands`, `.toolbar(id:)`, `NavigationSplitView` | identical to 3 | identical | 52 | `swift-toolbar_commands`, `swift-toolbar_id`, `swift-split` |
| 9 | AppKit `titlebarAppearsTransparent`, `.fullSizeContentView`, no toolbar | nothing | as 1 | 0 | `appkit-transparent`, `appkit-fullsize` |
| 10 | AppKit toolbar plus `.fullSizeContentView` | toolbar pinned | as 3 | 52, but `safeAreaInsets.top` = 52 so content is not pushed (SwiftUI reports the same 52 as safe area) | `appkit-toolbar-fullsize` |
| 11 | AppKit `presentationOptions` `.autoHideToolbar` (via `window(_:willUseFullScreenPresentationOptions:)`) | toolbar hidden | toolbar (52 px) slides in with the menu bar, lights in it | 0 | `appkit-opt-autohidetoolbar` |
| 12 | Same plus `.hideMenuBar` | toolbar hidden | nothing, ever | 0 | `appkit-opt-toolbar-nomenubar` |
| 13 | `NSTitlebarAccessoryViewController`, `.bottom`, `fullScreenMinHeight` 0 | accessory hidden | accessory (68 px strip) slides in under the title strip | 0 | `appkit-accessory-bottom-min0` |
| 14 | Same, `fullScreenMinHeight` 40 | 40 px of the accessory stay pinned | full accessory | 40 | `appkit-accessory-bottom-min40` |
| 15 | `.hideMenuBar` / `.hideDock` plus `.fullScreen` (kiosk) | nothing | nothing, no menu bar, no title strip, no lights | 0 | `appkit-opt-fullscreen-only`, `-hidemenubar-hidedock` |
| 16 | Keep the Dock (`.autoHideMenuBar|.fullScreen`) | nothing | as 1, Dock stays reachable | 0 | `appkit-opt-keepdock` |
| 17 | Borderless plus `.fullScreenPrimary` plus `toggleFullScreen` (CSD) | nothing | menu bar plus an AppKit title strip with dimmed lights and the window title over the app's content | 0 | `appkit-borderless-primary` |
| 18 | Borderless, manual `setFrame` plus `.hideMenuBar|.hideDock` | app content | nothing, menu bar never reveals | 0 | `appkit-borderless-manual` |
| 19 | Same with `.autoHideMenuBar|.autoHideDock` | app content | menu bar and Dock reveal, no title strip | 0 | `appkit-borderless-manual-auto` |
| 20 | SwiftUI `.windowStyle(.plain)` | nothing happens | nothing happens | none | `swift-plain` |

Further findings:

- **Lights.** In every windowed-chrome design (1 to 14) the lights are only
  visible and clickable at the top edge; AX lists `AXCloseButton`,
  `AXMinimizeButton` and `AXFullScreenButton` only then. The green button has
  actions `AXPress`, `AXZoomWindow` and `AXShowMenu`; `AXZoomButton` is a
  different, windowed-only element. In designs 3 to 8 the lights exist
  mid-screen but are disabled (AX `enabled: false`) and not drawn.
- **Toolbar item.** The View menu in full screen offered only `Show/Hide
  Toolbar` and `Customize Toolbar…` for the AppKit and SwiftUI toolbar probes
  (also with `ToolbarCommands` and `.toolbar(id:)`). **"Always Show Toolbar in
  Full Screen" did not appear**, so no defaults key was observed (diffs of the app
  and global domains around the menu press were empty). `Hide Toolbar` in full
  screen removes it: shift 52 becomes 0 and the edge shows only the 32 px title
  strip (`appkit-toolbar-menu-hide`). The system "auto-hide toolbar" behaviour
  is reachable only through `.autoHideToolbar` (row 11).
- **`collectionBehavior`.** `.fullScreenNone`, `.fullScreenAuxiliary` and a
  non-resizable window do not enter full screen through `toggleFullScreen`
  (`appkit-cb-none`, `-auxiliary`, `appkit-fixed`); a plain borderless window
  without `.fullScreenPrimary` does not either (`appkit-borderless`).
  `.fullScreenAllowsTiling` and `.fullScreenDisallowsTiling` changed nothing
  observable here.
- **Coercion.** `presentationOptions` without `.fullScreen`, or with
  `.hideMenuBar` alone, is coerced by AppKit (`opts=default` or `.fullScreen`
  added) and also kills the title strip reveal: reveal needs `.autoHideMenuBar`.
- **Not diagnosed.** `NSTitlebarAccessoryViewController` with
  `layoutAttribute = .top` (`appkit-accessory-top`, `-top-min0`): the process
  never opened a window or wrote a log in three runs (idle, 0% CPU). Cause not
  investigated; no result for this design.

## The kit against those designs

Cases from `crates/bk/examples/full_screen.rs`, logs `kit-title-bar`,
`kit-hidden-title-bar`, `kit-plain`.

| Kit style | Compared with | Reproduced | Differs |
| --------- | ------------- | ---------- | ------- |
| `TitleBar` (Titled, server decoration) | rows 1 and 2 | Yes. Same native strip at the edge: 32 px strip slides 30 px, lights at the same positions, title "Full screen", shift 0, same 3 bounds and ~550 ms transition. | The kit's own `Titlebar` element is invisible in full screen (the overlay uses `focus_bar_progress`, which is 0 on macOS), so toolbar content placed in the kit titlebar disappears. Swift rows 3 to 8 keep the toolbar pinned and shift the content; the kit has no equivalent. |
| `HiddenTitleBar` (Titled plus `.fullSizeContentView`, transparent title bar, empty unified `NSToolbar`) | rows 2, 4 and 9 | Partly. At the top edge the native lights slide down and are clickable (`AXCloseButton`, `AXZoomButton` at y 48), as in row 4. Shift 0. | **The kit's bar does not reappear at the top edge**: the screenshot shows only the menu bar and the lights over blank content; the app title is not drawn anywhere. Row 4 shows the toolbar pinned mid-screen and sliding with the lights at the edge. An empty `NSToolbar` window of 66 px is kept (alpha 1, invisible). |
| `Plain` (Borderless, client decoration) | rows 17 to 20 | Closest to row 18: `toggle_borderless_fullscreen` with `HideMenuBar|HideDock`, menu bar and Dock never reveal. Unlike Swift `.plain` (row 20), it does enter. | Client-drawn lights are removed in full screen (AX lists no `Cerrar/Minimizar/Zoom`); the in-flow `Titlebar` stays visible with no reveal behaviour. No auto-hide variant (row 19) and no native strip (row 17). |

Content shift: the kit shifts nothing in any of the three cases (0 px), same
as Swift rows 1, 2 and 9, and unlike rows 3 to 8, 10 and 14. Not reproduced at
all: pinned toolbar with content shift (3 to 8, 10), `.autoHideToolbar` (11),
title-bar accessories (13, 14), keeping the Dock (16), AppKit's borderless
native full screen (17) and `.autoHideMenuBar` for borderless windows (19). The
`FullscreenBar` and `fullscreen_push` options were not exercised (the recipe
uses defaults), so their macOS behaviour is unverified.

## Animation

Evidence: CGWindowList bounds every ~4 ms ("bounds" = distinct frames seen) and
60 fps recordings ("run" = consecutive changing frames). The Space transition is
not visible in CGWindowList, so a titled window shows only 3 bounds (start,
the final frame, and one intermediate) while the recording shows ~33 changing
frames (~550 ms). The zoom (maximize) is a real frame animation: ~55 bounds and
20 to 28 frames (~330 to 470 ms).

| Style | Enter full screen | Exit full screen | Zoom in | Zoom out |
| ----- | ----------------- | ---------------- | ------- | -------- |
| TitleBar, SwiftUI | animated, 36 fr / 583 ms | animated, 30 fr / 483 ms | animated, 21 fr / 333 ms | animated, 21 fr / 333 ms |
| TitleBar, kit | **same**, 35 fr / 567 ms | **same**, 34 fr / 550 ms | **same**, 21 fr / 333 ms | **same**, 28 fr / 450 ms |
| HiddenTitleBar, SwiftUI | animated, 33 fr / 533 ms | animated, 30 fr / 483 ms | animated, 21 fr / 333 ms | animated, 29 fr / 467 ms |
| HiddenTitleBar, kit | **same**, 34 fr / 550 ms | **same**, 31 fr / 500 ms | **same**, 21 fr / 333 ms | **same**, 20 fr / 317 ms |
| Plain, SwiftUI `.plain` | does not enter (1 bounds) | n/a | no zoom (Window > Zoom pressed, 1 bounds) | n/a |
| Plain, kit | **not animated**: 2 bounds, one frame jump | **not animated**: 2 bounds, one jump (a 183 ms fade of the Dock and menu bar) | **same as Swift**, 21 fr / 333 ms, when driven through `ToggleMaximize` | **same**, 21 fr / 333 ms |

Notes on the table:

- Zoom was triggered with `AXZoomWindow` on the native green button (titled
  styles; Option-click gave the same result) and, for the kit's plain window,
  by setting `green_button = "zoom"` in a temporary `window.domain` and
  clicking the kit-drawn green light. A synthetic double click on the kit
  `Titlebar` (default action `Zoom`) did not zoom the plain window in three
  attempts (up to 90 ms between clicks); native title bars do not zoom on
  double click on this system either. The cause for the kit was not investigated;
  `NSWindow.zoom` itself, called on a borderless AppKit window, animates like a
  titled one (`appkit-zoom-borderless`: 49/54 bounds, 20 fr / 317 ms).
- Borderless full-screen options, measured on AppKit probes: `setFrame` without
  animation jumps like the kit (2 bounds). `setFrame:display:animate:YES` runs a
  20 frame / 317 ms frame animation (`appkit-borderless-manual-animated`).
  `.fullScreenPrimary` plus `toggleFullScreen` on a borderless window gives the
  full Space transition, 34 fr / 550 ms, identical to titled
  (`appkit-borderless-primary`), but adds the AppKit title strip at the top
  edge (row 17).
- The lead's trace is confirmed by the recordings: titled kit windows animate
  like SwiftUI (3 bounds), kit Plain jumps (2 bounds), SwiftUI `.plain` never
  enters full screen.

## Proposed fixes, ranked

Fix 2 is in the kit since 2026-11-23, with `HideMenuBar|HideDock` returned
from `window:willUseFullScreenPresentationOptions:`, which keeps AppKit's
title strip off the top edge ([csd-window-designs.md](csd-window-designs.md),
row D). The tables above describe the kit before it.

1. **Plain enter/exit animation, smallest change.** In
   `toggle_borderless_fullscreen` (`crates/bk/src/platform/macos/mod.rs`) call
   `setFrame:display:animate:` with `animate = true` instead of
   `setFrame_display`. Measured in AppKit: 317 ms frame animation in both
   directions. Cost: one argument. Risks, not tested in the kit: AppKit runs the
   animation synchronously on the main thread, so resize and `fullscreen_changed`
   callbacks arriving inside it can hit `with_runtime`'s `try_borrow_mut` and be
   dropped, which needs a final surface resync; the presentation options should be
   applied around the animation (hide after entering, restore before exiting) to
   avoid a menu bar flash. It is a frame animation, not the Space transition,
   so it will not look the same as titled windows.
2. **Plain with the native transition.** Give the borderless window
   `.fullScreenPrimary` and use `toggleFullScreen` for it (550 ms, same as
   titled). Cost: moderate, since `Native::FullScreen` loses its titled/non-titled
   branch and the delegate already reports `fullscreen_changed`. Risk: AppKit
   adds a title strip with dimmed lights and the window title over the app's
   content at the top edge (row 17), which would sit over the kit's own
   client-drawn bar; whether `presentationOptions` can suppress it was not tested.
3. **Hybrid: draw the kit bar at the top edge.** Mid-screen nothing, edge-revealed
   bar like row 2/4. Needs a macOS signal for "the edge is revealed" (the
   menu bar visibility or the `NSToolbarFullScreenWindow` position) to drive
   `focus_bar_progress`, or keeping the bar pinned by giving the empty
   `NSToolbar` real height (row 4 geometry: shift 52). Not tried; the signal
   source is the open question.
4. **Toolbar pinned in full screen.** Only reachable with a native `NSToolbar`
   (rows 3 to 8) or an accessory (rows 13, 14); a kit-drawn toolbar cannot
   reproduce the content shift and slide without that. Larger change, left as
   a decision for the lead.
5. **Document, change nothing.** State in 09-titlebars.md that Plain jumps in
   and out of full screen and that SwiftUI `.plain` has no full screen at all.

## Not verified

- "Always Show Toolbar in Full Screen" and its defaults key: item absent in the
  five toolbar probes; it may require conditions not reproduced here.
- `.top` title-bar accessories (no window opened).
- Kit Plain zoom through the `Titlebar` double click (synthetic events only).
- Behaviour of `FullscreenBar::Glass/ProgressiveGlass` and `fullscreen_push` on macOS.
- Multiple displays, tiling from the green button menu, Stage Manager.
- Fix 1 and 2 were measured in AppKit probes, not in the kit.
