# Native window chrome on macOS (level 2)

Goal: decide whether the kit declares the window chrome (toolbar, sidebar, title bar) and macOS draws it, while Freya draws the content. Level 1 is what the runner does today: the kit draws the chrome and macOS draws the backdrops and the glass under it ([blur](macos-arm-port.md#blur-three-proposals)). Level 3, native controls inside the content, is rejected: the kit keeps drawing every control so the twin bench still measures it.

Scope: the experiments below and a proposal. No code of `crates/bk` changes with this document.

Reader: the owner and the sessions that implement the runner.

Inputs: macOS 27.2, SDK 27.2, a 1x display, `docs/research/macos-arm-port.md` (hybrid mode, E8), `docs/research/swiftui-api-gap.md` (item 7, the declarative toolbar). Output: the measurements, the API mapping and a phased plan. The probes live outside the repository in `~/src/probes` (`nchrome.swift`, `nchrome.sh`, `nchrome-all.sh`, `nchrome-events2.sh`, `bandlag.swift`); the captures were deleted after reading.

## Steps

1. Build `nchrome`: an `NSWindow` (`titled`, `fullSizeContentView`, unified toolbar) with an `NSSplitViewController` (a sidebar item and a content item) and an `NSToolbar` (sidebar toggle, tracking separator, a button, a two-segment group, flexible space, a button, a search item). Both columns hold a `CAMetalLayer` view that stands for the Freya surface: the content draws 12 pt gray rows and a red band that moves at 480 pt/s, the sidebar is transparent and draws rows.
2. Run it as an app bundle under `/tmp/barrierkit-screen.lock`, in light and dark, and read the view tree, the event log and the screenshots.
3. Link the same binary against SDK 14 and 15 with `vtool` and compare.
4. Read the measures into the table below and write the plan.

## What AppKit draws around a Metal surface

The tree of the window (light, 900 x 560, scale 1):

| Part | Class | Measure |
|---|---|---|
| Title bar | `NSTitlebarContainerView` | 52 high; the content view's `safeAreaInsets.top` is 52 |
| Each toolbar item | `NSToolbarPlatterView` with an `NSGlassEffectView` (regular) | 36 high at top 8, radius 18; one platter per item, one for the whole two-segment group (74 wide) |
| Sidebar toggle | `NSToolbarToggleSidebarItem` | 36 x 36 at x 96, next to the lights (x 19, 42, 65) |
| Search | `NSSearchToolbarItem` | a 289 wide platter at the right edge, a real `NSSearchField` |
| Top of the content | `NSScrollPocket` inside `NSTitlebarBackgroundView` | exists although the content is a Metal view and not an `NSScrollView` |
| Sidebar | `NSGlassEffectView` 200 x 560, radius 0, hosting the kit's view | the Metal view is above the glass and not opaque |
| Content | `_NSSplitViewItemViewWrapper` at x 200 | `safeAreaInsets.top` 52, left 0 |

The numbers agree with the kit's tokens (52 bar, 36 slots; [macos-control-metrics](macos-control-metrics.md)).

## Measurements

| # | Question | Result |
|---|---|---|
| a | Does the glass or the title bar lag behind a Metal layer that moves? | 16 shots in light and 16 in dark of the red band: its centroid in the content, in the title-bar rows (y 2 to 6) and in the search capsule differs by at most 0.5 px in the title bar and 1.3 px in the capsule (a band half under the capsule's edge is left out). One frame at 60 Hz is 8 px. The whole window is composed by WindowServer, so no lag comes from AppKit's side |
| a2 | What does the title bar do to the content under it? | The tree holds an `NSScrollPocket` and the rows 0 to 51 over the gray rows are flat (deviation 0, tone 246 at the top, 233 at row 45, 236 at row 51 in light): the content is blurred and veiled without any `NSScrollView`. A kit pocket in the same bar would draw twice |
| b | Window drag | A drag from the title, and from the title-bar rows above the capsules over the Metal view, moves the window (4 moves, total +99, -49 for a +100, +50 drag). A drag from the content 18 pt under the bar is delivered to the Metal view and does not move the window. The first click after launch only activates |
| b2 | Resize | A drag from inside the bottom-right corner resizes natively (16 steps in 0.34 s). A drag from just outside the corner was not delivered to the window in this run |
| c | Keyboard | A key goes to the Metal view that clicked last. `performKeyEquivalent` reaches it before the menu (Command+K, twice, then `keyDown`). A click on a toolbar button fires its action and leaves the first responder in the Metal view (the next key arrived there). A click in the search field takes the keyboard (the next key did not arrive). A click in the sidebar's Metal view arrives as `mouseDown` |
| d | Toolbar items | `NSToolbarItem` with `isBordered`, `NSToolbarItemGroup(selectionMode: .selectOne)`, `NSSearchToolbarItem`, `.toggleSidebar`, `.sidebarTrackingSeparator`, `.flexibleSpace` all get a glass platter with no code. The SDK has `style` (macOS 26), `badge`, `backgroundTintColor`, `navigational`, `visibilityPriority`. Not measured: overflow at a narrow width, customization, `prominent` |
| e | Sidebar and collapse | The glass is native and flat, light and dark. `toggleSidebar` takes 0.29 s: the content view grows 700 to 900 pt, the sidebar view keeps its 200 pt frame (it slides). The Metal drawable followed the view within 7 pt at the 0.1 s and 0.19 s samples: one display tick |
| f | Does the look follow the SDK the binary links against? | Yes (below) |
| d2 | Reduce Transparency (system setting on, the owner's) | The tree is the same (6 glass views, the pocket, 5 platters) and AppKit draws them opaque by itself: the title-bar rows are flat 255 in light, the capsules and the sidebar are solid in dark, and the red band never shows through the title bar or a capsule in 16 shots of each appearance. The kit needs no code for it in native chrome |
| g | Light and dark | Both draw the glass, the pocket and the sidebar; the lights, the title and the platters change with the window appearance (`NSApp.appearance`) |

Not measured: Increase Contrast (a system setting; `~/src/probes/nchrome-scroll.sh <aqua|darkAqua> <dir>` runs it), scale 2, a real Freya surface in place of the probe's Metal, an inspector column, and VoiceOver.

### The SDK the binary links against

| Linked SDK | Glass views | Pocket | Toolbar button | Sidebar |
|---|---|---|---|---|
| 27.2 (the installed one) | 6 | yes | 36 x 36 glass platter | glass |
| 15.0 and 14.0 (`vtool -set-build-version`) | 0 | no | 46 x 40, no platter | `NSVisualEffectView` sidebar material (7) and a 52 high titlebar material (3) |

The kit's own binaries are built with minimum macOS 11.0 and SDK 27.2 (`vtool -show-build target/debug/examples/button_label`), so they get the new look because the Xcode that builds them is new. A build on an older Xcode gets the old look. The Rust build can set the field with a linker argument (`-platform_version`); it was not tried from Cargo, only through `vtool`.

## What level 2 adds to level 1

Level 1 already has the glass capsule (13.3% of its pixels differ from SwiftUI's), the glass sidebar and the pocket as a kit effect ([M5](macos-arm-port.md#blur-three-proposals)). Level 2 adds:

| Gain | Why it is not in level 1 |
|---|---|
| Real SF Symbols in the items, as AppKit draws them | the kit draws its own SVG symbols |
| Item layout, spacing, grouping and overflow rules that change with the system | the kit copies today's numbers |
| A native search field: editing, IME, menu, clear button, focus | the kit's `ToolbarSearch` is a kit text field |
| Accessibility of the toolbar, full keyboard access, tooltips | the kit exposes its own tree |
| The pocket and the title-bar veil | the kit draws them as an effect |

The pixel difference between level 1 and level 2 was not measured.

## Mapping

The kit has no declarative toolbar: `Titlebar::left` and `right` take any element, so the runner cannot tell a button from a search field (`swiftui-api-gap.md`, item 7). Level 2 needs data, as the native menu bar did with `MainMenu` and `MenuHandle` (a snapshot, a revision and an action queue).

| Kit declaration (SwiftUI name) | Native | On Linux |
|---|---|---|
| `Toolbar` with `ToolbarItem(placement:)` | `NSToolbar` and `NSToolbarItem` | the kit's `Titlebar` draws the same items (today's `ToolGroup` capsules) |
| `ToolbarItemGroup` | `NSToolbarItemGroup` (`selectOne`, `momentary`) | one `ToolGroup` |
| `ToolbarSpacer` | `.flexibleSpace`, `.space` | a gap |
| `ToolbarItem(placement: .navigation)` | after the tracking separator | leading slot |
| `ToolbarItem(placement: .primaryAction)` | trailing | trailing slot |
| `.searchable` or `ToolbarSearch` | `NSSearchToolbarItem` | `ToolbarSearch` |
| `SidebarToggle` | `.toggleSidebar` | `SidebarToggle` |
| `SplitView` (`NavigationSplitView`) | `NSSplitViewController`, a sidebar item and a content item, one surface per column | one surface, today's layout |
| `Label` symbol | `NSImage(systemSymbolName:)`, needs an SF name per `Symbol` | the SVG |
| the item action | the `NSToolbarItem` target posts the id; `menu_actions`-like dispatch | the callback |
| `.help` | `toolTip` | the kit's tooltip |

## Cost and risks

1. The API is the larger part: a `Toolbar` model with ids, placements, state and a revision, a `WindowOptions::toolbar`, a Linux renderer of the same model, the glossary rows, a recipe and `agents/`. It is item 7 of the gap list either way.
2. The runner needs an `NSToolbarDelegate` and an action target (two `define_class!` objects like `ContentView`), about 600 lines, and the SF name table.
3. One surface per column means one `AppInstance` per column, as popups already are. State must be shared through the app's signals; the sidebar and the content can no longer be one Freya tree with one layout. This is the biggest risk and is not tried with Freya.
4. The kit must read the safe area and the native sidebar's frame each tick instead of animating `SidebarControl` itself.
5. Apps built with an SDK older than 26 get the old chrome (table above): the docs must name the floor.
6. The first click after activation only activates, as it does for any macOS window.
7. Unmeasured: overflow, customization, Increase Contrast, 2x.

## Proposal

Phases, each one commit in `main` and each testable:

1. The declarative `Toolbar` (model, handle, Linux renderer, glossary, recipe), no macOS change. It is mergeable alone, it is gap item 7, and its mapping to items is pure and testable on Linux.
2. macOS: the runner builds the `NSToolbar` from the model in hybrid mode (items, group, spacer, search, toggle, SF Symbols) and the kit draws nothing in that bar. Checked with a twin against SwiftUI (`notes`).
3. macOS: `SplitView` on `NSSplitViewController` with the sidebar column as its own surface; `SidebarControl` follows the native frame.
4. The inspector column, overflow priority, badges and `prominent`.
5. Remove the kit's own top pocket from windows with a native toolbar.

Decisions for the owner:

1. Whether to start with phase 1.
2. Whether the docs state SDK 26 as the build floor, and whether `bk` sets `-platform_version` itself.
3. One surface per column, or one surface over a native split view that only supplies the glass and the divider.
