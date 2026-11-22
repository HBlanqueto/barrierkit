# Compositor-composed backdrop: protocol, phases and GPU runbook

**Goal:** declare how a BarrierKit app asks for a material and Malaysia composes
every blur, inside the window too, so the kit can delete its Skia backdrop
blur.
**Scope:** the protocol that landed in `infinity-protocol` (version 10), the
rules a compositor follows, the compositor algorithm, the kit changes, the
phase 0 results, the GPU measurement that decides whether the in-window blur
ships on by default, the text of the ADRs
and the open questions. It starts at the decision in
[`blur-ownership.md`](blur-ownership.md) and ends at the open questions. It
adds no kit or compositor rendering code.
**Reader:** whoever implements a phase, runs the GPU measurement or pastes the
Malaysia ADR.
**Input:** the decision of 2026-11-21 (all blur is the compositor's), the macOS
measurements in [`blur-ownership.md`](blur-ownership.md), the phase 0
experiment in [`blur-phase-0/`](blur-phase-0/README.md) and the code of both
repos.
**Output:** a protocol, an algorithm, a phase table with its status, the phase 0
results with their labels, a GPU runbook with a pass criterion, three ADR
texts and the questions still open.

## Status

| What | State on 2026-11-22 |
|---|---|
| Decision | Taken by the owner: the compositor composes all blur; the kit drops its Skia in-window blur |
| Requirements | The owner's, in [the Requirements section](#requirements) |
| Phase 0 | Done, in software. The lag test passes; the frame time on a GPU is **NOT measured** |
| Phase 1 | Done: `infinity_window_style_v1` version 10, [ADR 0091](../decisions.md) and [ADR 0092](../decisions.md), the text for Malaysia in [section 8.3](#83-malaysias-adr-ready-to-paste) |
| Phase 2 | Started on 2026-11-22 as M2, before the GPU frame time is measured. The criterion of [section 10](#10-gpu-runbook-the-frame-time-of-the-band-pass) decides whether it ships on by default |
| Order of execution | M1 done (Malaysia `df9efc81`); M2 and K3 in parallel; K4 last ([section 7.1](#71-the-phases)). The owner decided on 2026-11-22 that the kit's Skia blur is deleted at the end, that Malaysia applies the measured values in every shell component, and that M2 and K3 start before the GPU measurement |
| Algorithm | M1 replaced Dual Kawase with the measured Gaussian pipeline. The phase 0 timings and the runbook of section 10 are of Dual Kawase and must be redone on it |
| Verification | The VM (container, llvmpipe, no display) cannot verify K3 and K4 end to end: a real session on the owner's laptop is needed |
| Blur values per material | Not in this document. They come from [`macos-materials.md`](macos-materials.md), [`macos-blur.md`](macos-blur.md) and [`macos-liquid-glass.md`](macos-liquid-glass.md), which the three sessions of [Who measures what](#who-measures-what) are measuring |
| Other choices of phase 1 | Pending the owner's confirmation ([ADR 0091](../decisions.md)) |
| Does version 10 carry all of macOS's configuration? | To review when the measurements end ([section 9.3](#93-to-review-when-the-measurements-end)) |

## Requirements

The owner's requirements for every blur, from 2026-11-21 and 2026-11-22. The
rest of this document serves them.

1. **Identical to macOS 27.2.1**, on the desktop and in windows.
   - Desktop: the menu bar, menus, the Dock, Control Center, Notification
     Center, banners, Spotlight, Mission Control, Launchpad, the lock screen
     and every other surface of the shell.
   - Windows: the 14 materials of `NSVisualEffectView` in `withinWindow` and
     `behindWindow`, sidebars, toolbars, title bars, sheets, popovers, HUD
     windows, tooltips, scroll edge effects and Liquid Glass
     (`NSGlassEffectView`, SwiftUI `glassEffect`).
2. **Exact values, measured, with provenance.** Each value comes from a
   measurement and records its background, appearance, state, scale, date and
   method. "Identical" is shown by comparing a capture of macOS with the
   model's output over the same background. A value not measured is written
   "not measured"; it is never invented and never copied from another surface.
   A reading taken with `~/src/probes/sample` before its fix on 2026-11-21 has
   shifted mid-tones (it read the raw bytes as Generic RGB with gamma 1.8), so
   such a value is read again from the raw PNG.
3. **A 1:1 replica of how it works and how it is configured.** The compositor
   composes as macOS does, and the protocol exposes the same configuration
   ([section 9.3](#93-to-review-when-the-measurements-end)).
4. **Malaysia's algorithm is redone where the measurements demand it.** The
   Kawase passes and the per-surface veils of today are a starting point. If a
   measurement shows macOS blurs, saturates or tints differently, the
   compositor changes. The GPU criterion of section 10 measures the algorithm
   of phase 0; if the algorithm changes, the measurement is repeated.
5. **Everything documented, in English**, dated, in the commit that changes it:
   each measurement, decision, API and token.

### Who measures what

| Area | Measured by | Document | Covers |
|---|---|---|---|
| Materials in windows | session `blur-macos-ventanas` | [`macos-materials.md`](macos-materials.md) | the 14 `NSVisualEffectView` materials in both blending modes, light and dark, active and inactive; sidebars, toolbars, title bars, sheets, popovers, HUD windows and tooltips. Tokens: the `material` group |
| Desktop surfaces | session `blur-macos-escritorio` | [`macos-blur.md`](macos-blur.md) | the menu bar, menus, Dock, Control Center, Notification Center, banners, Spotlight, Mission Control, Launchpad and the lock screen, in light and dark. Tokens: the desktop surfaces' |
| Liquid Glass | session `blur-liquid-glass` | [`macos-liquid-glass.md`](macos-liquid-glass.md) | the lens, refraction, highlights, shadows, adaptation, interaction and merging of a container, in SwiftUI and AppKit. Tokens: the `liquid-glass` group. Before it, only the blur extent of `NSGlassEffectView` regular and clear was measured ([`blur-ownership.md`](blur-ownership.md#14-what-the-app-sets-and-what-the-system-decides)) |

All three also measure the algorithm (the kernel, the color space and the
order of the filters) and the configuration, and say whether Malaysia's Dual
Kawase reproduces what they measured (requirement 4).

Their tokens go to Tengen (`crates/ctdesign/tengen/components.json`, read by
`ctdesign/build.rs`, [ADR 0032 · 0046](../decisions.md)), never to the
compositor or to `bk`. Each session names its tokens under the glossary rules
and adds them to [the glossary](../barrierkit/20-glossary.md) in its own
commit. The windows session adds the per-material table that section 5, step
3 reads; the desktop session refits the shell values that exist today
(`background-blur`, and `ctappearance`'s `menu_glass` and `dock_surface`). This
document and the protocol carry no value.

## Steps

1. Read the requirements.
2. Read the decision and its consequences in section 1.
3. Read the protocol in section 3 and the rules in section 4.
4. Read the phase 0 results in section 7.2 and the phase table in section 7.1.
5. Run section 10 on a Linux machine with a GPU. Record the result in section 7.2.
6. Session M1 pasted section 8.3 into Malaysia's `docs/decisions.md` with the
   pin (`df9efc81`). Session M2 implements section 5 and session K3 makes the
   kit declare the material, both before the GPU measurement.
7. When the measurement sessions report, review section 9.3.

---

## 1. The decision

The compositor composes every blur. The app declares a material and a style
and draws no blur. This is the split macOS has
([`blur-ownership.md`](blur-ownership.md#12-what-the-three-images-show)).

Consequences:

1. The kit deletes the Skia backdrop blur of `crates/bk/src/glass.rs` once
   the compositor path works (phase 4). Until then the kit draws it and its
   chapters say so.
2. Without the compositor path (another compositor, or a Malaysia that
   advertises version 9), the kit draws the solid it already draws under
   Reduce Transparency (`FullscreenBar::Solid`, the `solid` of `Material`).
3. Kit goldens and doc shots cannot show compositor blur. The tests of this
   blur live in Malaysia, on its headless renderer.
4. Malaysia advertises version 10 of `infinity_window_style_v1` only when it
   serves the new requests (section 3, "Versions").

## 2. What the kit declares

| Kit element | AppKit and SwiftUI | Request |
|---|---|---|
| `FullscreenBar::Glass`, a rail's glass band | `ScrollEdgeEffectStyle.hard` | `set_scroll_edge` with `hard` |
| `FullscreenBar::ProgressiveGlass`, `TopGlass` | `ScrollEdgeEffectStyle.soft` | `set_scroll_edge` with `soft` |
| `Material` | `Material`, `NSVisualEffectView` with `withinWindow` | `set_backdrop` with `within_window` |
| `GlassContainer`, `RoundButton` | `NSGlassEffectView` | later phase: merging needs its own design |
| Layer surfaces (menu bar, Dock) | `NSVisualEffectView` with `behindWindow` | `ext-background-effect-v1` today; `set_backdrop` with `behind_window` when a per-material radius is wanted |

No public kit name changes
([glossary](../barrierkit/20-glossary.md)). The kit enum for the edge style
would be `ScrollEdgeEffectStyle` with `Automatic`, `Soft` and `Hard`, the
SwiftUI name first (AGENTS.md, "Nombres y glosario", rule 2). Which
`NSVisualEffectView.Material` each `styles::Thickness` takes is an open
question (section 9).

## 3. The protocol

Version 10 of `infinity_window_style_v1` in
[`infinity-window-style-v1.xml`](../../crates/infinity-protocol/protocols/infinity-window-style-v1.xml).
The XML is the only statement of each request; this section lists what it
holds and what changed from the first draft.

| Item | Values or arguments | Name it follows |
|---|---|---|
| enum `blending_mode` | `behind_window`, `within_window` | `NSVisualEffectView.BlendingMode` |
| enum `material` | `titlebar`, `selection`, `menu`, `popover`, `sidebar`, `header_view`, `sheet`, `window_background`, `hud_window`, `full_screen_ui`, `tool_tip`, `content_background`, `under_window_background`, `under_page_background`, `glass_regular`, `glass_clear` | the 14 non-deprecated values of `NSVisualEffectView.Material`, in the order of the header, then `NSGlassEffectView.Style` |
| enum `edge` | `top`, `leading`, `bottom`, `trailing` | SwiftUI `Edge` |
| enum `scroll_edge_effect_style` | `automatic`, `soft`, `hard` | SwiftUI `ScrollEdgeEffectStyle` |
| request `set_backdrop` | `surface`, `blending`, `material`, `radius`, `rects` | `NSVisualEffectView` |
| request `set_scroll_edge` | `surface`, `edge`, `style`, `depth` | `scrollEdgeEffectStyle(_:for:)` |

The materials come from the macOS 27.2 SDK:
`/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/Frameworks/AppKit.framework/Headers/NSVisualEffectView.h`
(`NSVisualEffectMaterial`; the five values it deprecates, `appearanceBased`,
`light`, `dark`, `mediumLight` and `ultraDark`, are left out) and
`.../NSGlassEffectView.h` (`NSGlassEffectViewStyle`: `regular`, `clear`). The
values on the wire are the protocol's own, 0 to 15, not the SDK's raw
numbers. What each material looks like stays in the compositor and in
`ctdesign`; nothing in the XML or here gives a radius, a saturation or a tint.

What changed from the first draft of this document:

1. The enums sit inside the interface, as the existing ones do, and follow the
   AppKit and SwiftUI names: `blending_mode` (was `blending`) and
   `scroll_edge_effect_style` (was `scroll_edge_style`).
2. `material` lists all 14 semantic materials of the header (the draft had 10:
   `selection`, `window_background`, `content_background` and
   `under_page_background` were missing) and the two glass styles.
3. `edge` is physical: `leading` is the left side and `trailing` the right side
   whatever the layout direction, because the compositor cannot know it (right
   to left is decision D2 of the studies, still open).
4. `within_window` is a request of its own with an explicit blending, not a
   region on the root surface (phase 0: the root's region feeds Malaysia's
   wallpaper blur, `blur.rs:491`). Rule 8 of section 4 says what wins if a
   surface sends both.
5. The request applies with the parent's commit on a synchronized
   sub-surface, and tears on a desynchronized one (rule 3).
6. A window capture includes `within_window` material and not `behind_window`
   material (rule 9).
7. The blur radius is in logical pixels at every output scale (rule 5).
8. A value the version does not define makes the request ignored. There is no
   error enum.
9. The two requests are served for any surface in the tree of a toplevel of a
   BarrierKit client, not only for the toplevel's own surface, because the
   requester is a sub-surface.

### Versions

A request newer than the version a client bound is a protocol error, not an
ignored message (`wayland-backend` 0.3.17, `src/rs/server_impl/client.rs:580`).
So each pairing behaves as follows:

| Compositor advertises | Kit | Result |
|---|---|---|
| Version 9 (Malaysia today) | binds `1..=10` | binds 9, `version() >= 10` is false, the kit draws the solid and sends nothing |
| Version 10 | binds `1..=9` (the kit of today) | binds 9; every old request works as before |
| Version 10 | binds `1..=10` | sends `set_backdrop` and `set_scroll_edge` |
| No global (KWin, Mutter, Hyprland...) | any | the kit draws the solid |

The new requests come after `set_knobs`, so every old opcode is unchanged. The
test `crates/infinity-protocol/tests/window_style.rs` fixes the opcode and
`since` of all twelve requests and the signatures of the two new ones.

Malaysia's `window_style.rs:259` has `_ => {}`, so a Malaysia built at the new
kit rev compiles and ignores the two requests. It must keep advertising 9
(`window_style.rs:99`) until it serves them: a version 10 global with a match
that ignores the requests would leave the transparent rectangle empty.

`ext-background-effect-v1` stays for every other client (GTK, Qt, terminals).

## 4. Rules

1. **Below.** "Below the surface" means every surface of the same toplevel
   that its tree draws before it: its parent, the siblings placed under it
   and their children. A sub-surface above its parent, a sub-surface below
   it and a root surface with a sub-surface placed under it all work
   (phase 0, section 7.2); the rule needs no knowledge of which one the kit
   chooses.
2. **Within the window only.** `within_window` never samples another toplevel
   or the wallpaper, as Apple documents `withinWindow`. What it shows where
   the toplevel draws nothing below the rectangle is not measured on macOS
   (section 9).
3. **Same commit.** The request is double-buffered. On a synchronized
   sub-surface it applies with its parent's commit, so material and content
   change in one frame. On a desynchronized sub-surface it applies at the
   sub-surface's own commit and a frame can show the new region over the old
   content (phase 0: 23 of 23 mid-frames torn).
4. **Root surface.** `within_window` on a surface with nothing below it has no
   effect. A root with a sub-surface placed under it has something below it.
5. **Compositor decides.** Blur radius, saturation, tint, lens and Reduce
   Transparency and Increase Contrast come from `ctappearance` and `ctdesign`,
   as for the menus. The measured extent of each material is in
   [`blur-ownership.md`](blur-ownership.md#14-what-the-app-sets-and-what-the-system-decides).
   The radius is in logical pixels: the Kawase offset and passes of
   `blur.rs` are in physical pixels, so at 2x the compositor widens them to
   reach the same logical distance (phase 0 measured the halving).
6. **No global, no effect.** A client that sees a version below 10 draws the
   solid fallback.
7. **Only BarrierKit clients.** As `set_glass` and `set_knobs`, the compositor
   ignores the requests from other clients, and from surfaces outside the tree
   of a BarrierKit toplevel.
8. **One request per surface.** A surface that sends `set_backdrop` and
   `ext_background_effect_surface_v1.set_blur_region` gets the Infinity one.
   The in-window request never feeds the wallpaper path that reads the root's
   region. The region of a sub-surface stays ignored.
9. **Capture.** A capture of the toplevel alone includes `within_window`
   material and excludes `behind_window` material, as `screencapture -l` does
   ([`blur-ownership.md`](blur-ownership.md#12-what-the-three-images-show),
   points 2 and 3).

## 5. Compositor algorithm

All in Malaysia. "Tested" means the phase 0 patch runs it in software
([section 7.2](#72-phase-0-results)); the rest is not written.

Since M1 (`df9efc81`) the blur pass is the measured Gaussian pipeline of
`blur/gaussian.rs`: a box reduction by 2, 4 or 8, a Gaussian of the remaining
sigma, renormalised at the region's border. The Kawase offsets, passes and
reach in step 3 are phase 0's; M2 takes the reach from `reach_in_pixels` in
`gaussian.rs`.

1. Store the requests per `wl_surface` in a `Cacheable` state, as
   `GlassPanels` and `Knobs` do in `window_style.rs`. Smithay applies a
   sub-surface's cached state in the parent's transaction. [tested, with the
   state of `ext-background-effect`]
2. In the window path of `scene_elements.rs`, where
   `render_elements_from_surface_tree` builds `surface_elements`, split the
   list at the element whose `Id` is `Id::from_wayland_resource(requester)`.
   The requester and what is above it stay in front. The rest is the source.
   [tested]
3. Render the source of each rectangle, expanded by the blur reach, into an
   offscreen texture with an `OutputDamageTracker`, as `render_behind` does,
   and run the Kawase passes of `blur.rs`. The reach is
   `offset * 2^passes`: 3 * 8 = 24 physical pixels at `BACKGROUND_BLUR_PASSES`
   3 and `BACKGROUND_BLUR_OFFSET` 3.0 (margin check in section 7.2, finding 4).
   Take passes and
   offset per material from a `ctdesign` table, widened at 2x (rule 5).
   [tested for the first rectangle with the wallpaper values; the per-material
   table and the 2x compensation are not written]
4. For `soft`, blend the sharp source with the blurred levels by a mask along
   the edge, as the kit does today with `layer_mask`; for `hard`, use one
   level and draw the edge. [not tested]
5. Draw the result as a `RoundedTextureElement` between the two halves of the
   list. [tested with radius 0]
6. The tracker invalidates the cache when the source's damage touches the
   rectangle plus the reach. A scrolling window therefore blurs every frame.
   In every phase 0 configuration the client damaged the whole parent; a
   band-only damage case was not run. [partly tested]
7. When `drawn(state)` is false (Reduce Transparency, Increase Contrast),
   draw the material's solid instead, as the Dock does with `dock_solid`.
   [not tested]
8. Window capture (`capture.rs:281-283`, `capture_window_png` at
   `capture.rs:592`) builds its elements with `build_window_elements`
   (`scene_elements.rs:1246-1263`), a bare `render_elements_from_surface_tree`
   with no compositor blur. Run the split of step 2 there for `within_window`
   material (rule 9). [measured: the capture band deviates 0.0 from the sharp
   content, the split path 83.7]
9. Several rectangles of one request, rounded corners and the Reduce
   Transparency gate are not tested: `split` uses the first rectangle only.

The headless tests of `blur.rs` (`try_test_renderer`, `solid` elements) are
the pattern for the tests: a red element below a rectangle must tint it, a
blue one beside it must not. The phase 0 tests are the model for the lag test:
[`compositor-within.patch`](blur-phase-0/compositor-within.patch).

## 6. Kit changes

1. Two surfaces per window that has in-window glass: the content and the
   chrome. The chrome surface covers only the strip (the bar and the depth of its
   glass, or a rail), so its raster cost stays small.
2. `RenderPipeline` of Freya 0.5.3 loops over `tree.layers` and draws into
   one canvas. The kit draws layers below a split into the content canvas and
   the rest into the chrome canvas. The titlebar and toolbar must sit on the
   higher layers.
3. A second swapchain and Skia surface per window (`platform/surface.rs`,
   `vulkan.rs`), a `wl_subcompositor` binding, a synchronized sub-surface.
4. Input and focus across the two surfaces: pointer enter and leave, scroll,
   cursor shape, IME and xdg popups all map to one Freya app.
5. The bind range of `infinity_window_style_v1` in `platform/runtime.rs`
   goes from `1..=9` to `1..=10`, a `BACKDROP_VERSION` of 10 beside
   `KNOB_VERSION`, and an outbox for the requests like `GlassOutbox`, flushed
   before the commit (`flush_glass` is the model).
6. `glass.rs` loses the `Glass` and `MaterialElement` backdrop code once the
   phases below pass. `05-controls.md`, `09-titlebars.md`,
   `15-backdrops-and-media.md` and `18-platform.md` change in that commit.

## 7. Phases and the first experiment

### 7.1 The phases

| Phase | Output | Sessions | Status on 2026-11-21 |
|---|---|---|---|
| 0 | The experiment of section 7.2 | 1 to 2 | Done, in software. GPU frame time **NOT measured** |
| 1 | Protocol version 10 in `infinity-protocol`, the ADRs | 1 to 2 | Done |
| 2 | Compositor: state, split, blur, mask, gate, capture, tests | 3 to 5 | Started on 2026-11-22 (M2), before the GPU frame time is measured. The criterion of section 10 decides whether it ships on by default |
| 3 | Kit: sub-surface, layer split, input, outbox | 4 to 6 | Started on 2026-11-22 (K3); keeps Skia as the fallback |
| 4 | Migrate the four elements, delete the Skia blur, update docs | 2 to 3 | Not started |

If the GPU frame time fails the criterion of section 10, the in-window path
does not ship on by default: the work of M2 and K3 may be wasted, a risk the
owner accepted on 2026-11-22, and K4 waits for the criterion to pass because it
removes the fallback. M1 stays: it serves surfaces that already blur.

#### Order of execution

Decided by the owner on 2026-11-22 ([ADR 0091](../decisions.md)): once the
measurements end, Malaysia applies the measured values in every shell
component, and the kit's own Skia backdrop blur is deleted at the end.

| Step | Phase | Owner | Output | Condition |
|---|---|---|---|---|
| M1 | between 1 and 2 | session `malaysia-blur-nucleo`, in Malaysia's repository (done: `df9efc81`) | the pin; Malaysia's ADR pasted (section 8.3); the measured algorithm in `grade-graphics` and `blur.rs`; the measured tokens wired to every shell surface that blurs today; tests on software EGL. It does not raise `VERSION` from 9 | the measurements of "Who measures what" |
| M2 | 2 | session `malaysia-backdrop-m2`, in Malaysia's repository | `set_backdrop` and `set_scroll_edge` in the compositor, and the split of the surface tree (section 5) | the GPU criterion of section 10 |
| K3 | 3 | session `kit-backdrop-k3` | the kit declares the material and the style when the compositor advertises version 10: the sub-surface, the split of Freya's layers, input and the outbox. It keeps Skia as the fallback and does not delete it | the GPU criterion, and M2 serving the requests |
| K4 | 4 | a kit session | the Skia backdrop blur of `crates/bk/src/glass.rs` is deleted | last: only when the compositor serves the requests and a real session shows no regression |

1. M2 and K3 run in parallel and start before the GPU frame time is measured
   (the owner's decision of 2026-11-22). The owner accepts the risk that the
   work is wasted if the criterion fails; the criterion still decides whether
   the in-window path ships on by default. M1 did not depend on it: the
   surfaces it serves already blur.
2. M1 changed the algorithm (requirement 4): Dual Kawase gave way to the
   measured Gaussian pipeline of `blur/gaussian.rs` (`df9efc81`). The phase 0
   timings and the runbook of section 10 are of Dual Kawase, so the measurement
   is redone on the new pipeline, with a band case that M2 adds: M1's own timing
   test (`blur/timing.rs`) covers the whole output, the Dock and a menu, not a
   96 px band.
3. Until K4 the kit keeps its Skia blur as the fallback when the compositor
   does not advertise version 10. After K4 it draws the solid
   ([ADR 0092](../decisions.md)).
4. Tests of M1 and M2 are headless and run on software EGL in the VM. The VM
   cannot verify K3 and K4 end to end (input across two surfaces, a real scroll,
   real presentation): a real session on the owner's laptop must.

### 7.2 Phase 0 results

Run on 2026-11-21 in a scratch copy of Malaysia `1a45e115` (Smithay `65aa86f`),
inside a Linux container on llvmpipe (Mesa 26.2.5, OpenGL ES 3.2, aarch64 VM,
6 CPUs). Code, scripts and the output:
[`blur-phase-0/`](blur-phase-0/README.md). Nothing was committed to Malaysia.
The blur pass it measured is Dual Kawase, which M1 replaced on 2026-11-22
([section 7.1](#71-the-phases)).
Labels: **[software]** ran on llvmpipe; **[code]** read in the source;
**[not run]** the experiment did not run it.

The harness is a lockstep stand-in for a nested session: a Wayland client over
a socket pair, no xdg toplevel, no frame callbacks, no presentation clock. The
client makes an opaque parent with stripes that scroll 7 px per frame, a
synchronized sub-surface with a transparent band and one opaque mark, and a
blur region over a 96 px band. Per frame it commits the sub-surface, then the
parent, and the compositor renders after each message and compares pixels.

**The five answers**

| Question | Answer | Label |
|---|---|---|
| Does the blur follow the scrolling without a frame of lag, with the sub-surface committed in sync mode? | Yes. In six configurations (sub-surface above or below the parent; strip fixed or window-sized with a moving band; 1x and 2x), 24 frames each: no pixel outside the band differs, the mark stays sharp (24 of 24), the band equals a from-scratch render (24 of 24), a frame with age 1 equals a full repaint, and between the sub-surface's commit and the parent's the output equals the previous frame (23 of 23). The negative control, a desynchronized sub-surface, tears 23 of 23 mid-frames | [software] |
| What is the frame time of the blur pass at 1x and 2x on the user's GPU, with the band of `FULLSCREEN_GLASS` (96 px)? | **NOT measured.** On llvmpipe: 3.57 ms at 1x and 9.11 ms at 2x (table below). They are not GPU numbers and are not evidence for or against the criterion of section 10 | [software] |
| Does a window capture include the blur? | No, not today. The capture path builds its elements without the blur: the band deviates 0.0 from the sharp stripes, against 83.7 on the split path. It needs the same split. `./dev.sh screencopy-window` was not run (it needs a nested compositor on a display) | [software] for the deviation, [not run] for the probe |
| Does Smithay deliver the sub-surface's `BackgroundEffectSurfaceCachedState` with the parent's commit? | Yes. `Cacheable::commit` clones it (`background_effect/mod.rs:73-91`), `MultiCache::commit` walks every cached type of the surface (`cache.rs:206-212`), `PrivateSurfaceData::commit` merges the pending transaction of every effectively synchronized child into the parent's (`tree.rs:281-312`) and `Transaction::apply` applies all cached types, children first and the parent last, inside one dispatch (`transaction.rs:261-278`) | [code], and the 24-frame lag test above |
| Is a sub-surface below the parent as workable as one above it? | Yes for composing the blur: the Below results equal the Above ones. `place_below` reorders any entry of the parent's `children` against any other (`tree.rs:419-448`), and `render_elements_from_surface_tree` walks top to bottom (`mod.rs:351-357`). Input, `xdg_surface` geometry and hit testing of the Below layout were not exercised | [software], [code]; input [not run] |

**Frame time of the band pass** (source render, 3 down and 3 up Kawase passes
and `glFinish`; 1280x800 logical window, 96 logical rows; 40 frames; renderer
`llvmpipe (LLVM 21.1.8, 128 bits)`; output of
[`results.log`](blur-phase-0/results.log)). **[software, not a GPU]**

| Scale | Band source (physical) | Share of the output | Band pass median (min, max) | Share of a 60 Hz and a 120 Hz frame | Full-output Kawase of the same content (the shipping wallpaper pass) |
|---|---|---|---|---|---|
| 1x | 1280x120 | 15 % | 3.57 ms (3.29, 3.79) | 21.4 % and 42.8 % | 16.16 ms |
| 2x | 2560x216 | 13.5 % | 9.11 ms (8.85, 10.58) | 54.6 % and 109.3 % | 59.67 ms |
| GPU | | | **NOT measured** | | |

The other five runs of the same code (four of the first phase 0 session, one
rerun from this repository) gave medians of 3.45 to 3.64 ms at 1x and 9.12 to
9.37 ms at 2x.

**Other findings**

1. **The root's region feeds the wallpaper blur.** The window path reads
   `blur_region(window.surface())` of the root (`blur.rs:491`; for layers,
   `scene_elements.rs:1410`) and draws the wallpaper blur under all of the
   window's surfaces. With the chrome as the root and the content as a
   sub-surface placed below (Below), a region on the root would meet that
   path. With the chrome as a sub-surface above (Above), the root has no
   region and a sub-surface's region is ignored today, so there is no clash.
   The explicit `within_window` request of section 3 removes the clash for
   both layouts. [code]
2. **Scale.** The Kawase offsets and steps are in physical pixels, so at 2x the
   logical radius halves, as the wallpaper blur does today. Rule 5 makes the
   compositor compensate. [software]
3. **Software EGL.** Inside the nix shell libglvnd finds no vendor file, so
   `eglQueryDevicesEXT` returns no device and the EGL tests return early.
   `__EGL_VENDOR_LIBRARY_DIRS=<mesa>/share/glvnd/egl_vendor.d` is enough for
   Mesa to expose `EGL_MESA_device_software` (llvmpipe). With it Malaysia's
   288 library tests and 15 binary tests pass in three of three runs. In one of
   six earlier runs, with the experiment's CPU-heavy tests in the same
   four-thread batch, the existing
   `session::tests::a_failed_create_immed_is_drained_and_posts_the_protocol_error`
   failed (`session/tests.rs:694`); it did not fail on the pristine tree in six
   runs. The cause was not proven. [software]
4. **Reach margin.** The band blurred from a source grown by 24 physical pixels
   against one grown by 120 pixels differs by at most 1/255 in a channel at 1x
   with a fixed strip, 4/255 at 1x with a moving band and 12/255 at 2x.
   [software]

**What phase 0 did not run:** anything on a GPU; a nested session on a
display; the wiring into `append_window_frame` and `clip_to_frame`
(`scene_elements.rs:1053-1085`) and into `capture.rs`; the Reduce
Transparency and Increase Contrast gate; rounded corners and the soft mask of
section 5, step 4; several rectangles; a band-only damage case.

## 8. ADR texts

### 8.1 The kit ADRs

They are [ADR 0091](../decisions.md) (the compositor composes every blur: the
kit side of Malaysia's «Blur and wallpaper tinting») and
[ADR 0092](../decisions.md) (in-window glass is a request), in the kit's
`docs/decisions.md`.

### 8.2 What stays in each repo

ADR 0091 holds the decision that both repos share. ADR 0092 holds the kit's
consequences. Malaysia's entry below holds the compositor's.

### 8.3 Malaysia's ADR, ready to paste

For `docs/decisions.md` of Malaysia. It replaces the whole entry
"0020 · 0038 · 0065 · 0071 — Blur and wallpaper tinting". The bullets on the
compositor composing every blur, the Infinity requests, the capture, the scale
and the version are new; the old second bullet (a surface gets blur only in
the region it sends) is replaced by the third and fourth; the first bullet has
one added sentence; "Why", "Rejected", "Pending" and "Code" changed. The
bullets on the cached wallpaper blur, the shaders and the wallpaper tinting are
the current ones. Session M1 pasted it with the pin (`df9efc81`) and edited it
there to the measured pipeline: Malaysia's `docs/decisions.md` is the source
from now on, and this block is the text as first written. Its lines on the
Infinity requests, the in-window blur and the capture state the target: they
hold once M2 serves the requests, and the compositor advertises version 9 until
then.

```markdown
## 0020 · 0038 · 0065 · 0071 · 0091 — Blur and wallpaper tinting
Decision:
- Blur is drawn only while `appearance.reduce_transparency` and `appearance.contrast` are `false` (both default `false`), except the glass of kit menus: it survives `reduce_transparency`, as on macOS, and stops only with `contrast`. `ext_background_effect_manager_v1` advertises `blur` while that glass is drawn. With either setting on, a BarrierKit material is drawn as its solid.
- The compositor composes every blur: behind a window, behind a layer surface and inside a BarrierKit window. A client declares a material and never draws a backdrop blur.
- Behind a window, a surface gets blur in the region it sends through `ext-background-effect-v1`. A BarrierKit client may send `infinity_window_style_v1.set_backdrop` with `behind_window` instead, to get the radius of a material; that request wins over the region.
- Inside a window, a BarrierKit client sends `set_backdrop` with `within_window` or `set_scroll_edge` (`infinity_window_style_v1`, version 10) on the surface that draws over the content, and the compositor blurs what the toplevel's surfaces draw below it: the parent, the sub-surfaces under the requester and their children. It never reads the region of `ext-background-effect-v1` for this, so the wallpaper blur of the root surface is not touched.
- The request applies with the parent's commit on a synchronized sub-surface. A window capture includes `within_window` material and not `behind_window` material.
- The compositor decides radius, saturation, tint and lens per material and appearance from `ctdesign`. The radius is in logical pixels: at a larger output scale it widens the passes to reach the same distance.
- The compositor advertises version 10 of `infinity_window_style_v1` only while it serves these requests, and serves them only for BarrierKit clients.
- By default the blur shows the `Background` layer, blurred once per output into a cache. The Dock, notification cards and glass popups blur what is really behind them, each from its own cached render.
- The shaders are Dual Kawase in `grade-graphics`. `ctdesign::BACKGROUND_BLUR_PASSES` and `BACKGROUND_BLUR_OFFSET` set the strength of the wallpaper blur; no user option does.
- `appearance.wallpaper_tinting` (default `true`, dark mode only unless `wallpaper_tinting_in_light`): the compositor averages the wallpaper grid cells behind each window and sends the color in the `wallpaper` event of `infinity_focus_surface_v1`. The kit, the patched libadwaita and the compositor's titlebar mix it in.
Why: macOS composes every blur outside the app, in both blending modes, and the app declares the material; one place then decides radius and accessibility. No client sees other clients' pixels, so only the compositor can blur or sample what is behind a window, and it already samples a client's own pixels for `set_knobs`. The [protocol](https://wayland.app/protocols/ext-background-effect-v1) leaves strength to the compositor and has no material. `set_glass`, `set_plates` and `set_knobs` already paint for clients.
Rejected: blurring every window by default (GPU cost); a blur drawn by the client as the shipping path (two places for Reduce Transparency, two blur algorithms); `ext-background-effect-v1` on sub-surfaces (KWin, Hyprland and niri ignore their regions); the compositor painting client backgrounds the client did not ask it to paint.
Pending: the frame time of the in-window pass on a real GPU, which is not measured and decides whether the in-window path ships on by default (the owner decided on 2026-11-22: median of at most 1.0 ms at 2x for a 96 px band to pass, over 2.5 ms to stop, no frame of 40 at 8.33 ms or more), and the blur values per material (being measured).
Code: `crates/malaysia-compositor/src/blur.rs`, `crates/malaysia-compositor/src/window_style.rs`, `crates/malaysia-compositor/src/scene_elements.rs`, `crates/malaysia-compositor/src/capture.rs`, `crates/malaysia-compositor/src/focus_notice.rs`
```

## 9. Questions

### 9.1 Resolved

1. **Sub-surface above or below the parent?** Both compose the same blur
   (section 7.2). The rule of section 4 needs no choice. Input and
   `xdg_surface` geometry still decide which the kit uses (9.2, item 1).
2. **Does a request on the root meet the wallpaper blur?** It would if it were
   `ext-background-effect-v1`. The explicit `within_window` request does not
   (rules 4 and 8).
3. **Does a window capture have the blur?** Not today; rule 9 and section 5,
   step 8 say what it must do.
4. **What happens to the radius at 2x?** The compositor widens the passes
   (rule 5).
5. **May an old compositor or an old kit meet the new requests?** Version
   negotiation keeps them apart (section 3, "Versions").

### 9.2 Open

1. Input, focus, cursor shape and `xdg_surface` geometry with the chrome as a
   sub-surface above, or as the root with the content below. The phase 0
   harness has no xdg toplevel.
2. Which `NSVisualEffectView.Material` does each `styles::Thickness` take? The
   extents in [`blur-ownership.md`](blur-ownership.md) give two groups (about
   35 px and about 48 px in light).
3. How does `GlassContainer` merge shapes in the compositor
   (`NSGlassEffectContainerView.spacing`)?
4. What does `automatic` resolve to? A soft and a hard
   `preferredScrollEdgeEffectStyle` looked the same in the probe. Until it is
   measured the kit sends `soft` or `hard` itself.
5. Should layer surfaces move from `ext-background-effect-v1` to
   `set_backdrop` with `behind_window`, to get a radius per material?
6. How do kit doc shots show compositor glass: captures of a nested Malaysia
   session, or none?
7. What does `within_window` show where the toplevel draws nothing below the
   rectangle? The probe's within tiles sat over a checker. Not measured.
8. Is the blur extent of macOS the same in points at 2x? The probes ran at
   scale 1. Not measured; rule 5 assumes it.
9. The frame time of the band pass on a real GPU. **NOT measured.** Section 10
   measures it.
10. Several rectangles, rounded corners, the soft mask, the Reduce
    Transparency gate and a band-only damage case were not run (section 7.2).
11. The blur values per material (radius, saturation, tint) are being measured
    in [`macos-materials.md`](macos-materials.md) and
    [`macos-blur.md`](macos-blur.md). The protocol needs none of them.
12. Does version 10 carry all of macOS's configuration? To review when the
    measurements end (section 9.3).

### 9.3 To review when the measurements end

Does version 10 carry all the configuration macOS has, or must it grow before
Malaysia serves it? Until Malaysia advertises version 10 nothing sends these
requests, so version 10 can still change in place; afterwards each addition is
a new version. The XML does not change now. When the three measurement
sessions of "Who measures what" finish, their reports complete this section.
Review each row against them: a setting is added only if the look of macOS
changes with it, and the measurement says so.

| macOS setting | Where | Version 10 |
|---|---|---|
| `material` | `NSVisualEffectView` | `set_backdrop`, `material` (14 values) |
| `blendingMode` | `NSVisualEffectView` | `set_backdrop`, `blending` |
| `state` (`followsWindowActiveState`, `active`, `inactive`) | `NSVisualEffectView` | not carried: the compositor follows the toplevel's active state |
| `isEmphasized` | `NSVisualEffectView` | not carried |
| `maskImage` | `NSVisualEffectView` | not carried: `rects` and `radius` give rounded rectangles only |
| `style` (`regular`, `clear`) | `NSGlassEffectView` | `material` values `glass_regular` and `glass_clear` |
| `cornerRadius` | `NSGlassEffectView` | `set_backdrop`, `radius` |
| `tintColor` | `NSGlassEffectView` | not carried |
| `effectIsInteractive` (macOS 27.0) | `NSGlassEffectView` | not carried |
| `spacing` | `NSGlassEffectContainerView` | not carried (open question 3) |
| `Glass.tint(_:)` and `.interactive()` of `glassEffect(_:in:)` | SwiftUI | not carried; `.regular` and `.clear` as `style`, `.identity` is no request |
| `glassEffectID`, `glassEffectUnion`, `glassEffectTransition`, `GlassEffectContainer(spacing:)` | SwiftUI | not carried |
| `scrollEdgeEffectStyle(_:for:)`, `scrollEdgeEffectHidden` | SwiftUI | `set_scroll_edge` (`edge`, `style`, `depth`); hidden is no request |

The AppKit properties are in `NSVisualEffectView.h` and `NSGlassEffectView.h`
of the macOS 27.2 SDK ([section 3](#3-the-protocol) names the folder); the
SwiftUI ones are in [`swiftui-api-gap.md`](swiftui-api-gap.md).

## 10. GPU runbook: the frame time of the band pass

**Goal:** measure the cost of one in-window blur band on a real GPU, the
criterion that decides whether the in-window path ships on by default.
**Status on 2026-11-22:** stale until M2 adds the band case. The steps below
measure the Dual Kawase pass of phase 0 against Malaysia `1a45e115`; M1 replaced
that pass (`df9efc81`) and the patch was not rebased. M1's test
`the_blur_passes_cost_this_much_per_frame` (`blur/timing.rs`, `#[ignore]`: run
it with `--ignored --nocapture --test-threads=1`) prints `BLUR TIMING` lines
with the renderer and the share of a 60 and a 120 Hz frame for the whole
output, the Dock and a menu at 1x and 2x, not for a 96 px band. The criterion
below is read from the band case M2 adds. The checkout, the dev shell and the
probe still apply with Malaysia at `df9efc81` or later and without the patch;
this was not tried.
**Scope:** two tests of the phase 0 patch on a Linux machine with a GPU. It
starts at a Malaysia checkout and ends at two `BLURP0 TIMING` lines and a
decision. It measures one band over one window, nothing else.
**Reader:** the owner, or whoever has the machine.
**Input:** a Linux x86_64 machine with a GPU and nix with flakes, a checkout of
this repository and network access. Malaysia's flake has `x86_64-linux`
outputs only. The commands were assembled from what ran in the container and
from Malaysia's `flake.nix`; they were **not run on a GPU**, so the first run
may need the change the probe points to.
**Output:** the two lines, the probe's first block, and the GPU, driver and
display; then the criterion below.

### Steps

1. Check that the user can open a render node: `ls -l /dev/dri/renderD*`. The
   path to a GPU is Mesa's: Malaysia's dev shell exports Mesa's vendor file
   (`flake.nix`, `shellHook`). A proprietary NVIDIA driver is not covered.
2. Get Malaysia at the base revision and apply the patch. `KIT` is this
   repository.

   ```sh
   KIT=$HOME/src/barrierkit
   git clone https://github.com/HBlanqueto/malaysia.git malaysia-blurp0
   cd malaysia-blurp0
   git checkout 1a45e1159039478d5c9e5dedc17d1844f91a497f
   patch -p1 < "$KIT/docs/research/blur-phase-0/compositor-within.patch"
   ```

3. Enter Malaysia's dev shell. The first time it builds Malaysia's patched
   libadwaita, which is slow.

   ```sh
   nix develop
   ```

   Run steps 4 and 5 inside it. Do not source `swegl.sh` and do not set `LIBGL_ALWAYS_SOFTWARE`. Those are
   the software run.
4. Check that the test will use the GPU:

   ```sh
   sh "$KIT/docs/research/blur-phase-0/egl-probe.sh"
   ```

   Read the block after `=== plain`. `eglQueryDevicesEXT ok=1 n=<N>` must have
   `N` of at least 1, and the first `GL_RENDERER=` line must name your GPU,
   not `llvmpipe`. The test uses the first device, and Mesa lists hardware
   devices before its software one (`src/egl/main/egldevice.c`,
   `_eglQueryDevicesEXT`). The other three blocks are software variants:
   ignore them. If `N` is 0, or the first renderer is `llvmpipe`, nothing below
   measures a GPU: run `unset __EGL_VENDOR_LIBRARY_DIRS LIBGL_DRIVERS_PATH` to
   use the system's driver and probe again (libglvnd looks in
   `/run/opengl-driver`, `/etc/glvnd/egl_vendor.d` and
   `/usr/share/glvnd/egl_vendor.d`; read from the `libEGL.so.1` of nix's
   libglvnd 1.7.0). On NixOS the first of them is the system's driver; on
   another distribution the host's driver library may not load inside nix's
   glibc. Neither was tried.
5. Run the two tests, on one thread, close to idle (no other GPU load):

   ```sh
   cargo test -p malaysia-compositor --lib -- the_band_blur_pass --test-threads=1 --nocapture
   ```

   Run it three times and keep the three medians.
6. Read the output. Each test prints one line like this (the numbers are the
   software run's):

   ```text
   BLURP0 TIMING renderer llvmpipe (LLVM 21.1.8, 128 bits): 2x window 1280x800 logical, band 96 logical rows: band blur pass median 9.11 ms (min 8.85, max 10.58) over 40 frames, 54.6 % of a 60 Hz frame (16.67 ms) and 109.3 % of a 120 Hz frame (8.33 ms); full-output Kawase of the same content (the shipping wallpaper pass) median 59.67 ms
   ```

   - `renderer` must be your GPU. `llvmpipe` means a software run.
   - `skipped: no EGL device` means nothing was measured; the test still
     passes.
   - `2x` is the line the criterion uses; `1x` is reported beside it.
   - The `BLURP0 Above-strip-fixed-...` line that follows has
     `equals_fresh=0` in these tests: they skip the reference render.
7. Apply the criterion below, add the row to the table of section 7.2 and
   record in section 7.1 whether the in-window path ships on by default.

### Pass criterion

**The owner decided this criterion on 2026-11-22**, approving the thresholds
below as proposed. They are not derived from a measurement. The numbers are the
median of the three runs of the 2x test, with the maximum of its 40 frames.

| Median band pass at 2x | Share of a 120 Hz frame | Share of a 60 Hz frame | Reading |
|---|---|---|---|
| 1.0 ms or less | 12 % or less | 6 % or less | Pass: the in-window path can ship on by default |
| Over 1.0 and up to 2.5 ms | Over 12 % and up to 30 % | Over 6 % and up to 15 % | Not decided on 2026-11-22: the owner decides when a measurement lands here, between a cheaper pass and not shipping it on by default |
| Over 2.5 ms | Over 30 % | Over 15 % | Stop: the in-window path does not ship on by default; the work of M2 and K3 may be wasted |

Also decided: no frame of the 40 at 2x may reach 8.33 ms, because one slow
frame shows as a stutter while the window scrolls.

Why these figures:

1. **The cost is paid per frame while content moves under a band.** The
   compositor blurs again each time the source's damage touches the band
   (section 5, step 6), so a scrolling window pays the whole pass every
   frame, and a desktop can show several scrolling windows at once.
2. **The worst common case is the reference.** 2x doubles the pixels per side
   of the band (a source of 2560x216 against 1280x120, section 7.2), and
   120 Hz halves the budget. The 1x and 60 Hz shares are printed on the same
   line.
3. **The band is one pass among others.** The frame also holds the
   composition, the Dock, menu and card blurs that already run live, and the
   client's own work. A band that takes a tenth of a 120 Hz frame leaves room
   for several; one that takes a third leaves room for none.
4. **The software run proves nothing.** llvmpipe needs 55 % of a 60 Hz frame
   at 2x and a GPU is expected to be far below that. A reference, not a budget: on
   llvmpipe the band pass is 15 to 22 % of the full-output pass, because it
   touches 13.5 to 15 % of the pixels. On a GPU a ratio far above that
   points to a fixed cost, such as setup or synchronization, and the test should
   be repeated with a wider band before concluding.

### What to send back

The two `BLURP0 TIMING` lines of the median run, the `=== plain` block of the
probe, the GPU model, the Mesa version (`GL_VERSION` in the probe) and the
refresh rate and scale of the display the owner targets.
