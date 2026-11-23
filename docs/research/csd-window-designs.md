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
| A | `WindowGroup`, no toolbar (system bar) | `WindowStyle::TitleBar` | measured, same ([macos-windows.md](macos-windows.md)) | animation same, 21 fr / 333 ms; look owed | same in Swift and kit, nothing changes but the frame: Left `0,30 1280×1325`, Top Left `1280×662`, Fill and a drag to the top `2560×1325`, Center moves only; corner 16, shadow 56 px, lights and focus as windowed; Swift `isZoomed` is true after Fill only (`tile/swift-A-*`, `tile/kit-app-title-bar-*`) | same as Swift rows 1 and 2 of [fullscreen-titlebars.md](fullscreen-titlebars.md); the kit `Titlebar` disappears |
| B | `.toolbar{}` with `.windowToolbarStyle(.unified)`, `.unifiedCompact`, `.expanded` | `Titlebar::toolbar_style` (`Unified`, `UnifiedCompact`; no `Expanded`) | kit variants in [09](../barrierkit/09-titlebars.md#variants-in-a-normal-window); Swift twin owed | owed | Swift `.unified` as A, toolbar unchanged (`tile/swift-B-*`); the kit toolbar sits in A's `NSWindow`, not captured | Swift pins the toolbar and shifts the content 52 / 40 / 68 px (rows 3 to 8); `.autoHideToolbar` hides it (row 11); kit has neither |
| C | `.windowStyle(.hiddenTitleBar)` (alone, with `.toolbar{}`, `.unifiedCompact`, `.toolbarBackgroundVisibility(.hidden, for: .windowToolbar)`, `.toolbar(removing: .title)`, `NavigationSplitView`), AppKit `titlebarAppearsTransparent` + `.fullSizeContentView` | `WindowStyle::HiddenTitleBar` | same. Native lights at window (18, 18) under a unified toolbar (also with the background hidden, the title removed or a split view), (11, 12) under `.unifiedCompact`, (8, 8) with no toolbar; safe-area top 52 / 40 / 32. The kit puts them at (18, 18) under `Unified` and, since this change, (11, 12) under `UnifiedCompact` (was 18, 18); the compact bar now leaves their width free. Inactive grays the native lights in both; corner 16, system shadow (`*-0-active`, `*-1-inactive`). Differs: a kit window with no `Titlebar` keeps (18, 18) (no app needs the bare strip), and the compact bar puts its first item right after the lights where Swift leaves 20 px (spacing shared by every compact bar, design B). Captures `~/Desktop/barrierkit-capturas/csd-c/`, harness `~/src/probes/fs/csd-states.sh` | same: frame = visible frame 0,30 2560x1325, lights keep their window offset, corner and shadow unchanged (`*-2-zoomed`); a double click on the kit bar zooms through AppKit | as A in Swift and kit: Left 0,30 1280x1325, Fill 0,30 2560x1325, lights keep their windowed offset, (8, 8) in Swift with no toolbar, (18, 18) in the kit; corner and shadow unchanged. The kit tiles from the Window menu of an `Application` and by an edge drag of a `run_window` window, which has no Window menu (`tile/swift-C-*`, `tile/kit-app-hidden-title-bar-*`, `tile/kit-notes-hidden-title-bar-*`, `csd-c/swift-*-4-tiled-fill`) | Swift: nothing (no toolbar) or the toolbar pinned with shift 52 / 40; at the edge it slides 30 px with the lights (rows 2, 4). Kit: mid-screen the bar stays at y 0 with no lights gap and no content shift. At the edge, before `907de855` the menu bar covered it and the lights showed at (18, 48) over blank content (`fs/kit-hidden-title-bar-2-fs-top-edge`); since then its glass controls ride beside the lights in AppKit's bar ([ADR 0116](../decisions.md#0116--in-full-screen-the-title-bar-row-of-a-hiddentitlebar-window-rides-with-appkits-bar)), whose style now follows `toolbar_style`. The content does not shift 52 / 40 as under Swift's pinned toolbar, by that ADR |
| D | `.windowStyle(.plain)`, borderless `NSWindow` | `WindowStyle::Plain` | kit lights drawn by the kit | zoom by the green light same as Swift; by title-bar double click does not zoom (not diagnosed) | Swift `.plain` never tiles: the Window menu and a drag do nothing (`tile/swift-D-*`). Kit tiles through Window > Fill, Center and Move & Resize of an `Application` with A's frames, keeping corner 16.8 and its 23 px shadow; not by a drag to an edge (the kit moves the window itself) nor from its lights; a `run_window` window has no Window menu (`tile/kit-app-plain-*`, `tile/kit-notes-plain-*`) | Swift `.plain` never enters; kit enters with one frame jump (2 bounds), hides menu bar and Dock for good |

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
