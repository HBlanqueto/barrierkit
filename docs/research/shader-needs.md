# Shader needs: what SwiftUI and AppKit ask of the GPU

**Goal:** decide who can implement each shader-level or GPU-composition
capability of SwiftUI, AppKit, Core Animation and Core Image (the kit with
Skia, the compositor, or nobody), at what cost, and which ones are not worth
building.
**Scope:** the SDK APIs that run a function per pixel, composite layers, filter
a view or sample what is behind it. It ends at the table, the implementation
options and the API sketch. It does not repeat who applies the blur; that is
the blur-ownership research (not on `main` when this file was written).
**Reader:** anyone who plans `bk` shader, material or modifier work, or a
Malaysia effect.
**Input:** the Xcode 27.2 SDK (`SwiftUICore.swiftinterface`,
`SwiftUI.swiftinterface`, AppKit, QuartzCore and CoreImage headers),
`freya-skia-safe` 0.104.0 and `freya-core` 0.5.3 from `Cargo.lock`, the kit's
`crates/bk/src`, Malaysia's `crates/grade-graphics`, and a throwaway Skia probe
run in the `bk` container.
**Output:** one table of 28 rows, two prerequisites, the option costs, an API
sketch, the plan for P1 and the open questions.

## Steps

1. Read [Findings](#findings).
2. Read [Prerequisites](#prerequisites): most kit rows wait on P1.
3. Find a capability in [the table](#the-table).
4. Before building a shader, read [Options and costs](#options-and-costs).
5. To add the API, follow [API sketch](#api-sketch) and
   [`20-glossary.md`](../barrierkit/20-glossary.md).

## Findings

1. **SwiftUI shaders cannot be copied as code.** `ShaderLibrary` loads a
   precompiled Metal library (`init(data:)`, `init(url:)`, `bundle(_:)`,
   `default`); no API takes shader source. `grep -ci -E "metal|stitchable|metallib|msl|MTL"`
   gives 0 in both interfaces. Apps that use shaders ship Metal; the kit can
   only offer SkSL strings.
2. **The shader API is small.** Three kinds (`colorEffect`, `distortionEffect`,
   `layerEffect`), a fill (`Shader: ShapeStyle`, `Shading.shader`), ten argument
   kinds (`float`, `float2`, `float3`, `float4`, `floatArray`, `boundingRect`,
   `color`, `colorArray`, `image`, `data`) and `dithersColor`. No `Shader.Options`.
   None of the three kinds reads the backdrop: they see the view's own layer.
3. **Skia does all of it in the app.** A `RuntimeEffect` (shader, color filter,
   blender) and the `runtime_shader` image filter compile and run in the pinned
   crate. Freya 0.5.3 already ships `ShaderFill`, and the kit already ships two
   SkSL shaders (`glass_container.rs`, `app_icon.rs`).
4. **The real blocker is not shaders.** Freya's render loop dims and blurs per
   element, with no hook around a subtree. `compositingGroup`, `mask`, `blur`,
   `colorEffect`, `layerEffect` and every color modifier on a container need one
   ([P1](#prerequisites)). The kit already works around it twice
   (`level.rs:207`, `controls/slider.rs:290`).
5. **Only behind-window pixels need the compositor.** No Wayland protocol
   uploads shader code (`grep -rni shader` over `wayland-protocols` 0.32.13 and
   `wayland-protocols-plasma` 0.3.12: 0 matches). `ext-background-effect-v1`
   has one capability, `blur`; KDE's contrast protocol adds contrast, intensity,
   saturation and a frost color. Malaysia's `infinity-window-style-v1` carries
   the rest.
6. **Measured in Skia:** a backdrop filter of `blur` then a veil-and-cap tone
   shader flattens an 8 px checker to 153/255, the value the model of
   [`macos-blur.md`](macos-blur.md) predicts (E9). So in-window materials need
   no compositor. Superseded on 2026-11-21: the owner decided that the
   compositor composes in-window blur too ([ADR 0091](../decisions.md)); E9
   stays as the measurement.
7. **Sharing GLSL and SkSL is possible only for pure functions.** Three of
   Malaysia's five `.glsl` files (`oklab.glsl`, `window_shape.glsl`,
   `lens.glsl`) compile unchanged as SkSL (E2). Entry points, samplers,
   `precision`, `varying` and `#define` do not. `naga` cannot bridge them.
8. **Not worth building:** vibrancy over a desktop backdrop, the
   other ~230 Core Image filters, MSL and Core Image kernel loading, the private
   `CAFilter` set, HDR.
9. **SwiftUI mixes gradients in OKLab, premultiplied, by default.** Measured on
   macOS 27.2.1 with `ImageRenderer` (E14): red to blue at 50% is (140,83,162),
   white to black is 99; both equal an OKLab mix within 1/255, and a stop that
   fades to transparent mixes premultiplied. Skia's OKLab interpolation gives the
   same values (E13). The kit's `Gradient` now mixes the same way by
   default; the two controls measured against AppKit (the small square bezel and
   the tab pill) keep `.color_space(GradientColorSpace::Device)`.

## Method

Two subagents read the SDK (SwiftUI interfaces; AppKit, QuartzCore and
CoreImage headers) and returned tables with line numbers. I re-checked by
sampling against the files and corrected three counts:

| Claim | Report | Checked |
|---|---|---|
| `BlendMode` cases on `View` (SwiftUICore:8091) | 20 | 21 (`grep` of the enum) |
| `GraphicsContext.BlendMode` (SwiftUICore:9011) | 27 | 28 (`grep -c` of the statics) |
| `Shader.Argument` kinds | 11 | 10 (`float2` has four overloads) |

Verified by me: the `ShaderLibrary`, `ShaderFunction`, `Shader` block
(SwiftUICore:8865-8990), `MeshGradient`, `_ConditionalCompositingGroupEffect`
(`anyAppleOS 27.0`), `NSView.backgroundFilters` at `NSView.h:275`, the
`NSGlassEffectView` header, 239 typed `CIFilterBuiltins` factories, 243
`_kCAFilter*` symbols in `QuartzCore.tbd` (no public header declares `CAFilter`).
Both SwiftUI interfaces carry no doc comments, so "samples behind the view" comes
from Apple's DocC pages and is marked where only DocC says it.

## What each side has

| Side | What it offers | Source |
|---|---|---|
| Skia in the kit | `RuntimeEffect::make_for_shader`, `make_for_color_filter`, `make_for_blender`; `image_filters::runtime_shader`, `runtime_shader_with_options` (sample radius); `color_filters::matrix_row_major`, `hsla_matrix`, `luma`, `table`; `Blender::arithmetic`; 29 `BlendMode` variants; `Canvas::draw_patch`, `draw_vertices`, `concat_44` | `freya-skia-safe-0.104.0/src/effects`, `src/core` |
| Freya 0.5.3 | `Fill::Shader(ShaderFill)` for backgrounds and text; `opacity`, `backdrop_blur`, `rotation`, `scale` per element; `ElementExt::render` hands the kit the canvas | `freya-core-0.5.3/src/style/shader.rs`, `data.rs` (`EffectData`), `element.rs` |
| The kit today | `glass.rs` (backdrop blur, `DstIn` masks), `glass_container.rs` (SkSL smooth union over 8 shapes), `app_icon.rs` (SkSL plate edge), `symbol.rs` (color filters) | `crates/bk/src` |
| Malaysia | GLSL ES 1.00 (`varying`, `gl_FragColor`, `texture2D`) for Smithay's `compile_custom_texture_shader`: blur down/up, glass, rim, shadow, Dock, minimize, display filter; 5 `.glsl` and 15 `.frag` files, 828 lines with `lib.rs`. The wallpaper uses a second dialect: GLSL 450 (Shadertoy `mainImage`) and WGSL through wgpu and naga | `~/src/malaysia/crates/grade-graphics`, `malaysia-wallpaper/src/shader.rs` |
| Other compositors | `ext-background-effect-v1` (blur only); KDE `org_kde_kwin_blur` and `org_kde_kwin_contrast` | `wayland-protocols-0.32.13`, `wayland-protocols-plasma-0.3.12` |

The kit already asks for the blur: `shell/blur_region.rs` sends
`ext_background_effect_v1.set_blur_region`.

## Prerequisites

| ID | What | Why | Cost |
|---|---|---|---|
| P1 | A subtree layer hook ([plan](#p1-plan)). The kit calls `RenderPipeline` itself (`platform/surface.rs:256`) and `Tree` fields are public, so the kit can copy the ~170-line loop of `freya-core-0.5.3/src/render_pipeline.rs` and draw a group's descendants into one layer that carries an image filter, a blend mode or a mask. | The loop paints breadth-first, one z-layer per tree depth, and applies each ancestor's opacity per node with `save_layer_alpha_f`. No SwiftUI effect on a container can be built on that. | L |
| P2 | A shader registry: compile each SkSL source once, keep the `RuntimeEffect`, read uniform names, types and order with `RuntimeEffect::uniforms()`, check arguments at bind time. **Done for uniforms (2026-11-22):** `crates/bk/src/shader.rs` (`Program::get`, `pack`, `BindError`); `glass_container.rs` binds its eight uniforms by name through it. Children (`image` arguments) and `app_icon.rs` are not moved. | Each shader packed uniform bytes by hand in a fixed order. Reflection works: the probe printed `uniforms` and `children` counts for every effect (E1), and the test of `shader.rs` reads `box`, `ramp`, `tint` at byte offsets 0, 16 and 24: Skia packs the uniforms with no padding to 16 bytes, as the hand-kept array did. | M |

A whole-window variant is cheap (render the tree into an offscreen surface, then
draw it through a filter), but no SwiftUI API needs only that.

## The table

Owner: **Kit** the kit with Skia alone, **Comp** the compositor must sample
behind the window, **None** neither, or with visible loss. Cost: **S** under a
day, **M** days with a probe, **L** a week or an architectural change. `P` is
the probability that the approach works as described.

| ID | Capability (SwiftUI / AppKit) | Owner | Approach and gain | Cost | P and assumptions | Blocker |
|---|---|---|---|---|---|---|
| 1 | `Shader`, `ShaderLibrary`, `ShaderFunction`, `Shader.Argument`; `Shader` as `ShapeStyle` and `Shading.shader` | Kit | `ShaderLibrary` holds named SkSL sources; arguments bind in uniform declaration order through reflection; `image` becomes a child shader; `floatArray` and `colorArray` become uniform arrays (E7 compiles `float4 stops[8]`). Fills use `Fill::Shader`. Gain: apps write custom fills and effects. | M | 0.9: SkSL compiles on Vulkan as on raster | P2; `data` has no direct SkSL type |
| 2 | `colorEffect` | Kit | A shader that reads `content.eval(position)` and returns a color; stock `ColorFilter` when it ignores position. Gain: per-pixel tints and gradients on any view. | S leaf, M container | 0.85: P1 for containers | P1 |
| 3 | `layerEffect` | Kit | `runtime_shader` image filter on a layer; `sample_radius` = `maxSampleOffset` (E3 moved a red/blue edge 8 px). Gain: ripples, pixelation, blurs of a view. | M | 0.8: P1; `CGSize` offset folds to one radius (the larger) | P1 |
| 4 | `distortionEffect` | Kit | Same path; the user function returns the source position. Gain: wave and magnify effects. | M | 0.8 as row 3 | P1 |
| 5 | `Shader.compile(as:)` (async prewarm) | Kit | Compile at registration (errors surface at once); warm the GPU pipeline with a 1x1 draw. Gain: no stutter on first use. | S | 0.5: GPU first-draw cost not measured (no GPU here) | none |
| 6 | `[[stitchable]]` MSL, `.metallib` (`init(data:)`, `init(url:)`, `default`); Core Image Metal kernels, string kernels (deprecated 10.14) | None | No compensa: no translator reads MSL into SkSL. Apps port by hand; document an MSL to SkSL table. Gain of a translator: none that justifies it. | n/a | n/a | no tool found |
| 7 | `visualEffect`, `VisualEffect`, `scrollTransition` | Kit | A closure over a geometry value that returns modifiers; offset, scale, rotation, opacity already exist in Freya. Gain: scroll and geometry-driven effects. | M | 0.6: blur and color members need P1 | P1 |
| 8 | `blendMode` (21 modes), `GraphicsContext.BlendMode` (28), `ShapeStyle.blendMode`, `compositingFilter` | Kit | 20 of 21 map to Skia `BlendMode` (29 variants compile by name, E12); `plusDarker` is a SkSL blender (compiles, E12; numeric match with macOS unverified). Blends only with the window's own pixels, as macOS. Gain: Multiply, Screen, Overlay and the rest. | M | 0.85: P1 for groups | P1 |
| 9 | `compositingGroup`, `drawingGroup` | Kit | This is P1. Gain: a faded container fades as one (the slider and level workarounds go away). | L | 0.65: see P1 risk | P1 |
| 10 | `mask`; `NSVisualEffectView.maskImage` | Kit | `DstIn` layer over the subtree (the kit does it for glass, `glass_container.rs:257`). Gain: any view as a mask. | M | 0.8 | P1 |
| 11 | `blur(radius:opaque:)` on a view; `CIGaussianBlur` | Kit | Blur image filter on the layer; `opaque` picks the tile mode. Gain: blurred content and transitions. | M | 0.85 | P1 |
| 12 | `saturation`, `brightness`, `contrast`, `hueRotation`, `grayscale`, `colorInvert`, `colorMultiply`, `luminanceToAlpha`; `CIColorControls`, `CIHueAdjust`, `CIColorInvert`, `CIMaskToAlpha` | Kit | Stock `ColorFilter` (matrix, `hsla_matrix`, `luma`, blend); the kit already uses them on images (`symbol.rs:768-790`). Measured on macOS 27.2.1 (E14), on sRGB-encoded values: `saturation(s)` mixes toward the luma 0.2126/0.7152/0.0722 (the same at 8 bits as SVG's 0.213/0.715/0.072); `grayscale(g)` is `saturation(1 - g)`; `brightness(b)` adds b; `contrast(k)` is (c - 0.5)k + 0.5; `hueRotation` is the SVG `hue-rotate` matrix; `colorInvert` is 1 - c; `colorMultiply` multiplies. Gain: the eight modifiers on any view. | S leaf, M container | 0.95: formulas match every sampled value within 1/255 | P1 for containers |
| 13 | `MeshGradient`, `Shading.meshGradient` | Kit | `Canvas::draw_patch` per cell (Coons patch, corner colors) or an SkSL with point and color arrays (compiles, E7). Gain: the macOS 15 mesh backgrounds. | M | 0.6: Apple's interpolation is unmeasured | probe |
| 14 | `AngularGradient`, `EllipticalGradient`, `Gradient.ColorSpace.perceptual`, `Color.mix` | Kit | Done 2026-11-21: `Gradient::angular`, `Gradient::elliptical` and `.color_space(GradientColorSpace)` draw a Skia gradient through Freya's `Fill::Callback` (default `Perceptual`: OKLab, premultiplied; SwiftUI's angular starts right and runs clockwise, its elliptical radii are half the width and half the height; four tests). Not offered: center, start and end angles or radii. `Color.mix` is not done. Gain: all SwiftUI gradients. | S | 0.9 | none |
| 15 | `Canvas`, `GraphicsContext` (`addFilter`, `drawLayer`, `clip`, `Shading.backdrop`) | Kit | A leaf element drawing with the Skia canvas (the `glass.rs` pattern); `addFilter` and `drawLayer` are `save_layer` plus a filter (E11). `backdrop` samples the window only. Gain: immediate-mode drawing. | M | 0.8 | none |
| 16 | `TextRenderer`, `Text.Layout` | Kit | Skia `Paragraph::visit` exposes glyph runs, but Freya draws paragraphs itself, so the kit would own those text views. No compensa now: apps get gradient or shimmering text from `ShaderFill` on text. Gain would be per-glyph animation only. | L | 0.5 | Freya text path |
| 17 | `ImageRenderer` | Kit | Render a subtree into an offscreen surface with the P1 loop. No compensa until an app asks: `doc_shots.rs` covers the kit's own captures. | M | 0.6 | P1 |
| 18 | `rotation3DEffect`, `transformEffect`, `projectionTransform` | Kit | `Canvas::concat_44` takes a 4x4 matrix; Freya has 2D rotation only. Gain: card flips, perspective. | M | 0.7 | P1 |
| 19 | `Material` and `NSVisualEffectView` `.withinWindow`; `.background(.ultraThinMaterial)` | Kit | Backdrop filter = blur then a veil-and-cap tone shader (E9: 153 against 153 expected). The 14 current materials need measured values, not new technique. Gain: materials over the app's own content. Superseded on 2026-11-21 by [ADR 0091](../decisions.md): the compositor composes this blur and the kit sends `set_backdrop` with `within_window`. | M | 0.8: values come from [`macos-materials.md`](macos-materials.md) | measurements |
| 20 | `NSVisualEffectView` `.behindWindow`; window and sidebar materials over the desktop | Comp | Malaysia: `ext-background-effect-v1` blur plus `set_glass`/tone shaders. Other compositors: plain blur; the app adds an alpha veil but cannot clamp luminance (the cap) or saturate; KDE's contrast request adds saturation. Gain: desktop-tinted sidebars. Ownership detail: blur-ownership research. | M | 0.8 on Malaysia; 0.5 elsewhere (unverified: no other compositor run) | Wayland has no shader upload |
| 21 | Vibrancy: `allowsVibrancy`, `NSAppearance.vibrant*`, `.secondary` over a material | None | No compensa. Vibrant ink blends with the desktop pixels; nothing lets an app do that. The kit keeps measured ink colors (`ctappearance::menu::dark_ink_over`). Gain of more: none visible. | n/a | 0.2 for a faithful result | no protocol |
| 22 | `glassEffect`, `Glass` (`regular`, `clear`, `tint`, `interactive`), `GlassEffectContainer`, `glassEffectID/Union/Transition`; `NSGlassEffectView`, `NSGlassEffectContainerView`, `effectIsInteractive` (27.0) | Comp for the glass over the window's own backdrop; Kit for the static pieces | Measured 2026-11-22 ([`macos-liquid-glass.md`](macos-liquid-glass.md)): a half-resolution backdrop, a lens along the SDF normal (the profile is a function of depth over height), a Gaussian of sigma 0 to 28 px by depth, a luma-scaled tone, a rim and shadow table and a smooth union of the container's shapes. The shader descriptions (GLSL for the compositor, SkSL compiled in the container for the kit) are in section 9 of that file. `interactive`: hovered and pressed are an affine map of the output colour per scheme (one multiply-add); what triggers it and its shimmer are unmeasured. Gain: the glass of every macOS 26 app. | L | 0.7 for the tone, blur and union (CPU reference within 2.5 levels); 0.5 for the lens near the edge (high-frequency backdrops differ by 1 to 2 px); GPU cost unmeasured | pointer trigger for `interactive`; GPU run |
| 23 | `scrollEdgeEffectStyle` (`automatic`, `soft`, `hard`); `NSScrollEdgeEffectStyle` | Kit | Checked by reading the kit: `ScrollView::edge_effect(bool)` is the `soft` fade (24 px, `SCROLL_BAR.edge_height`) and `TopGlass` is a glass band; there is no `hard`, no `automatic` and no `Edge.Set`, and no capture backs the pixels. Layered blur and mask gradients can give `hard`. Gain: finish the partial row. | S | 0.8 | a window probe: `ImageRenderer` cannot show a live scroll edge (unverified) |
| 24 | `backgroundExtensionEffect`; `NSBackgroundExtensionView` | Kit | Checked by reading the kit: `SplitView::background_extension` stretches the first pixel column to the window edge (`10-sidebar.md`); Apple mirrors and blurs. A mirror tile plus blur (`TileMode::Mirror`) is easy for an `Image`; an arbitrary view needs P1. Gain: the macOS 26 look under the sidebar. | S for an image, M after P1 | 0.8 | P1 for arbitrary views |
| 25 | `NSView.backgroundFilters`, `contentFilters`, `compositingFilter`; `CALayer.filters`, `backgroundFilters`, `compositingFilter` | Kit; behind-window None | Map the Core Image filters named in rows 11 and 12 and the blend filters (30 `CICategoryCompositeOperation` entries) to Skia. Behind a window: none. Other ~230 of 239 typed Core Image filters: no compensa; an app can write any as SkSL. Gain: the common filters. | S per filter | 0.85 | P1 for containers |
| 26 | Private `CAFilter` (243 `_kCAFilter*` symbols) | None | No compensa. Undocumented; semantics only by probe. Gain: none. | n/a | n/a | not public |
| 27 | Accessibility Display: Invert Colors, color filters, display contrast | Comp | Exists in Malaysia (`display_filter.frag`: matrix, invert, contrast). The kit must not filter its own windows too (double apply). Gain: every app, not only the kit's. | done | n/a | none |
| 28 | HDR and EDR: `Color.ResolvedHDR`, `allowedDynamicRange`, `CALayer.preferredDynamicRange` | None | No compensa now. Needs a float swapchain, `wp_color_management_v1` (staging in `wayland-protocols`) and a compositor that composites it; Malaysia's GLES path is 8-bit. Gain: HDR media only. | L | 0.2: unverified | compositor renderer |

Rows not in the table because nothing is missing: `shadow`, `ShadowStyle`
(`.inner`, `.drop`), `symbolEffect` (not GPU), `Material.bar`. SwiftUI has no
particle or texture effect (`grep -ci -E "ParticleEffect|Emitter|TextureEffect"`: 0).

## Options and costs

### SkSL in the kit

Already in use. A shader is a string, a `thread_local` `RuntimeEffect` and a
byte block of uniforms (`glass_container.rs:135`). The probe measured
`make_for_shader` at 3 to 520 µs on CPU for 8 to 60 line shaders; that is parse
and IR, not the GPU pipeline. The uniform block is separate from the source, so
changing a value never recompiles (`RuntimeEffect::make_shader(uniforms, children, matrix)`).
Cache one effect per source in P2. The first Vulkan draw compiles a pipeline;
that cost and its stutter are unmeasured (no GPU in the container, no Vulkan on
the Mac host).

SkSL as Skia 0.104.0 builds it (the Chrome m153 source in `nix/skia.nix`) is
GLSL ES 2.0 plus children. Probed limits (E1, E2, E7):

| GLSL ES 1.00 / Malaysia | SkSL runtime effect |
|---|---|
| `void main()`, `gl_FragColor` | `half4 main(float2 p)`, returns the color |
| `texture2D(tex, uv)`, `uniform sampler2D` | `uniform shader t; t.eval(p)`; pixel coordinates, not 0 to 1; `sample(t, p)` is unknown |
| `varying`, `precision` | reserved names: errors |
| `#define`, `#if defined` | `#define` is an unsupported directive; `#version` is accepted |
| loops with dynamic bound, `%`, `dFdx` | all rejected; constant bounds, `break` and `const int` work |
| `vec2`, `mix`, `step`, `pow`, `smoothstep`, `out` parameters, `mat3`, arrays | accepted unchanged |

### GLSL in the compositor

A new effect is a `.frag` file in `grade-graphics`, a `pixel_shader!` or
`texture_shader!` line and a uniform list in the compositor (see `minimize.rs`,
`blur.rs`). It is the only place that samples the desktop. Malaysia has two more
dialects: GLSL 450 for wallpapers and WGSL for fixed passes.

### One description for two backends

| Tool | Reads | Writes | Verdict |
|---|---|---|---|
| `naga` 30.0.1 | WGSL, SPIR-V, GLSL 440+ with Vulkan semantics | SPIR-V, WGSL, MSL, HLSL, GLSL 330+ and ES 300+ | No SkSL either way; cannot read ES 1.00 or write it. README, `front/glsl/mod.rs` |
| Skia's own compiler | SkSL | internal GLSL, SPIR-V, MSL | Not exposed: `runtime_effect.rs` has no emit function (`grep -n "pub fn"`) |
| glslang, SPIRV-Cross, shaderc | GLSL, HLSL, SPIR-V | various | Not tried; none is known to read SkSL (unverified) |

No translator fits. The workable design is shared **pure functions** plus a thin
entry per backend. Measured: `oklab.glsl` (52 lines), `window_shape.glsl` (67)
and `lens.glsl` (7) compile verbatim as SkSL under a `half4 main` wrapper. Not
tested: `dock_frame.glsl`. Not compilable: `glass_fill.glsl` (sampler, `#if`).
The rule for a shared file: no preprocessor, no sampler, no `precision`, only
plain uniforms with equal names. A test renders the SkSL on Skia's raster
backend, which runs in the Linux container with no GPU (E9, E11), and compares
it with the Rust function in `ctdesign` and `ctappearance`. The GLSL side has no
GPU-free test here.

Malaysia depends on the kit at a pinned `rev`, and `grade-graphics` lives in
Malaysia. A shared directory therefore belongs to the kit, and `grade-graphics`
would include it. That reverses today's layout; it is a decision ([see below](#decisions)).

### Accessibility and other compositors

Built-in materials follow `Resolved::reduce_transparency` and Increase Contrast
as `glass.rs` already does (solid fill). SwiftUI documents no such gate for user
shaders, so user shaders run as written (unverified on macOS). Without
Malaysia, a kit window gets the compositor's plain blur (`ext-background-effect-v1`),
the app's own veil and no cap or saturation, except KWin's contrast request.

## API sketch

Names follow [`20-glossary.md`](../barrierkit/20-glossary.md) and the rules of
`AGENTS.md`.

| SwiftUI | Kit name | Note |
|---|---|---|
| `Shader` | `Shader` | `Shader::new(function, arguments)`, as `init(function:arguments:)`. `freya_engine::prelude::Shader` is Skia's: check that `bk::prelude` does not re-export it |
| `ShaderLibrary` | `ShaderLibrary` | `ShaderLibrary::new(sksl)`; no `default`, `bundle` or `url` |
| `ShaderFunction` | `ShaderFunction` | `library.function("name")` replaces `@dynamicMemberLookup` |
| `Shader.Argument` | `ShaderArgument` | flattened: `float`, `float2`, `float3`, `float4`, `float_array`, `bounding_rect`, `color`, `color_array`, `image`, `data` |
| `Shader.UsageType` | `ShaderUsageType` | only if `compile` stays |
| `.colorEffect(_:isEnabled:)` | `.color_effect(shader)` | no `enabled`: the kit's `maybe` does it |
| `.layerEffect(_:maxSampleOffset:isEnabled:)` | `.layer_effect(shader, max_sample_offset)` | |
| `.distortionEffect(_:maxSampleOffset:isEnabled:)` | `.distortion_effect(shader, max_sample_offset)` | |
| `Shader.dithersColor` | `Shader::dithers_color` | |

What cannot be mirrored:

- **`[[stitchable]]` Metal functions** compile at build time with a fixed
  signature. SkSL is a string compiled at run time; the error is a `Result`
  at `ShaderLibrary::new`, not a build error.
- **Positional arguments** become uniform declaration order. SwiftUI passes
  `position`, `color` and the layer as leading Metal parameters. The kit adds a
  `main` per usage kind that calls the user's function: `colorEffect(position,
  color)` with `color = content.eval(position)`; `layerEffect(position)` with a
  `layer_sample(float2)` helper; `distortionEffect(position)` returning the
  source position. One `runtime_shader` path serves all three.
- **`maxSampleOffset` is a size;** Skia takes one radius.
- **One `image` argument per shader** in SwiftUI (DocC); SkSL allows several
  children.

Shipped with the kit (SkSL beside its element, values in `ctdesign`): the glass
union and plate edge (exist), the material tone (veil, lift, cap, chroma:
`MENU_GLASS_*` and `MATERIAL` exist), mesh gradient, the `plusDarker` blender.
The app brings its own effects. Nothing is exposed as a library function the
kit does not use (rule 2 of «Código mínimo»).

## P1 plan

**Goal:** a kit-owned render loop that draws a subtree into one layer with its
own opacity, blend mode, mask and filter, and changes no pixel until a group is
used.
**Scope:** the loop and the group container. The effects that sit on it (table
rows 2 to 4, 8 to 12, 17, 18, 25) follow one by one. P2 is separate.

Facts that shape the plan:

1. The loop is breadth-first. `LayerState::update` gives a node
   `parent.layer + relative + 1` (`freya-core-0.5.3/src/data.rs:262-279`), and
   `render` visits the sorted layer keys, reading each layer's nodes from a hash
   set. A subtree is not contiguous in paint order, so one `save_layer` and
   `restore` pair around a node cannot hold its descendants.
2. Each node is self-contained. Between `save` and `restore_to_count` it applies
   its ancestors' clips, rotations and scales, and the product of their opacities.
   Opacities are a list of values, not node ids, so the loop cannot tell a group's
   opacity from an ancestor's.
3. Only painting uses this loop. Hit testing reads `Tree::layers` in
   `events/measurer.rs:42`, and nothing else outside the loop does
   (`grep -rn "\.layers\b"`). Layer numbers stay as they are.
4. `ElementExt` extends `Any`, the `Tree` fields the loop reads are public and
   the workspace `rust-version` is 1.98, so the loop can read a kit payload from
   an element by upcasting.

Design:

1. **Payload.** A kit group value holds opacity, blend mode, color filter, image
   filter and mask. It lives on a kit container element that wraps Freya's `rect`
   and forwards its `ElementExt` methods, as rule 3 of «Código mínimo» asks of
   view wrappers. `ViewModifiers` methods wrap a view in it. The group keeps its
   opacity in the payload, not in `EffectData`, so its descendants do not dim
   themselves again.
2. **Loop.** Breadth-first as today. At a group root the loop draws the root and
   every descendant depth-first, in depth order, inside one `save_layer`, marks
   them done, and restores with a paint built from the payload. A descendant
   with `Layer::Overlay` or `OverlayLevel` stays out of the group and paints in
   the main pass.
3. **Bounds.** The union of the descendants' visible areas mapped by their
   scales, outset for shadows as the loop already does for opacity. Start with a
   conservative bound. A layer effect gets a local matrix so its `position` is
   the group's own space.
4. **Cost control.** No layer when the payload is the identity. Groups nest.

Milestones, in order:

| M | Work | Cost | Done when |
|---|---|---|---|
| M0 | Run the golden scenes and the `bk` tests in the container and record which fail today | S | A baseline list exists |
| M1 | Copy the loop into the kit unchanged; call it from `platform/surface.rs`; add a script that fails when the pinned `freya-core` loop file differs from the copy's source version | S | Every golden is byte-identical |
| M2 | `compositing_group` with opacity only; glossary row, chapter and `agents/` entry in the same commit (rule 4 of «Nombres y glosario») | M | Two overlapping children at 50% show no seam (pixel assertion); scenes without a group are unchanged |
| M3 | The depth-first group pass of design 2 and 3 | L | One golden each: nested groups, clipped parent, scaled parent, rotated parent, shadow at the edge |
| M4 | Use a group in the slider and the level indicator instead of per-part dimming (`controls/slider.rs:290`, `level.rs:207`) | S | Goldens equal or better; the two comments go |
| M5 | Effects, one commit each: `mask`, `blur`, color modifiers (stock `ColorFilter`), `blend_mode`; then the three shader effects once P2 exists | S to M each | One golden per effect |

M0 to M4 are sequential. M5 starts after M3.

Risks:

- **Paint order.** Overlapping elements at the same depth in different subtrees
  paint in hash-set order today, so grouping cannot make it worse; a descendant
  that must paint above everything needs a `Layer` overlay.
- **Cost per frame.** The loop redraws everything each frame (its own `TODO: Use
  incremental rendering`), so each active group allocates a layer every frame.
  Measure on a Vulkan machine before wide use; none is available here.
- **Upgrade.** The copy stops matching when the `=0.5.3` pin moves; the M1 script
  makes that visible.
- **Goldens.** `UPDATE_GOLDEN=1` rewrites every golden; review the diff and
  commit only the intended images.

## Experiments

Throwaway crate outside the repo, built in the `bk` container against
`freya-skia-safe` 0.104.0 (`nix develop`, Skia prebuilt by `nix/skia.nix`).
E1 to E13 ran on Skia's **CPU raster**; no GPU ran. E14 ran on macOS 27.2.1 in a Swift script outside the repo.

| E | Question | Result |
|---|---|---|
| E1 | Do color filter, child-sampling shader, blender and a 9-tap loop compile? | Yes; `sample(layer, p)` fails, `layer.eval(p)` works |
| E2 | Do Malaysia's GLSL files compile as SkSL? | `oklab`, `window_shape`, `lens`: yes, verbatim. `precision`, `varying`/`gl_FragColor`, `sampler2D`: no |
| E3 | Does a backdrop `runtime_shader` move pixels? | Yes: a red/blue edge moved 8 px |
| E4 | Does a SkSL blender work as a paint blend? | Yes: difference of white over (200,100,50) gave (55,155,205) |
| E5 | Compile cost | 1.8 to 2.0 µs (200 tiny effects), 3 to 520 µs for the E1 set; CPU, no pipeline |
| E7 | Preprocessor, `out`, loops, `mat3`, arrays, `sk_FragCoord`, `dFdx`, `%`, dynamic bound | `#define`, `sk_FragCoord`, `dFdx`, `%` and a dynamic bound fail; the rest compile |
| E8 | Stock filters | `luma`, `matrix_row_major`, `hsla_matrix`, `Blender::arithmetic` build |
| E9 | Blur then veil-and-cap tone as a backdrop filter over an 8 px checker | 153 at the centre, 153 expected; outside the panel unchanged; 1.4 ms on CPU |
| E11 | A layer with a color filter applied on restore | Red inverted to cyan inside the layer; outside untouched |
| E12 | All 29 Skia blend modes by name; `plusDarker` as a blender | Named in source; the blender compiles |
| E13 | Does Skia's OKLab gradient match SwiftUI's? | Red to blue at 25, 50 and 75%: (197,73,110), (140,83,162), (81,71,209) against SwiftUI's (196,74,110), (140,83,162), (81,71,210); `in_premul` Yes gives (128,0,0,128) for red to transparent blue, as SwiftUI |
| E14 | What do SwiftUI's color modifiers and gradients compute? A Swift script, `ImageRenderer` at scale 1, pixels read as sRGB | The formulas in row 12; default gradients are OKLab (max difference 1 of 255 over 12 samples); `Color.mix(in: .perceptual)` equals the default gradient; angular starts at +x clockwise; elliptical radii are 0.5 of width and height |

## Not verified

- GPU behavior: first-draw pipeline compile, cache hits, frame cost on Vulkan.
- Whether SwiftUI shader output lives in sRGB-encoded or linear space, and
  premultiplication details; Skia surfaces here do math on encoded values.
- Whether macOS leaves user shaders alone under Reduce Transparency and Increase
  Contrast.
- Numeric match of macOS `plusDarker` and `MeshGradient` against the Skia ports.
- Pixel behavior of `scrollEdgeEffectStyle` (`soft`, `hard`) and of the real
  background extension; both need a window probe.
- `Color.mix`, and the center, angle and radius variants of the gradients.
- the pointer trigger and shimmer of `Glass.interactive`, `scrollEdgeEffectStyle.hard`, vibrancy recipe; the GPU
  cost of the Liquid Glass pass (blur levels, lens, table).
- Any compositor other than Malaysia. The `ext-background-effect-v1` and KDE
  behavior is read from the protocol XML only.
- glslang, SPIRV-Cross and shaderc as bridges.
- P1: that the group pass keeps clip, scale and rotation right (M3 goldens test it).
- The first effects driven by a time uniform (the edge of a scroll, a glass morph on the GPU): not built. The morph of `GlassContainer` interpolates its frames on the CPU and retargets from them (`plan`, `Frame::lerp`), so a `from`, `to` and `t` in the shader would duplicate that state; an effect that needs elapsed time, such as a scroll-edge glow, needs P1 for its container and a clock the shader reads. The registry already binds a `time` float by name.

## Rows for the coverage matrix

[`swiftui-appkit-coverage.md`](swiftui-appkit-coverage.md) has no row for these
(`grep -ci` of each name gives 0). Proposed for its section 15, status
`missing` unless noted:

| SwiftUI | AppKit | Pri |
|---|---|---|
| `colorEffect`, `layerEffect`, `distortionEffect`, `Shader`, `ShaderLibrary` | `CIKernel` | P3 |
| `blendMode` | `compositingFilter` | P2 |
| `compositingGroup`, `drawingGroup` | `canDrawSubviewsIntoLayer` | P2 |
| `mask` | `NSVisualEffectView.maskImage` | P2 |
| `blur`, `saturation`, `brightness`, `contrast`, `hueRotation`, `grayscale`, `colorInvert`, `colorMultiply`, `luminanceToAlpha` | `contentFilters`, `backgroundFilters` | P2 (`implemented` on images) |
| `MeshGradient` | — | P3 |
| `AngularGradient`, `EllipticalGradient`, `Gradient.ColorSpace` | `NSGradient` | P2 (`partial`) |
| `Canvas`, `GraphicsContext` | `NSGraphicsContext` | P2 |
| `TextRenderer` | — | P3 |
| `ImageRenderer` | — | P3 |
| `rotation3DEffect`, `transformEffect` | `CATransform3D` | P3 |
| `visualEffect`, `scrollTransition` | — | P3 |

## Decisions

1. **Fund P1** (a kit-owned render loop, L; [plan](#p1-plan)). It unlocks rows 2 to 4, 7 to 12, 17,
   18 and 25 and removes two existing workarounds. Without it the kit ships leaf
   effects only (images, symbols, shapes, `Canvas`). Planned on 2026-11-21; nothing of P1 is implemented (P2's uniforms are, see the table).
2. **Where shared shader functions live.** In the kit, with `grade-graphics`
   including them (one source, reversed dependency), or kept apart with a
   raster test that compares each pair.
3. **Whether to publish the SkSL API now.** `Shader`, `ShaderLibrary` and
   `.layer_effect` with leaf support first, or keep SkSL internal until P1
   lands. The name `ShaderArgument` is also open.
