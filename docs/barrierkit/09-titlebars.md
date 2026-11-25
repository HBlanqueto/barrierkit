# 9. Titlebars

**Goal:** give a window its title bar, choose one of SwiftUI's window designs
and know how the bar looks windowed, in full screen and at the top edge.
**Scope:** `Titlebar`, its `WindowToolbarStyle`, `TrafficLights`,
`WindowChrome` and the bar in Malaysia's focus mode. The window
options and state they read, `WindowStyle` included, are in
[8. Windows](08-windows.md). The sidebar that holds the traffic lights is in
[10. Sidebar](10-sidebar.md).
**Reader:** anyone who designs the window of an app.
**Input:** a window opened with `Application` or `run_window`
([8](08-windows.md#running-a-window)).
**Output:** a window whose bar matches the SwiftUI design it copies, in every
window state.

## Steps

1. Pick the design in [Window designs](#window-designs). A window that keeps
   AppKit's bar (`WindowStyle::TitleBar`, the default) needs nothing more; on
   Linux `Application` draws its `Titlebar`.
2. For any other design, open the window as `WindowStyle::HiddenTitleBar` and
   put a `Titlebar` at the top of the root view:

   ```rust
   Application::new("com.example.notes")
       .window(tr!("notes"), || {
           rect()
               .expanded()
               .child(Titlebar::new().title(tr!("notes")).toolbar(ToolbarItem::new(
                   ToolbarItemPlacement::PrimaryAction,
                   IconButton::new(Symbol::Plus, tr!("new-note")),
               )))
               .child(content)
       })
       .options(|w| w.window_style(WindowStyle::HiddenTitleBar))
       .run()
   ```

3. Choose the bar with `.toolbar_style(..)` and drop the title with
   `.shows_title(false)` when the design does.
4. With a sidebar, pass `.with_sidebar(true)` and `Sidebar::traffic_lights(true)`
   ([10](10-sidebar.md)).
5. To keep the bar out of full screen until the pointer reaches the top edge,
   set `WindowToolbarFullScreenVisibility::OnHover`
   ([8](08-windows.md#window-options)).
6. On Malaysia, if the bar should hide, wrap the window in `WindowChrome`
   ([Linux and Malaysia](#linux-and-malaysia)).

## Window designs

Each SwiftUI design has one kit form. The program that opens every one is
[`window_designs.rs`](../../crates/bk/examples/window_designs.rs)
([recipe 50](../../.claude/skills/barrierkit-app/recipes.md#50-every-swiftui-window-design)); the
sidebar design is [`notes_sidebar.rs`](../../crates/bk/examples/notes_sidebar.rs).
Each pair was captured next to its SwiftUI twin, active and inactive, windowed,
in full screen and with the pointer at the top edge, and matches within 1 to
2 px and one color level in Dark
([the measurement](../research/csd-window-designs.md#2-designs-against-states)).

| SwiftUI | Kit | Bar |
|---|---|---|
| `WindowGroup` (`.windowStyle(.titleBar)`) | `WindowStyle::TitleBar`, no `Titlebar` | AppKit's 32 px bar and lights |
| `.windowStyle(.hiddenTitleBar)`, no toolbar | `HiddenTitleBar`, no `Titlebar` | None: the native lights over the content |
| `.toolbar { }` (`.windowToolbarStyle(.unified)`) | `HiddenTitleBar`, `Titlebar` with items or `.toolbar_style(Unified)` | 52 px, title at the start in `TOOLBAR_TITLE` |
| `.unified(showsTitle: false)` | `Unified` and `.shows_title(false)` | 52 px; the items follow the lights |
| `.windowToolbarStyle(.unifiedCompact)` | `.toolbar_style(UnifiedCompact)`, or a `Titlebar` with only a title | 40 px, title at the start |
| `.unifiedCompact(showsTitle: false)` | `UnifiedCompact` and `.shows_title(false)` | 40 px; the items follow the lights |
| `.windowToolbarStyle(.expanded)` | `.toolbar_style(Expanded)` | 68 px: a 28 px title row over a 40 px toolbar row |
| `NavigationSplitView` with a `.toolbar` | `SplitView`, `Sidebar::traffic_lights(true)`, `Titlebar::with_sidebar(true)` | 52 px over the detail; the lights in the sidebar |
| `.windowToolbarFullScreenVisibility(.onHover)` | `WindowOptions::window_toolbar_full_screen_visibility(WindowToolbarFullScreenVisibility::OnHover)` | As its design windowed; no bar at rest in full screen |

`WindowStyle::Plain` is not a SwiftUI design: it is a borderless frame drawn as
on Linux, with the kit's own lights. SwiftUI's `.windowStyle(.plain)` is
`WindowStyle::Borderless`: no bar, no lights, only the content
([8](08-windows.md#window-style)).

## Titlebar

`Titlebar` (`bk::titlebar`, in the prelude) is the bar the kit draws: the
traffic lights (or room for the native ones), a left item, the toolbar items
and the title.

| Method | Default | Effect |
|---|---|---|
| `Titlebar::new()` | — | A bar that reads the title and state from `WindowControls` |
| `.title(text)` | the window title | Replaces the title shown |
| `.subtitle(text)` | none | SwiftUI's `navigationSubtitle`: in a unified bar the title turns `TOOLBAR_TITLE_SUBTITLED` (13 semibold) over the subtitle in `SUBHEADLINE`; in an expanded bar the title row reads title – subtitle, through `NSWindow.subtitle` where AppKit draws that row |
| `.shows_title(bool)` | `true` | `false` drops the title and lays the trailing items after the lights, as SwiftUI's `.unified(showsTitle: false)` and `.unifiedCompact(showsTitle: false)` |
| `.toolbar(content)` | none | Adds a `ToolbarItem` or a `ToolbarSpacer`; call it once per item ([Toolbar items](#toolbar-items)) |
| `.toolbar_style(WindowToolbarStyle)` | `Unified` with a sidebar, a `.left` view or items, else `UnifiedCompact` | `Unified`, `UnifiedCompact` or `Expanded`, as SwiftUI's `windowToolbarStyle`; in a `HiddenTitleBar` window it also moves the native lights ([8](08-windows.md#window-style)) |
| `.with_sidebar(bool)` | `false` | Aligns the bar with a sidebar: the title at the start of the detail, the lights in the sidebar |
| `.left(view)` | empty; with a `SplitView` above, the `SidebarToggle` | Items before the navigation items |
| `.spacing(px)` | 0; `SETTINGS.toolbar_gap` (13) beside a sidebar | Gap between the bar's items. Beside a sidebar it is the gap before the title, so the title sits 20 from the sidebar's edge |
| `.close_only(bool)` | `false` | Only the close button works, for dialogs and panels |
| `.hairline(bool)` | `false` | Draws `TitlebarStyle::line()` under the bar, as `NSTitlebarSeparatorStyle.line` |
| `Titlebar::preview(status, title, fixed)` | — | Draws the bar for a given `WindowStatus` without a window, for previews and tests |

Layout:

- **Unified:** 52 high. The title is `SETTINGS.toolbar_lights_gap` (22) after
  the lights, in `TOOLBAR_TITLE`. The first item (the sidebar toggle, a `.left`
  view or a `Navigation` item) starts `TITLEBAR_TOOLBAR_START.unified` (16) after
  the lights, and the title follows it `SETTINGS.toolbar_gap` (13) later.
- **Compact:** 40 high, the title in `TITLEBAR.font` where an item would start,
  `TITLEBAR_TOOLBAR_START.compact` (22) after the lights, as macOS 26 does; the
  first item lands at x 94, where SwiftUI puts it. Only a `.close_only(true)`
  bar centers its title.
- **Expanded:** a `TITLEBAR_EXPANDED_TITLE_HEIGHT` (28) title row, its title
  `TITLEBAR_EXPANDED_TITLE_GAP` (15) after the lights in the dimmer
  `TitlebarStyle::row_title` ink, over a `TITLEBAR_CLIENT_HEIGHT` (40) toolbar
  row. Its `Navigation` and `PrimaryAction` items sit together from
  `TITLEBAR_TOOLBAR_START.expanded` (11), `SETTINGS.toolbar_gap` (13) apart,
  centred in the row less `TITLEBAR_EXPANDED_TOOLBAR_BOTTOM` (8) at its
  bottom, in a group 30 high with 37.5 slots (`TOOLGROUP.expanded_*`) whose
  glass shows 28 high: their top 4 under the title row, as SwiftUI's.
- **No title:** with `.shows_title(false)` the trailing items join the
  navigation items after the lights.
- **Beside a sidebar:** the title goes at the start of the detail, next to
  `left`, in `TOOLBAR_TITLE`. While the sidebar shows, the lights live in it and
  keep their place while it folds.
- A press on the bar moves the window; a double click does what
  `double_click_titlebar` says ([Traffic lights](#traffic-lights)).
- `.close_only(true)`: a transparent bar `TITLEBAR.thickness` (32 px) tall,
  with only close enabled. Utility and HUD windows switch to it by themselves
  ([8](08-windows.md#utility-windows)).
- Under a system title bar (`DecorationResponse::Server`) the bar takes no
  space and its toolbar items do not show; the kit warns once in the log. On
  macOS, `WindowStyle::HiddenTitleBar` asks for the bar
  ([8](08-windows.md#window-style)).

### Toolbar items

`.toolbar(content)` is SwiftUI's `.toolbar { }`: it takes a
`ToolbarItem::new(placement, view)` or a `ToolbarSpacer` (`bk::views::toolbar`,
in the prelude) and lays it into the zones above. Call it again for more items;
they keep the order of the calls. Adjacent items whose view is a bare
`IconButton` share one `ToolGroup`, one glass, as SwiftUI groups adjacent
toolbar items; a `ToolbarSpacer` or another view starts a new one.

```rust
Titlebar::new()
    .toolbar(ToolbarItem::new(ToolbarItemPlacement::Navigation, back_and_forward))
    .toolbar(ToolbarItem::new(ToolbarItemPlacement::Principal, Text::new(page)))
    .toolbar(ToolbarSpacer::flexible())
    .toolbar(ToolbarItem::new(ToolbarItemPlacement::PrimaryAction, new_button))
```

The recipe is [`toolbar_items.rs`](../../crates/bk/examples/toolbar_items.rs).

| `ToolbarItemPlacement` | Where it goes |
|---|---|
| `Navigation` | The left zone, after the sidebar toggle |
| `Principal`, `Status` | The title's place: the item replaces the title text and is centered on the window, as AppKit centers it |
| `PrimaryAction` | The right zone |
| `Automatic` | The right zone; with a flexible spacer later in the list, the left zone |

- `ToolbarSpacer::flexible()` ends the left side: the `Automatic` items before
  it go next to the `Navigation` items and the ones after it go to the right
  zone. `ToolbarSpacer::fixed()` adds one gap (`TitlebarStyle::sidebar_padding_x`)
  to the zone where it falls.
- Items in a zone are `sidebar_padding_x` apart. A `.left` view comes first in
  the left zone and replaces the sidebar toggle.
- A bar with toolbar items is the unified one.
- There is no overflow: the bar does not measure its items, so items that do
  not fit are clipped, not moved to a menu.
- macOS 27.2 (probe `toolbarplace`, 2026-11-22): `.navigation` items follow the
  sidebar toggle before the inline title, `.principal` and `.status` items sit
  in the center, and `.primaryAction` and `.automatic` items trail in the order
  declared. Apple's page for `primaryAction` says "leading edge" for macOS; the
  probe disagrees and the kit follows the probe. The same probe showed no effect
  of `ToolbarSpacer` in a plain `.toolbar { }`, so the spacer follows Apple's
  description of it (a gap, or a push).

### Style

`Titlebar::style` takes a `TitlebarStyle`: heights, paddings, background,
title color, font and position, and the hairline. The fields are in
[Styles and themes](../../crates/bk/src/styles/titlebar.rs).

```rust
Titlebar::new().style(|s: TitlebarStyle| TitlebarStyle {
    title_font: HEADLINE,
    ..s
})
```

## On macOS: window, full screen and top edge

A `HiddenTitleBar` window keeps AppKit's frame, so AppKit gives it what it
gives a SwiftUI window: the native lights, the wallpaper tint of the window
background, the inactive look and the full-screen strip. The kit draws only the
rows AppKit does not.

### Windowed

- **AppKit's bar under the kit's row.** Over content that does not scroll
  under it, a unified or compact bar lies on AppKit's own title bar and
  separator, as SwiftUI's toolbar does; over a scroll view the bar has no line
  at rest.
- **Expanded.** AppKit draws the 28 px title row and its separator; the kit
  draws only the toolbar row on AppKit's bar, so the bar is 68 px with
  AppKit's tint.
- **Beside a sidebar.** The detail's bar is the window material under white at
  `TITLEBAR_SYSTEM_VEIL`, with AppKit's separator unless a scroll view of the
  detail reaches under it. In Light the veil adds nothing, as in SwiftUI.
- **Inactive.** AppKit grays the lights and dims the tint; the title takes the
  inactive ink.

### Full screen

The bar stays pinned at the top, as SwiftUI's toolbar does. Without lights the
title and the first item start `TITLEBAR_FULLSCREEN_START` from the edge
(22 unified, 14 compact, 11 expanded). The expanded bar is 52 high: a 12 px
title row, the 40 px toolbar row and a separator. Beside a sidebar the
detail's bar takes the toolbar background, not the sidebar layout's veil.

### The top edge

With the pointer at the top edge, the menu bar comes down with AppKit's strip.
The kit's row rides in that strip with the native lights and the title, as
SwiftUI's. AppKit fills its full-screen bar with an opaque background that no
public setting clears, so the kit's own row cannot show through it:

- the title of a unified or compact bar rides on a layer of its own;
- in the expanded bar AppKit draws the title row, the toolbar row rides
  centered in AppKit's 36 px accessory, and the pinned separator hides, as
  SwiftUI's does.

### `OnHover`

`WindowToolbarFullScreenVisibility::OnHover` is AppKit's `.autoHideToolbar`.
In full screen there is no bar at rest: the content and the sidebar's rows
start at the top. At the top edge the row comes down with the menu bar, in the
same places as SwiftUI's, over the content, which does not move; the scroll
pocket then spans the window, so the row sits on blurred content, and over the
detail it lays the window color at 0.56 alpha with a hard edge, as SwiftUI's
(white reads 130/255 there on macOS 27.2). A press in the row leaves no stale
frame. Windowed it changes nothing.

## Traffic lights

Close, minimize and zoom, drawn as macOS 27 draws them: 14 pt discs with a
vertical gradient, a 0.5 pt ring, a halo in the light appearances and a glyph
while the pointer is over the group. The measurement is in
[macOS control metrics](../research/macos-control-metrics.md#traffic-lights-macos-27).
They are the most seen control of a window, so the kit draws them from that
measurement.
`Titlebar` and `Sidebar::traffic_lights(true)` place them with
`TrafficLights::for_window(focused, fixed)`, which wires them to
`WindowControls` and the user's settings. In a fixed-width window
(`WindowControls::fixed_width`) the green light is a plus and zooms to the full
height. Use it when a view covers the title bar and needs its own lights. A
`HiddenTitleBar` window shows AppKit's own lights instead.

In a bar the kit draws (Linux, `Plain`) the lights sit `TITLEBAR.buttons.inset_compact` (12) from the left when
it is `TITLEBAR_CLIENT_HEIGHT` (40) tall and `TITLEBAR.buttons.inset` (9) when it
is `TITLEBAR.thickness` (32) tall, centred vertically, so 13 and 9 from the top.
`ctappearance::traffic::inset(bar_height)` is the one rule; the `Titlebar` and
the compositor both use it.

The kit stops at the compact style. macOS also has a 66 px toolbar title bar
whose lights sit 19 from the left and 19 from the top, not centred. No window of
the kit uses it, so it has no style, `Bar` value or token here; the
measurement stays in [macOS windows](../research/macos-windows.md).

For a standalone set, `TrafficLights` (`bk::traffic`):

```rust
use bk::traffic::TrafficLights;
use ctdesign::ButtonKind;

TrafficLights::new()
    .focused(true)
    .on_action(|kind: ButtonKind| { /* Close, Minimize or Maximize */ })
```

| Method | Default | Effect |
|---|---|---|
| `.focused(bool)` | `true` | Unfocused lights are gray until the pointer hovers them |
| `.zoom_supported(bool)` | `true` | Without zoom, the green light is gray, has no glyph and does nothing |
| `.fixed_width(bool)` | `false` | The zoom glyph is a plus, for fixed-width windows |
| `.close_only(bool)` | `false` | Only close is colored and clickable |
| `.fullscreen(bool)` | `false` | The full-screen state ([States](#states)); `for_window` reads it from `WindowControls` |
| `.input(TrafficInput)` | — | Draws a fixed state (focus, hover, press) and ignores the pointer, for previews |
| `.on_action(f)` | — | Receives the `ButtonKind` pressed |

Each light has the accessible role button and the name "Close", "Minimize" or
"Zoom". Hovering the group shows the glyphs.

`.style(..)` takes a `TrafficLightsStyle`: size, gap, ring width, halo, and the
`ButtonVisual` of each light in each state
([Styles and themes](../../crates/bk/src/styles/traffic_lights.rs)). The lights that
`TrafficLights::for_window` place take the theme and the containers:

```rust
Theme::tengen().style(|s: TrafficLightsStyle| TrafficLightsStyle {
    spacing: 12.0,
    ..s
})
```

### States

| State | Look |
|---|---|
| Focused window | The lit disc, no glyph |
| Pointer over any light | The three discs with their glyphs: × − and the full-screen arrows (a plus in a fixed-width window) |
| Pressed | The pressed light grows to 16 pt and brightens; the others keep the hover look |
| Full screen | Minimize is disabled (the unfocused gray, no glyph); zoom shows its exit glyph, two triangles that meet at the center. Entering or leaving full screen cross-fades the two zoom glyphs in `TRAFFIC_GLYPH_FADE_MS` (150 ms, `Animation::Ease`; none with Reduce Motion) |
| Unfocused window | Translucent gray discs; the pointer lights them with their glyphs |
| Disabled (zoom without zoom, or `close_only`) | The unfocused gray, no glyph, no press |

With `differentiate_without_color` the glyphs show without the pointer.

### Color

The user decides the rest. The appearance key `traffic_lights` (`standard`,
`accent`, `wallpaper`) is in
[4](04-appearance-and-tokens.md#the-appearance-file). `standard` is the measured
look of each light in each appearance, light, dark and Increase Contrast, and it
is exact. `accent` and `wallpaper` give the lights a tone: close and minimize
the accent (or the wallpaper's dominant color) darkened by
`TRAFFIC_CLOSE_MINIMIZE_DARKEN`, zoom lightened by `TRAFFIC_ZOOM_LIGHTEN`.
`ctappearance::traffic::looks` then recolors the measured look of each light
with that tone: it keeps the alpha, the lightness and the saturation of every
stop relative to the light's base, takes the tone's hue, and keeps the glyph at
3:1 against its fill (WCAG 2.2, 1.4.11). The gray unfocused look does not change.
The old key `traffic_light_shape` is ignored and removed on the next write: the
lights are discs in macOS 27. The behavior keys live in
`~/.config/malaysia/window.domain`
([4](04-appearance-and-tokens.md#window-behavior)); the compositor's own
title bars use the same functions. What each value does here:

| `green_button` | Normal window | Fixed size |
|---|---|---|
| `focus` (default; `fullscreen` is an alias) | Full screen | Maximizes |
| `zoom` | Maximizes | Maximizes |
| `none` | Nothing; the light is gray and has no glyph | Nothing |

| `double_click_titlebar` | Normal window | Fixed size |
|---|---|---|
| `zoom` (default), `fill` | Maximizes | Nothing |
| `minimize` | Minimizes | Minimizes |
| `none` | Nothing | Nothing |

A double click is two presses within `TITLEBAR_DOUBLE_CLICK.interval_ms`
(400 ms) and `slop` (4 px). `use_window_drag(fixed)` is the press handler
that moves the window or runs the double-click action; put it on your own
draggable areas.

## Linux and Malaysia

On Linux the kit draws the whole bar and its lights. The designs above keep
their layout; focus mode and the hidden bars follow Malaysia.

### Corner radius

The `Titlebar` tells the compositor which bar it draws with
`infinity_window_style_v1.set_bar`, and Malaysia rounds the window with that
bar's radius
([Malaysia ADR 0059](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisions.md)).
The app does nothing:

| `Titlebar` | `Bar` | Height | Radius (`WINDOW_CORNER`) |
|---|---|---|---|
| Panel (`UtilityWindow`) | `Utility` | `TITLEBAR.thickness` (32) | `utility` (16) |
| `.close_only(true)` or HUD window | `Titlebar` | `TITLEBAR.thickness` (32) | `titlebar` (16) |
| `Unified` (with a sidebar or items) | `Unified` | `SETTINGS.toolbar_height` (52) | `unified` (16) |
| `UnifiedCompact` | `UnifiedCompact` | `TITLEBAR_CLIENT_HEIGHT` (40) | `unified_compact` (16) |
| `Expanded` | `Titlebar` | 68 | `titlebar` (16) |

`Bar` (`bk::platform`) comes from the Infinity protocol and has four
values, each an AppKit window kind: `Titlebar` (no toolbar), `UnifiedCompact`
(`NSWindow.ToolbarStyle.unifiedCompact`), `Unified` (`.unified`) and
`Utility` (a panel, radius 16). Every radius is 16, as macOS 27.2.1 gives every window kind; the silhouette to draw is `CORNER_WINDOW.fitted` (16.8, [Corners](04-appearance-and-tokens.md#corners)).

An app that draws its own
bar without `Titlebar` declares it with
`WindowControls::get().set_bar(Bar::Titlebar)`; otherwise Malaysia uses
`unified` for a client-decorated window. The user can force one radius for
every window with `corner_radius` in `window.domain`.

### Window chrome

`WindowChrome` (`bk::chrome`) wraps the window and decides when the bar
shows. `Titlebar` and `Sidebar` inside it read that decision.

```rust
use bk::chrome::{Reveal, WindowChrome};

WindowChrome::new(window_content).style(|s| WindowChromeStyle {
    reveal: Reveal::Drawer,
    ..s
})
```

| `WindowChromeStyle` field | Default | Values |
|---|---|---|
| `reveal` | `Always` | When the bar shows (below) |
| `resting_edge` | `None` | What stays while a hidden bar is away: `RestingEdge::None`, `Line` (`TITLEBAR_REVEAL.edge_line`, 1 px) or `Accent` (`edge_accent`, 4 px, accent color) |

The style also holds the pill, its shadow and the animation
([Styles and themes](../../crates/bk/src/styles/window_chrome.rs)); a `Theme`
sets them for every window.

| `Reveal` | What happens |
|---|---|
| `Always` | The bar is always there |
| `Drawer` | Hidden. The pointer within `TITLEBAR_REVEAL.reveal_zone` (8 px) of the top edge slides it in over the content, with a shadow |
| `Push` | Hidden. It grows in from the top edge and pushes the content |
| `Corner` | The bar stays; only the traffic lights fade in while the pointer is in the top-left `reveal_corner` (96 px) |
| `Popup` | The traffic lights leave the bar. The pointer at the edge slides in a pill with them, which stays while the pointer is in the corner |

The reveal animates over `DURATION_BASE`. In Malaysia's focus mode the chrome
behaves as `Always`.

### Full screen and focus mode

The state comes from [`WindowStatus`](08-windows.md#window-state).

- **Maximized:** the bar stays as in a window; the edges stop resizing.
  Malaysia keeps the frame (corners and shadow).
- **Full screen on another compositor:** the bar stays; the traffic lights
  hide.
- **Malaysia focus mode**
  ([Malaysia ADRs 0051 and 0054](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisions.md)):
  the bar leaves the layout and the content fills the window. When the pointer
  nears the top, the compositor slides its bar down and the kit asks for
  `set_overlay` through `infinity_focus_v1` version 2, so the window does not
  move: `titlebar_overlaid()` is true and the `Titlebar` comes down over the
  content, right below the compositor bar, on an opaque band of
  `FullscreenBarStyle::tint`. The sidebar's traffic lights move down by the same
  amount. Bar and band slide and fade over `DURATION_BASE`.

## What the app does

1. With `Titlebar`, `WindowChrome`, `Sidebar` or `SettingsWindow`, nothing:
   every state above already works.
2. On Malaysia, it leaves room for the bar in content that sits under the
   overlaid title bar:

   ```rust
   let top = try_consume_root_context::<WindowControls>()
       .map_or(0.0, |controls| controls.fullscreen_content_top());
   rect().padding(Gaps::new(top, 0.0, 0.0, 0.0)).child(content)
   ```

3. If a view covers the title bar (a video player that fills the window), it
   adds its own traffic lights, so the window can leave full screen:

   ```rust
   TrafficLights::for_window(WindowControls::get().status.read().activated, false)
   ```

4. For its own behavior (an immersive mode that hides panels), it reads the
   state:

   ```rust
   let immersive = WindowControls::get().status.read().fullscreen;
   rect().maybe_child((!immersive).then(tool_panel))
   ```

5. To offer maximize or full screen in a menu, it calls `toggle_maximize()`
   or `toggle_fullscreen()` from an action ([13. App](13-app.md)).

## Helpers

| Function | Returns |
|---|---|
| `titlebar::sidebar_bar_height()` | Height of the title bar row beside a sidebar |
| `titlebar::use_window_drag(fixed)` | The move and double-click handler of the title bar ([Traffic lights](#traffic-lights)) |

Next: [10. Sidebar and split view](10-sidebar.md).
