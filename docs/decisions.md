# Decisions

**Goal:** know why BarrierKit is built this way before you change it.
**Reader:** anyone who changes the kit's structure, render stack, look or public names.

Each entry gives the decision as it holds today, with its ADR number. When a
decision changes, edit its entry; when it no longer holds, delete it. Git keeps
the history. The compositor, session and shell decisions are in
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
Code: `crates/ctdesign/tengen/`, `crates/ctdesign/build.rs`, [Tengen](architecture/tengen.md)

## 0033 · 0034 · 0056 — Render stack
Decision:
- `bk` uses Freya's core crates pinned `=0.5.3`: `freya-core`, `freya-components` (no default features), `freya-edit`, `freya-animation`, `freya-engine`, `freya-clipboard`, `torin`, `ragnarok`.
- Never `freya`, `freya-winit` or any `otto-*` crate.
- Its own Wayland runner on smithay-client-toolkit and `calloop`.
- Skia runs only in clients. `ctdesign`, `ctappearance` and `infinity-protocol` pull no Freya or Skia, so the compositor never links them.
- The runner draws with Vulkan through `ash`, with no GL fallback. `freya-engine` builds Skia with only `vulkan`.
- One shared device and `DirectContext`, one swapchain per surface, `MAILBOX` when offered, else `FIFO`.
Why: winit would own the Wayland connection, and Freya's Vulkan driver lives in `freya-winit`. Frame callbacks already pace redraws, so `FIFO` would throttle twice.
Rejected: a kit from `otto-kit`; Skia on EGL/GLES.
Code: `crates/bk/src/platform/vulkan.rs`, `scripts/check-client-only-deps.sh`, [Platform](barrierkit/18-platform.md)

## 0035 — Own app identification and CSD
Decision: a window's `app_id` is `ctdesign::NATIVE_APP_ID_PREFIX` plus the app name, unless `WindowOptions::app_id` sets another. Every window requests client-side decoration; if the compositor answers server-side, the kit turns off its edge resizing.
Why: the prefix lets the compositor tell kit windows from the rest and leave their chrome to the kit.
Code: `crates/bk/src/platform/window.rs`

## 0039 — Infinity v1
Decision: `infinity_window_manager_v1` is a private Wayland protocol for the shell's window management. `infinity-protocol` generates the client side (feature `client`, used by `bk`) and the server side (feature `server`, used by the compositor). `bk` binds it only with the `shell` feature; the compositor decides which clients may bind it.
Why: the standard protocols do not cover what the Dock and the menu bar need from the compositor.
Code: `crates/infinity-protocol/protocols/`, `crates/bk/src/shell/infinity.rs`, [Wayland protocols](barrierkit/18-platform.md#wayland-protocols)

## 0041 · 0057 · 0065 · 0078 · 0079 — Sidebar: flat, backdrop and glass
Decision:
- The sidebar is flat and full height, as in macOS 27: no inset, rounded corner, rim or edge line; only its colour sets it apart from the content. Its geometry lives in `components.window.sidebar`; an app changes it through `SidebarStyle`. The floating island of ADR 0041 was dropped on 2026-11-20 (ADR 0079): macOS 27 does not draw it, and the user's `sidebar_style` key went with it. A file that still has the key loads; the key is ignored.
- A window with a sidebar draws the unified 52 title bar (`Bar::Unified`), with the traffic lights at (19, 19) inside the sidebar.
- Over a `Backdrop` the sidebar is transparent and sends no glass panel.
- Otherwise the sidebar is a glass panel: the kit leaves a hole and sends `set_glass` (Infinity window style, version 6) with the rectangle, corner, `base`, `solid` and `chroma`. There is no public API.
- The compositor fills the panel with `base` plus the blurred wallpaper's chroma and draws the rim; unfocused, it shows flat `solid`. Towards the edge that touches the content, `base` veils the fill, so the wallpaper's colours fade but stay.
- With no such compositor, or with reduce transparency or contrast, the kit paints the opaque sidebar itself.
- In full screen with a glass bar, the glass covers the whole top strip, not one column.
Why: only the compositor sees the wallpaper. The macOS sidebar keeps the app's lightness and takes only part of the wallpaper's chroma. The island was measured against macOS 27 (`docs/research/macos-control-metrics.md`, "Sidebar and window (macOS 27)") and does not exist there.
Rejected: a translucent fill over a plain blur region; it lets the wallpaper's lightness through and cannot say a corner. Keeping the island as a user option: nothing in macOS 27 backs it.
Code: `crates/bk/src/sidebar.rs`, `crates/bk/src/glass_panel.rs`, [Sidebar](barrierkit/10-sidebar.md)

## 0047 — BarrierKit in layers
Decision: `bk` is one crate in four layers (platform, primitives, views, patterns) plus a cross-cutting `app`. A layer uses only the layers below it, and patterns compose only the public API of views. Code only `malaysia-shell` uses lives in `src/shell/`, behind the `shell` feature.
Why: the kit must build any desktop app, like AppKit or libadwaita, without each app rebuilding sidebars, search fields and controls.
Pending: nothing is built until an app needs it. Partial damage waits for Freya: 0.5.3 repaints the whole tree every frame.
Code: `crates/bk/src/lib.rs`, [Layers](barrierkit/03-layers.md)

## 0048 — BarrierKit in its own repository
Decision: `bk`, `ctdesign`, `ctappearance` and `infinity-protocol` live in `github.com/HBlanqueto/barrierkit`. malaysia pins all four by one git `rev` in `Cargo.toml` and takes barrierkit as a flake input, reusing its prebuilt Skia. `./dev.sh kit-local` in malaysia tests a local checkout before publishing.
Why: the toolkit carries its style and its reading of the appearance; keeping them in malaysia would make the two repos depend on each other.
Code: `flake.nix`

## 0082 — Traffic lights as macOS 27 draws them
Decision:
- A light is the 14 pt disc of macOS 27: a vertical gradient fill (four stops), a 0.5 pt ring (three stops, a vertical gradient too), a halo in the light appearances and a glyph while the pointer is over the group. A pressed light grows to 16 pt and brightens (`TRAFFIC_PRESS`).
- Every color is a token measured per appearance, light, dark and Increase Contrast (`themes.*.traffic-lights`, `traffic-lights-contrast`); none depends on the accent.
- `standard` is that look as it is. `accent` and `wallpaper` give each light a tone and `ctappearance::traffic::looks` recolors its measured look with it: alpha, and lightness and saturation relative to the light's base, stay; the hue is the tone's; the glyph stays at 3:1.
- Skia rasterizes each light once per look at the window's scale (`bk/src/traffic_raster.rs`), like the symbol tiles.
- `traffic_light_shape` is gone: macOS 27 has no square light, and the ring, the halo and the press are defined for a disc. A file that has the key still loads, and a write drops it.
Why: the lights are the most seen control of a window, and the measurement says what they are (`docs/research/macos-control-metrics.md`, "Traffic lights (macOS 27)").
Rejected: keeping a corner radius in the style, which would need its own ring and halo for no macOS look; blending the glyph with plus-darker like macOS, which Freya cannot do, so the glyph is its measured final color.
Code: `crates/ctappearance/src/traffic.rs`, `crates/bk/src/traffic.rs`, `crates/bk/src/traffic_raster.rs`, `crates/ctdesign/tengen/`

## 0061 · 0062 · 0063 — SwiftUI-shaped API
Decision:
- An app depends only on `bk` and writes `use bk::prelude::*;`.
- `Application::new(name).window(title, content).run()` opens a window with background and `Titlebar`; `run_window` stays for windows that draw their own bar.
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

## 0080 — Accent colors
Decision:
- The accents are the nine of System Settings: Multicolor and eight named ones (`ctdesign::ACCENTS`). Multicolor is no entry: with no `accent` the controls are blue and an app keeps its own accent.
- Each named accent holds the colors it decides in four looks, light, dark and Increase Contrast light and dark, as measured on macOS 27.2.1 with the accent set per process. Blue points to the theme tokens, so no value is written twice.
- The user's choice is `accent = "<name>"` in `appearance.domain`. `accent_color` stays for a color macOS has no name for, and one that equals the light `control` of a named accent reads as that accent. `wallpaper_accent` stays; a monochrome wallpaper falls back to graphite.
- A color without a name has only its `control`. The rest follows a rule fitted to the named accents (`ACCENT_RULE`): the text selection color is exact, the selection and focus colors are an OKLab shift within a few levels, the control fills are the `control`. Where the ink does not reach 4.5:1, the fill darkens (WCAG 2.2 §1.4.3); a named accent keeps macOS's value.
- Text selection draws `selected_text`, the accent over the window at 30%.
Why: macOS has per-appearance values for every accent and no custom one; a single color for light and dark, and blue pressed and disabled fills under any accent, were wrong. The rule covers what macOS gives no value for.
Rejected: one hex per accent for both appearances (the dark values of red, yellow, purple and graphite differ); deriving the named accents from their `control` (the best formula misses the selection color by up to 6 levels and the focus ring by 14, so each accent has its own).
Code: `crates/ctdesign/tengen/components.json` (`settings.accents`, `settings.accent-rule`), `crates/ctappearance/src/accent.rs`, `crates/ctappearance/src/resolve.rs`, [Accents](barrierkit/04-appearance-and-tokens.md#accents)

## 0081 — Icons: glyphs in the kit, icon files in the theme
Date: 2026-11-20
Decision:
- A glyph is a one-color symbol the kit draws inside a control, menu, toolbar, sidebar, list, form or state. It is a `Symbol`: SVG paths on a 16 grid in `currentColor`, named after its SwiftUI name, fitted to AppKit at 16 pt (20 px).
- An icon is a file with a freedesktop name that other apps, the shell and malaysia load by name: apps, places and folders, MIME types, devices, and the `-symbolic` actions and status. It lives in `malaysia-icon-theme`.
- Neither repository holds a file from Apple. The paths and the theme files are drawings made for the project; the ink boxes and masses they were fitted to stay out of the repository.
- The kit does not read the theme and the theme does not read the kit. A glyph moves to the theme only when the theme lacks its freedesktop name and its owner asks for it.
Why: the kit draws a glyph at any size, stroke and color from one path, which a file lookup by name and size cannot do. The theme already ships the macOS-style set the desktop's apps need: about 3,700 symbolic icons, 50 folders, 235 file types, 63 devices; every freedesktop name that matches a kit glyph was found in it (50 checked). Copying the glyphs would draw each twice and mix the kit's MIT with the theme's LGPL, CC BY-SA and GPL licenses.
Rejected: every glyph in the theme (the kit would load files at run time or embed GPL art, and lose the stroke and the state colors); every icon in the kit (Linux apps look icons up by name and the kit is not an icon theme); drawing the theme's symbols from the kit's paths (it would replace the MacTahoe copies, which is the theme's decision).
Code: `crates/bk/src/symbol.rs`, `crates/bk/tests/kit/appkit_icons.rs`, `scripts/appkit-compare.py` (`icons`), [Macos icons](research/macos-icons.md)

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

## 0084 — Labels: owned text or a view
Date: 2026-11-21
Decision:
- Visible text a control takes (label, title, placeholder, caption, item) is `impl Into<Cow<'static, str>>`. A literal or a `tr!` result stays borrowed; a `String` is owned and needs no leak. The few that take `impl Into<String>` today, such as `Button::new`, change too.
- Identifiers stay `&'static str`: menu ids, `app_id`, pane ids and `GlassContainer` item ids.
- A control that shows a label takes a closure that returns the view, through a builder named `label` (SwiftUI's `label:`), which replaces the drawn text. The string given to `new` stays the accessible name. The control runs the closure inside a `TextStyle` scope that holds its font and the ink of its current state, and sets the same font and ink as Freya's inherited text style: `Text`, `Label` and a Freya `label()` take them.
- Order: the text type first (done 2026-11-21); then `Label` (a title and a `Symbol`, done 2026-11-21: `Label::new(title, Symbol)` in `crates/bk/src/elements.rs`, which shadows Freya's `Label` in the prelude); then the `label` builder: done 2026-11-21 for `Button` (`Button::label(|| view)`), the other controls when an app needs it (ADR 0047).
Why: a list built at run time (networks, files, users) leaks every text: `i18n::intern` keeps each distinct one for the life of the process. SwiftUI takes text or any view. The text change is source compatible: literals, `tr!`, `String`, `.map(Type::new)` and `fn(&'static str)` pointers still compile (checked with `rustc` on 2026-11-21), so each control written in the old form only adds one more signature to change. A view built by the app cannot see the control that will draw it: `Text` and `Image` fix their font and color when they are built, so an eager view in a `bordered_prominent` button drew the label ink and 750 to 2300 channels differed from the drawn text (2026-11-21), while a Freya `label()` sets neither and inherits. A closure runs in the control's render, after its scope exists. Freya's `Callback` never compares equal (`freya-core-0.5.3/src/event_handler.rs`), so a button with a label re-renders with its parent and the closure cannot show a stale value. Freya cannot read a name out of a view (it overwrites the accessibility label in `create_node` and has no text runs), so the string stays the AT-SPI name.
Rejected: an eager view for `label`, for the reason above, and `NoArgCallback` for the closure, which compares always equal and would leave a stale label; `impl Into<String>`, which copies every literal each time a view is built; one `impl IntoElement` parameter for both, since Freya turns a `&str` into a `label` with `to_string()` (`freya-core-0.5.3/src/elements/label.rs:73`) and the control would lose its accessible name; keeping `intern` for runtime lists, which grows without bound.
Pending: the `label` builder on controls other than `Button`. `Button` proves it: a view label is pixel equal to the drawn label in light and dark, plain, prominent, disabled and pressed, and in an unfocused window (`crates/bk/tests/kit/button_label.rs`, `crates/bk/src/platform/tests.rs`). Two limits: a bare `Image::system` keeps its own color, so a symbol goes in a `Label`; with a text size other than the default a kit `Text` or `Label` scales and the drawn label does not.
Pending, text type: collections still take `&'static`: the columns of `Table::new` (`Vec<&'static str>`), `Slider::labels`, the swatch names of `ColorWell` and `RuleKind::Choice`. The `Option` parameters (`GroupBox::new`, `PaneGroup::new`, `Form::highlighted`) take `Option<Cow<'static, str>>`, so a caller writes `Some("x".into())`. A caller that passed a borrowed `&str` or `&String` to an old `impl Into<String>` parameter passes a `String` now. `i18n::intern` stays: ids built at run time (pane ids) still need `&'static str`, and malaysia calls it 37 times until its rev moves.
Code: `crates/bk/src/controls/button.rs`, `crates/bk/src/styles/theme.rs`, `crates/bk/src/menu/model.rs`, `crates/bk/src/i18n.rs`, [SwiftUI API gap](research/swiftui-api-gap.md)

## 0085 — No `Binding`, no action routing or app lifecycle callbacks
Date: 2026-11-21
Decision:
- The kit has no `Binding`. A text control takes `State<String>`; every other control takes its value and a callback, and keeps a private copy until the app passes a new value (`use_controlled`).
- An action reaches every handler of the window: `use_menu_action` registers a handler and `dispatch` calls them all. The kit adds no routing to the focused view, no close veto (`windowShouldClose`), no quit call (`applicationShouldTerminate`) and no open-URL event (`application(_:open:)`). A close request closes the window at once, except a sheet's; closing the last window ends the process.
- Both are non-goals until an app asks for them. [SwiftUI to the kit](barrierkit/21-swiftui-to-the-kit.md) tells the app author the same.
Why: Freya 0.5.3 has `Writable<T>` and `IntoWritable` (`freya-core-0.5.3/src/lifecycle/writable.rs`), but its `PartialEq` returns `true` for any two values (line 60), so a component that holds one cannot show the diff that its source changed. Neither malaysia (working tree of 2026-11-21) nor curseterm (`1375c72`) uses `Writable`, `IntoWritable` or `use_radio` (grep). A grep of both for a close veto, a quit call and an open-URL event finds none; the only hits are winit's `CloseRequested` and `infinity_force_quit` in malaysia's compositor, which are not kit apps. The three repositories have no issue that asks for any of them (`gh issue list`, 2026-11-21). AGENTS.md rule 2 forbids adding what nobody uses.
Rejected: controls taking `impl IntoWritable` beside value plus callback, and a `Binding` type of the kit's own; no app uses either.
Pending: the action routing, the veto, `quit` and the open-URL event return when an app of the kit needs one; a `Binding` returns when two components must edit one value and a `State<T>` in a context cannot serve them.
Code: `crates/bk/src/controlled.rs`, `crates/bk/src/app/action.rs`, `crates/bk/src/platform/window.rs`, [Swift syntax in Rust](research/swift-syntax-in-rust.md)
