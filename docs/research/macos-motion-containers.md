# macOS motion of containers and navigation

**Goal:** record the animation that each container and navigation element of
SwiftUI and AppKit plays by default on macOS, with a duration and a curve the
kit can copy.
**Scope:** sidebar and inspector show and hide, column resize, split view
collapse, list and table rows (insert, remove, move, selection, hover), outline
and disclosure expand, tab switch, `NavigationStack` push and pop, overlay
scroll bars, scroll-to, toolbar show and hide. It ends at the table and the
kit comparison. It does not design kit API and does not cover the desktop
shell.
**Reader:** whoever changes a duration, spring or transition of a kit
container, or writes a container that should move like the system one.
**Input:** the probe in `/Users/humbe/src/probes/containers` (SwiftUI in
`NSHostingView` and AppKit, sampled in process), the public
`NSAnimationContext` default, and [macOS animations](macos-animations.md) and
[UIKit animations](uikit-animations.md) for the method.
**Output:** the [motion table](#2-motion-table), the
[kit comparison](#3-kit-comparison), the [list of what is not
measured](#4-not-measured) and the [commands](#5-how-to-reproduce).

## Steps

1. Read [Method and limits](#1-method-and-limits).
2. Take the row of the element in [the table](#2-motion-table). Trust a fit
   only with its rmse.
3. Compare with the kit in [the kit comparison](#3-kit-comparison).
4. Rerun a row with [the commands](#5-how-to-reproduce).

## 1. Method and limits

Snapshot: macOS 27.2.1, Apple-silicon Mac, 2560x1440 at 1x, probe windows
forced Aqua, Reduce Motion off (`NSWorkspace.accessibilityDisplayShouldReduceMotion`
reads false), 2026-11-25. One run per row unless a range is shown.

- **Sampling.** A 2 ms timer on the main run loop reads the model frame and
  the `presentationLayer` of the views (and, for SwiftUI content, of every
  layer under the hosting view: opacity, x, y, width). No video. The state
  change is made by the probe, so no pointer is involved, except for the drag
  rows, where `NSEvent`s posted into the probe window drive the divider.
- **Hook.** `CALayer.add(_:forKey:)` is swizzled and logs every `CAAnimation`.
  Most container motion does **not** go through it: `NSSplitView`, the
  `NSScroller` fade and the SwiftUI springs change the model value each
  display refresh, so the curve is read from the samples.
- **Words.** `t10`, `t50`, `t90` are the times after the trigger at which the
  travel reaches 10, 50 and 90 %; `settle` is the last time it is 1 % away
  from the end. A **fit** is a curve fitted to the samples, searched over a
  delay and a duration; it describes the motion, it is not Apple's parameter.
  `ease-in-out` is the Bézier (0.42, 0, 0.58, 1), `ease-out` is (0, 0, 0.58,
  1), `CA default` is (0.25, 0.1, 0.25, 1). Springs are critically damped
  (`ζ` = 1) and given by `ω0` in rad/s; SwiftUI `Spring(duration:, bounce: 0)`
  has `duration = 2π / ω0`.
- **Delay.** Every fit has a delay of 0 to 12 ms: one display frame between
  the call and the first changed frame. It is not part of the curve.
- **Not tested.** Interruption (retargeting in flight) was not tried on any
  row. Reduce Motion was only read, never changed: its effect on every row is
  unmeasured.
- **Selection colours** were read with `cacheDisplay`, which draws the model
  state, so a Core Animation fade of a highlight would not show; the hook saw
  no animation on those rows.
- **Screen.** The probe windows were on screen while the probe ran, under the
  shared lock `/tmp/barrierkit-screen.lock`. No capture and no recording.
- **Earlier notes.** The memory note on tab view, scroller and table probes
  holds layouts only, no motion; nothing was reused from it.

## 2. Motion table

Widths are points. "Rows below" means the views under the changed row.

### Split views, sidebar, inspector

| Element and trigger | Animated | Duration and curve | Source | Kit status |
|---|---|---|---|---|
| SwiftUI `NavigationSplitView` sidebar hide and show, binding changed inside `withAnimation` | Width of the sidebar pane 220 to 0 and back (the pane keeps its place, the glass shrinks) | `ease-in-out` over 248 ms, delay 10 ms; t10 66, t50 133, t90 205, settle 241 ms; rmse 0.004 (2 runs, 0.004 to 0.005) | measured | `split_view.rs` `animation`: `DURATION_BASE` 250, `ease_in_ease_out`: matches |
| The same, binding changed **without** `withAnimation` | Same property | None: hidden in one frame | measured | `SplitViewStyle::animation` is the default of `with_animation` only: the sidebar is instant unless the toggle (`SidebarControl::toggle`) or `with_animation` asks: matches |
| The toolbar toggle, `toggleSidebar:` sent through the responder chain to a SwiftUI split | Same | Same curve, settle 235 to 241 ms (2 runs) | measured | matches |
| AppKit `NSSplitViewController.toggleSidebar(_:)`, `splitViewItem.animator().isCollapsed` | Width of the sidebar 300 to 0 and back; the content item grows by the same amount on the same curve | `ease-in-out` over 248 ms; t50 128, t90 200, settle 238 ms; rmse 0.007 (4 runs) | measured | matches |
| AppKit `splitViewItem.isCollapsed = true` (no animator) | Same | None, one frame | measured | n/a |
| The same with `NSAnimationContext.runAnimationGroup { duration = 0.5 }` | Same | Still 248 ms: the split view item ignores the context duration | measured | A kit duration token is the only knob |
| `NSAnimationContext.current.duration` | n/a | 0.25 s (read) | doc and read | `duration.base` 250 |
| AppKit `splitView.setPosition(_:ofDividerAt:)`, with and without `animator()` | Divider position | None, one frame in both | measured | The kit's key resize (← and →) is one frame: matches |
| AppKit live drag of the divider (posted events, 400 ms drag) | Width of the sidebar | Follows the pointer 1:1, within one 8 ms event (width 300 to 379 for pointer 303 to 380); no smoothing | measured | `resize_animation` removed; the width follows the pointer 1:1 in `SplitView`: matches (not run against a native pair) |
| Drag past the maximum (pointer 700, maximum 400) | Width | Clamps at 400 in the frame the pointer passes it; no animation | measured | `resize_to` clamps to `sidebar_range`: matches (not run against a native pair) |
| Drag below the minimum (minimum 160, pointer 300 to 40 in 400 ms) | Width | Holds 160 until the pointer is at about 76 to 71, then goes to 0 in one frame (no animation, the content takes the space in that frame) | measured | hides in one frame below half the minimum and shows again above it: matches (threshold is half, not 71 to 76 of 160; not run against a native pair) |
| Double-click on the divider of an expanded sidebar (events posted) | Width | 300 to 160, the minimum, in one frame | measured (1 run) | `ColumnGrip` double click sets the minimum in one frame: matches |
| SwiftUI `.inspector(isPresented:)` show and hide | Width of the inspector pane 0 to 260 and back | `ease-in-out` over 248 ms, settle 233 to 245 ms; rmse 0.004 to 0.007 (4 runs). **It animates with and without `withAnimation`** | measured | `animation` of the split view: matches the curve; the inspector always animates: matches |
| AppKit inspector item, `animator().isCollapsed` | Width 0 to 300 and back | Same curve, settle 238 to 240 ms | measured | matches |
| AppKit inspector item, `isCollapsed` (no animator) | Width | None | measured | n/a |

### Rows, selection, outline

| Element and trigger | Animated | Duration and curve | Source | Kit status |
|---|---|---|---|---|
| `NSTableView.insertRows` and `removeRows`, options `.effectFade`, `.effectGap`, `.slideDown`, `.slideUp` | `position` of the rows below by one row (24 pt); `.effectFade` also fades the new row 0 to 1 (remove: 1 to 0) | `ease-out` over 200 ms, delay 4 to 10 ms; t10 19, t50 75, t90 153, settle 189 to 194 ms; rmse 0.005 to 0.008 (8 runs). The four options move the rows alike | measured | differs: `Table` follows SwiftUI's list (next rows); ease-out 200 ms only through `TableStyle.animation` |
| `NSTableView` insert or remove with no options (`[]`) | Rows below | None, one frame | measured | n/a |
| `NSTableView.moveRow(at: 2, to: 6)` | Rows between | `ease-out`, 400 ms for a move of 4 rows (t50 143, t90 305, settle 375 ms; rmse 0.002). The duration for other distances was not measured | measured (1 run) | differs: a moved row slides (FLIP) along SwiftUI's 248 ms CA default |
| `NSTableView.reloadData()` | Rows | No row moves; a cross-fade `CATransition` is added to the table layer (its duration reads 0 in the hook) | measured | n/a |
| SwiftUI `List` insert, remove, move inside `withAnimation` | `position` of the rows below | `CA default` over 248 ms, delay 6 to 10 ms; t50 80 to 85, t90 163 to 165, settle 221 to 227 ms; rmse 0.005 to 0.008 (3 runs). `.smooth` and `.easeInOut` give the same curve: the list ignores the SwiftUI curve | measured | matches: `List` and `Table` rows, `ease_default` over `DURATION_LIST_ROW_MS` 248, keyed by the row strings; not run against the probe |
| The same without `withAnimation` | Rows | None, one frame | measured | n/a |
| SwiftUI `List(children:)` (outline), click on the disclosure triangle (event posted) | Rows below the group | `CA default` over 248 ms; t50 80 to 87, settle 222 to 229 ms; rmse 0.0045. Collapse keeps the child rows mounted until about 250 ms | measured | `DisclosureGroup` (the kit's tree of them): `ease_default`, `DURATION_DISCLOSURE_MS` 248, both ways, child rows mounted until the end; matches |
| SwiftUI `DisclosureGroup` in a `List`: binding in `withAnimation`; click | Rows below | `CA default`, 248 ms (rmse 0.005); same for expand and collapse | measured | `DisclosureGroup`: matches (`DisclosureGroupStyle::animation`) |
| The same, binding without `withAnimation` | Rows | None | measured | n/a |
| SwiftUI `Section(isExpanded:)` in a sidebar `List`, `withAnimation` | Rows below | `CA default`, 248 ms (rmse 0.004); without `withAnimation`, one frame | measured | `SidebarSection`: `ease_default` over `DURATION_DISCLOSURE_MS` 248; matches |
| `NSOutlineView.animator().expandItem` and `collapseItem` (also `expandChildren: true`) | Rows below | `ease-out` over 200 ms, delay 8 to 12 ms; t50 78 to 83, t90 158 to 164, settle 191 to 196 ms; rmse 0.007 (4 runs). The scroll clip bounds animate on the same curve | measured | `OutlineView`: only an expansion animates, `ease_out_appkit` (CA `easeOut`, matches t50 and t90) over `DURATION_OUTLINE_EXPAND_MS` 200 by a drawer of the revealed rows; collapse is instant. The scroll clip is not animated; the delay of 8 to 12 ms is not copied |
| `NSOutlineView.expandItem` (no animator) | Rows | None | measured | n/a: the kit animates a press on the chevron, as a click does |
| Selection of a row: `NSTableView` styles inset, source list, plain; SwiftUI sidebar `List` | Highlight | None: the highlight changes in one sample; no `CAAnimation` is added | measured | not checked |
| The window loses and regains the key state (inactive tint) | Highlight colour | None seen (one step; see the limit on `cacheDisplay`); no animation on SwiftUI layers either | measured | not checked |
| Pointer moving over rows of a SwiftUI `List` (events posted) | Row | No layer changes in 1 s: a plain `List` has no hover effect | measured (1 run) | not checked |
| Table header click (sort indicator) | Indicator | One step; no `CAAnimation` | measured | not checked |
| Table column reorder by dragging a header | Header cells | `CABasicAnimation` on `position` and `bounds` of the other header cells (CA default function) and an opacity 0.6 on the dragged cell; the duration was not read | measured (hook only, 1 run) | missing |

### Tabs, navigation, scrolling, toolbar

| Element and trigger | Animated | Duration and curve | Source | Kit status |
|---|---|---|---|---|
| SwiftUI `TabView(selection:)`, tab changed with and without `withAnimation` | Content | None: no cross-fade, content replaced in one frame; no layer of the first ten varying ones moves more than 1 pt | measured | kit has no tab animation: matches |
| AppKit `NSTabView` (classic tabs), `selectTabViewItem(at:)` | The selection pill of the tab strip, x 2 to 47 then 47 to 91 | Spring `ζ` 1, `ω0` 18.0, delay 8 ms (rmse 0.007 to 0.008); t10 38 to 43, t50 98 to 103, settle 380 ms. No effect on the content | measured (2 runs) | `StripPill` in `tabs.rs`: one pill slides on `TabViewStyle::animation`, `SPRING_SHEET_MS` 344 (`ω0` 18.3, no 8 ms delay): matches |
| `NSTabViewController`, `selectedTabViewItemIndex` | Content | None. `transitionOptions` defaults to `.allowUserInteraction` (4096): no `.crossfade` unless the app sets it | measured and read | matches |
| SwiftUI `NavigationStack` push and pop inside `withAnimation` | The pushed view slides in from the right edge by its own width (x 400 to 0) and out again on pop; the root view stays, with no parallax and no dimming | Spring `ζ` 1, `ω0` 12.5 (rmse 0.005); t10 54, t50 147, t90 327, settle 540 ms (3 runs) | measured | `navigation_stack.rs`: `SPRING_PUSH_MS` 503 (`ω0` 12.5), `NAVIGATION_PARALLAX` and `NAVIGATION_DIM` 0; matches inside `withAnimation`; a path change outside it also animates, SwiftUI's is instant |
| The same without `withAnimation` | Views | None | measured | n/a |
| SwiftUI `withAnimation { }` with no curve and `.smooth`, a width 100 to 300 | Frame | Identical: spring `ζ` 1, `ω0` 12.5 (duration 0.5, bounce 0; rmse 0.005); t50 148, t90 325, settle 551 ms. `.snappy`: settle 376 ms, overshoot 0.3 % | measured | `Animation::Spring` exists |
| Overlay scroll bar, `flashScrollers()` (`-AppleShowScrollBars WhenScrolling`, 2 runs) | Opacity of the scroller layer | Fade in 0 to 1: t10 115, t50 253, t90 394, full at 501 ms (S curve). Holds 1 until 1008 ms. Fade out: t90 1060, t50 1128, t10 1201, 0 at 1257 ms | measured | `SCROLL_BAR.fade_in_ms` 501 and `fade_out_ms` 249 along `ease_in_ease_out`, hidden `hide_delay_ms` 1008 after the last scroll: matches |
| `NSClipView.animator().setBoundsOrigin` (a 800 pt jump) | Scroll offset | `CA default` over 244 ms; t50 76 to 84, settle 218 to 226 ms; rmse 0.007 to 0.008 (2 runs) | measured | `scroll_to` animates with the `Animation` the app gives: no default curve; `Animation::Custom` of 244 ms with `ease_in_ease_out` approximates it |
| `NSClipView.scroll(to:)` | Scroll offset | None | measured | n/a |
| SwiftUI `ScrollViewReader.scrollTo` inside `withAnimation` (a 600 pt jump) | Scroll offset | Default curve: `ease-in-out` over 212 ms, rmse 0.009; t50 108 to 111, settle 190 to 191 ms (2 runs). It follows the curve given: `.linear(duration: 0.5)` runs 500 ms linear (rmse 0.003); `.snappy` settles in 362 ms with 0.7 % overshoot | measured | `move_to` follows the `with_animation` curve and duration, through `effective` (speed, Reduce Motion): matches; the 212 ms default is not a kit default, the app passes it |
| The same, no `withAnimation` | Scroll offset | None | measured | matches: `Animation::None` without a transaction |
| `NSToolbar.isVisible`, `NSWindow.toggleToolbarShown(_:)` | Title bar height 52 to 32 and the content top | None: one frame, both ways | measured | not checked |

## 3. Kit comparison

Kit tokens are in `crates/ctdesign/tengen/tokens.json`; the durations are
listed in [Animations](../barrierkit/14-animations.md#durations).

1. **Matches.** Sidebar and inspector show and hide (`ease-in-out`, 250 ms),
   no tab cross-fade, the tab strip pill (`ω0` 18.3) and the navigation push
   and pop (`ω0` 12.5, no parallax, no dimming). Selection and hover were not
   checked in the kit.
2. **Differs.** Sidebar drag eases over 120 ms where AppKit follows the pointer
   1:1. A programmatic `NavigationPath` change animates in the kit; SwiftUI
   without `withAnimation` is instant. Sidebar section collapse uses the 344 ms sheet spring;
   SwiftUI uses a 250 ms `CA default` curve.
3. **Missing.** Row insert, remove and move (AppKit: 200 ms `ease-out`;
   SwiftUI: 250 ms `CA default`), outline and disclosure expand.
4. The two list timings differ because SwiftUI's `List` sets its own
   transaction: a SwiftUI `List` is not the `NSTableView` default.

## 4. Not measured

- **Reduce Motion** for every row. The setting is the owner's; it was only
  read.
- **Interruption** of every row (retarget in flight).
- **Swipe actions.** A phased horizontal scroll built with
  `NSEvent(cgEvent:)` and sent to the window revealed nothing in 1.5 s (12
  events); a real trackpad is needed.
- **Table column resize by dragging the header boundary.** Two attempts with
  posted events changed no width. Column reorder moved the other header cells
  with a Core Animation animation whose duration was not read.
- **`NavigationLink` press.** The accessibility tree of the hosted view gave no
  element to press; only the binding path was measured.
- **Form and `GroupBox`.** A `DisclosureGroup` in a grouped `Form` gave
  layer motion that came 800 ms late in the `withAnimation` run and could not
  be read; `GroupBox` has no state of its own to animate and was not probed.
- **New row in a SwiftUI `List`.** The probe could not tell whether the
  inserted row fades.
- **Sidebar and content column widths of a three-column `NavigationSplitView`**
  (live drag, divider double-click reset, snap) in SwiftUI. Only AppKit was
  driven. The SwiftUI detail pane did not resize while the sidebar moved
  (its frame stayed 900 wide; the content is inset by the safe area).
- **`NSScrollView.scrollRowToVisible`** (the table was too short to scroll),
  **real scroll-wheel** scroll bar fade, legacy (always visible) scrollers.
- **Drag-to-reorder of list rows**, `Table` sort animation, dragging a sidebar
  collapse back open.
- A Catalyst row exists for list insert and remove (0.3 s, `ease-in-out`) in
  [UIKit animations](uikit-animations.md#2-motion-table); it is a different
  platform.

## 5. How to reproduce

The probe sources are `/Users/humbe/src/probes/containers/*.swift` plus
`build.sh`, `run.sh`, `an.sh`, `analyze.py` and `series.py`. They stay outside
the repository. `run.sh` holds `/tmp/barrierkit-screen.lock` while the probe
windows are up.

```sh
cd /Users/humbe/src/probes/containers
./build.sh                                   # swiftc -> $S/probe
./run.sh L splitui splitak tableak tableot listui tabsui nstab sel tcols toolbar hover
./run.sh L scroll -AppleShowScrollBars WhenScrolling
RUN=L ./an.sh sui-split-hide-anim -fit -k '^ew3$'       # rows of a case
RUN=L python3 -I series.py ak-drag-collapse ew0,ptr 25 0 800   # raw series
```

Families: `splitui`, `splitak` (sidebar, inspector, drag), `tableak`, `tableot`
(`NSTableView`, `NSOutlineView`), `listui` (SwiftUI `List`, outline,
disclosure, section), `tabsui` (tabs, `NavigationStack`, default animation),
`nstab`, `sel`, `tcols`, `scroll`, `toolbar`, `hover`.
