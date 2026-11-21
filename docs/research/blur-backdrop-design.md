# Compositor-composed backdrop: protocol and ADR draft

**Goal:** design how a BarrierKit app declares a material and Malaysia
composes every blur, inside the window too, so the kit can delete its Skia
backdrop blur.
**Scope:** the draft of an Infinity request, the compositor algorithm, the
kit changes, the first experiment and the text of the two ADRs. It starts at
the decision in [`blur-ownership.md`](blur-ownership.md) and ends at the
open questions. It changes no code.
**Reader:** whoever runs the first experiment on Malaysia or implements a
phase.
**Input:** the decision of 2026-11-21 (all blur is the compositor's), the
macOS measurements in [`blur-ownership.md`](blur-ownership.md) and the code
of both repos.
**Output:** a protocol draft, an algorithm, a phase list with acceptance
tests, two ADR drafts and the questions the experiment must answer.

## Steps

1. Read the decision and its consequences in section 1.
2. Read the protocol draft in section 3 and the rules in section 4.
3. Run the experiment in section 7 before any other phase.
4. Move the ADR drafts of section 8 into the two `decisions.md` files when
   phase 1 lands.

---

## 1. The decision

The compositor composes every blur. The app declares a material and a style
and draws no blur. This is the split macOS has
([`blur-ownership.md`](blur-ownership.md#12-what-the-three-images-show)).

Consequences:

1. The kit deletes the Skia backdrop blur of `crates/bk/src/glass.rs` once
   the compositor path works.
2. Without the compositor path (another compositor, or an old Malaysia), the
   kit draws the solid it already draws under Reduce Transparency
   (`FullscreenBar::Solid`, the `solid` of `Material`).
3. Kit goldens and doc shots cannot show compositor blur. The tests of this
   blur live in Malaysia, on its headless renderer.

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

## 3. Protocol draft

An addition to `infinity_window_style_v1`, version 10, in
`crates/infinity-protocol/protocols/infinity-window-style-v1.xml`. Names
follow the AppKit and SwiftUI names the interface already uses for `bar`,
`panel` and `knob`.

```xml
<enum name="blending" since="10">
  <description summary="where a material takes its pixels, as NSVisualEffectView.BlendingMode"/>
  <entry name="behind_window" value="0"
         summary="what lies behind the toplevel: the wallpaper and the windows below"/>
  <entry name="within_window" value="1"
         summary="what the toplevel's own surfaces draw below the requesting surface"/>
</enum>

<enum name="material" since="10">
  <description summary="how a material looks, as NSVisualEffectView.Material"/>
  <entry name="titlebar" value="0"/>
  <entry name="header_view" value="1"/>
  <entry name="sheet" value="2"/>
  <entry name="hud_window" value="3"/>
  <entry name="popover" value="4"/>
  <entry name="menu" value="5"/>
  <entry name="sidebar" value="6"/>
  <entry name="tool_tip" value="7"/>
  <entry name="full_screen_ui" value="8"/>
  <entry name="under_window_background" value="9"/>
</enum>

<enum name="edge" since="10">
  <entry name="top" value="0"/>
  <entry name="leading" value="1"/>
  <entry name="bottom" value="2"/>
  <entry name="trailing" value="3"/>
</enum>

<enum name="scroll_edge_style" since="10">
  <description summary="as ScrollEdgeEffectStyle"/>
  <entry name="automatic" value="0"/>
  <entry name="soft" value="1"/>
  <entry name="hard" value="2"/>
</enum>

<request name="set_backdrop" since="10">
  <description summary="the surface has a material over what lies below it">
    The array holds int32 quadruples x, y, width and height in
    surface-local logical pixels, clipped to the surface, at most 16.
    The compositor draws the material in each rectangle, under the
    surface's own pixels, around a continuous corner of radius. The client
    leaves the rectangle transparent where the material shows and draws
    its content over it. The compositor chooses the radius, saturation and
    tint, and the behavior under Reduce Transparency and Increase Contrast.
    Double-buffered; an empty array removes the backdrop.
  </description>
  <arg name="surface" type="object" interface="wl_surface"/>
  <arg name="blending" type="uint" enum="blending"/>
  <arg name="material" type="uint" enum="material"/>
  <arg name="radius" type="uint"/>
  <arg name="rects" type="array"/>
</request>

<request name="set_scroll_edge" since="10">
  <description summary="the surface has a scroll edge effect">
    The compositor blurs what lies below the surface in a strip of depth
    logical pixels along edge, from the surface's own edge inward. With
    soft the blur fades to none across the depth. With hard it is uniform
    and ends in a line. Double-buffered; a depth of 0 removes it.
  </description>
  <arg name="surface" type="object" interface="wl_surface"/>
  <arg name="edge" type="uint" enum="edge"/>
  <arg name="style" type="uint" enum="scroll_edge_style"/>
  <arg name="depth" type="uint"/>
</request>
```

`ext-background-effect-v1` stays for every other client (GTK, Qt, terminals).

## 4. Rules

1. **Below.** "Below the surface" means every surface of the same toplevel
   that its tree draws before it: its parent, the siblings placed under it
   and their children. A sub-surface above its parent and a parent over a
   sub-surface placed below it both work; the rule needs no knowledge of
   which one the kit chooses.
2. **Within the window only.** `within_window` never samples another toplevel
   or the wallpaper, as Apple documents `withinWindow`. Where the window is
   transparent below the rectangle the compositor uses the window's
   background color.
3. **Same commit.** The request is double-buffered. On a synchronized
   sub-surface it applies with its parent's commit, so material and content
   change in one frame.
4. **Root surface.** `within_window` on a surface with nothing below it has
   no effect.
5. **Compositor decides.** Radius, saturation, tint and Reduce Transparency
   and Increase Contrast come from `ctappearance` and `ctdesign`, as for the
   menus. The measured radius of each material is in
   [`blur-ownership.md`](blur-ownership.md#14-what-the-app-sets-and-what-the-system-decides).
6. **No global, no effect.** A client that sees version below 10 draws the
   solid fallback.
7. **Only BarrierKit clients.** As `set_glass` and `set_knobs`, the compositor
   ignores the requests from other clients.

## 5. Compositor algorithm

All in Malaysia.

1. Store the requests per `wl_surface` in a `Cacheable` state, as
   `GlassPanels` and `Knobs` do in `window_style.rs`.
2. In the window path of `scene_elements.rs`, where
   `render_elements_from_surface_tree` builds `surface_elements`, split the
   list by owning surface. The elements of the requesting surface and the ones
   above it stay in front. The ones below become the source.
3. Render the source of each rectangle, expanded by the blur reach, into an
   offscreen texture with an `OutputDamageTracker`, as `render_behind` does,
   and run the Kawase passes of `blur.rs`. Take passes and offset from a
   per-material table in `ctdesign` (measured extents: 35 px for `sidebar`
   in light, about 48 px for the others).
4. For `soft`, blend the sharp source with the blurred levels by a mask along
   the edge, as the kit does today with `layer_mask`; for `hard`, use one
   level and draw the line.
5. Draw the result as a `RoundedTextureElement` between the two halves of the
   list.
6. The tracker invalidates the cache when the source's damage touches the
   rectangle plus the reach. A scrolling window therefore blurs every frame.
7. When `drawn(state)` is false (Reduce Transparency, Increase Contrast),
   draw the material's solid instead, as the Dock does with `dock_solid`.

The headless tests of `blur.rs` (`try_test_renderer`, `solid` elements) are
the pattern for the tests: a red element below a rectangle must tint it, a
blue one beside it must not.

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
5. Outbox for the requests, like `GlassOutbox`, flushed before the commit
   (`flush_glass` is the model).
6. `glass.rs` loses the `Glass` and `MaterialElement` backdrop code once the
   phases below pass. `05-controls.md`, `09-titlebars.md`,
   `15-backdrops-and-media.md` and `18-platform.md` change in that commit.

## 7. Phases and the first experiment

| Phase | Output | Sessions |
|---|---|---|
| 0 | The experiment below | 1 to 2 |
| 1 | Protocol version 10 in `infinity-protocol`, both ADRs | 1 to 2 |
| 2 | Compositor: state, split, blur, mask, gate, tests | 3 to 5 |
| 3 | Kit: sub-surface, layer split, input, outbox | 4 to 6 |
| 4 | Migrate the four elements, delete the Skia blur, update docs | 2 to 3 |

### Phase 0: the experiment

Run it where Malaysia builds and has a GPU. Use a scratch copy of the
compositor; commit nothing.

1. A minimal sctk client: an opaque parent surface with moving stripes and a
   synchronized sub-surface above it that is transparent in a band and has a
   blur region over that band.
2. In the scratch compositor, read the sub-surface's region and, for that
   band, blur the parent's elements below it, as section 5 says.
3. Look at the result in a nested session, scrolling the stripes.
4. Run the window capture probe (`./dev.sh screencopy-window`) and check
   whether the captured window has the blur.

Answers it must give:

- Does the blur follow the scrolling without a frame of lag, with the
  sub-surface committed in sync mode?
- What is the frame time of the blur pass at 1x and 2x on the user's GPU,
  with the band of `FULLSCREEN_GLASS` (96 px)?
- Does a window capture include the blur?
- Does Smithay deliver the sub-surface's `BackgroundEffectSurfaceCachedState`
  with the parent's commit?
- Is a sub-surface below the parent as workable as one above it?

If the frame time or the lag fails, stop: no later phase has value.

## 8. ADR drafts

For `docs/decisions.md` of Malaysia, replacing the blur lines of
"0020 · 0038 · 0065 · 0071":

```markdown
## 0020 · 0038 · 0065 · 0071 · 00NN — Blur and wallpaper tinting
Decision:
- The compositor composes every blur: behind a window, behind a layer surface and inside a BarrierKit window. A client declares a material and never draws a backdrop blur.
- Behind a window, a surface gets blur in the region it sends through `ext-background-effect-v1`. Inside a window, a BarrierKit client sends `infinity_window_style_v1.set_backdrop` or `set_scroll_edge` on the surface that draws over the content, and the compositor blurs what the toplevel's surfaces draw below it.
- The compositor decides radius, saturation and tint per material and appearance, and drops the blur under Reduce Transparency and Increase Contrast.
- [the remaining lines of the current decision stay]
Why: macOS composes every blur outside the app, in both blending modes; the app declares the material. One place then decides radius and accessibility. `set_glass`, `set_plates` and `set_knobs` already paint for clients.
Rejected: a blur drawn by the client as the shipping path (two places for Reduce Transparency, two blur algorithms); `ext-background-effect-v1` on sub-surfaces (KWin, Hyprland and niri ignore it).
Code: `crates/malaysia-compositor/src/blur.rs`, `crates/malaysia-compositor/src/window_style.rs`
```

For `docs/decisions.md` of BarrierKit, a new entry:

```markdown
## 00NN — In-window glass is a request
Decision:
- `FullscreenBar::Glass`, `ProgressiveGlass`, `TopGlass` and `Material` draw no blur. The kit leaves the rectangle transparent, draws its chrome on a sub-surface over the content and sends `set_scroll_edge` or `set_backdrop`.
- Without version 10 of `infinity_window_style_v1` the kit draws the solid of Reduce Transparency.
- The kit tests of this glass check the request and the solid; the blur is tested in Malaysia.
Why: the compositor composes every blur, as macOS does.
Rejected: keeping the Skia backdrop blur as a second path.
Code: `crates/bk/src/glass.rs`, `crates/bk/src/platform/glass.rs`
```

## 9. Open questions

1. Sub-surface above the parent (chrome on top, as section 6) or below it
   (content under a root that is the chrome)? The rule in section 4 works
   for both. Input and `xdg_surface` geometry decide.
2. Which `NSVisualEffectView.Material` does each `styles::Thickness` take?
   The extents in [`blur-ownership.md`](blur-ownership.md) give two groups
   (about 35 px and about 48 px in light).
3. How does `GlassContainer` merge shapes in the compositor
   (`NSGlassEffectContainerView.spacing`)?
4. What does `automatic` resolve to? A soft and a hard
   `preferredScrollEdgeEffectStyle` looked the same in the probe.
5. Should layer surfaces move from `ext-background-effect-v1` to
   `set_backdrop` with `behind_window`, to get a radius per material?
6. How do kit doc shots show compositor glass: captures of a nested Malaysia
   session, or none?
