# BarrierKit on macOS arm64

**Goal:** decide how a BarrierKit app runs on macOS arm64, with the window
frame drawn by the system (SSD) or by the kit (CSD), and what to do about the
blur and the effects that macOS owns.
**Scope:** from `cargo build` of `bk` on an Apple-silicon Mac to a window on
screen that matches a native one. It covers the build, the runner, the window
modes, the desktop services, the parity test and the blur. It does not cover
the Swift translator ([swift-app-translation.md](swift-app-translation.md)),
iOS or UIKit, or the `shell` feature (layer-shell and lock surfaces have no
macOS counterpart).
**Snapshot:** `main` at `dc9656b2`, 2026-11-22. Xcode 27.2, Swift 6.4.2,
macOS 27.2.1 on an Apple-silicon Mac, Rust 1.100.
**Reader:** the owner, who decides, and the engineer who builds the port.
**Input:** the code of `crates/`, ADR 0034, 0035, 0056, 0091, 0092, 0098 and
0100, [Platform](../barrierkit/18-platform.md), and four experiments on this
Mac ([Evidence](#evidence)).
**Output:** a verdict, a recommended design, milestones, three blur proposals
and the decisions that wait for the owner.

## Steps

1. Read the [verdict](#verdict) and the [decisions](#decisions-for-the-owner).
2. To build the port, follow the [milestones](#milestones) in order.
3. To check a milestone, run the [parity test](#parity-test).
4. Before any blur work, the owner picks one [proposal](#blur-three-proposals).

## Verdict

The port is feasible, and the cost is in one place. Everything the kit draws
(Freya, Torin, Skia, the controls, the styles) builds on macOS arm64. What does
not build is the Linux stack under `platform/`: Wayland, Vulkan, xkbcommon,
inotify, D-Bus and AT-SPI. The kit has no macOS backend and says so:
[`agents/prompt.md`](../../agents/prompt.md) lists «No macOS, no Windows, no
X11», and `flake.nix` builds for Linux only.

Recommended: a second runner written on AppKit (`objc2`) and Metal, behind the
same `run_window` API, as ADR 0034 did for Wayland. macOS is the compositor of
this design: its WindowServer composes the blur, draws the frame and owns the
menu bar, so most of the kit's Wayland protocol maps onto native views.

## Evidence

| # | Experiment | Result |
|---|---|---|
| E1 | `cargo check -p ctdesign` on macOS arm64 | builds |
| E2 | `cargo check -p ctappearance` | fails: `crates/ctappearance/src/watch.rs:6`, `nix::sys::inotify` does not exist on macOS |
| E3 | `cargo check -p bk` | fails in the build script of `wayland-sys` 0.31.11: no `pkg-config` and no `libwayland` |
| E4 | A scratch crate with `freya-engine =0.5.3` (feature `metal`), `freya-core`, `freya-components`, `freya-edit`, `freya-animation`, `freya-clipboard`, `freya-testing`, `torin` and `ragnarok`, all `=0.5.3` | builds in 26 s: the prebuilt Skia with Metal downloads and links. Nothing was drawn |

The scratch crate is eight lines of `Cargo.toml`; the features are those of
`freya-engine-0.5.3/Cargo.toml` (`metal = ["freya-skia-safe/ganesh", "freya-skia-safe/metal"]`).

How much of `bk` is Linux-bound (lines counted by `find … | wc -l`, tests
included):

| Part | Lines | Linux stack |
|---|---|---|
| `crates/bk/src` | 54,967 | |
| `platform/` and `shell/` | 12,866 | Wayland, Vulkan, xkbcommon, calloop, AT-SPI |
| 11 files outside them | not counted | nine use `zbus` (`notifications`, `portal`, `secrets`, `status_item`, `session_bus`, `main_menu*`, `app/application`); `glass_panel` and `menu/tests` use the protocol types |

So about three quarters of the kit never touches the Linux stack. Not
verified: that this three quarters compiles on macOS once the rest is gated.
Only M0 shows it.

## Options for the runner

| | A. Own runner on `objc2` + Metal (recommended) | B. Runner on `winit` for macOS only |
|---|---|---|
| Who owns `NSWindow`, `NSView`, the menu | The kit | winit |
| SSD, CSD and hybrid frames | all three | the window attributes winit exposes |
| Blur, glass and chrome layers as native views | direct | needs `raw-window-handle` plus `objc2` calls on winit's view anyway |
| IME, pasteboard, cursors, popups as child windows | `NSTextInputClient`, `NSPasteboard`, `NSCursor`, `NSPanel` | partial; popups and sheets are not winit's |
| Reuses the kit's API | yes, behind `run_window` | yes, but the runner is the kit's own either way |
| New dependencies | none: `objc2` 0.6.5 and `objc2-app-kit` 0.3.2 are in `Cargo.lock` already | `winit` |
| Rule it meets | ADR 0034 (the kit owns its runner) | ADR 0034 names `freya-winit` and `winit` as «never» for Wayland; its reason does not apply on macOS, but the gain is small |

Rejected for now: Freya's `freya-winit` (the kit's `platform::runtime` is its
own, so the kit's popups, chromes and menus would not run on it); a Linux VM
or Asahi (it is not macOS).

## Window modes: SSD, CSD and hybrid

`WindowOptions` already carries `decoration` with `DecorationResponse::Client`
or `Server` ([8](../barrierkit/08-windows.md)). Under `Server` the `Titlebar`
takes no space. On macOS the answer is a property of the window the runner
creates.

| Mode | `NSWindow` | Traffic lights | Frame, corner, shadow | Use |
|---|---|---|---|---|
| SSD | `titled`, `closable`, `miniaturizable`, `resizable` | native | WindowServer | apps that look like any Mac app; default |
| Hybrid | the same plus `fullSizeContentView` and `titlebarAppearsTransparent` | native (`standardWindowButton`) | WindowServer | a kit `Titlebar` and toolbar under the native lights: Settings, Notes, Finder |
| CSD | `borderless`, `hasShadow` | the kit's `TrafficLights` | the kit (corner 16, continuous; ADR 0097) | titleless apps, HUDs, and the Linux look |

Hybrid is how Mac apps with a unified toolbar are made. The kit already
measures the insets it needs (`macos-windows.md`, `macos-control-metrics.md`).
Full screen, Mission Control, Stage Manager and window tabs come free in SSD
and hybrid. They are open work in CSD: `FullscreenBar` and `WindowTabs` exist,
and `NSWindow.toggleFullScreen` needs `titled`.

## Design of the macOS runner

The runner has the shape of ADR 0047: a seam under `run_window`, with the
Wayland code on one side and AppKit on the other.

1. Gate by `cfg(target_os)` in `crates/bk/Cargo.toml`: `sctk`, `wayland-*`,
   `xkbcommon`, `ash`, `accesskit_unix` and `nix` for Linux; `objc2*`,
   `accesskit_macos` and the Metal feature of `freya-engine` for macOS. The
   `shell` feature stays Linux-only.
2. Keep the common types (`WindowOptions`, `WindowControls`, `PopupSpec`,
   `Timers`) in `platform/` and gate the Wayland code by `cfg`; macOS files
   live in `platform/macos/` ([M0 result](#m0-result-2026-11-22-uncommitted)).
3. Run the loop on `NSApplication`. A Freya redraw request is a `CFRunLoop`
   source; a `WakeHandle` is a `CFRunLoopSourceSignal`; a timer is a
   `CFRunLoopTimer`. `calloop` stays on Linux.
4. Draw into a `CAMetalLayer` of a content `NSView` with one Skia Metal
   `DirectContext` per process. The scale is `backingScaleFactor`; there is
   no fractional-scale protocol to bind.
5. Map `NSEvent` to Freya events in the content view; take the IME from
   `NSTextInputClient`, and the cursor from `NSCursor` (the `CursorIcon` shapes that `cursor_shape_from_icon` maps today).
6. Open popups, menus and tooltips as child `NSPanel`s positioned with the
   same `PopupAnchor` and `PopupGravity` the kit already computes, sheets as
   `beginSheet`.
7. Replace the D-Bus services with the native ones:

| Linux today | macOS |
|---|---|
| `org.freedesktop.Notifications` | `UNUserNotificationCenter` |
| Desktop portal (files, URIs) | `NSOpenPanel`, `NSSavePanel`, `NSWorkspace` |
| Secret Service | Keychain |
| `StatusNotifierItem` | `NSStatusItem` |
| `dbusmenu` global menu | the native `NSMenu` of the app, built from `MainMenu` |
| `org.freedesktop.Application` (single instance) | `NSRunningApplication` |
| AT-SPI through `accesskit_unix` | `accesskit_macos` |
| inotify on `appearance.domain` | `kqueue` or the `notify` crate |

The kit keeps its own appearance. On macOS it can also read the live system
state (`NSApp.effectiveAppearance`, `AppleAccentColor`, Increase Contrast,
Reduce Transparency, Reduce Motion) and feed `ctappearance::Resolved`, with
`appearance.domain` as an override. The kit's `ctdesign` tokens were measured
on this system, so the two agree.

Size, an estimate and not a measurement: the Linux files this replaces
(`window` 976, `runtime` 736, `surface` 590, `popup` 605, `vulkan` 728,
`clipboard` 470, `ime` 172, `timer` 133 lines, plus input) are about 5,000
lines. AppKit supplies IME, pasteboard and popups natively, so the macOS side
should be smaller.

## Milestones

| M | Output | Check |
|---|---|---|
| M0 | The gate and the seam. `ctappearance` watches with `kqueue`. D-Bus services compile to stubs. Blur draws the opaque solid that ADR 0092 defines | `cargo check -p bk` passes on `aarch64-apple-darwin`; `freya-testing` renders the golden scenes on the CPU |
| M1 | One SSD window: `hello_world` and `counter` open, take the mouse and the keyboard, resize | the two recipes run; Retina scale correct |
| M2 | Popups, menus, sheets, tooltips, clipboard, IME, cursors, timers | the `menu` and `alert` scenes of the bench open |
| M3 | Hybrid and CSD windows; the native menu bar from `MainMenu` | Settings window recipe in the three modes |
| M4 | Notifications, panels, Keychain, status item, accessibility | VoiceOver reads a window |
| M5 | Blur and glass by the chosen proposal | [parity test](#parity-test) of the blur scenes |

M0 is useful alone: it lets the bench run on the Mac that holds the probes.

### M0 result (2026-11-22, uncommitted)

M0 is built: 15 files changed (+147, −34) and one new file,
`platform/macos/clipboard.rs`. Checks on this Mac and in the Linux container:

| Check | Result |
|---|---|
| `cargo check -p bk` on `aarch64-apple-darwin` | passes; 60-odd dead-code warnings, all items only the Linux runner uses |
| `cargo test -p bk -p ctappearance -p ctdesign --lib` on macOS | 214, 211 and 10 pass |
| `cargo test -p bk --test kit` on macOS (CPU render) | 117 pass, 5 fail, 19 ignored. The five are golden images and a label comparison; Inter, Adwaita Sans and Geist are not installed on this Mac, which is the likely cause (not isolated) |
| `cargo clippy -p bk -p ctappearance -p infinity-protocol --all-targets --features bk/shell -- -D warnings` in the Linux container | clean |
| `cargo test --lib` for `bk` and `ctappearance` in the Linux container | 223 and 272 pass |

What it did, and what it found:

1. Dependencies by `target_os`. Linux keeps `sctk`, `wayland-*`, `xkbcommon`,
   `ash`, `accesskit_unix` and the Vulkan feature; macOS takes the Metal
   feature and `calloop`. The workspace `wayland-client` loses `system` and
   `bk` adds it back for Linux, so `infinity-protocol`'s client code (and with
   it `Bar`, `PopoverShape` and `Knob`) builds on macOS with the Rust backend.
2. `smithay-client-toolkit` 0.20 does not build on macOS
   (`rustix::pipe::pipe_with` is Linux-only), so no Wayland runner code can be
   compiled there and the split had to be by `cfg`, file by file.
3. The seam is not where the plan put it. `platform/app.rs` (`AppInstance`,
   the Freya driver) reads about twenty fields of `Runtime` (ping, queues,
   timers, popups, fonts, defaults revision, capability checks), so it stays
   Linux-only. M1 must pull those fields into a backend-neutral host struct
   that both runners fill; that is the first task of M1.
4. `WindowControls` holds the Wayland clipboard type. On macOS a stand-in
   (`platform/macos/clipboard.rs`) gives the system pasteboard text through
   `copypasta` and no primary selection.
5. `run_window` on macOS returns the new `RunError::NoRunner`.
6. `ctappearance::watch` is Linux-only; M1 needs a `kqueue` watcher.

### M1 result (2026-11-22, uncommitted)

M1 is built as an AppKit runner in `platform/macos/` (about 1,050 lines):
`Host` (the fields `AppInstance` reads from its runner) was extracted from
`Runtime`, so the driver is shared; the macOS `Runtime` fills it. Reference:
Freya's own Metal driver, `crates/freya-winit/src/drivers/metal.rs` in
[marc2332/freya](https://github.com/marc2332/freya), which fixed the objc2
versions and the `CAMetalLayer` settings (`framebufferOnly` false).

Checked on this Mac, display scale 1, captures in the session's scratchpad:

| Check | Result |
|---|---|
| `hello_world` | opens: system frame, traffic lights, text drawn by the kit |
| `counter`, three clicks | shows 3 |
| `binding`, click a field and type `hi` | field shows `hi`, focus ring, label `Hello, hi` |
| `stopwatch`, Start | `00:02.19` after about 2.2 s: timers and the timeline run |
| Resize to 700 × 500 through System Events | content laid out again, centered |
| Close the window | the process exits with status 0 |

Findings:

1. `calloop`'s poll descriptor is itself pollable: a `CFFileDescriptor` on it
   in the main run loop delivers pings (three of three in a spike). Its
   timers are not descriptor-based, so `Timers` use `dispatch_after`.
2. A `mouse click` that posts down and up with no gap loses the up event
   (probe tool artifact; down, 150 ms, up works).
3. `NSApplication.activateIgnoringOtherApps` is deprecated; `activate()` is used.

### M2 result (2026-11-22, uncommitted)

Popups, sheets, utility windows and cursors are built in `platform/macos/`
(`popups.rs`, `place.rs`, additions to `view.rs`, `mod.rs` and `input.rs`).
A popup is a `BKPanel` (borderless, non-activating, able to become key) added
as a child window, with an `NSVisualEffectView` behind the Metal layer: this is
P1 for popups. The material is `menu` for a grabbing popup, `popover` and
`toolTip` by shape, and the corner is the measured one (`CORNER_MENU`,
`CORNER_POPOVER`, `CORNER_TOOLTIP`). `place.rs` is `xdg_positioner` (anchor,
gravity, offset, flip, slide, resize) against the screen's visible frame, with
five tests.

Checked on this Mac with a throw-away example (not kept):

| Check | Result |
|---|---|
| `PopUpButton` click | a menu opens below the button, rounded, with shadow and material |
| Pick Beta | label `Beta`, `selected 1`, menu closes |
| Click outside the menu | the menu closes (a grabbing popup closes when it loses the key window) |
| `.help` over a label | the tooltip appears after the delay, with the tooltip material |
| `Alert` | opens as an attached sheet over a dimmed parent; Save closes it |
| Title bar and effect views | follow the kit's scheme (light) instead of the system's (dark) |

Findings and limits:

1. The popover arrow band is the compositor's on Linux; here the band stays
   empty and the popover is a rounded rectangle.
2. The kind of a popup is guessed: a popover shape if it has one, else a menu
   if it grabs, else a tooltip.
3. The first click after launching a bare binary may only activate the app.
4. `NSCursor::columnResizeCursor` needs macOS 15, so the floor is macOS 15.
5. Cursors are implemented with cursor rects and were not looked at on screen.

### IME result (2026-11-22, uncommitted)

`ContentView` implements `NSTextInputClient` (`view.rs`). A key goes to the
input method while it composes, and for the characters of a focused text field
(no Command or Control, not a named key); what it does not take is sent to the
kit as before. `insertText` pushes the key events the Wayland runner pushes on
`commit_string`, `setMarkedText` sets `WindowControls::ime_preedit`, and
`firstRectForCharacterRange` converts the area of `set_ime_cursor_area` to
screen coordinates. `flagsChanged` now feeds `modifier_flags()`.

Checked: on this Mac (Spanish ISO layout) a dead acute then `a` types `á` in
the `binding` field; Option+e types `€`; plain letters and the earlier
mouse and key checks still work. Not checked: a CJK input method and its
candidate window (switching the input source is a system setting, so it was
left to the owner), the preedit text drawn in a field (`TextField` shows
nothing while a dead key waits), and an IME inside a popup's search field.

Still not done after M2: scale 2, the native menu bar,
the live system appearance, the title bar mode choice (SSD only), clippy on the
macOS code (the toolchain has no `cargo-clippy`), and tests for the runner.

### M3 result: frame modes (2026-11-22)

`WindowOptions::window_style(WindowStyle)` ([8](../barrierkit/08-windows.md#window-style))
picks the frame of a main window; `platform/macos/frame.rs` maps it to the
style mask and to `DecorationResponse` (four unit tests).

| `WindowStyle` | Result |
|---|---|
| `TitleBar` | The SSD window of M1, still the default |
| `HiddenTitleBar` | The cheap hybrid: `fullSizeContentView`, transparent title bar, hidden title, native lights over the content, `Server` (the kit `Titlebar` takes no space). Run with `notes_sidebar`: the native lights sit over the sidebar, and `Sidebar::traffic_lights(true)` draws a second set there, so an app in this mode leaves it off |
| `Plain` | The CSD window: a `BKToplevel` (an `NSWindow` that can be key and main) that is borderless, not opaque, with shadow, and a content layer with the continuous corner `CORNER_WINDOW.fitted`; `Client`, so the kit draws `Titlebar` and `TrafficLights` |

Checked on this Mac with `notes_sidebar`, `custom_window` and
`titlebar_over_gradient`: a press on the `Titlebar` moves the window
(`performWindowDragWithEvent` on the press, from `WindowCommand::Move`); the
edge of a borderless window with `Resizable` resizes it natively, so the kit's
`resize_edge_at` band is not needed; close and minimize from the kit lights
work. `notes_sidebar` has a Swift twin (`~/src/probes/csdtwin.swift`) of the
same size, texts, scheme and position: the two lights, the sidebar edge (x 220)
and the sidebar rows agree to 1 px, and the silhouette corner differs by at most
2 px in the first row.

Differences left: the shadow of a borderless window is the system's smaller one;
the kit draws a hairline under the title bar and a darker, heavier title; the
sidebar toggle has no ring; `ContentUnavailableView` is smaller and not bold;
the sidebar icons are badges, not plain symbols; the kit's default
`traffic_lights = "wallpaper"` tints the lights (the twin sets `"standard"`).
Not done: full screen of a `Plain` window (`toggleFullScreen` needs `titled`),
the kit `Titlebar` drawn under native lights (needs a way to reserve their room),
the Settings recipe in the three modes, and the zoom button on a borderless
window.

### M3 result: native menu bar (2026-11-22)

`platform/macos/menu.rs` builds the menu bar from `MenuHandle::snapshot()` of
the key window. The standard app menu (About, Hide, Hide Others, Show All, Quit
with Command+Q; texts in `bk.ftl`) stays first; the kit's menus follow it.

| Rule | Where |
|---|---|
| Ctrl is Command, Alt is Option, Shift is Shift, Super is Control; letters lowercase, named keys and F1 to F12 become AppKit's function characters | `key_equivalent` (16 unit tests with the rest of the mapping) |
| `CheckState` On is a checkmark (radio too, as AppKit's own menus), `Mixed` the dash, a disabled item or submenu is disabled | `entries`, `state` |
| Palettes, search fields and search results are dropped; a header is a disabled item; icons and badges are not shown | `entries` |
| The menu follows the key window; a window without `options.menu` keeps the last one; closing the owner clears it | `register`, `show`, `forget` |
| A new `MenuHandle::revision()` patches the items in place, so an open menu keeps its items | `sync` from `poll_all`, `apply` |
| An action calls `menu_actions.dispatch(id)` of the window that owns the menu, then polls and redraws | `fire` |
| `performKeyEquivalent:` of `ContentView` gives a Command chord to a focused text field when `edits_text` matches, so Command+C reaches the field and not the menu; the field gets it as META, because Freya's editor reads Command as META on macOS (before, Command+C typed a `c`) | `view.rs`, `input::edit_chord` |

## Parity test

**Goal:** show that a kit app and a native app look the same on one Mac.
**Input:** the probes of `~/src/probes`, the kit's bench
(`crates/bk/tests/kit/appkit.rs`) and `scripts/appkit-compare.py`.

1. Run the probe window and the kit window of one scene, at 1x and 2x, in
   light, dark and Increase Contrast.
2. Capture each with `screencapture -l <window id>`, raw, as the probes do.
3. Compare with `appkit-compare.py`. Use the golden tolerance (channels that
   differ by more than 3) and report `% differ`.
4. Compare the frames: size and position of the window and of its lights from
   `CGWindowListCopyWindowInfo` (`~/src/probes/winlist`).

Three apps to build first, each in SSD and in CSD or hybrid:

| App | Kit pieces | Native counterpart |
|---|---|---|
| Settings | `SettingsWindow`, `Sidebar`, `Form` | System Settings |
| Notes | `SplitView`, `List`, `TextEditor`, toolbar | Notes |
| Calculator | a grid of `Button`s, no title | Calculator |

Known differences the first run will show: the kit draws Inter or Adwaita Sans
and macOS draws SF Pro (the matrix puts 9.6% to 18.4% of the pixels of the
time and date-time pickers down to it); the kit's goldens come from
`freya-testing` in the Linux VM with the nix fonts, and the Mac would draw
with CoreText and Metal.

## Blur: three proposals

The kit says the compositor composes every blur (ADR 0091, 0092, 0100) and
the protocol's `material` values are the 14 `NSVisualEffectView.Material`
values plus the two `NSGlassEffectView` styles. On macOS the compositor is
WindowServer, so the protocol maps onto native views. Nothing below is
implemented or measured on a kit window.

| Proposal | What it does | For | Against |
|---|---|---|---|
| P1 native views (recommended) | Each declared backdrop becomes an `NSVisualEffectView` or `NSGlassEffectView` next to the Metal layer. A chrome sub-surface is a second `CAMetalLayer` view above it. The scroll edge is an `NSScrollPocket` | identical to native; Reduce Transparency and Increase Contrast come from the system; no Skia blur, as ADR 0100 decided; Liquid Glass lens and merging (`NSGlassEffectContainerView`) come free | untested: whether `withinWindow` vibrancy samples a Metal layer below it; soft edges and rounded shapes need a mask; `NSGlassEffectView` needs macOS 26 |
| P2 Skia blur on macOS | The kit blurs its own pixels again | works on any macOS version | reverts the owner's decision of 2026-11-22; cannot blur what lies behind the window, so no translucent sidebar; two places for Reduce Transparency |
| P3 native with a Skia residue | P1, plus Skia only where a native view cannot serve (a material per control, `ShapeStyle::Material(Thickness)`) | covers the one case ADR 0100 left blocked | the residue is the code ADR 0100 wants gone |
| Baseline | The opaque solid of Reduce Transparency, which ADR 0092 already defines | no work; ships in M0 | no translucency |

Decided by the owner on 2026-11-22: P1. M0 still ships the baseline.

First experiment for P1 (E8, 2026-11-22, `~/src/probes/metalblur.swift`,
captures in `~/src/captures/metalblur/`): a borderless window with a
`CAMetalLayer` view of 4 pt stripes, an `NSVisualEffectView` (`hudWindow`,
`withinWindow`, active) over x 100 to 300 pt, and a transparent `CAMetalLayer`
view above with an opaque red square and a half-transparent green one. Dark
appearance, scale 1, `screencapture -l`, read through the capture's ICC
profile.

| Measure | Result |
|---|---|
| Stripes under the effect view | flat gray 93 over the whole region: the effect samples the Metal layer below it |
| Stripes outside it | 0 and 255, untouched; the edge is hard (0 at x 99, 93 at x 100) |
| Opaque red square above | 255, 0, 0 |
| Half-transparent green square above | 85, 175, 72, the same as over a plain sRGB gray plate of 93 (control run), so the chrome composites over an effect view as over any layer |

So P1's layering works for this case: Metal content below, an effect view,
Metal chrome above. Still not measured: a scroll that follows without a frame
of lag (the layers present with a transaction or not), a rounded or soft mask,
`behindWindow`, `NSGlassEffectView` and its container, scale 2, light and
Increase Contrast, and the other 13 materials.

Other things macOS draws that the kit does not, with the proposal:

| Item | State | Proposal |
|---|---|---|
| SF Pro | the kit draws Inter or Adwaita Sans | on macOS resolve `system-ui` through the CoreText font manager (`fonts.rs` already sets `FontMgr::default()` as the default) and never bundle SF; not verified that the family list resolves it |
| Vibrancy of labels | `partial` in the matrix | `NSVisualEffectView` vibrancy needs the label in the effect view's subtree; propose it after P1 |
| Window shadow and corner | measured, CSD only | free in SSD and hybrid |
| Symbol effects, variable symbols | missing | not part of the port |

## Decisions for the owner

1. ~~The runner~~ decided: A, own `objc2` (2026-11-22).
2. The default window mode on macOS: SSD or hybrid.
3. ~~The blur~~ decided: P1 (2026-11-22). Open: when P3's Skia residue is
   admitted, if ever.
4. Whether macOS reads the live system appearance, or only `appearance.domain`.
5. Whether the goldens stay the Linux ones (`freya-testing`, nix fonts) and
   the Mac bench is a second report, or the Mac becomes a source of goldens.
6. Whether `agents/prompt.md` changes now («Linux on Wayland») or when M1
   lands (`AGENTS.md`, rule 3 of «Documentación»).

## Not verified

- That the 42,000 lines outside `platform/` and `shell/` compile on macOS.
- That Skia with Metal draws a kit scene: E4 compiles and links only.
- Everything under [Blur](#blur-three-proposals) beyond E8's one case.
- The 5,000-line estimate.
