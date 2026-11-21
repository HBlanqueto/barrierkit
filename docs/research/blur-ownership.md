# Who composes the blur: macOS, BarrierKit and Malaysia

**Goal:** find out who composes each kind of blur on macOS, what the app
declares and what the system decides, and whether BarrierKit and Malaysia
should split the work the same way.
**Scope:** blur and glass materials only. It starts at the AppKit and SwiftUI
API and ends at the options table and the open questions. It implements
nothing and closes no decision.
**Reader:** whoever decides if the kit keeps drawing in-window blur with Skia
(`crates/bk/src/glass.rs`) or asks the compositor to do it.
**Input:** the SDK 27.2 headers, three probes in `~/src/probes`, the captures
in `~/src/captures/blur`, the code of both repos and the Wayland sources
linked in each section.
**Output:** the macOS truth with its evidence, the current split, four
options with cost, risk and probability, and the questions that decide.

## Steps

1. Read the evidence tiers in section 1.1; every claim carries one.
2. Read section 2 to see what the repos do today.
3. Read section 3, the constraint that shapes every option.
4. Compare the options in section 5 and the recommendation in section 6.
5. Answer the questions in section 7 before any implementation starts.

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
| **measured** | Pixels from a probe, reproducible with the commands in section 8 |
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

## 2. Today in BarrierKit and Malaysia

### 2.1 Who does what

| Blur | Who composes | Where |
|---|---|---|
| Behind a layer surface or a popup (menu bar, Dock, cards, lock) | Malaysia, from `ext_background_effect_v1` | `crates/bk/src/shell/blur_region.rs`; `crates/malaysia-compositor/src/blur.rs` |
| Behind a toplevel (a HUD, a translucent window) | Malaysia, from the same protocol on the root surface | `blur::blur_region(window.surface())` |
| Sidebar and whole-window glass | Malaysia, under a transparent hole the app leaves; `infinity_window_style_v1.set_glass` (v6) | `crates/bk/src/glass_panel.rs`; `crates/malaysia-compositor/src/glass.rs` |
| Dock plates | Malaysia; `set_plates` (v8) | [18-platform](../barrierkit/18-platform.md) |
| Held slider and switch knobs | Malaysia draws a lens above the surface and refracts the pixels the app committed there; `set_knobs` (v9) | `crates/malaysia-compositor/src/knobs.rs` |
| Content under a bar: `FullscreenBar::Glass`, `ProgressiveGlass`, `TopGlass` | The app, with a Skia `save_layer` backdrop blur | `crates/bk/src/glass.rs` |
| `Material` and `GlassContainer` | The app, the same way | `crates/bk/src/glass.rs`, `glass_container.rs` |

Malaysia's own shell draws no in-window glass: the menu bar, Dock,
notifications and lock only send `set_blur_region`.

### 2.2 Facts that matter for the options

1. **The compositor already samples a client's own pixels.** `knobs.rs` walks
   the surface tree of the window and runs a shader on it. What it does not
   do is blur those pixels.
2. **The compositor already blurs live content with damage tracking.** The
   `Behind` caches of the Dock, cards and popups render the windows and the
   wallpaper behind the glass into a texture with an `OutputDamageTracker`
   and run Kawase passes on it (`refresh_behind`, `behind_elements`).
3. **A window's blur is the wallpaper only.** `Detail::Blurred` is one blurred
   wallpaper texture per output. A window behind another window does not show
   through the blur. macOS blurs the windows behind too (section 1.2, point 3).
   [code]
4. **Only the root surface's blur region is read.** `blur_region` is called on
   `window.surface()`; a sub-surface's region is ignored. [code]
5. **`ext-background-effect-v1` is a region and nothing else.** Version 1 has
   `set_blur_region` and the `blur` capability. The text speaks of the
   background "behind" a surface and leaves the algorithm to the compositor.
   It has no strength, no mask or gradient, no tint and no word about
   sub-surfaces. [spec, `wayland-protocols` 0.32.13, staging]
6. **BarrierKit owns its surfaces.** The runtime is sctk plus a Vulkan
   swapchain plus Skia per `wl_surface` (`crates/bk/src/platform/surface.rs`,
   `vulkan.rs`). Freya 0.5.3 supplies the tree and the layout. The kit uses no
   sub-surface today. [code]

### 2.3 Documents that contradict the code

1. [`05-controls.md`](../barrierkit/05-controls.md) says "the kit never
   draws blur or glass itself". `glass.rs` draws blur, and
   [`09-titlebars.md`](../barrierkit/09-titlebars.md) and
   [`15-backdrops-and-media.md`](../barrierkit/15-backdrops-and-media.md)
   document it. The sentence is true of knobs only.
2. Malaysia ADR 0020 · 0038 · 0065 · 0071 rejects "the compositor painting
   client backgrounds" and "a blur request in Infinity". `set_glass`,
   `set_plates` and `set_knobs` are compositor painting driven by Infinity
   requests. The ADR needs a line that says where the boundary lies.
3. [`macos-blur.md`](macos-blur.md) classifies shell surfaces by host
   measurement. It says nothing about who composes the blur, so it does not
   contradict this file, but it should link to it.

---

## 3. The constraint: in-window glass needs two layers

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

## 4. What Wayland and other compositors give

Sources and tags (spec, code, issue) are in the notes behind this file; the
links below are the primary ones.

### 4.1 The protocol

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

### 4.2 Sub-surface behavior by compositor

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

### 4.3 Blur policy differs per compositor

The same region gives a different look in each one: KWin defaults to strength
15, noise 5 and saturation 150 %; Mutter to radius 24, saturation 1.25 and
noise 0.015; niri shares one wallpaper blur ("xray") by default. Malaysia
takes its values from `ctdesign`. The kit cannot reach macOS pixels on
another compositor through the protocol; only its own Skia path can.

### 4.4 Toolkits and other systems

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

## 5. Options

Cost is in sessions: one session is one focused piece of work ending in one
commit, as the parity sessions so far. Probability is a judgment, with the
assumptions that carry it. Nothing here ran on a compositor with a GPU.

### Option A: keep Skia, fix the documents

The app draws every in-window blur. Malaysia keeps the behind-window blur and
the sidebar, plate and knob glass. The documents say so.

- **Same as macOS:** the declaration side of `Material` and `TopGlass`; the
  behind-window half of the split.
- **Not the same:** the app composes the in-window blur; Gaussian in Skia
  instead of the compositor's Kawase; the kit's `transparency_reduced()` and
  the compositor's `reduce_transparency` gate are two places.
- **Cost:** 1 session. `05-controls.md` line 332 and the Malaysia ADR
  0020 wording change; this file links from `macos-blur.md`.
- **Risk:** none new.
- **Probability of success:** 95 %. It changes text.

### Option B: sub-surface split and `ext-background-effect` on the chrome

The window becomes a parent surface (content) and a synchronized sub-surface
above it (chrome over a transparent hole) with the blur region. Malaysia
reads sub-surface regions and blurs the pixels of the surfaces below.

- **Same as macOS:** the compositor composes the in-window blur from layers
  the app declared; works with the standard protocol.
- **Not the same:** the protocol carries no material, no strength and no
  mask, so the soft scroll edge, the progressive fade and the per-material
  radius of section 1.4 stay app-side or cannot be expressed. Sub-surface
  semantics are unspecified (section 4.2), so the Skia path stays as the
  fallback on KWin, Hyprland, niri and any compositor without the capability.
- **Cost:** 10 to 16 sessions.
  - Compositor: read sub-surface regions, render the surface tree below the
    glass into a damage-tracked cache and blur it (the `Behind` caches are the
    pattern), 3 to 4.
  - Kit: `wl_subcompositor`, a second Vulkan swapchain and Skia surface,
    synchronized commits, viewport and scale per surface, 3 to 4.
  - Render split: `RenderPipeline` in Freya 0.5.3 is a loop over
    `tree.layers` and draws everything into one canvas. The kit would copy
    the loop with a layer range, or upstream a range. The kit's chrome must
    sit on a higher layer than its content, 2 to 3.
  - Tests, goldens, doc shots, fallback switch, 2 to 3.
- **Risks:**
  - GPU: the blur runs every frame while content scrolls under it.
  - Sync: content and chrome must land in one frame. A sync sub-surface
    commits with its parent, which is the intended mechanism.
  - Damage: Freya re-renders the whole tree each frame (the source says
    "Use incremental rendering" is a TODO), so two surfaces can double the
    raster work.
  - Capture: whether a toplevel capture of the window includes a
    compositor-made blur is not known; the Skia path always includes it.
- **Probability of success:** 35 %, where success is Malaysia composing
  `FullscreenBar::Glass`, `ProgressiveGlass` and `TopGlass` with the Skia
  look kept and no scroll lag. It assumes the render split works without a
  Freya fork, synchronized commits hold in Smithay, and the per-frame blur
  fits the frame budget on the GLES renderer.

### Option C: Infinity request with a material

Same split as B, but the sub-surface (or a region of it) carries an explicit
request, for example `infinity_window_style_v1.set_backdrop(surface, material,
rects)` with a `material` enum named after `NSVisualEffectView.Material` and
a scroll edge style (`automatic`, `soft`, `hard`) named after
`ScrollEdgeEffectStyle`. It says "sample the surface tree below this
surface". The compositor picks radius, saturation, tint and the Reduce
Transparency and Increase Contrast behavior.

- **Same as macOS:** the whole split of section 1: the app declares a
  material and a style, the compositor decides the rest, and a mask can
  express the soft edge. Section 2's `set_glass` and `set_knobs` become the
  same family.
- **Not the same:** the compositor still uses Kawase; other compositors keep
  the Skia path. Malaysia ADR 0020 · 0038 · 0065 · 0071 rejected "a blur
  request in Infinity"; it needs a new decision that says why that changed.
- **Cost:** B plus 3 to 5 sessions: protocol, a per-material radius table in
  `ctdesign`, masked blur in the compositor, the ADR.
- **Risks:** B's, and a protocol only two projects use. Versioning follows
  `infinity_window_style_v1`, which is at version 9.
- **Probability of success:** 30 %, or 85 % given that B worked; the extra
  risk is design, not feasibility.
- **API names:** no new public kit name. `Material`, `TopGlass`,
  `GlassContainer` and `FullscreenBar` keep theirs
  ([glossary](../barrierkit/20-glossary.md)); the request is
  `platform`-level. A kit enum for the edge style would be
  `ScrollEdgeEffectStyle` with `Automatic`, `Soft` and `Hard`, the SwiftUI
  name first (AGENTS.md, "Nombres y glosario", rule 2).

### Option D: closer look without moving ownership

Two cheaper variants found while researching:

1. **Align the numbers.** Take the system blur extent per material from
   section 1.4 and check that the Skia path and the compositor's Kawase give
   the same extent for the same `ctdesign` token. Cost 1 to 2 sessions,
   probability 65 %: the comparison needs a GPU compositor, which this host
   does not have, so only the Skia half is testable here.
2. **Move only what has no chrome above it.** `set_glass`, `set_plates` and
   `set_knobs` already do. A compositor-drawn overlay over one surface
   (the `set_knobs` form) cannot serve a bar, because the bar's text and
   controls would be covered (section 3). No further case exists today.

---

## 6. Comparison and recommendation

| | A: Skia + docs | B: sub-surface + ext | C: Infinity material | D1: align numbers |
|---|---|---|---|---|
| Who composes in-window blur | App | Compositor (Malaysia only) | Compositor (Malaysia only) | App |
| App declares a material | Yes | No: region only | Yes | Yes |
| Radius, tint, RT and IC decided by | App and compositor | Compositor | Compositor | App and compositor |
| Soft edge and per-material radius | Yes | No | Yes | Yes |
| Other compositors | Same | Skia fallback | Skia fallback | Same |
| Cost (sessions) | 1 | 10 to 16 | 13 to 21 | 1 to 2 |
| Success probability | 95 % | 35 % | 30 % | 65 % |
| New risk | None | GPU, sync, damage, capture | B's plus protocol | Host cannot run Kawase |

### What the user gains from an identical split

1. **Pixels:** nothing by itself. The extent and tint of a blur come from
   the algorithm and the tokens, not from who runs it. [inferred]
2. **Wallpaper tint:** in-window glass over the app's own content takes no
   wallpaper tint, so there is little to gain. The behind-window parts
   already use it.
3. **One place for Reduce Transparency:** the compositor already gates blur
   on it. The gain is real only if the Skia path disappears, and it cannot,
   because other compositors need it (section 4.2). Two places remain.
4. **Performance:** unmeasured. The compositor does the same blur work the
   app does today, now plus a second surface. The kit's `glass` span can
   measure the Skia cost first.
5. **Captures:** macOS includes the within-window blur in a window-only
   capture (section 1.2). The Skia path always does. A compositor-made blur
   may not be in a toplevel capture. Unverified.

Gains 1 to 5 are small or unknown, and every one of them is bought with a
second code path. The evidence supports "it does not pay today" more than
"it pays".

### Recommendation

1. Do option A now: fix the three documents of section 2.3.
2. Measure before choosing B or C. Two experiments cost under a session each:
   - Trace the `glass` span (`tracy` feature) on a scrolling window with
     `FullscreenBar::ProgressiveGlass`. If it costs well under a millisecond
     per frame, the performance argument is gone.
   - Run a minimal client on Malaysia: a parent, a synchronized sub-surface
     with a transparent hole and a blur region. It shows what the
     compositor does with the pattern and whether captures keep the blur.
3. Choose C over B if the split is wanted, because B cannot express a
   material or a soft edge. Take B only to prove the sub-surface pattern.

Option A has probability 95 %. If the split is wanted, C has probability
30 % and costs 13 to 21 sessions.

---

## 7. Questions that decide

1. **Is a second code path acceptable?** The Skia path must stay for other
   compositors. Is ownership parity on Malaysia worth 13 to 21 sessions and
   that second path, or is documented app-side blur enough?
2. **Which in-window glass matters?** Content under the bar
   (`FullscreenBar::Glass`, `ProgressiveGlass`, `TopGlass`) needs the soft
   edge and a mask. `Material` and `GlassContainer` cards need only a region.
   The answer decides between B and C.
3. **May Infinity grow a material request?** Malaysia ADR 0020 rejected it,
   while `set_glass`, `set_plates` and `set_knobs` already paint for
   clients. Should that ADR be rewritten first?

---

## 8. Evidence and reproduction

The captures and probes live outside the repo, as the rest of the macOS
research.

| Item | Where |
|---|---|
| Blur tiles probe | `~/src/probes/blurown.swift`, run with `~/src/probes/blurown.sh <checker\|split\|bands> <aqua\|darkAqua> <out>` |
| Scroll edge probe | `~/src/probes/scrolledge.swift <aqua\|darkAqua> <none\|automatic\|soft\|hard> <out>` through `capture.sh` |
| Background extension probe | `~/src/probes/bgext.swift <aqua\|darkAqua> <out>` through `capture.sh` |
| Captures | `~/src/captures/blur/tiles-*`, `edge-*`, `bgext-light*` (`-cache.png` is the app's render, `-screen.png` the window as WindowServer composes it, `-full.png` the screen) |
| Analysis | `~/src/captures/blur/analyze.py <backdrop> <mode>`, with the venv of [`macos-observations.md`](macos-observations.md) |
| Wayland notes | the sources named in section 4; the compositor claims are code reading, none was run |

Limits:

- The edge widths are lower bounds above 45 px: the tile is 92 px.
- Reduce Transparency and Increase Contrast were not re-measured.
- A soft and a hard `preferredScrollEdgeEffectStyle` looked the same in the
  probe; the cause is unknown.
- No compositor with a GPU ran here, so every claim about Malaysia's blur is
  code reading.
- Class names in section 1.2 are runtime observations, not API.
