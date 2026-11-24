# Decisions

**Goal:** know why BarrierKit is built this way before you change it.
**Reader:** anyone who changes the kit's structure, render stack, look or public names.

Each entry is an architecture decision as it holds today, with its ADR number.
The list is closed: no entry is added. The why of a new feature is one line in
the chapter that documents it, or in its commit. When a decision changes, edit
its entry; when it no longer holds, delete it. Git keeps the history. The compositor, session and shell decisions are in
[Malaysia's decisions](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisions.md).

## 0011 — Design values as Rust constants
Decision: every design value is a public compile-time constant of `ctdesign`, and no other crate defines one. A color that changes with window focus is a `FocusPair`.
Why: each value has one typed source; a missing token fails the build instead of hiding as a literal.
Code: `crates/ctdesign/build.rs`, `crates/ctdesign/src/lib.rs`, `scripts/check-design-tokens.sh`

## 0032 · 0046 — Tengen as the design source
Decision:
- The visual language is Tengen. Its source is `crates/ctdesign/tengen/tokens.json` and `components.json`.
- They hold colors, radii, spacing, typography, shadows and component metrics.
- `ctdesign/build.rs` generates `$OUT_DIR/tengen.rs` from them.
- The themes are `light` (default), `dark` and `dark-tint`.
- The kit draws the look itself; no alias keeps an old name or path.
Why: the kit, the shell and the compositor read the same values from one place.
Code: `crates/ctdesign/tengen/`, `crates/ctdesign/build.rs`, [Tengen](barrierkit/tengen.md)

## 0033 · 0034 · 0056 — Render stack
Decision:
- `bk` uses Freya's core crates pinned `=0.5.3`: `freya-core`, `freya-components` (no default features), `freya-edit`, `freya-animation`, `freya-engine`, `freya-clipboard`, `torin`, `ragnarok`.
- Never `freya`, `freya-winit` or any `otto-*` crate.
- Its own Wayland runner on smithay-client-toolkit and `calloop`.
- Skia runs only in clients. `ctdesign`, `ctappearance`, `ctshell` and `infinity-protocol` pull no Freya or Skia, so the compositor never links them.
- The runner draws with Vulkan through `ash`, with no GL fallback. `freya-engine` builds Skia with only `vulkan`.
- One shared device and `DirectContext`, one swapchain per surface, `MAILBOX` when offered, else `FIFO`.
Why: winit would own the Wayland connection, and Freya's Vulkan driver lives in `freya-winit`. Frame callbacks already pace redraws, so `FIFO` would throttle twice.
Rejected: a kit from `otto-kit`; Skia on EGL/GLES.
Code: `crates/bk/src/platform/linux/vulkan.rs`, `scripts/check-client-only-deps.sh`, [Platform](barrierkit/18-platform.md)

## 0035 — Own app identification and CSD
Decision: a window's `app_id` is the reverse-DNS id the app passes to `Application::new` or `WindowOptions::new`; the kit adds no prefix. Every window requests client-side decoration; if the compositor answers server-side, the kit turns off its edge resizing.
Why: an app built on the kit is not Malaysia's, so its id, settings folder and bus name are its own (2026-11-24).
Code: `crates/bk/src/platform/window.rs`

## 0039 — Infinity v1
Decision: `infinity_window_manager_v1` is a private Wayland protocol for the shell's window management. `infinity-protocol` generates the client side (feature `client`, with `shell` for this protocol) and the server side (feature `server`, used by the compositor). Only `bk-shell` binds it; the compositor decides which clients may bind it.
Why: the standard protocols do not cover what the Dock and the menu bar need from the compositor.
Code: `crates/infinity-protocol/protocols/`, `crates/bk-shell/src/infinity.rs`, [Globals](shell/bk-shell.md#globals)

## 0047 — BarrierKit in layers
Decision: `bk` is one crate in four layers (platform, primitives, views, patterns) plus a cross-cutting `app`. A layer uses only the layers below it, and patterns compose only the public API of views. Code only Malaysia's shell uses lives in `crates/bk-shell`, which reaches the runner through `bk`'s hidden `shell` feature.
Why: the kit must build any desktop app, like AppKit or libadwaita, without each app rebuilding sidebars, search fields and controls.
Pending: nothing is built until an app needs it. Partial damage waits for Freya: 0.5.3 repaints the whole tree every frame.
Code: `crates/bk/src/lib.rs`, [Layers](barrierkit/03-layers.md)

## 0048 — BarrierKit in its own repository
Decision: `bk`, `bk-shell`, `ctdesign`, `ctappearance`, `ctshell` and `infinity-protocol` live in `github.com/HBlanqueto/barrierkit`. malaysia pins them by one git `rev` in `Cargo.toml` and takes barrierkit as a flake input, reusing its prebuilt Skia. `./dev.sh kit-local` in malaysia tests a local checkout before publishing.
Why: the toolkit carries its style and its reading of the appearance; keeping them in malaysia would make the two repos depend on each other.
Code: `flake.nix`

## 0061 · 0062 · 0063 — SwiftUI-shaped API
Decision:
- An app depends only on `bk` and writes `use bk::prelude::*;`.
- `Application::new(name).window(title, content).run()` opens a window with background and `Titlebar`; `run_window` stays for a bare window; an app that draws its own bar sets `HiddenTitleBar` or `Plain` (ADR 0115).
- The crates are `bk`, `ctdesign` and `ctappearance`; prose still says BarrierKit.
- Every public view is a type built with `Type::new(..)` that forwards Freya's traits. Modifiers are methods of `ViewModifiers` (`.help`, `.context_menu`, `.focus_ring`).
- Low-level primitives stay functions, like `text(..)`, `rect()` and `label()`.
- Names follow SwiftUI, then AppKit, then plain English; the glossary lists each one.
Why: the common case needs one import and one expression, and the code reads like its SwiftUI version.
Code: `crates/bk/src/prelude.rs`, `crates/bk/src/app/scene.rs`, [Glossary](barrierkit/20-glossary.md)

## 0064 — Styles and themes
Decision:
- Each control has a style struct (`SliderStyle`) that implements `ControlStyle`. Its `tengen()` builds the default from `ctdesign`.
- An override is a closure from the inherited style to a new one.
- Levels, later wins: Tengen, `Theme` (`Application::theme`, `WindowOptions::theme`), `.control_style` on a container, `.style` on the instance.
- Paint is a `ShapeStyle`; `Role`, `Accent` and `Backdrop` follow the system, `Color` does not.
- The user's accessibility settings apply last: reduce motion, reduce transparency and the focus ring.
Why: an app changes any visual property, or builds a whole theme, without copying Tengen's numbers.
Code: `crates/bk/src/styles/theme.rs`, `crates/bk/src/styles.rs`, `crates/bk/src/modifiers.rs`

## 0081 — Icons: glyphs in the kit, icon files in the theme
Date: 2026-11-20
Decision:
- A glyph is a one-color symbol the kit draws inside a control, menu, toolbar, sidebar, list, form or state. It is a `Symbol`: SVG paths on a 16 grid in `currentColor`, named after its SwiftUI name, fitted to AppKit at 16 pt (20 px).
- An icon is a file with a freedesktop name that other apps, the shell and malaysia load by name: apps, places and folders, MIME types, devices, and the `-symbolic` actions and status. It lives in `malaysia-icon-theme`.
- Neither repository holds a file from Apple. The paths and the theme files are drawings made for the project; the ink boxes and masses they were fitted to stay out of the repository.
- The kit does not read the theme and the theme does not read the kit. A glyph moves to the theme only when the theme lacks its freedesktop name and its owner asks for it.
Why: the kit draws a glyph at any size, stroke and color from one path, which a file lookup by name and size cannot do. The theme already ships the macOS-style set the desktop's apps need: about 3,700 symbolic icons, 50 folders, 235 file types, 63 devices; every freedesktop name that matches a kit glyph was found in it (50 checked). Copying the glyphs would draw each twice and mix the kit's MIT with the theme's LGPL, CC BY-SA and GPL licenses.
Rejected: every glyph in the theme (the kit would load files at run time or embed GPL art, and lose the stroke and the state colors); every icon in the kit (Linux apps look icons up by name and the kit is not an icon theme); drawing the theme's symbols from the kit's paths (it would replace the MacTahoe copies, which is the theme's decision).
Code: `crates/bk/src/symbol.rs`, [Macos icons](research/macos-icons.md)

## 0083 — What the kit does not cover
Date: 2026-11-21
Decision:
- Out of scope, so the [SwiftUI API gap](research/swiftui-api-gap.md) counts them apart and no backlog row asks for them:
  - Core Data: `FetchRequest`, `FetchedResults`, `SectionedFetchRequest`, `SectionedFetchResults`. The app owns its data and hands `List` its rows.
  - Network: `bk` fetches no URL. `AsyncImage` shows files and bytes (`use_artwork`, `use_artwork_from`); `asyncImageURLSession` has no counterpart.
  - HDR: `allowedDynamicRange` (modifier and environment value), `Color.headroom`, `Color.exposureAdjust`, `Image.DynamicRange`. The kit draws in SDR.
- Not out of scope: `Shader`, `ShaderLibrary`, `colorEffect`, `layerEffect`, `distortionEffect` and `MeshGradient`. Skia runs them in the app ([Shader needs](research/shader-needs.md)); the Metal libraries SwiftUI loads do not port, so the kit takes SkSL.
- Not a decision: WidgetKit, control widgets, visionOS spaces and the other Apple-only API are `n/a` in the gap study. Linux has no such host or hardware.
Why: before this entry no decision excluded an API, so every absent item counted as work. The first two are frameworks beside the toolkit that an app already has a better source for (its database, its HTTP client). HDR needs `wp_color_manager_v1`, an HDR swapchain and compositor support; malaysia does not advertise the protocol. ADR 0047 says "not yet"; this entry says "not in the kit".
Rejected: excluding `Shader` and `MeshGradient` as graphics frameworks, since Skia draws both and a plan exists; listing the Apple-only API here, since the platform already rules it out.
Pending: the document model (`DocumentGroup`, `FileDocument` and 24 more types). The [coverage matrix](research/swiftui-appkit-coverage.md) puts document frameworks outside its scope and its backlog D.7 plans document scenes; decide before building scenes. HDR returns when malaysia advertises `wp_color_manager_v1`.
Code: [SwiftUI API gap](research/swiftui-api-gap.md), [Shader needs](research/shader-needs.md)

## 0085 — No `Binding`, no action routing or app lifecycle callbacks
Date: 2026-11-21
Decision:
- The kit has no `Binding` type. Since 2026-11-24 every control takes the `State<T>` it edits in `new`, as a SwiftUI control takes a binding, and lists and tables take `.selection(state)`.
- An action reaches every handler of the window: `use_menu_action` registers a handler and `dispatch` calls them all. The kit adds no routing to the focused view, no close veto (`windowShouldClose`), no quit call (`applicationShouldTerminate`) and no open-URL event (`application(_:open:)`). A close request closes the window at once, except a sheet's; closing the last window ends the process.
- Both are non-goals until an app asks for them; for documents the close veto, the quit and the open-URL event are built ([Documents and file dialogs](barrierkit/08-windows.md#documents-and-file-dialogs)). [SwiftUI to the kit](barrierkit/16-swiftui-to-the-kit.md) tells the app author the same.
Why: Freya 0.5.3 has `Writable<T>` and `IntoWritable` (`freya-core-0.5.3/src/lifecycle/writable.rs`), but its `PartialEq` returns `true` for any two values (line 60), so a component that holds one cannot show the diff that its source changed. Neither malaysia (working tree of 2026-11-21) nor curseterm (`1375c72`) uses `Writable`, `IntoWritable` or `use_radio` (grep). A grep of both for a close veto, a quit call and an open-URL event finds none; the only hits are winit's `CloseRequested` and `infinity_force_quit` in malaysia's compositor, which are not kit apps. The three repositories have no issue that asks for any of them (`gh issue list`, 2026-11-21). AGENTS.md rule 2 forbids adding what nobody uses.
Rejected: controls taking `impl IntoWritable` beside value plus callback, and a `Binding` type of the kit's own; no app uses either.
Pending: the action routing and `quit` return when an app of the kit needs one; a `Binding` returns when two components must edit one value and a `State<T>` in a context cannot serve them.
Code: `crates/bk/src/controlled.rs`, `crates/bk/src/app/action.rs`, `crates/bk/src/platform/window.rs`, [Swift syntax in Rust](research/swift-syntax-in-rust.md)

## 0091 — The compositor composes every blur
Date: 2026-11-21
Decision:
- Malaysia's compositor composes every blur, as macOS's render server does: behind a window, behind a layer surface and inside a BarrierKit window. An app declares a material and a style and draws no backdrop blur. This is the kit side of Malaysia's «Blur and wallpaper tinting», whose blur lines it replaces (Malaysia ADRs 0020 · 0038 · 0065 · 0071); Malaysia's `decisions.md` holds that text.
- `infinity_window_style_v1` version 10 adds `set_backdrop(surface, blending, material, radius, rects)` and `set_scroll_edge(surface, edge, style, depth)`. `material` lists the 14 `NSVisualEffectView.Material` values the macOS 27.2 SDK does not deprecate (`NSVisualEffectView.h`) and `glass_regular` and `glass_clear` (`NSGlassEffectView.Style`); `blending` is `behind_window` or `within_window`. The protocol carries no blur radius, saturation or tint: the compositor takes them from `ctdesign` per material and appearance, and the blur reaches the same distance in logical pixels at every output scale.
- `within_window` blurs what the surfaces of the toplevel draw below the requesting surface: its parent, the sub-surfaces under it and their children. The requester may be a sub-surface above its parent, below it, or the root with a sub-surface under it. It never reads the region of `ext-background-effect-v1`, so a request on the root surface does not meet Malaysia's wallpaper blur.
- On a synchronized sub-surface the request applies with the parent's commit, so material and content change in one frame. A desynchronized sub-surface tears (phase 0: 23 of 23 mid-frames).
- A capture of a window alone includes `within_window` material and not `behind_window` material, as `screencapture -l` does on macOS 27.2.1.
- `ext-background-effect-v1` stays for every client that is not BarrierKit. The compositor serves `set_backdrop` and `set_scroll_edge` only for BarrierKit clients.
- Existing requests keep their opcode and meaning. A client sends the new ones only when it bound version 10 or later: a request newer than the object's version is a protocol error (`wayland-backend` 0.3.17, `src/rs/server_impl/client.rs:580`), not an ignored message. The kit binds `1..=9` until it sends them, and Malaysia advertises 10 only when it serves them.
Why: macOS composes the blur outside the app in both blending modes and the app declares the material (`docs/research/blur-ownership.md` 1.2, measured on macOS 27.2.1). One place then decides the radius, the tint and Reduce Transparency, and one algorithm gives each material its measured extent. Phase 0 (software rendering) showed that a synchronized sub-surface and its parent's content reach the compositor in one transaction (Smithay `transaction.rs:261-278`) and that the blur then follows the scroll without lag, above and below the parent, at 1x and 2x.
Rejected: a blur drawn by the client as the shipping path (two places for Reduce Transparency, two blur algorithms); `ext-background-effect-v1` on sub-surfaces (KWin, Hyprland and niri ignore their regions, and it has no material, strength or soft edge); reading the region of the root surface for the in-window blur (it meets the wallpaper path); one surface with partial alpha (the compositor cannot tell chrome from content).
Decided by the owner on 2026-11-22: the criterion for the in-window blur is the frame time of the band pass at 2x for a 96 px band on a real GPU. A median of at most 1.0 ms passes, over 2.5 ms stops, and no frame of the 40 may reach 8.33 ms; between 1.0 and 2.5 ms the owner decides then ([criterion](research/blur-backdrop-design.md#10-gpu-criterion-the-frame-time-of-the-band-pass)). That frame time is NOT measured.
Decided by the owner on 2026-11-22: M2 and K3 start before the GPU frame time is measured; the owner accepts the risk that the work is wasted if the GPU criterion fails; the criterion still decides whether it ships on by default ([order of execution](research/blur-backdrop-design.md#71-the-phases)).
Decided by the owner on 2026-11-22: the kit's own Skia backdrop blur is deleted at the end of the plan, and Malaysia applies the measured values in every shell component ([order of execution](research/blur-backdrop-design.md#71-the-phases)).
Pending: confirmation by the owner of the protocol's shape (physical `leading` and `trailing`, `automatic` left to the compositor, the glass styles inside `material`, `set_backdrop` winning over `ext-background-effect-v1`, the radius in logical pixels at every scale). To review when the measurements end: whether version 10 carries all of macOS's configuration (`state`, `isEmphasized`, `maskImage`, `tintColor`, `effectIsInteractive`, container `spacing`, SwiftUI `tint` and `interactive`) or must grow before Malaysia serves it ([design 9.3](research/blur-backdrop-design.md#93-to-review-when-the-measurements-end)). The blur values per material are being measured ([materials](research/macos-materials.md), [blur](research/macos-blur.md)); the extent at 2x on macOS and what `automatic` resolves to are not measured.
Code: `crates/infinity-protocol/protocols/infinity-window-style-v1.xml`, [Backdrop design](research/blur-backdrop-design.md), [Who composes the blur](research/blur-ownership.md)

## 0092 — In-window glass is a request
Date: 2026-11-21
Decision:
- Target, from phase 3 on of [the design](research/blur-backdrop-design.md#7-phases-and-the-first-experiment): `FullscreenBar::Glass`, `ProgressiveGlass`, `TopGlass` and `Material` draw no blur. The kit leaves the area transparent, draws its chrome on a synchronized sub-surface over the content and sends `set_scroll_edge` (`Glass` hard; `ProgressiveGlass` and `TopGlass` soft) or `set_backdrop` with `within_window` (`Material`). `GlassContainer` and `RoundButton` stay as they are until merging nearby glass has a design.
- Without a compositor that offers version 10 the kit draws the solid it draws under Reduce Transparency (`FullscreenBar::Solid`, the opaque `MEDIA.material_tint`). It keeps no blur path of its own.
- The requests go out before the commit that carries the buffer, from an outbox like `flush_glass`. The sub-surface stays synchronized.
- Kit goldens and doc shots show the solid. The tests of the blur live in Malaysia, on its headless renderer.
Why: the compositor composes every blur ([ADR 0091](decisions.md)). A buffer that holds content and chrome cannot be blurred under the chrome by the compositor, so the window needs two surfaces, which is the sub-surface tree.
Rejected: keeping the Skia blur as a second path; chrome drawn with partial alpha on the content's surface (the compositor cannot recover the content); the compositor drawing the chrome (titles and controls are the app's).
Pending: confirmation by the owner of which `material` each `Thickness` takes (not mapped; macOS gives two extents in light, about 35 and about 48 px), of the chrome sub-surface above the parent rather than below it (both composed in phase 0; input and `xdg_surface` geometry are untested), and of how doc shots show glass.
Code: `crates/bk/src/glass.rs`, `crates/bk/src/platform/glass.rs`, `crates/bk/src/media.rs`, [Backdrops and media](barrierkit/15-backdrops-and-media.md), [Title bars](barrierkit/09-titlebars.md)

## 0098 — A glass window is a content surface and a chrome sub-surface above it
Date: 2026-11-22
Decision:
- The kit side of ADR 0091 and 0092, first slice (K3). When the compositor offers `infinity_window_style_v1` at version 10 or later and `wl_subcompositor`, a window's glass is declared and the kit draws no blur for it. The kit binds the global at the lower of what the compositor offers and 10 (`window_style::bind`), so a version 9 compositor gets version 9 and the legacy path: `glass.rs` draws Skia exactly as before, no request newer than the bound version is sent, and no golden changes. The decision is per window, in a root context (`BackdropSupport`) like `GlassSupport`.
- **Above, not below.** The window's root surface, the `xdg_toplevel`, stays the content. Each glass is declared on a synchronized sub-surface above it, the chrome (`ChromeSurface`). The root keeps what the rest of the kit already relies on: popups and menus parent it, as do the window geometry, the fractional scale and its viewport, the opaque region, keyboard focus and IME. Phase 0 composed both layouts, and a chrome below would make the root a transparent shell that every one of those paths would have to learn; a root region on it would also meet Malaysia's wallpaper blur ([design 7.2](research/blur-backdrop-design.md#72-phase-0-results), finding 1).
- **What is chrome.** The container that holds a declared glass: a `Declared` leaf (what `TopGlass` and `Material` become) names its parent, and that parent's subtree draws on the chrome surface, above the material, as the elements put over a SwiftUI `.background(.material)` do. That is the layout the chapters already ask for: the glass is the first child of the element it sits behind. Containers nest by merging into the outer one. A container with nothing beside it in the tree is no chrome: nothing is drawn below its glass, and all the nodes would leave the content canvas. The strip is the union of the container's area and its glass areas, cut to the window, in whole logical pixels, and the compositor places it with `wl_subsurface.set_position`.
- **How Freya's render splits.** `RenderPipeline` of Freya 0.5.3 draws every node of `tree.layers` into one canvas and the kit does not fork it. `render_passes` takes the chrome nodes out of `tree.layers` (a cost proportional to the chrome, not the tree), renders the content, then puts back only each chrome's nodes for its own pass, and restores the map. `render_pass` shifts the canvas by the strip's origin and clears a chrome to transparent. A chrome draws above every node of the content, whatever layer the content has, and anything it paints outside its strip is cut.
- **One swapchain per surface.** Each chrome has its own `Graphics` on the window's `GraphicsShared` (the same code a popup uses): a `VkSurfaceKHR` and a swapchain for its `wl_surface`, and a `wp_viewport` where the root has one. A draw acquires the root and every chrome, renders, presents the chromes first and the root last, so the parent's commit applies the chrome's cached commit with its position and its requests: one frame, no tear (ADR 0091). A chrome that fails is dropped and its nodes draw in the content; a lost device recreates them.
- **The declaration is read from the tree, not from an outbox.** The leaves cannot use hooks, and the strip must be known before the render, so `collect` reads the laid-out tree once per layout. `Requests::of` turns a chrome into one `set_backdrop` (`within_window`, the first material and radius, the rectangles of the leaves relative to the strip; the compositor keeps 16) and one `set_scroll_edge` (the edge of the first leaf and its depth from the strip's edge to the bottom of the glass); `send_requests` sends only what changed, before the buffer, and removes with empty rectangles and a depth of 0. `GlassOutbox` and `KnobOutbox` stay for the declarations of components.
- **Input and focus.** A window is still one Freya app, and the kit maps nothing. Malaysia delivers every pointer and touch event of a toplevel to its root surface in window coordinates, even over a chrome (`WindowStore::surface_under` never hit-tests sub-surfaces; M2, 2026-11-22), and keeps keyboard focus on the root. So the strip answers through the same tree, resize band, cursor shape and press serial as the rest. The first commit of this slice (`4936435866a3`) mapped events of a chrome sub-surface into the window (`at_in_window`, `Runtime::window_of`); no compositor that offers version 10 sends them, so the mapping was removed as unused. If Malaysia starts giving a sub-surface pointer focus, the kit must map it again from that commit, or the strip stops answering.
- **Elements.** `TopGlass` declares a soft top scroll edge as deep as its metrics (`FULLSCREEN_GLASS.height`). `Material` declares `set_backdrop` with `MaterialStyle::material`, whose Tengen value is `HudWindow`, and with the corner radius it was given. Both paint nothing, so the kit's tint, blur, rim and line are not drawn in this mode and the compositor draws its measured material. Under Reduce Transparency they stay the kit's opaque solid, as before.
Why: a buffer that holds the content and the chrome cannot be blurred under the chrome by the compositor (ADR 0092), and a sub-surface above an unchanged root changes the least of what already works. Rendering the chrome apart costs one traversal of the chrome nodes. The container rule needs no new API: it is the layout `Material` and `TopGlass` already document.
Rejected: a chrome below the root (see above); the chrome as large as the window with partial alpha (a second full-window buffer every frame); a strip sized by a declared rectangle instead of by the container (the content drawn over the glass would fall outside it); a layer threshold for the split (Freya layers are relative to the parent, and what sits over a glass is its container's children); forking Freya's `RenderPipeline` (the owner's decision on patching or forking Freya is open); an outbox for the leaves.
Pending: confirmation by the owner of the chrome above the root and of `HudWindow` as the material of `Material` (not measured against the kit's `MEDIA` look; `Thickness` is still not mapped). Not done in this slice: `FullscreenBar::Glass` and `ProgressiveGlass` and the rail over the content (their glass fades with the focus bar and version 10 has no way to fade a material), `GlassContainer` and `RoundButton`, drawing and presenting a chrome only when its nodes change (today both surfaces draw every frame), keeping the swapchain while a strip's size animates, more than one material or scroll edge per chrome (and Malaysia serves 4 requester surfaces per toplevel, and a chrome's material does not blur another's). Not verified: the draw, the swapchains and the commit order with a real compositor and GPU, and the hover over the strip, which needs Malaysia's M2 on the owner's laptop ([chapter 18](barrierkit/18-platform.md#glass-over-the-windows-content)); that Mesa's Wayland WSI commits a sub-surface inside `vkQueuePresentKHR` and does not throttle a synchronized one with its own frame callback.
Code: `crates/bk/src/platform/backdrop.rs`, `crates/bk/src/platform/linux/chrome_surface.rs`, `crates/bk/src/platform/window_style.rs`, `crates/bk/src/platform/linux/surface.rs`, `crates/bk/src/glass.rs`, `agents/recipes/glass_window.rs`, [Backdrop design](research/blur-backdrop-design.md), [Backdrops and media](barrierkit/15-backdrops-and-media.md), [Platform](barrierkit/18-platform.md)

## 0100 — The compositor draws every in-window glass it can; Skia keeps only what it cannot
Date: 2026-11-22
Decision:
- The rest of K3 and K4 ([ADR 0098](decisions.md), [design 7.3](research/blur-backdrop-design.md#73-k3-the-slices-and-what-they-need)). Every blur consumer of the kit, classified:
  - **Migrated in ADR 0098:** `TopGlass` (soft top edge) and `Material` (`within_window`, `MaterialStyle::material`).
  - **Migrated here:** `FullscreenBar::Glass` (hard) and `ProgressiveGlass` (soft), in the overlaid title bar and in the rail over the content. The title bar puts a declared leaf in the same holder as the bar (a leaf placed `Position::new_global()` across the window), so the bar draws on the chrome above the glass; the rail already holds its glass and its items. The `Sidebar` and `WindowGlass` panels: with version 10 the kit sends `set_backdrop` with `behind_window` and the `sidebar` material, from the window's own surface, which leaves the panel transparent as before; `within_window` would blur nothing, because nothing of the window lies below a docked sidebar (design rule 4). With version 6 to 9 it sends `set_glass`, which is not a Skia path and stays.
  - **Nothing to migrate:** popovers, menus and tooltips (the compositor draws them as popup surfaces), sheets and the traffic lights (no backdrop blur).
  - **Blocked, kept with Skia and the opaque solid of Reduce Transparency:** `GlassContainer` (the lens and the merging of Liquid Glass, which M2 serves only without lens: M2 `blur/look.rs`); the per-control `ShapeStyle::Material(Thickness)` blur (`backdrop_blur`: `Thickness` is not mapped to a material, and a material per control would exceed Malaysia's 4 requesting surfaces per toplevel); `behind_window` for popups and layer surfaces (M2's later phase, design 9.2 item 5).
- **K4.** The Skia backdrop blur of `Glass` and `MaterialElement` (`glass.rs`), the layer mask and the line under the band are deleted. With a compositor below version 10, or under Reduce Transparency, `TopGlass`, `Material`, `FullscreenBar::Glass` and `ProgressiveGlass` draw the opaque solid, as ADR 0091 says. `MaterialStyle::blur`, `FullscreenBarStyle::band` and `FullscreenBarStyle::line` had no reader left and are removed; `ctdesign` keeps the metrics, which Malaysia reads.
- **Malaysia's limits are the kit's.** It serves 4 requesting surfaces per toplevel and the sidebar's is the window's own, so the kit opens at most 3 chromes, the first by position, and a chrome's material does not blur another's.
- **The glass does not fade.** The glass of a full-screen bar used to fade with the focus bar and version 10 carries no opacity: the bar and its items fade over the chrome, and the material is there from the first frame the bar is down.
Why: macOS composes every blur outside the app (ADR 0091); what the compositor covers needs no second implementation in the client, and a client that keeps one would have two places for Reduce Transparency. What it cannot yet serve (a lens, a material per control, popups) keeps its Skia path until Malaysia covers it, rather than being drawn wrong.
Rejected: `within_window` for the sidebar (nothing below it); a container-less glass for the title bar (the bar must be in the chrome); fading the material (no opacity in version 10); deleting the Skia blur of `GlassContainer` and `ShapeStyle::Material` now (they would draw differently from today with no compositor path to replace them).
Pending: confirmation by the owner that the full-screen glass may appear without a fade and that the sidebar of a version 10 window leaves `set_glass`; the look of a sidebar over `behind_window` against `set_glass` (not compared); the scroll edge's values (not measured; M2 uses `FULLSCREEN_GLASS` and `FULLSCREEN_BAND`); the doc images of the glass bars, rendered before this change, still show the old Skia blur. Not run end to end: the sidebar, the title bar and the rail over a real compositor.
Code: `crates/bk/src/glass.rs`, `crates/bk/src/titlebar.rs`, `crates/bk/src/platform/glass.rs`, `crates/bk/src/platform/backdrop.rs`, `crates/bk/src/styles/fullscreen_bar.rs`, `crates/bk/src/styles/material.rs`, [Backdrops and media](barrierkit/15-backdrops-and-media.md), [Titlebars](barrierkit/09-titlebars.md)

## 0103 — A compositing group draws its subtree into one layer; the kit paints the tree
Date: 2026-11-22
Decision:
- `bk::group::render` is Freya's `RenderPipeline::render` copied into the kit and called from `platform::backdrop::render_pass`, the one place the kit paints a tree (T7: no fork, [Shader needs](research/shader-needs.md#p1-plan)). A scene without a group paints byte for byte as Freya does (a test compares both).
- A group is a container with a marker child (`CompositingGroup`). The loop finds the markers, takes the descendants of each parent as its members (an overlay and what hangs from it stay in the main pass) and paints them, in layer order, between one `save_layer` and its restore, at the place of the root. The root's own clips, rotations and scales apply once outside the layer; each member applies the entries of its inherited lists after the root's (`beyond`), which holds because those lists only grow by pushing. The root's inherited opacity multiplies the layer's.
- Effects: `Opacity`, `Blur`, `ColorMatrix` and an SkSL `Shader` image filter bound through the registry of P2; a uniform named `time` is the seconds since the group appeared, driven by a Freya animation that restarts every second so the tree redraws each frame.
Why: Freya paints breadth-first per node with each ancestor's opacity applied again, so no effect on a container could be built; the kit already owned the call.
Rejected: forking Freya; reading a shadow's reach (its fields are `pub(crate)`: a node with a shadow takes an unbounded layer); a `Layout` wrapper element for the group (the marker leaves every container's layout alone); `Timers` for the clock (they need the compositor's ping and do not run in tests).
- `GroupEffect::ScrollEdge` is the pocket of a scroll view under a toolbar, from stock image filters (blur, color matrix, crop, merge) because what macOS draws is a blur and a tone; `progress` is a uniform the app moves each frame. The numbers are fitted to the pixels of macOS 27.2.1 (`scrolledge` probe): the pocket tones a mean `m` to `0.5 m + 136.4` in light and `0.4 m + 24.9` in dark (three input means, linear within 0.4), blurs an 8 px checker as a Gaussian of sigma 3.4 (variable: 4.0 to 3.3 light, 3.8 to 3.0 dark from the top of the pocket to its edge; not modelled) and ends in a hard step. The `GlassContainer` shader mixes the `from` and `to` frames by the progress it receives, as the CPU did (a test compares both within 1 level).
- `scripts/check-render-loop.sh` fails when the pinned `freya-core` version or its `render_pipeline.rs` is not the one `group.rs` copies; `./dev.sh check` runs it. The groups also draw on a Vulkan device: with Mesa's lavapipe in the container the ignored test `vulkan::groups_draw_on_vulkan_as_on_the_raster` matches the raster within 10 levels (the Gaussians differ).
Pending: the variable blur of the pocket; its fade in and out as content scrolls under (the `progress` curve is not measured: the apps move it with a spring); the cost per frame of a layer on a real GPU; incremental rendering; `blend_mode` and `mask` as effects.
Code: `crates/bk/src/group.rs`, `crates/bk/src/shader.rs`, `crates/bk/src/glass_container.rs`, `crates/bk/src/platform/backdrop.rs`, `scripts/check-render-loop.sh`, `crates/ctdesign/tengen/components.json` (`scroll-pocket`), [Backdrops and media](barrierkit/15-backdrops-and-media.md#compositing-groups)

## 0114 — The kit follows the system's light or dark
Date: 2026-11-23
Decision:
- **Rule.** The desktop decides light or dark; an app built with the kit follows it, live. `ctappearance::set_system_scheme(Option<ColorScheme>)` records the system's choice, and `load_appearance()` applies it over `color_scheme`, so the runners, `resolved()` and `use_appearance` all read the same answer. `None` (no preference, no portal) leaves the domain's scheme.
- **Dark Tint.** A `dark_tint` domain stays `dark_tint` when the system asks for dark. The system's scheme turns `automatic_scheme` off.
- **macOS.** `platform/macos/system.rs` reads `NSApp.effectiveAppearance` (best match of Aqua and Dark Aqua) and observes it by key-value observing, as Apple documents for Dark Mode. A change re-resolves the appearance and gives every `NSWindow` the new `NSAppearance`.
- **Linux.** `platform/linux/system.rs` reads `org.freedesktop.appearance` `color-scheme` with the Settings portal's `ReadOne` before the first frame, as GTK does, and a thread follows `SettingChanged` and pings the event loop. The `shell` feature skips the portal: Malaysia's shell is the desktop, and its portal backend serves the domain's `color_scheme`, so other apps under Malaysia see the same scheme.
Why: macOS apps follow the system unless they opt out, and the freedesktop portal is the cross-desktop equivalent (GNOME, KDE and Malaysia serve it). A kit app on a dark Mac drew light because it read only `appearance.domain`.
Rejected: a per-app opt-in (the owner asked for a default every app gets); `AppleInterfaceThemeChangedNotification` (undocumented); reading `gsettings` or KDE files (desktop-specific; the portal covers them).
Pending: Reduce Transparency, Increase Contrast and Reduce Motion from the portal (`contrast`, `reduced-motion`) on Linux; the accent colour (`accent-color`); popups already open when the scheme changes keep the old `NSAppearance` until they close.
Code: `crates/ctappearance/src/config.rs`, `crates/bk/src/platform/macos/system.rs`, `crates/bk/src/platform/macos/mod.rs`, `crates/bk/src/platform/linux/system.rs`, `crates/bk/src/platform/linux/runtime.rs`, [The system's light or dark](barrierkit/04-appearance-and-tokens.md#the-systems-light-or-dark)

## 0115 — Standard menus by default; Edit items reach the focused text field; macOS menus are AppKit's
Date: 2026-11-23
Decision:
- **Default.** `Application::new` starts with `Commands::new()`, as SwiftUI builds its menus always; `.commands(..)` replaces them, and a scene with its own `WindowOptions::menu` keeps it. Standard View menu: Enter Full Screen (`TOGGLE_FULL_SCREEN`, F11; the kit toggles full screen).
- **Own title bar.** An `Application` window whose `window_style` is `HiddenTitleBar` or `Plain` gets no kit `Titlebar`: the app draws its own, as with SwiftUI's `.hiddenTitleBar`. So an app with its own bar has the scenes, the menus and their handlers without `run_window`.
- **Edit items.** Undo, Redo, Cut, Copy, Paste and Select All go to a focused text field before the app's handlers, as AppKit's responder chain sends them to the first responder (`main_menu::field_edit_key`, `AppInstance::edit_focused_field`). The field gets its own shortcut, pressed with Freya's `ctrl_or_meta()`: Command on macOS. The edit context menu used Ctrl and typed the letter into the field on macOS; it now uses the same key.
- **macOS.** The app menu carries the bundle's `CFBundleName`, About, Settings… (while the window's menu has an enabled `SETTINGS`), Services, Hide, Hide Others, Show All and Quit; the kit's own app-name menu is not repeated. Submenus auto-enable: an item's tag holds the kit's enabled state and `validateMenuItem:` also enables the Edit items a focused field serves. Enter Full Screen, Minimize, Zoom, Bring All to Front and an unserved Help are AppKit's own selectors (`toggleFullScreen:`, `performMiniaturize:`, `performZoom:`, `arrangeInFront:`, `showHelp:`), so AppKit validates them, retitles full screen, adds Minimize All, Zoom All and Arrange in Front and places its tiling items. The Window and Help menus are `NSApp.windowsMenu` and `helpMenu` (Fill, Center, Move & Resize, Full Screen Tile; the search field). Windows are excluded from AppKit's window list, because the kit lists them.
Why: the owner asked for the menus a Swift app has, by default and working. Measured against the Swift twin of the Resonance notes app with the accessibility menu tree: before, the kit app had only the app menu, titled with the app id; after, it has the same menus and items except those below.
Rejected: AppKit's `cut:`/`copy:` selectors on the Edit items (Freya's editor is not an `NSTextView`, so the responder chain would not reach it); a routing layer for every command (ADR 0085 keeps its rule for the app's own commands).
Pending: Cut, Copy and Undo enabled by the field's selection and history rather than by its focus; Close All and the tab items; Writing Tools, AutoFill and Dictation need an `NSTextView` (AppKit adds them, disabled); Edit items on Linux stay disabled until the app enables them, since the menu does not learn the focus; the Spanish title of Enter Full Screen on Linux is not measured.
Code: `crates/bk/src/app/commands.rs`, `crates/bk/src/app/scene.rs`, `crates/bk/src/app/windows.rs`, `crates/bk/src/main_menu.rs`, `crates/bk/src/platform/app.rs`, `crates/bk/src/platform/main_menu_export.rs`, `crates/bk/src/platform/macos/menu.rs`, `crates/bk/src/platform/macos/mod.rs`, [Standard menus](barrierkit/13-app.md#standard-menus)

## 0118 — On macOS AppKit composes every blur and glass, and SwiftUI is the reference
Date: 2026-11-23
Decision:
- On macOS the kit draws no blur or glass of its own. Each declaration that [ADR 0091](decisions.md) sends to Malaysia as a request becomes the native view that composes the same effect in WindowServer: an `NSVisualEffectView` for a material, an `NSGlassEffectView` (in an `NSGlassEffectContainerView` when shapes merge) for Liquid Glass, an `NSSplitViewItem` for the sidebar and inspector panes, an `NSPopover` for a popover, AppKit's own window background, and AppKit's own title bar under a bar that no scroll view reaches. [Backdrops and media](barrierkit/15-backdrops-and-media.md#on-macos) lists each one.
- The kit's Metal layer draws the content and the ink over the glass: text, symbols, and the selection, hover and pressed colors. Those colors are measured against a SwiftUI twin on the same macOS, in light and dark.
- SwiftUI on the running macOS is the reference. Each piece is compared with its SwiftUI twin at the same frame. What no public AppKit API reproduces is recorded as a ceiling, not imitated with a shader.
- The kit calls AppKit from Rust through `objc2` and holds no Swift code. Interactive glass (`effectIsInteractive`) needs macOS 27. Before macOS 26 there is no `NSGlassEffectView`, and the controls paint their own fill.
- An app may still add its own effects on top of the system's (`GroupEffect`, a Skia shader). They are extras; the system's materials are never redrawn by the kit.
Why: the kit gives apps the look and behavior of their desktop, as SwiftUI does on macOS. When one owner, the compositor, draws every blur, every app stays in step with the desktop. The radius, the wallpaper tint, Reduce Transparency, Increase Contrast and the look of each new release come from the system, the same in every app, and the glass never lags the content. macOS offers that owner through public views. Malaysia offers it on Linux through `infinity_window_style_v1` ([ADR 0091](decisions.md)). So one declaration in the kit maps to the same model on both systems, and the macOS port is the test that the model holds. Liquid Glass recreated in Skia would drift with each macOS release and could not match the lens, the wallpaper tint or the system settings.
Rejected: Freya or Skia shaders for the system's materials (a second implementation that drifts from the system); Swift or SwiftUI code inside the kit (a second language and build step for what AppKit already gives). Swift is to be considered again only for an effect that exists only in SwiftUI, such as `glassEffectID`, `glassEffectUnion` and `glassEffectTransition`.
Pending: the ceilings measured on 2026-11-23:
- a held knob's lens fills with the track;
- the scroll edge under the bar over kit content;
- the glass morph transitions.

The press feedback on macOS 26 is not verified.
Code: `crates/bk/src/platform/macos/blur.rs`, `crates/bk/src/platform/macos/view.rs`, `crates/bk/src/platform/macos/mod.rs`, `crates/bk/src/platform/macos/popups.rs`, [Backdrops and media](barrierkit/15-backdrops-and-media.md#on-macos), [Liquid Glass measurements](research/macos-liquid-glass.md), [Who composes the blur](research/blur-ownership.md)
