# Who composes the blur: macOS, BarrierKit and Malaysia

**Goal:** find out who composes each kind of blur on macOS, what the app
declares and what the system decides, and whether BarrierKit and Malaysia
should split the work the same way.
**Scope:** blur and glass materials only. It starts at the AppKit and SwiftUI
API and ends at what Wayland and other compositors give. It implements
nothing.
**Reader:** whoever decides if the kit keeps drawing in-window blur with Skia
(`crates/bk/src/glass.rs`) or asks the compositor to do it.
**Input:** the SDK 27.2 headers, three probes in `~/src/probes`, the captures
in `~/src/captures/blur`, the code of both repos and the Wayland sources
linked in each section.
**Output:** the macOS truth with its evidence, the constraint of in-window
glass and what Wayland gives.

**Status (2026-11-21):** the owner chose to compose all blur in the compositor.
The protocol, the phases and the first experiment are in
[`blur-backdrop-design.md`](blur-backdrop-design.md) and
[ADR 0091](../decisions.md).

## Steps

1. Read the evidence tiers in section 1.1; every claim carries one.
2. Read section 2, the constraint of in-window glass.
3. Read section 3 for what Wayland and other compositors give.

---

## 1. macOS: who composes the blur

### 1.1 Method and evidence tiers

Only public API, SDK headers, Apple documentation and observed behavior are
used; nothing is taken from Apple binaries (method rules in
[`macos-observations.md`](macos-observations.md)).

| Tier | Meaning |
|---|---|
| **header** | Stated in an SDK 27.2 header |
| **doc** | Stated by Apple in a session or documentation page |
| **measured** | Pixels from a probe, reproducible with the commands in section 4 |
| **observed** | Runtime class names read from the probe's own process; not API, may change |
| **inferred** | A conclusion drawn from the tiers above |

The probe `blurown` draws one borderless, non-opaque window. It holds
`NSVisualEffectView` tiles of every material, both `withinWindow` and
`behindWindow`, and `NSGlassEffectView` tiles. Under the within tiles are an
8 px checker, a white/black edge and a red/green edge. Under the behind tiles
is a window of another process (`backdrop`). Each run takes three images:

- `cacheDisplay` of the window's view tree: what the app renders itself.
- `screencapture -l`: the window as WindowServer composes it, alone.
- `screencapture` of the whole screen: the final composite.

The host is macOS 27.2.1 at 2560x1440, scale 1.

### 1.2 What the three images show

1. **The app's own render has no blur.** In `cacheDisplay` the checker stays
   sharp under every tile (residual standard deviation 25 to 63 of 255), and
   an `NSGlassEffectView` is an almost empty rounded rectangle with no lens.
   The tint is drawn, so the veil is app-side layer content and the blur is
   not. [measured]
2. **WindowServer composes the within-window blur.** In `screencapture -l`
   the same tiles flatten the checker completely (standard deviation 0.0) and
   `NSGlassEffectView` shows a lens at its edge and keeps the pattern's colors more saturated than the other materials. [measured]
3. **The compositor level composes the behind-window blur.** The probe's
   `CGPreflightScreenCaptureAccess()` is `false`: it cannot read other
   windows. Yet its behind tiles show the blurred pixels of the other process
   in the full-screen capture. A window-only capture does not contain them.
   [measured, `tiles-bands-*`, `tiles-split-*`]
4. **Toolbar, titlebar, scroll edge and background extension follow the same
   path.** Their blur is in the WindowServer image and absent from
   `cacheDisplay`. [measured, `edge-*`, `bgext-light`]
5. **AppKit builds a layer tree; the render server samples it.** The view tree
   shows `CABackdropLayer`s and `CAPortalLayer`s under `NSTitlebarView`,
   `NSToolbarPlatterView` > `NSGlassEffectView`, an `NSScrollPocket`
   (`PocketMask`, `PocketBlur`, `LuminanceAdjustment`) inside the scroll view,
   and `_NSCoreHostingView<BackgroundExtensionContainer>` for
   `NSBackgroundExtensionView`. [observed]
   The toolkit decides the geometry, the mask and the style. The render
   server does the sampling and the blur. [inferred from 1 to 5]

Point 3 and the window-only capture give the split Wayland also has:
the blur of what lies outside the window is the compositor's, and the blur
of what lies inside it is sampled from the window's own layers.

### 1.3 Who composes each kind

| Kind | App declares | Composed by | Evidence |
|---|---|---|---|
| Menus | `NSVisualEffectMaterialMenu` (the material used by menus) | WindowServer, behind the menu window | header; a `menu` tile composes like the others; a real `NSMenu` was not re-probed here (earlier panel samples: [`macos-blur.md`](macos-blur.md)) |
| Sidebar | `NSVisualEffectMaterialSidebar`, or the split view item | WindowServer | header; not re-measured; the tree of the existing sidebar probe has an `NSVisualEffectView` at the window root |
| Titlebar and toolbar | `NSVisualEffectMaterialTitlebar`; the toolbar | WindowServer (`CABackdropLayer` in `NSTitlebarView`) | measured, observed |
| Toolbar platters | none: the toolbar makes `NSGlassEffectView`s | WindowServer, with lens | measured, observed |
| Scroll edge effect | `preferredScrollEdgeEffectStyle` (`automaticStyle`, `softStyle`, `hardStyle`, macOS 26.1) on a titlebar accessory; in SwiftUI `scrollEdgeEffectStyle(_:for:)` and `scrollEdgeEffectHidden` | AppKit adds an `NSScrollPocket`; WindowServer blurs | header, doc ([WWDC25 310](https://developer.apple.com/videos/play/wwdc2025/310/): automatic under floating content), observed |
| `NSBackgroundExtensionView` | `contentView`, `automaticallyPlacesContentView` | WindowServer (portal and backdrop layers) | header, measured, observed |
| `NSGlassEffectView` | `contentView`, `cornerRadius`, `tintColor`, `style`, `effectIsInteractive` | WindowServer, with lens | header, measured |
| `withinWindow` | `NSVisualEffectBlendingModeWithinWindow`: "area behind the view in the window" | WindowServer, from the window's own layers | header, measured |
| `behindWindow` | `NSVisualEffectBlendingModeBehindWindow`: "area behind the window (such as the Desktop or other windows)" | WindowServer, from the screen | header, measured |

SwiftUI `Material` acts like `withinWindow`, and like `behindWindow` when it
is a container background, according to an Apple engineer on the
[developer forums](https://developer.apple.com/forums/thread/806416) (a
forum hint, not documentation).

### 1.4 What the app sets and what the system decides

| App sets (header) | System decides |
|---|---|
| `material`, `blendingMode`, `state`, `isEmphasized`, `maskImage` on `NSVisualEffectView` | The blur radius, the saturation, the tint and the lens |
| `style`, `tintColor`, `cornerRadius`, `effectIsInteractive` on `NSGlassEffectView`; `spacing` on `NSGlassEffectContainerView` | Merging of nearby glass in a container |
| The scroll edge style: `automaticStyle`, `softStyle`, `hardStyle` | Where the pocket sits, its mask and its luminance adjustment |
| The appearance of the view | Light and dark values per material |

The header says materials are dynamic and that their look depends on
`effectiveAppearance`, `blendingMode`, `state`, `emphasized` "and possibly
other factors". No API sets a radius. [header]

Measured blur extent (10 to 90 % edge width in px; the tile is 92 px wide, so
values near 50 are truncated and are lower bounds):

| Material | Light within | Light behind | Dark within | Dark behind |
|---|---|---|---|---|
| titlebar | 47 | 49 | 47 | 50 |
| headerView | 48 | 50 | 48 | 50 |
| hudWindow, fullScreenUI | 48 | 49 | 48 | 49 |
| popover | 47 | 50 | 47 | 49 |
| menu | 48 | 50 | 48 | 49 |
| sidebar, underWindowBackground, toolTip | 35 | 36 | 47 to 49 | 50 |
| sheet | opaque | opaque | no blur, veil only | no blur, veil only |
| contentBackground | opaque | opaque | opaque | opaque |
| `NSGlassEffectView` regular | 13 | 18 | 8 | 9 |
| `NSGlassEffectView` clear | 2 | 2 | 2 | 2 |

Within and behind give the same extent per material, so one system decision
per material and appearance serves both. [measured, `analyze.py`]

Reduce Transparency and Increase Contrast were not re-measured. A per-process
flag does not change a material, and the system switch is a setting the
probes do not touch (see [`macos-observations.md`](macos-observations.md)).
The earlier finding that the Dock
becomes a solid and menus keep their veil is in [`macos-blur.md`](macos-blur.md).

---

## 2. The constraint: in-window glass needs two layers

Glass over scrolling content has the content below it and the chrome (title,
controls) above it. The blur sits between the two.

One `wl_surface` holds one buffer. If the app draws content and chrome into
it, the compositor cannot tell them apart, so a blur it adds over the region
would also blur the chrome. `set_knobs` works only because nothing of the app
lies above a knob.

macOS solves this with a layer tree: AppKit sends layers with backdrop
properties and the render server orders them (section 1.2, point 5). The
Wayland form of a shallow layer tree is the sub-surface tree. Therefore every
compositor-side design needs the app to split a window into at least a lower
surface (content) and an upper sub-surface (chrome over a transparent hole),
and the compositor to blur the lower one under the upper one's region.

Two ways around the split exist and both fail:

1. The app draws the chrome with partial alpha and the compositor keeps it.
   The buffer has already merged chrome and content, so the compositor
   cannot recover the content.
2. The compositor draws the chrome. It cannot: titles, toolbars and controls
   are the app's.

---

## 3. What Wayland and other compositors give

Sources and tags (spec, code, issue) are in the notes behind this file; the
links below are the primary ones.

### 3.1 The protocol

`ext-background-effect-v1` is staging, version 1, in
[`wayland-protocols`](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/staging/ext-background-effect/ext-background-effect-v1.xml)
since 1.45 (2025-06-13); 1.46 fixed the `blur` capability value. The pinned
crate is 0.32.13.

- It offers `set_blur_region` and the `capabilities` event, and nothing else.
- The blurred thing is "the background behind" the surface. The text does not
  define "background", does not mention sub-surfaces and has no strength,
  mask, tint or material.
- The algorithm is "subject to compositor policies".
- The merge request
  ([!272](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/merge_requests/272))
  left rounded corners, fractional scaling and blur strength for a later
  version. The older
  [!43](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/merge_requests/43)
  asked whether sub-surfaces matter and was never answered in the text.
- No `material`, `backdrop` or in-surface blur proposal exists upstream.
  Open items: [#318](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/work_items/318)
  and [!550](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/merge_requests/550)
  (capability semantics), [!449](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/merge_requests/449)
  (corner shape).

Smithay at the pinned rev `65aa86f` stores the region per `wl_surface` in
`BackgroundEffectSurfaceCachedState` and renders nothing. It also has
framebuffer-effect elements (`capture_framebuffer`) that make the damage
tracker include the area behind an element.

### 3.2 Sub-surface behavior by compositor

Read from source, none run:

| Compositor | Blur region on a sub-surface | Evidence |
|---|---|---|
| KWin 6.7 | Ignored; only the main surface's region counts, blurred before the window is drawn | code |
| Hyprland 0.56 | Decided from the main surface; backdrop is a snapshot taken before the window renders, so the parent is never blurred | code |
| niri 26.04 | Unsupported, written in its docs | spec |
| Mutter 51 | Kept per surface; the blur is painted before that surface, from what is already painted, so an upper sub-surface likely blurs its parent | code, unverified |
| COSMIC | One blur element per surface in front-to-back order, so likely the same as Mutter | code, unverified |
| Malaysia | Ignored: `blur_region` reads the root surface | code |
| wlroots, scenefx | State only, or one blur per container | code |

Therefore a blur region on a sub-surface has no portable meaning. A client
that relies on it shows no in-window blur on KWin, Hyprland and niri.

### 3.3 Blur policy differs per compositor

The same region gives a different look in each one: KWin defaults to strength
15, noise 5 and saturation 150 %; Mutter to radius 24, saturation 1.25 and
noise 0.015; niri shares one wallpaper blur ("xray") by default. Malaysia
takes its values from `ctdesign`. The kit cannot reach macOS pixels on
another compositor through the protocol; only its own Skia path can.

### 3.4 Toolkits and other systems

- GTK 4.24 blurs in-process (copy and paste render nodes, CSS
  `backdrop-filter`) and derives a compositor region from the render tree
  only when the backdrop is still the plain window background
  ([MR 10145](https://gitlab.gnome.org/GNOME/gtk/-/merge_requests/10145)).
  There is no public blur API.
- Qt Quick blurs in-process (`ShaderEffectSource`, `MultiEffect`); the
  compositor blur needs `KWindowEffects` or the protocol. winit exposes
  `set_blur(bool)` for the whole window.
- Android composes background blur in SurfaceFlinger, per layer, including
  siblings of the same app
  ([window blurs](https://source.android.com/docs/core/display/window-blurs));
  `RenderEffect` blurs a view's own content in the app.
- Windows composes the window backdrop in DWM
  ([backdrop types](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type));
  in-app acrylic is built in the app's own composition tree (unverified).
- macOS documents the two modes and not the process that composes them; the
  probes of section 1 answer that.

macOS and Android keep in-window blur on the server and rest on a layer tree.
Every Wayland toolkit keeps it in the app. That is the gap this file weighs.

---

## 4. Evidence and reproduction

The captures and probes live outside the repo, as the rest of the macOS
research.

| Item | Where |
|---|---|
| Blur tiles probe | `~/src/probes/blurown.swift`, run with `~/src/probes/blurown.sh <checker\|split\|bands> <aqua\|darkAqua> <out>` |
| Scroll edge probe | `~/src/probes/scrolledge.swift <aqua\|darkAqua> <none\|automatic\|soft\|hard> <out>` through `capture.sh` |
| Background extension probe | `~/src/probes/bgext.swift <aqua\|darkAqua> <out>` through `capture.sh` |
| Captures | `~/src/captures/blur/tiles-*`, `edge-*`, `bgext-light*` (`-cache.png` is the app's render, `-screen.png` the window as WindowServer composes it, `-full.png` the screen) |
| Analysis | `~/src/captures/blur/analyze.py <backdrop> <mode>`, with the venv of [`macos-observations.md`](macos-observations.md) |
| Wayland notes | the sources named in section 3; the compositor claims are code reading, none was run |

Limits:

- The edge widths are lower bounds above 45 px: the tile is 92 px.
- Reduce Transparency and Increase Contrast were not re-measured.
- A soft and a hard `preferredScrollEdgeEffectStyle` looked the same in the
  probe; the cause is unknown.
- No compositor with a GPU ran here, so every claim about Malaysia's blur is
  code reading.
- Class names in section 1.2 are runtime observations, not API.
