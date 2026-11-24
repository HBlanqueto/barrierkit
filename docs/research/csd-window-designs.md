# Window designs and window states: Swift against the kit

**Goal:** list the window designs a Swift app on macOS can use to draw its
own title bar, record how each one looks and behaves in every window state the
system has, and say which kit option reproduces it. Each design row ends in
a kit change or in a documented ceiling.
**Scope:** main windows on one display: the system title bar, the system
title bar with a toolbar, the hidden title bar (the app's bar beside the
native lights) and the borderless window (the app draws everything). States:
active, inactive, zoomed, tiled, full screen and minimized. It excludes
panels, HUDs, sheets, tabs, Stage Manager and several displays.
**Snapshot:** macOS 27.2, swiftc 6.4, repo `273c9fca`, 2026-11-23.
**Reader:** anyone who changes `WindowStyle`, `WindowToolbarStyle`,
`Titlebar`, `WindowChrome` or `crates/bk/src/platform/macos`.
**Input:** [fullscreen-titlebars.md](fullscreen-titlebars.md) (20 full-screen
designs, measured), [macos-windows.md](macos-windows.md) (windowed geometry
and lights), [macos-corners.md](macos-corners.md),
[macos-native-chrome.md](macos-native-chrome.md) and one probe pass per
design (section 4).
**Output:** the matrix of section 2, filled by the sessions of section 4, and
the kit changes they commit.

## 1. The states, by their system names

| State | How the user enters it | AppKit | SwiftUI | Kit today |
| ----- | ---------------------- | ------ | ------- | --------- |
| Active | click the window | `isKeyWindow`, `isMainWindow` | `@Environment(\.appearsActive)` | `WindowStatus::activated` |
| Inactive | another window is key | `windowDidResignKey` | `appearsActive == false` | `activated == false` |
| Zoomed | green button with Option, title-bar double click (setting "Zoom"), Window > Zoom | `zoom(_:)`, `isZoomed` | none | `WindowStatus::maximized` |
| Tiled | green button menu or Window > Move & Resize (halves, quarters, Fill, Center), drag to an edge | no public state | none | not reported |
| Full screen | green button, Control-Command-F | `toggleFullScreen(_:)`, `.fullScreen` in `styleMask` | `.windowFullScreenBehavior` | `WindowStatus::fullscreen` |
| Minimized | yellow button, Command-M | `miniaturize(_:)`, `isMiniaturized` | none | not reported (no surface) |

## 2. Designs against states

"Measured" cells point at their source. "Owed" cells are filled by the
session named in section 4.

| # | Swift design | Kit option | Active / inactive | Zoomed | Tiled | Full screen |
| - | ------------ | ---------- | ----------------- | ------ | ----- | ----------- |
| A | `WindowGroup`, no toolbar (system bar) | `WindowStyle::TitleBar` | same in Swift and kit (the kit asks AppKit for the bar): 32 px bar, lights at (9, 9), title ink at x 85 in gray (165 of 255 in Dark), corner 16 (diagonal inset 5 px), shadow 56 px active and 23 px inactive (`csd-ab/swift-A-*`, `csd-ab/kit-A-*`) | same in Swift and kit: Zoom fills the visible frame `0,30 2560×1325`; bar, lights, corner 16 and the 56 px shadow do not change; animation 21 fr / 333 ms | same in Swift and kit, nothing changes but the frame: Left `0,30 1280×1325`, Top Left `1280×662`, Fill and a drag to the top `2560×1325`, Center moves only; corner 16, shadow 56 px, lights and focus as windowed; Swift `isZoomed` is true after Fill only (`tile/swift-A-*`, `tile/kit-app-title-bar-*`) | same in Swift and kit: no shift, the 32 px title strip waits at `y −32` (alpha 0) and slides to `y 30` with the menu bar at the top edge (rows 1 and 2 of [fullscreen-titlebars.md](fullscreen-titlebars.md)) |
| B | `.toolbar{}` with `.windowToolbarStyle(.unified)`, `.unifiedCompact`, `.expanded` | `WindowStyle::HiddenTitleBar` and `Titlebar::toolbar_style` (`Unified`, `UnifiedCompact`, `Expanded`); recipe `full_screen.rs hidden-title-bar unified\|compact\|expanded` | Swift (`csd-ab/swift-B-*`): `.unified` 52 px, lights (19, 19), title white at the start, items 36 high; `.unifiedCompact` 40 px, lights (12, 13), items 24; `.expanded` 68 px = a 28 px title row (lights (9, 9), title gray at x 85, as A) over a 40 px toolbar row whose `.navigation` and `.primaryAction` items sit together at the start, 28 high. Corner 16 and shadow 56 / 23 px as A in all three. Kit (`csd-ab/kit-B-*`): unified and compact bars of the same heights and lights; `Expanded` added in this pass: 28 + 40 rows, native lights at (9, 9) through an `.expanded` empty `NSToolbar`, title ink at x 85, y 12 (Swift 11). Matched in the second pass: the first item's ink starts at x 95 / 94 / 11 as in Swift (`TITLEBAR_TOOLBAR_START`, was 86 / 75 / 7), expanded items are 28 high (`TOOLGROUP.expanded_*`, back ink 11–40 and group 54–121 against Swift 11–40 and 54–122, y 32–58 against 32–59), and the expanded title row uses the measured title-row ink, 165 active and 106 inactive in Dark (Swift 165 / 106; Light 88 / 183, `titlebar-row-title`). Still differs: the unified title ink starts at x 147, y 21 (Swift 145, 19); the compact title stays centered where Swift puts it after the items (x 137); the kit compact group draws a divider Swift does not; the kit window background is 30 where Swift's bar is 41 | as A in Swift and kit: `0,30 2560×1325`, the bar keeps its height and items, corner 16, shadow 56 px (`*-zoomed*`) | Swift `.unified` as A, toolbar unchanged (`tile/swift-B-*`); the kit's B is a `HiddenTitleBar` window and tiles as C (`tile/kit-notes-hidden-title-bar-*`); a `run_window` window has no Window menu and its green-button menu is not in the AX tree, so `csd-ab` has no kit tile | Swift pins the bar and shifts the content by its height, 52 / 40 / 68 px (safe area `t52` in full screen), lights hidden; at the top edge the strip slides to `y 30` with the menu bar and shows the lights. `.windowToolbarFullScreenVisibility(.onHover)` is AppKit's `.autoHideToolbar` (row 11): strip at `y −52` alpha 0, shift 0, slides in at the edge (`csd-ab/swift-B-onhover-*`). Kit: the bar keeps its windowed place at the top, so the shift is the same 52 / 40 / 68 px; the expanded title row is empty, as in Swift. At the top edge AppKit's 32 px strip covers the bar; since `907de855` ([Full screen](../barrierkit/09-titlebars.md#full-screen)) the glass controls of the first 52 px ride down beside the lights. Since the second pass the expanded toolbar row rides too, in a bottom titlebar accessory under AppKit's 28 px title strip with the empty toolbar hidden: a 68 px strip at `y 30`, lights at y 38–51 as in Swift, items 3 to 4 px lower than Swift's (65–91 against 61–88). The expanded title is not drawn at the edge (AppKit's strip covers the kit's row; Swift shows it). Owed: `.onHover` (Swift: the strip waits at `y −52` alpha 0, content from `y 0`, and slides to `y 30` in about 150 ms, back in 120). A kit attempt through the focus overlay (`WindowToolbarFullScreenVisibility::{Automatic, OnHover}`, `focus_overlay` on macOS) hid the bar at rest and rode the row, but after a press in the revealed row the screen kept stale frames until the row hid; not committed |
| C | `.windowStyle(.hiddenTitleBar)` (alone, with `.toolbar{}`, `.unifiedCompact`, `.toolbarBackgroundVisibility(.hidden, for: .windowToolbar)`, `.toolbar(removing: .title)`, `NavigationSplitView`), AppKit `titlebarAppearsTransparent` + `.fullSizeContentView` | `WindowStyle::HiddenTitleBar` | same. Native lights at window (18, 18) under a unified toolbar (also with the background hidden, the title removed or a split view), (11, 12) under `.unifiedCompact`, (8, 8) with no toolbar; safe-area top 52 / 40 / 32. The kit puts them at (18, 18) under `Unified` and, since this change, (11, 12) under `UnifiedCompact` (was 18, 18); the compact bar now leaves their width free. Inactive grays the native lights in both; corner 16, system shadow (`*-0-active`, `*-1-inactive`). Differs: a kit window with no `Titlebar` keeps (18, 18) (no app needs the bare strip), and the compact bar puts its first item right after the lights where Swift leaves 20 px (spacing shared by every compact bar, design B). Captures `~/Desktop/barrierkit-capturas/csd-c/`, harness `~/src/probes/fs/csd-states.sh` | same: frame = visible frame 0,30 2560x1325, lights keep their window offset, corner and shadow unchanged (`*-2-zoomed`); a double click on the kit bar zooms through AppKit | as A in Swift and kit: Left 0,30 1280x1325, Fill 0,30 2560x1325, lights keep their windowed offset, (8, 8) in Swift with no toolbar, (18, 18) in the kit; corner and shadow unchanged. The kit tiles from the Window menu of an `Application` and by an edge drag of a `run_window` window, which has no Window menu (`tile/swift-C-*`, `tile/kit-app-hidden-title-bar-*`, `tile/kit-notes-hidden-title-bar-*`, `csd-c/swift-*-4-tiled-fill`) | Swift: nothing (no toolbar) or the toolbar pinned with shift 52 / 40; at the edge it slides 30 px with the lights (rows 2, 4). Kit: mid-screen the bar stays at y 0 with no lights gap and no content shift. At the edge, before `907de855` the menu bar covered it and the lights showed at (18, 48) over blank content (`fs/kit-hidden-title-bar-2-fs-top-edge`); since then its glass controls ride beside the lights in AppKit's bar ([Full screen](../barrierkit/09-titlebars.md#full-screen)), whose style now follows `toolbar_style`. The content does not shift 52 / 40 as under Swift's pinned toolbar, by that section |
| D | `.windowStyle(.plain)`, borderless `NSWindow` | `WindowStyle::Plain` | Swift: style mask 0 (borderless, not resizable), 900×450, corner 0, no border or visible shadow; it never becomes key, so `appearsActive` stays false and both states look the same. Kit: corner 16.8, 23 px shadow, its lights in color when key and gray when not, title dimmed (probes `~/src/probes/csdplain/`, captures `~/src/captures/csd-plain/`: `states/swift-plain-states-*`, `states/kit-plain-states-*`) | Swift: Window > Zoom does nothing (1 bounds). Kit: green light with `zoom` 21 fr / 333 ms; title-bar double click zooms to `0,30 2560×1325` and back, 12 of 12, corner and shadow kept (`states/kit-plain-states-zoomed.png`). The old miss was `performWindowDragWithEvent` on the first press, gone since 42c5c6cd; under the 60 fps recorder presses were still missed, not reproduced without it | Swift `.plain` never tiles: the Window menu and a drag do nothing (`tile/swift-D-*`). Kit tiles through Window > Fill, Center and Move & Resize of an `Application` with A's frames, keeping corner 16.8 and its 23 px shadow; not by a drag to an edge (the kit moves the window itself) nor from its lights; a `run_window` window has no Window menu (`tile/kit-app-plain-*`, `tile/kit-notes-plain-*`). Ceiling: the kit light gets no long-press or hover menu, since no public API shows the system's one and its items call private actions | Swift `.plain` never enters, also with `.windowFullScreenBehavior(.enabled)` (style mask 0, `swift-plain-fsenabled`). Kit: AppKit's transition (`.fullScreenPrimary`, `toggleFullScreen`) and a Space of its own: enter 4 bounds, the window grows from 1.70 to 2.05 s at 60 fps; exit 33 fr / 533 ms (AppKit twin `appkit-borderless-primary-kiosk`: 34 fr / 550 ms, 31 fr / 500 ms). `HideMenuBar|HideDock` keep AppKit's title strip off (y −32, alpha 0): the top edge shows the kit bar alone, no menu bar, no lights, corner 0 (`fix2/`). Ceiling: with the menu bar (`.autoHideMenuBar`) AppKit lays its strip with dimmed lights over the kit bar (row 17) |

Tiled is a ceiling: macOS 27.2 has no public tiled state. The SDK has
`isZoomed` and the full-screen tile behaviors only, the Window menu items
call private actions (`_zoomLeft:`, `_zoomFill:`), and AppKit posts only
`windowWillStartLiveResize`, `windowDidResize` and `windowDidEndLiveResize`.
Firefox needs a private API for the same state
([bug 2054999](https://bugzilla.mozilla.org/show_bug.cgi?id=2054999)). No
design looks different when tiled, so `WindowStatus` has no tiled field.
"Tiled windows have margins" is off on this Mac
(`EnableTiledWindowMargins = 0`); with margins on, nothing was measured.
Probe `~/src/probes/tile/`, captures `~/src/captures/tile/`.

Rows A and B: probe `~/src/probes/csd/` (`states.sh` drives one window
through every state, `greenmenu` presses the green button's menu,
`shape.py` reads corner and shadow, `swift-twins.swift` adds `.onHover` to the
`fs-swiftui` variants), captures `~/src/captures/csd-ab/`, display 2560×1440
at 1x, Dark. A SwiftUI `WindowGroup` with a toolbar opened no window at launch
from a fresh bundle; File > New Window opened it. The second pass closed the leading offset of the first toolbar item, the 28 px
expanded items, the title-row ink and the expanded toolbar row riding with the
menu bar. A third pass (2026-11-23, `~/src/csd-test`: eight SwiftUI designs and
their kit twins through `states.sh`) closed the compact title (now at the
start, x 94), `showsTitle: false` (`Titlebar::shows_title`), the bare
`.hiddenTitleBar` lights at (8, 8), AppKit's bar under the compact bar, the
pinned 52 px expanded strip with its separator, the title riding at the top
edge (its own layer in the unified and compact bars, AppKit's title in the
expanded strip), the full-screen start (`TITLEBAR_FULLSCREEN_START`) and the
frame size of a `TitleBar` window; all within 1 px of Swift. A fourth pass closed
the rest: the expanded row rides centered in AppKit's 36 px bottom accessory
(1 px from Swift), the windowed expanded bar lies on AppKit's bar with AppKit's
title row and its empty toolbar hidden (68 px, 41/40/46 against 40/40/45), the
sidebar layout's bar is the window material under white at
`TITLEBAR_SYSTEM_VEIL` with AppKit's separator (41/40/46 against 40/40/46; none
in Light, as Swift), and `.windowToolbarFullScreenVisibility(.onHover)` is
`WindowToolbarFullScreenVisibility::OnHover`: AppKit's `.autoHideToolbar`, no
kit bar at rest, the sidebar's rows from the top (16 lower before
2026-11-24) and the row riding at the edge over a window-wide scroll pocket,
veiled over the detail as Swift ([OnHover](../barrierkit/09-titlebars.md#onhover)); a press in the revealed row
leaves no stale frame. Nothing in rows A to C is owed.
Light was measured on the Swift twins only (`-light` sets `NSApp.appearance`);
the kit follows the system and was captured in Dark.

Rows 13 and 14 of [fullscreen-titlebars.md](fullscreen-titlebars.md)
(title-bar accessories) belong to design B; rows 15, 16 and 19 (kiosk, keep
the Dock, auto-hide the menu bar) to design D.

## 3. Rules every session follows

1. Build a Swift twin for the design and the kit window from the same recipe,
   and capture both in every state of section 1 (light, active and inactive).
   Use `~/src/probes/fs/` and its harness first; take
   `/tmp/barrierkit-screen.lock` (`mkdir`) for every capture.
2. Fill your row of section 2 with numbers (frame, content shift, corner,
   shadow, what the top edge reveals) and the capture names.
3. Implement in the kit what Swift does and the kit does not, with the
   SwiftUI name (glossary rule). A design Swift allows but no app needs stays
   out ("Nada por si acaso").
4. Record what cannot be reproduced without private API as a ceiling, with
   its cause.

## 4. Sessions

| Session | Designs | Files it owns |
| ------- | ------- | ------------- |
| `csd-titled-toolbar` | A and B | `titlebar.rs`, the `TitleBar` branch of `platform/macos` |
| `csd-hidden-titlebar` | C | the `HiddenTitleBar` branch of `platform/macos`, `chrome.rs` |
| `csd-plain` | D | the `Plain` branch of `platform/macos`, `traffic.rs` |
| `csd-tiling` | tiled column of A to D | `WindowStatus` if a public tiled signal exists |
