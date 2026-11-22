# macOS Liquid Glass

**Goal:** describe Liquid Glass as macOS 27.2.1 renders and configures it, with
measured values, so that a compositor and the kit can reproduce it and a
parity check can say how close they are.
**Scope:** `NSGlassEffectView`, SwiftUI's `glassEffect`, `GlassEffectContainer`,
the glass buttons, the glass the window chrome uses (toolbar, sidebar, tab
control, scroll-edge bar) and the real menus and popovers. It starts at the
layer stack and ends at the model, its error, the shader descriptions and what
is not measured. The ordinary materials are in
[`macos-materials.md`](macos-materials.md) and the desktop surfaces in
[`macos-blur.md`](macos-blur.md).
**Reader:** whoever writes the glass pass of the compositor, the kit's glass or
a parity check.
**Input:** a probe that builds the views in its own process, reads their layer
tree and captures their window ([Method](#2-method)).
**Output:** the measured model, the configuration per variant, state and size,
a parity procedure with its tolerance, and the shader descriptions.

## Steps

1. Read [Findings](#1-findings).
2. Read [Method](#2-method) before trusting a number: it says where each value
   comes from.
3. To configure a glass, take the look of [Configuration](#4-configuration) and
   the size rules of [Size](#5-size).
4. To draw it, follow [The model](#6-the-model) and
   [Shader descriptions](#9-shader-descriptions).
5. To check a result, run [Parity check](#8-parity-check).

---

## 1. Findings

1. **One filter.** A glass is a `CABackdropLayer` at layer scale 0.5 with the
   private filter `glassBackground` (77 named inputs) over a `CASDFLayer` that
   holds the shape. AppKit and SwiftUI build the same stack. `[R]`
2. **Half-resolution backdrop.** The backdrop is a 2x2 box average read back with
   bilinear filtering at the texel centres: a one-pixel line spreads as
   1/8, 3/8, 3/8, 1/8. The kit's test reproduces three line captures within one
   level. `[C]`
3. **The glass refracts.** Inside the edge, over `InnerRefractionHeight` pixels,
   the lens reads the backdrop from further in, along the normal, by a
   scale-free exponential profile ([section 6.2](#62-lens)). The earlier verdict
   of "no lensing" ([section 11](#11-corrections)) came from the shell
   surfaces and 1x boundary tests; a lens needs structure at the edge to show.
   `[C]`
4. **Blur.** `BlurRadius` is a Gaussian sigma in half-resolution texels, so
   2 x radius x the share at that depth in pixels at scale 1, capped near
   28 px. The share runs from `BlurOpacity0` deep inside to `BlurOpacity1` one
   pixel inside. Clear has radius 0 and no blur in the active look. `[C]`
5. **Tone.** `out = black + (white - black) x saturate(c) x (1 - (1 - cap) x luma)`,
   mixed with the fill by its alpha, in the display's device values. It
   reproduces every variant without a blur fill within 0.6 levels (rms 0.35);
   the blur fill is a tone delta of 0.6 to 8.5 levels. `[C]`
6. **State changes the inputs.** The active look (a key window or a
   non-activating panel) and the inactive look (a non-key window) differ;
   Reduce Transparency and Increase Contrast change them again. They are read
   per process, with the probe's own flags. `[R]`
7. **Size changes the inputs.** Eleven inputs are piecewise linear in the shorter
   side `m` of the shape ([section 5](#5-size)); the blur disappears below
   `m` = 64. `[R]`
8. **Rim, ring and shadow** are two angle profiles times two distance profiles
   (rank 2: rms 0.7 to 1.3 levels over the circle captures). `[C]`
9. **Tint** is an alpha blend between the glass and the tint colour as the
   glass shows it. `[C]`
10. **A container** shares one backdrop and one SDF between its children;
    `smoothness` equals the spacing; the union is a smooth minimum whose
    blend is 0.8 of the spacing. `glassEffectUnion` merges the shapes into one
    element before the filter reads its size. `[R]` `[C]`
11. **Real menus and popovers** are this glass, the Dock's tone matches
    variant 3 (rms 2.4 levels) and the window sidebar is variant 18. `[C]` `[R]`
12. **Interaction.** Forcing `_interactionState` to hovered or pressed maps every
    pixel of the glass as `out' = gain x out + offset`, the same for regular and
    clear, within 1 level ([section 7](#7-interaction-merging-and-morphing)). `[C]`
13. **The Appearance › Liquid Glass slider** (`NSGlassTintAmount`, 0 to 1) moves
    the blur fill, the face fill and cap and the read scale of every
    `glassBackground`: `ctappearance::liquid_glass::tinted` reproduces the tone
    within 2.3 levels and the width of a step within 2 px
    ([section 13](#13-the-appearance--liquid-glass-slider-nsglasstintamount-measured-2026-11-22)). `[R]` `[C]`
14. **Not measured:** what makes the pointer trigger it (posted events changed
    nothing), scale 2, the system switches, tint at every alpha, the outer
    refraction ([section 12](#12-not-measured)).

## 2. Method

Three sources, in this order of trust. Each value says which one it comes from:
`[C]` captured, `[R]` read, `[A]` API.

1. `[C]` **Captured:** `screencapture -l <window>` of a probe window that holds
   the glass over a backdrop image drawn pixel for pixel. The reference is the
   same window without the glass, or the pixels outside the glass. PNGs are
   read raw (PIL on the file bytes): the panel's profile is wide gamut and the
   system mixes in the display's device space, so converting with `sips`
   moves saturated colours by 10 to 20 levels and leaves grays alone. A
   non-activating `NSPanel` shows the active look without the app being active,
   and a capture of it is pixel-identical to one of a key window (difference 0
   in the 7 pairs compared), so no focus is needed.
2. `[R]` **Read:** the layer tree of the probe's own process. The probe builds an
   `NSGlassEffectView` (or a SwiftUI `NSHostingView`), walks `CALayer.sublayers`
   and reads every input of every `CAFilter` and every property of every
   `CASDFEffect` with key-value coding. Nothing is injected and no setting
   changes: Reduce Transparency and Increase Contrast are flags on the probe's
   command line. A window that is not key reads the inactive look.
3. `[A]` **API:** the SDK headers and `.swiftinterface` files.

A captured value beats a read one. The probes are in
`~/src/probes/liquidglass/` (`README.txt`), the captures in
`~/src/captures/liquidglass/`, the 8 163 inputs in
[`macos-liquid-glass-inputs.csv`](macos-liquid-glass-inputs.csv).

**Limits.** One display, scale 1, 2560 x 1440. Scale 2 needs a HiDPI display
mode, which is not authorised: every 2x cell is `not measured`. The session
screen was locked for part of the run; window-id captures still worked.

## 3. How a glass is built

| Layer | What it is |
|---|---|
| `NSGlassEffectView` backing layer | holds the content-holder layer and a hosted SwiftUI root (`_NSCoreHostingView`) |
| `SwiftUI.SDFLayer` | the glass of one `glassEffect` or one container |
| `CALayer` with `vibrantColorMatrix` (backdrop-aware) | the interaction veil: layer opacity 0 at rest, 0.1 hovered, 0.05 pressed through `_interactionState`; its effect on the pixels is the affine map of [section 7](#7-interaction-merging-and-morphing) |
| `CABackdropLayer`, filter `glassBackground`, scale 0.5, group `SwiftUI:<id>` | the glass |
| `CASDFLayer` (`CASDFOutputEffect`, `smoothness` 8) with `CASDFElementLayer` children | the shape: `mode` bounds, `operation` union, `cornerRadius`, continuous curve |
| `SDFPortalLayer` | the SDF read by the content |
| `CASDFLayer` with `CASDFKeyFillHighlightEffect`, opacity 0, filter `vibrantColorMatrix` | the highlight of the interactive state; inert at rest |
| tint: `CASDFLayer` with `CASDFGradientEffect` (distances -1, 0, 10) and a `CASDFLayer` with `CASDFFillEffect` and `destIn` | present only with a tint |
| `_contentLensing` 2: filter `glassForeground` (refraction -16 height 16, aberration -5 height 6.6) | the lens for the content above the glass |

A container (`NSGlassEffectContainerView`, `GlassEffectContainer`) keeps one
backdrop layer and one `CASDFLayer` for all its children, with one element per
child and `smoothness` equal to the spacing (8 for a lone glass). The inputs are
computed from each child's size; `glassEffectUnion` makes the children one
element, so the size is the union's.

## 4. Configuration

All 77 inputs, in the active look of a 200 x 160 shape. The size rules of
[section 5](#5-size) change eleven of them. Inputs the model does not read
say so; their values are in the CSV.

| Group | `glassBackground` input | Meaning | Regular dark | Regular light | Clear |
|---|---|---|---|---|---|
| lens | `InnerRefractionAmount` | Pixels the lens reads inward at the edge (negative = inward); the displacement is 0.831 times its magnitude | -60 | -60 | -60 |
| lens | `InnerRefractionHeight` | Depth in pixels of the lens zone; the profile is a function of depth over height | 20 | 20 | 20 |
| lens | `OuterRefractionAmount` | Lens outside the shape; no displacement measured outside the edge | 40 | 40 | 32 |
| lens | `OuterRefractionHeight` | Height of the outer zone; not measured | 32 | 32 | 20 |
| lens | `RefractionDistance0` | Not measured | -1 | -1 | -1 |
| lens | `RefractionDistance1` | Not measured | 0 | 0 | 0 |
| lens | `RefractionOpacity` | Scales the outer refraction; not measured | 0.6 | 0.6 | 0 |
| blur | `BlurDistance0` | Depth where the share is BlurOpacity0 (minus half the shorter side) | -80 | -80 | -80 |
| blur | `BlurDistance1` | Depth where the share is BlurOpacity1 | -1 | -1 | -1 |
| blur | `BlurDistance2` | Distance of BlurOpacity2 | 0 | 0 | 0 |
| blur | `BlurDistance3` | Distance of BlurOpacity3 | 0 | 0 | 0 |
| blur | `BlurOpacity0` | Share of the radius at BlurDistance0 (deep inside) | 0.8 | 0.8 | 0 |
| blur | `BlurOpacity1` | Share at BlurDistance1 (1 px inside) | 0.4 | 0.4 | 0 |
| blur | `BlurOpacity2` | Share at distance 0 (the edge), first of two | 0.5 | 0.5 | 0 |
| blur | `BlurOpacity3` | Share at distance 0, second; outside the shape | 1 | 1 | 0 |
| blur | `BlurRadius` | Gaussian sigma in half-resolution texels: 2 x radius pixels at scale 1, capped near 28 px | 5 | 5 | 0 |
| face | `FaceColorMatrixBlack` | Output for a black input | 0.125 | 0.4 | 0.125 |
| face | `FaceColorMatrixFillColor` | Colour mixed over the tone by its alpha | (0.125, 0.125, 0.125, 0) | (1, 1, 1, 0) | (1, 1, 1, 0) |
| face | `FaceColorMatrixMaxLuma` | HDR cap, same as the SDR one in SDR | 0.45 | 1 | 1 |
| face | `FaceColorMatrixMaxLumaSDR` | Cap: the tone is scaled by 1 - (1 - cap) x luma | 0.45 | 0.94 | 1 |
| face | `FaceColorMatrixSaturation` | Saturation about the Rec. 709 luma, in device values | 1.3 | 1.2 | 1.06 |
| face | `FaceColorMatrixWhite` | Output for a white input (before the cap) | 1.125 | 1.03 | 1.08 |
| face | `FaceOpacity` | Share of the face tone; 0 leaves the half-resolution backdrop | 1 | 1 | 1 |
| blur fill | `BlurFillBlurRadius` | A second blur; its effect is a tone delta of 0.6 to 8.5 levels | 8 | 8 | 0 |
| blur fill | `BlurFillDarkenOpacity` | Dark scheme: darkens mid-tones by up to 6 levels | 0.675 | 0 | 0 |
| blur fill | `BlurFillLightenOpacity` | Light scheme: lightens mid-tones by up to 8 levels | 0 | 0.675 | 0 |
| blur fill | `BlurFillNormalOpacity` | Not measured | 0 | 0 | 0 |
| bleed | `BleedAmount` | Under 1.5 levels on gray; not modelled | 56 | 56 | 0 |
| bleed | `BleedBlurRadius` | idem | 56 | 56 | 0 |
| bleed | `BleedColorMatrixBlack` | idem | 0.125 | 0.9 | 0.75 |
| bleed | `BleedColorMatrixFillColor` | idem | nil | nil | nil |
| bleed | `BleedColorMatrixSaturation` | idem | 1 | 1.2 | 1.2 |
| bleed | `BleedColorMatrixWhite` | idem | 0.5 | 1 | 1 |
| bleed | `BleedDarkenBlend` | idem | false | true | true |
| bleed | `BleedDistance0` | idem | 1 | 1 | 1 |
| bleed | `BleedDistance1` | idem | 0 | 0 | 0 |
| bleed | `BleedHeight` | idem | 56 | 56 | 0 |
| bleed | `BleedOpacity` | idem | 0.8 | 0.5 | 0 |
| shadow | `ShadowAmount` | Clear only; no visible shadow | 0 | 0 | 75 |
| shadow | `ShadowBlurRadius` | Not measured | 0 | 0 | 0 |
| shadow | `ShadowColorMatrixBlack` | Not measured | 0 | 0 | 0 |
| shadow | `ShadowColorMatrixFillColor` | Shadow colour | (0, 0, 0, 0.3) | (0, 0, 0, 0.3) | (0, 0, 0, 0.1) |
| shadow | `ShadowColorMatrixSaturation` | Not measured | 1 | 1 | 1.2 |
| shadow | `ShadowColorMatrixWhite` | Not measured | 1 | 1 | 1 |
| shadow | `ShadowDistanceOffset` | Not measured | 0 | 0 | 0 |
| shadow | `ShadowHeight` | idem | 0 | 0 | 64 |
| shadow | `ShadowOffset` | Offset of the outer shadow (x, y) | (0, 8) | (0, 8) | (0, 8) |
| shadow | `ShadowOpacity` | Strength of the outer shadow (regular) | 0.6 | 0.4 | 0 |
| shadow | `ShadowRadius` | Blur of the outer shadow in points | 24 | 24 | 0 |
| shadow | `ShadowVibrancyContribution` | Not measured | 0 | 0 | 0 |
| ring shadow | `RingShadowBlurRadius` | Blur of the ring | 5 | 5 | 0 |
| ring shadow | `RingShadowMask` | Not measured | 1 | 1 | 0 |
| ring shadow | `RingShadowOffset` | Inner dark ring, pushed downward by this offset | 8 | 8 | 0 |
| ring shadow | `RingShadowOpacity` | 0.06 in the active regular glass; 0 in clear | 0.06 | 0.06 | 0 |
| ring shadow | `RingShadowStrokeWidth` | Width of the ring | 4 | 4 | 0 |
| highlight | `KeyFillHighlightAmount` | Strength of the lobe; 0 turns the side line white | 0.4 | 0.4 | 0.4 |
| highlight | `KeyFillHighlightAngle` | pi/2: the key light is above, the fill below | 1.5708 | 1.5708 | 1.5708 |
| highlight | `KeyFillHighlightColorBias` | Darkens the edge outside the lobe | -0.25 | -0.25 | -0.25 |
| highlight | `KeyFillHighlightEffectOffset` | Not measured | -1 | -1 | -1 |
| highlight | `KeyFillHighlightHeight` | Height of the 1 px key and fill line | 1 | 1 | 1 |
| highlight | `KeyFillHighlightSpread` | Angular width of the lobe | 1.309 | 1.6755 | 1.309 |
| highlight | `KeyFillHighlightSpreadSDR` | SDR spread | 1.309 | 1.85 | 1.309 |
| hdr | `Clamp` | No visible effect in SDR | 1.3083 | 1.0696 | 1.1917 |
| hdr | `ClampPreserveHue` | No visible effect | false | false | false |
| hdr | `MaxHeadroom` | HDR | 9999 | 9999 | 9999 |
| hdr | `SDRGradientDistance0` | HDR | 0 | 0 | 0 |
| hdr | `SDRGradientDistance1` | HDR | 0 | 0 | 0 |
| hdr | `SDRHoldingToneEnabled` | HDR | false | false | false |
| hdr | `SDRHoldingToneWhite` | HDR | 1 | 1 | 1 |
| hdr | `SDRShadowOpacity` | HDR | 0 | 0 | 0 |
| aberration | `AberrationAmount` | nil in every variant read | nil | nil | nil |
| aberration | `AberrationAngle` | nil | nil | nil | nil |
| aberration | `AberrationHeight` | nil | nil | nil | nil |
| aberration | `AberrationOffset` | nil | nil | nil | nil |
| wiring | `SourceSublayerName` | Name of the SDF layer the filter reads | @0 | @0 | @0 |

### 4.1 State

| Look | State | Blur r | Blur shares | White | Black | Sat | Cap | Fill | Lens |
|---|---|---|---|---|---|---|---|---|---|
| regular dark | active | 5 | (0.8, 0.4, 0.5, 1) | 1.125 | 0.125 | 1.3 | 0.45 | (0.125, 0.125, 0.125, 0) | -60/20 |
| regular dark | inactive | 5 | (0.8, 0.8, 1, 1) | 1.125 | 0.08 | 1.3 | 0.45 | (0.125, 0.125, 0.125, 0) | -60/20 |
| regular dark | Reduce Transparency | 50 | (1, 1, 0.1, 0.1) | 0.2 | 0.2 | 0.7 | 0.45 | (0, 0, 0, 0) | 0/0 |
| regular dark | Increase Contrast | 5 | (0.8, 0.4, 0.5, 1) | 0.3 | 0.1 | 0.8 | 0.45 | (0.12, 0.12, 0.12, 0.4) | -60/20 |
| regular dark | both | 50 | (1, 1, 0.1, 0.1) | 0.3 | 0.1 | 0.8 | 0.45 | (0.12, 0.12, 0.12, 0.4) | 0/0 |
| regular light | active | 5 | (0.8, 0.4, 0.5, 1) | 1.03 | 0.4 | 1.2 | 0.94 | (1, 1, 1, 0) | -60/20 |
| regular light | inactive | 5 | (0.8, 0.8, 1, 1) | 0.96 | 0.4 | 1.2 | 0.94 | (1, 1, 1, 0) | -60/20 |
| regular light | Reduce Transparency | 50 | (1, 1, 0.1, 0.1) | 1.03 | 0.9 | 0.9 | 0.94 | (1, 1, 1, 0) | 0/0 |
| regular light | Increase Contrast | 5 | (0.8, 0.4, 0.5, 1) | 1.03 | 0.5 | 0.8 | 0.94 | (1, 1, 1, 0.45) | -60/20 |
| regular light | both | 50 | (1, 1, 0.1, 0.1) | 1.03 | 0.5 | 0.8 | 0.94 | (1, 1, 1, 0.45) | 0/0 |
| clear dark | active | 0 | (0, 0, 0, 0) | 1.08 | 0.125 | 1.06 | 1 | (1, 1, 1, 0) | -60/20 |
| clear dark | inactive | 10 | (1, 1, 1, 1) | 0.8 | 0.05 | 1 | 1 | (1, 1, 1, 0.05) | -60/20 |
| clear dark | Reduce Transparency | 50 | (1, 1, 1, 1) | 0 | 0.125 | 0.7 | 1 | (0, 0, 0, 0.25) | 0/0 |
| clear dark | Increase Contrast | 0 | (0, 0, 0, 0) | 0.6 | 0.1 | 0.8 | 1 | (0, 0, 0, 0.35) | -60/20 |
| clear dark | both | 50 | (1, 1, 1, 1) | 0.6 | 0.1 | 0.8 | 1 | (0, 0, 0, 0.35) | 0/0 |
| clear light | active | 0 | (0, 0, 0, 0) | 1.08 | 0.125 | 1.06 | 1 | (1, 1, 1, 0) | -60/20 |
| clear light | inactive | 10 | (1, 1, 1, 1) | 0.95 | 0.2 | 1 | 1 | (1, 1, 1, 0.1) | -60/20 |
| clear light | Reduce Transparency | 50 | (1, 1, 1, 1) | 0 | 0.125 | 0.9 | 1 | (1, 1, 1, 0.25) | 0/0 |
| clear light | Increase Contrast | 0 | (0, 0, 0, 0) | 0.6 | 0.1 | 0.8 | 1 | (0, 0, 0, 0.35) | -60/20 |
| clear light | both | 50 | (1, 1, 1, 1) | 0.6 | 0.1 | 0.8 | 1 | (0, 0, 0, 0.35) | 0/0 |

The inactive look reads the same pipeline with other inputs: gray 128 gives 117
(regular dark), 171 (regular light), 116 and 158 (clear), which the face formula
reproduces within 1.5 levels. Reduce Transparency over regular dark is a
constant 51 (`white` = `black` = 0.2); its blur radius is 50 and its blur is
capped. Increase Contrast adds a fill of alpha 0.4 and narrows `white` to
`black`. Clear is the same in light and dark in the active look.
`subdued`, `scrim` and `contentLensing` change inputs too; they are in
the CSV only through the probe's `-set` flags, not captured.

### 4.2 Variants

`NSGlassEffectView._variant` has 23 values; 13 and 15 have no `glassBackground`.
Active look, dark, 200 x 160:

| Variant | Blur r | Blur shares | White | Black | Sat | Cap | Fill (dark) | Lens | Outer lens | Shadow | Seen as |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 5 | (0.8, 0.4, 0.5, 1) | 1.125 | 0.125 | 1.3 | 0.45 | (0.125, 0.125, 0.125, 0) | -60/20 | 40/32 | 0.6 | same as 1 |
| 1 | 5 | (0.8, 0.4, 0.5, 1) | 1.125 | 0.125 | 1.3 | 0.45 | (0.125, 0.125, 0.125, 0) | -60/20 | 40/32 | 0.6 | `style` regular, `.regular`, real menus |
| 2 | 0 | (0, 0, 0, 0) | 1.08 | 0.125 | 1.06 | 1 | (1, 1, 1, 0) | -60/20 | 32/20 | 0 | `style` clear, `.clear` |
| 3 | 0 | (0, 0, 0, 0) | 0.92 | 0.05 | 1.25 | 0.75 | (0, 0, 0, 0.02) | -28.8/12.8 | 0/0 | 0 | Dock (tone) |
| 4 | 0.5 | (1, 0.5, 0.5, 1) | 0.3 | 0.05 | 0.8 | 1 | (0, 0, 0, 0) | -41.6/9.6 | 0/0 | 0 |  |
| 5 | 0.5 | (1, 0.5, 0.5, 1) | 0.3 | 0.05 | 0.8 | 1 | (0, 0, 0, 0) | -60/20 | 0/0 | 0 |  |
| 6 | 3 | (0.5, 1, 0.5, 1) | 1 | 0.5 | 1 | 1 | nil | -57.733/6 | 0/0 | 0 |  |
| 7 | 0 | (0, 0, 0, 0) | 1.08 | 0.125 | 1.06 | 1 | (1, 1, 1, 0) | -60/20 | 32/20 | 0 |  |
| 8 | 0 | (0, 0, 0, 0) | 1.08 | 0.125 | 1.06 | 1 | (1, 1, 1, 0) | -60/20 | 32/20 | 0 |  |
| 9 | 0 | (0, 0, 0, 0) | 0.8 | 0.125 | 1.2 | 1 | (0, 0, 0, 0) | -80/25 | 11.424/5.712 | 0 |  |
| 10 | 5 | (0.8, 0.4, 0.5, 1) | 1.125 | 0.125 | 1.3 | 0.45 | (0.125, 0.125, 0.125, 0) | -60/20 | 40/32 | 0.6 |  |
| 11 | 0 | (0, 0, 0, 0) | 1 | 0 | 1 | 1 | nil | -50/20 | 10/16 | 0 |  |
| 12 | 5 | (0.8, 0.4, 0.5, 1) | 1.125 | 0.125 | 1.3 | 0.45 | (0.125, 0.125, 0.125, 0) | -60/20 | 40/32 | 0.6 |  |
| 13 | no `glassBackground` | | | | | | | | | | |
| 14 | 0 | (0, 0, 0, 0) | 1 | 0 | 1 | 1 | (0, 0, 0, 0) | 0/0 | 0/0 | 0 | `.identity` |
| 15 | no `glassBackground` | | | | | | | | | | |
| 16 | 2.5 | (1, 1, 0.32, 1) | 0.5 | 0.1 | 1.4 | 1 | (0.15, 0.15, 0.15, 0.75) | -48/20 | 0/0 | 0 |  |
| 17 | 5 | (0.8, 0.4, 0.5, 1) | 1.125 | 0.125 | 1.3 | 0.35 | (0, 0, 0, 0) | -60/20 | 40/32 | 0.6 |  |
| 18 | 5 | (0.8, 0.4, 0.5, 1) | 0.5 | 0.1 | 0.6 | 1 | (0, 0, 0, 0) | -60/20 | 40/32 | 0 | window sidebar chrome |
| 19 | 5 | (0.8, 0.4, 0.5, 1) | 0.5 | 0.1 | 0.6 | 1 | (0, 0, 0, 0) | -60/20 | 40/32 | 0 |  |
| 20 | 0 | (0, 0, 0, 0) | 1 | 0.06 | 1 | 1 | (1, 1, 1, 0) | -48/15.2 | 0/0 | 0.02 |  |
| 21 | 5 | (0.8, 0.4, 0.5, 1) | 1.125 | 0.125 | 1.3 | 0.45 | (0.125, 0.125, 0.125, 0) | -60/5 | 40/32 | 0.6 |  |
| 22 | 5 | (0.8, 0.4, 0.5, 1) | 1.125 | 0.125 | 1.3 | 0.45 | (0.125, 0.125, 0.125, 0) | -60/20 | 40/32 | 0.6 |  |

The tone formula reproduces the gray ramp of every variant without a blur fill
within 0.6 levels. Which surface uses which variant is known for: regular =
1 (`style` regular, menus and popovers: the application menu reads 32, 122, 146
over 0, 128, 255 as in section 5), clear = 2, the window sidebar = 18 (a read
match: two inputs differ, both size-dependent), `.identity` = 14 and the Dock =
3 (the Dock's gray ramp matches the formula of variant 3 within 2.4 rms; it is a
match of tone, not a read).

## 5. Size

Let `m` be the shorter side of a shape in points. In the active look these
inputs follow `clamp(a x m + b, lo, hi)`; all were read at 11 sizes from
20 x 20 to 560 x 400 and the rules reproduce every reading.

| Input | Regular | Clear |
|---|---|---|
| `InnerRefractionAmount` | `-clamp(m / 2, 0, 60)` | `-clamp(0.65 m, 0, 60)` |
| `InnerRefractionHeight` | `min(m / 4, 20)` | `min(0.36 m, 20)` |
| `BlurOpacity0` | `clamp((m - 64) / 120, 0, 0.8)` | 0 |
| `BlurOpacity1` | half of `BlurOpacity0` | 0 |
| `BlurDistance0` | `-m / 2` | `-m / 2` |
| `FaceColorMatrixMaxLumaSDR` (dark) | `clamp(0.84 - 0.00375 m, 0.45, 0.6)` | 1 |
| `BleedAmount`, `BleedHeight` | `0.35 m` | 0 |
| `OuterRefractionAmount` / `Height` | `max(m / 4, 16)` / `max(m / 5, 16)` | `0.2 m` / `0.125 m` |
| `ShadowOpacity` (dark / light) | `clamp(0.04 + 0.00583 (m - 64), 0.04, 0.6)` / `... 0.00375 ..., 0.4` | 0 |
| `ShadowRadius` | `clamp(4 + 0.2083 (m - 64), 4, 24)` | 0 |
| `ShadowAmount`, `ShadowHeight` | 0 | `min(0.625 m, 75)`, `0.4 m` |

## 6. The model

The model is a pure function of the backdrop, the shape and the look. Its Rust
reference is `ctappearance::liquid_glass`; tokens are
`components.liquid-glass` ([ADR 0095](../decisions.md)). Values are device
values 0 to 255; distances are points at scale 1; `d` is the signed distance to
the shape (negative inside) and `n` its outward normal.

1. **Half resolution.** `hr = bilinear(box2(backdrop))`: the backdrop averaged
   over 2x2 blocks and read at the texel centres.
2. **Lens** (inside, `0 <= -d < height`): `u = -d / height`,
   `s(u) = (exp(-k u) - exp(-k)) / (1 - exp(-k))` with `k` = 3.462 and the
   displacement `|amount| x 0.831 x s(u)` inward. The sample point is
   `p - n x displacement`.
3. **Blur.** A Gaussian of `sigma = min(2 x r x share(d), 28)` over `hr`, where
   `share` is the piecewise-linear profile through
   `(BlurDistance0, BlurOpacity0)`, `(BlurDistance1, BlurOpacity1)` and
   `(0, BlurOpacity2)`. The compositor reads blurred copies at sigma
   0, 1, 2, 3, 4, 6, 8, 10, 13, 16, 20, 24 and 28 and interpolates.
4. **Face.** `y` is the Rec. 709 luma of the sample;
   `sat = clamp(y + S (c - y), 0, 1)`;
   `tone = black + (white - black) x sat x (1 - (1 - cap) x y)`;
   `out = (1 - a) tone + a fill + delta(y) / 255` with `a` the fill's alpha and
   `delta` the nine-point table of the blur fill (dark `0.1, -6.0, -6.2, -5.3,
   -4.0, -3.1, -2.2, -1.8, -0.6`; light `2.0, 3.3, 4.3, 5.7, 7.0, 7.4, 6.5, 4.6,
   1.0`; others 0).
5. **Tint.** `out = (1 - alpha) glass + alpha T`, where `T` is the tint as the
   glass shows it. Over regular dark, red gives `(254, 4, 4)` at backdrop 32
   and `(251, 22, 22)` at 224; white gives 255 and black 0; over clear, red
   gives `255 x (0.62 + 0.37 g)` for a glass level `g`. The inputs that carry
   it are the `vibrantColorMatrix` of the tint layer (`0.7638 0.2145 0.0217`,
   `0.0638 0.9146 0.0217`, `0.0637 0.2146 0.7217`, bias 0.1 dark and -0.1
   light, clamp 1.2, alpha = the tint's alpha).
6. **Rim, ring and shadow.** `levels(d, angle) = A1(angle) V1(d) + A2(angle)
   V2(d)` added to every channel, for a normal at `angle` degrees folded to
   0 to 90 (0 = a side, 90 = top and bottom). `A` has 12 bins of 7.5 degrees
   and `V` 35 distance bins: 0.5 px from -8 to 4, then 2 px to 26. Over clear
   the top and bottom edges gain up to +66 on the first inside pixel and a tail
   of 23, 13, 9, 5, 3, 2 levels in the next pixels; the sides have a dark line of
   -39 to -85 on the first pixel outside. Over regular dark the edge gains +7 to
   +25 and the shadow lies outside: -15 at 1 px, -10 at 3 px, -6 at 10 px.
   The tables are measured on a 200 px circle; the shadow of smaller shapes is
   weaker by the `ShadowOpacity` rule and is not scaled in the model.
7. **Union.** Two shapes of a container with spacing `s` join through the smooth
   minimum `min(a, b) - h^2 k / 4` with `h = max(k - |a - b|, 0) / k` and
   `k = 0.8 s`.

8. **Slider** (`t` = `NSGlassTintAmount`, clamped to 0..1; the active look only;
   [section 13](#13-the-appearance--liquid-glass-slider-nsglasstintamount-measured-2026-11-22)).
   The backdrop is read through a box of 2, 4 or 8 pixels instead of 2 (the
   layer scale 0.5, 0.25, 0.125 at the thresholds of the tokens). A second
   Gaussian of sigma `2 x BlurFillBlurRadius` = 16 px (the blur fill, over the
   same read) is mixed with the lens-read blur by `BlurFillNormalOpacity` `n`:
   `seen = (1 - n) blur + n fill`. The face reads `seen` with the fill, the cap
   (`max_luma`) and the delta table of `t` (the table of step 4 plus the shift
   table of the position, interpolated every 0.25). The inputs that are linear
   in `t` are the knots of `GlassTint` at `t` = 0, 0.5 and 1.

### 6.1 Error

RMS of the reference against the captures, levels of 255, by distance to the
edge `d`:

| Capture | d < -40 | -40..-20 | -20..-8 | -8..-2 | -2..0 | 0..10 |
|---|---|---|---|---|---|---|
| clear, gray | 0.1 | 0.1 | 0.1 | 0.7 | 17 | 7 |
| clear, white box | 0.1 | 0.1 | 0.1 | 10.5 | 16 | 7 |
| clear, saw (smooth) | 0.3 | 0.3 | 1.3 | 4.3 | 16 | 8 |
| clear, 8 px checker | 0.3 | 0.6 | 38 | 79 | 81 | 10 |
| clear, 16 px grid | 0.3 | 0.9 | 35 | 55 | 59 | 9 |
| clear, gray ramp and colours | 2.2 | 2.2 | 2.0 | 2.6 | 12 | 7 |
| regular dark, gray | 1.5 | 1.5 | 1.2 | 1.6 | 6.7 | 5.8 |
| regular dark, white box | 3.1 | 6.3 | 3.8 | 18.5 | 30 | 1.9 |
| regular dark, edge | 7.6 | 5.9 | 5.0 | 5.7 | 12 | 18.5 |
| regular dark, gray ramp and colours | 6.8 | 4.7 | 4.2 | 5.7 | 12 | 11 |
| regular light, gray | 0.8 | 0.8 | 1.2 | 1.0 | 13 | 8.7 |
| regular light, gray ramp and colours | 12 | 11 | 10 | 11 | 19 | 8 |

The interior of clear is exact. The lens reproduces smooth backdrops down to 3
pixels from the edge and the structure of high-frequency ones (the checker's
bars and corner folds) but not their phase: 8 px cells put every pixel of the
zone within a displacement error of 1 to 2 pixels. The first pixel is the
rim. The regular blur is a Gaussian fitted at four depths (sigma 5.5, 6.5, 8.0,
8.5 against the model's 4.8, 5.2, 6.0, 7.2 at 40, 60, 100, 160 px in a
400 px shape): the model blurs the middle depths up to 2 px less than macOS.
Regular light chroma is the weakest cell: its blur fill lightens per channel.

### 6.2 Lens

Displacement along the inward normal, measured with a sawtooth backdrop
(`2 x (x mod 116)` levels) through clear glass against the same glass without
refraction (slope 0.955 levels per level, resolution 0.5 px). For
`InnerRefractionAmount` -60 and heights 10, 20 and 40 the readings collapse onto
one curve of `u = depth / height`; they are proportional to the amount (-30 gives
half, -120 gives double):

| u | 0.025 | 0.075 | 0.125 | 0.225 | 0.325 | 0.425 | 0.525 | 0.675 | 0.825 | 0.975 |
|---|---|---|---|---|---|---|---|---|---|---|
| displacement / 60 | 0.78 | 0.64 | 0.51 | 0.37 | 0.25 | 0.18 | 0.115 | 0.057 | 0.013 | 0 |

The fit `0.831 (exp(-3.462 u) - exp(-3.462)) / (1 - exp(-3.462))` has rms 0.63
px. The field is isotropic: the left, right, top and bottom edges agree to
0.5 px, and the inward normal at a corner follows the SDF gradient. No
displacement was measured outside the edge (`OuterRefraction*`), over clear
(`RefractionOpacity` 0) or regular (0.6).

## 7. Interaction, merging and morphing

- **Interactive.** `effectIsInteractive` changes no layer input at rest. The
  pointer response goes through `_NSFlexiglassInteractionDriver` and a
  presentation modifier. Forcing `_interactionState` (1 hovered, 2 pressed; 3
  crashes the probe) on the probe's own glass maps every pixel inside the shape
  as `out' = clamp(gain x out + offset)` on the device value, the same for
  regular and clear and with or without `effectIsInteractive` (captures
  `b10-int*`, 2026-11-22):

  | Scheme | Hovered (gain, offset) | Pressed (gain, offset) |
  |---|---|---|
  | dark | 0.9276, +0.0962 | 0.9676, +0.0466 |
  | light | 0.9328, -0.0262 | 0.9678, -0.0124 |

  A clear glass over a 16 px grid (195 to 202 distinct levels from 32 to 249)
  fits with rms 0.17 to 0.24 and at most 1.0 level; the regular glass over gray
  128 lands within 1 level (122 to 137 hovered and 130 pressed in dark; 188 to
  169 and 178 in light). Only gray was measured: the colour matrix is assumed
  to act per channel. `ctappearance::liquid_glass::interacted` applies the map
  `[C]`. A real pointer did not trigger the state: `mouse` events (move onto the
  glass, press, release; `NSEvent.mouseLocation` confirmed the position) over a
  key, interactive glass changed no pixel in six captures, regular and clear
  over gray and a grid. The scale, the shimmer and the durations are not
  measured.
- **Merging.** Two 100 x 60 capsules of a clear container: the glass at the
  middle of the gap is 6 rows thick at spacing 0, gap 0 (touching), 32 at
  spacing 20 gap 0, 4 at gap 10, 0 at gap 20; 44 at spacing 40 gap 0, 32 at gap
  10, 2 at gap 20, 0 at gap 30. The smooth minimum above reproduces them within
  8 rows.
- **Morphing.** A shape entering a container with `matchedGeometry` grows from
  an 8.8 px circle at its centre to its frame while its neighbour slides, both
  on the SwiftUI animation (`.smooth`: 90 % at 0.32 s, settled at 0.6 s);
  `materialize` starts at 1.13 x 1.2 times its frame and shrinks into it. The
  glass follows the layer frames; `smoothness` stays at the container's
  spacing `[R]`.
- **Window chrome.** Toolbar buttons are three 36 px circles in one backdrop
  (regular, small: no blur, `max-luma` 0.6); the tab control's pill and the
  scroll-edge bar (`safeAreaInset`, 48 px) are regular glass; the sidebar of a
  `NavigationSplitView` is variant 18 (blur radius 10, share 1, no tone cap).
  The soft scroll edge adds a `variableBlur` (radius 2, 1x128 gradient mask,
  scale 0.5) under the bar; the hard one a `gaussianBlur` of radius 6 at scale
  0.25 `[R]`.

## 8. Parity check

1. Build the backdrop PNG (`~/src/probes/liquidglass/patterns/`) and the window
   with `lgshot -panel -image <png> -glass x,y,w,h [-style 1] [-appearance aqua]`.
2. Capture the window (`screencapture -l`) and read it raw.
3. Render the same backdrop with `ctappearance::liquid_glass::render` and
   compare by distance band as in [section 6.1](#61-error).
4. Tolerances, justified by the measurement noise (one level of 8-bit
   quantisation, 0.5 px for displacement): interior of clear 1 level; gray tone
   of any look 2.5; colours 8 (clear), 32 (regular dark), 70 (regular light);
   the other states 20; the lens 2 px of displacement; the blur 2.5 px of
   sigma; the rim and shadow tables 10 levels; the union 8 rows; the hovered and pressed map 1.5 levels.
5. A result is not at parity where the table above is above these
   tolerances; the checker rows show it for high-frequency content next to
   the edge.

## 9. Shader descriptions

Both take the half-resolution backdrop and its blurred copies as inputs; the
lens, the blur mix, the face and the rim are one pass per glass.

**Compositor (GLSL ES 1.00; not compiled, no GPU here).**

1. Render the backdrop behind the glass to a half-size texture with linear
   filtering (a sample at a texel boundary is the 2x2 box); the dual Kawase
   downsample of the blur pass is not equivalent.
2. Build blurred copies at sigma 0, 2, 4, 8, 16, 28 (or the 13 levels of the
   model) from it.
3. In the fragment shader: `d`, `n` from the rounded-rectangle SDF (a loop over
   the container's elements with the smooth minimum of
   [section 6](#6-the-model)); `p' = p - n x |amount| x 0.831 x s(-d/height)`;
   `c = mix(copy_lo(p'), copy_hi(p'), t)` with `t` from `sigma(d)`; the face of
   step 4; the 2-D rim table sampled by `(angle, d)` and added; for `d >= 0`
   only the table (shadow and outline) over the backdrop.
4. Uniforms are `GlassLook` (blur, shares and distances, white, black,
   saturation, cap, fill, refraction, delta) and the size rules; the shape
   comes from `set_backdrop`'s rects and radius.

**Kit (SkSL, compiled in the container against Skia 0.104.0).**

```sksl
uniform shader sharp;
uniform shader soft;
uniform float4 shape;
uniform float radius;
uniform float4 lens;
uniform float4 blur;
uniform half4 face;
uniform half4 fill;
uniform half tone_delta[9];

const half3 LUMA = half3(0.2126, 0.7152, 0.0722);

float rounded_rect(float2 p) {
    float2 half_size = shape.zw * 0.5;
    float2 q = abs(p - (shape.xy + half_size)) - (half_size - radius);
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

half delta_at(half y) {
    float at = clamp(y * 8.0, 0.0, 7.999);
    half below = 0.0;
    half above = 0.0;
    for (int i = 0; i < 8; i++) {
        if (float(i) <= at && at < float(i + 1)) {
            below = tone_delta[i];
            above = tone_delta[i + 1];
        }
    }
    return mix(below, above, half(at - floor(at))) / 255.0;
}

half4 main(float2 p) {
    float d = rounded_rect(p);
    if (d >= 0.0) {
        return half4(0.0);
    }
    float2 normal = normalize(float2(
        rounded_rect(p + float2(0.25, 0.0)) - rounded_rect(p - float2(0.25, 0.0)),
        rounded_rect(p + float2(0.0, 0.25)) - rounded_rect(p - float2(0.0, 0.25))));
    float u = -d / lens.y;
    float falloff = u < 1.0 ? (exp(-lens.w * u) - exp(-lens.w)) / (1.0 - exp(-lens.w)) : 0.0;
    float2 at = p - normal * (abs(lens.x) * lens.z * falloff);
    float depth = clamp((-d - 1.0) / max(blur.z - 1.0, 1.0), 0.0, 1.0);
    float strength = clamp(mix(blur.x, blur.y, depth) / max(blur.w, 0.001), 0.0, 1.0);
    half3 seen = mix(sharp.eval(at).rgb, soft.eval(at).rgb, half(strength));
    half y = dot(seen, LUMA);
    half3 saturated = clamp(y + face.z * (seen - y), 0.0, 1.0);
    half3 tone = face.y + (face.x - face.y) * saturated * (1.0 - (1.0 - face.w) * y);
    half3 shown = mix(tone, fill.rgb, fill.a) + delta_at(y);
    return half4(clamp(shown, 0.0, 1.0), 1.0);
}
```

`sharp` is the half-resolution backdrop and `soft` the same blurred at
`blur.w` pixels; the mix of two copies approximates the Gaussian of an
intermediate sigma (not measured against macOS). The rim is a second pass over
the table. The kit cannot read behind its window, so this applies to content
inside it.

## 10. For Malaysia

1. The compositor can read `GlassSet` tokens through `ctappearance::liquid_glass`
   or copy `components.liquid-glass`; `looks(glass, scheme)` returns the five
   looks (active, inactive, reduced, increased contrast, both) and
   `adapted(look, scale, shorter)` the size-dependent inputs.
2. Protocol version 10 carries the style (`glass_regular`, `glass_clear`), rects
   and radius. For glass it still lacks: the **tint** (an RGBA, or none), the
   **interactive** flag with its pointer state (`hovered` or `pressed`;
   `interacted` maps the glass colour), the container **spacing** with a group id per
   element and a **union** id, and a **variant** for the chrome glass (sidebar
   = 18, Dock = 3 if confirmed). State, scheme, Reduce Transparency and the
   shorter side come from the compositor.
3. The slider: `tint(glass, scheme)` returns the `GlassTint` tokens and
   `tinted(&set, &tint, amount, shorter)` the active look of that position
   (`Tinted`: the look, the read factor 2, 4 or 8, the blur fill opacity and its
   sigma). At 0 it is the look of `adapted`. Only the active look was measured
   under the slider; the other states keep their looks.
4. The blur needs a variable sigma by depth up to 28 px (13 levels); the existing
   Dual Kawase cannot give the two-pixel resolution of the half-resolution read
   but can give the sigmas. The lens needs per-pixel displacement with a
   normal from the SDF.

## 11. Corrections

- [`macos-blur.md`](macos-blur.md) sections 7.1 and 8 and the earlier text of
  this file: "no lensing at 1x".
  True for the Dock, menu, Control Center and banners over uniform and boundary
  backdrops measured there (the blur of the regular glass flattens a
  1 px structure; the white/black boundary keeps its midpoint). False for
  `NSGlassEffectView` and `glassEffect`, whose lens bends a grid by 50 px at
  the edge of a clear glass.
- `glass_blur` as a quadratic of the slider (earlier section 3) is retired
  (ADR 0096): System Settings has a tint slider, `NSGlassTintAmount`, measured in
  section 13 ([`macos-settings.md`](macos-settings.md)), and no blur setting.
- The application menu is regular glass: 32, 122 and 146 over black, gray 128 and
  white, in raw device bytes. The `#a3` over white of the earlier menu tables
  was read through a probe that lifted mid-tones (the correction is in
  [`macos-blur.md`](macos-blur.md#9-corrections-to-the-2026-11-19-research)).

## 12. Not measured

| Cell | What it needs |
|---|---|
| What triggers the hovered and pressed state from a real pointer, its scale, shimmer and durations, and the map over colours | posted `mouse` events changed nothing (cause unknown); a hosted glass button or the user's own pointer; colour backdrops under `_interactionState` |
| Scale 2 | a HiDPI display mode (not authorised) |
| Reduce Transparency and Increase Contrast as system switches | the system switches; the per-process flags were used |
| Tint at every alpha and colour, tint over the inactive look | more captures; red, green, blue, gray, white and black at alpha 0.25 to 1 over regular dark and clear were captured |
| The outer refraction, the bleed and `BlurFill` beyond its delta | none showed an effect above 1.5 levels on the captured backdrops |
| `_contentLensing`, `subdued`, `scrim` | a window that sets them; their inputs are in the CSV |
| The 21 variants other than 1, 2, 3 (Dock tone), 14 and 18 | the surface that uses each |
| Shadow of shapes under 160 px | captures of 44 and 100 px circles |
| The depth profile of the blur between 0 and 40 px | a step at 5 to 30 px from the edge |

### 12.1 Global states

The cells that need a system setting run in `global-states.sh all`, which the user
runs: each of `dark`, `light`, `rt-on`, `ic-on`, `rt-ic-on`, `light-rt-on`,
`light-ic-on`, `light-rt-ic-on`, `hidpi-720` and `light-hidpi-720` has
`~/src/probes/states.d/<state>/70-liquidglass.sh`, which runs
`~/src/probes/liquidglass/lgstate.sh` under the screen lock. It writes
`~/src/captures/liquidglass/states/<state>/`: `live5.json` and `live3.json` (layer
tree and the 77 inputs of four looks and of 44 sizes), and raw window captures of
the regular (`reg-*`) and clear (`clr-*`) glass over `gray128`, `tone`, `grid16`,
`kern`, `sawx` (`reg-c3` and `reg-c7` for colour; `rampd` and `checker8` at scale 2).
`dark` and `light` add the per-process flags (`flag-rt-*`, `flag-ic-*`,
`flag-rtic-*`). The script was run alone in `dark`: 29 captures and 2 live files; its
`reg-gray128` and `clr-gray128` equal the earlier captures to 0 levels, and its
`flag-rt-live5.json` equals `live5-rt.json` in all 12 filter input sets.

| Cell | State | How it is decided |
|---|---|---|
| The system switch of Reduce Transparency and Increase Contrast is the per-process flag | `rt-on`, `ic-on`, `rt-ic-on`, `light-rt-on`, `light-ic-on`, `light-rt-ic-on` | `live5.json` equals `flag-<rt, ic, rtic>-live5.json` of `dark` (and `light`); the `reg-*` and `clr-*` tones equal the `flag-*` ones |
| A light system is a light window | `light` | `live5.json`, `live3.json` and the tones equal the aqua values of the 1x tables |
| Scale 2: blur, lens and size rules in points or in pixels | `hidpi-720`, `light-hidpi-720` | `live3.json` (sizes in points) equals the 1x inputs when the rules are in points; sigma and displacement in the captures are twice the 1x ones |
| Wallpaper adaptation of a window's glass | none | the glass reads the backdrop under it, which `lgshot` draws; the shell's glass over the wallpaper is in [`macos-blur.md`](macos-blur.md) |
| What triggers the hovered and pressed state | none | a hosted glass button or the user's own pointer; no global setting changes it |

After a run, fill the cells above and move them out of the table of this section.

## 13. The Appearance › Liquid Glass slider (`NSGlassTintAmount`), measured 2026-11-22

System Settings › Appearance has a slider named `Liquid Glass Tint Amount`, range 0 to 1, stored in the global domain as `NSGlassTintAmount` (0 on this Mac;
[`macos-settings.md`](macos-settings.md)). It is a per-process default too: `-NSGlassTintAmount <t>` on the command line of the probe gives that
process the glass of that slider position and changes no system setting. Values outside 0 to 1 clamp (checked at -0.5 and 1.5). With `0` the
live inputs equal those of a run with no argument, and the captures equal the earlier default ones (122 and 154 over gray 128, dark).

**Method.** `lgmulti` (the live inputs of a key window, `plans/live5.json`: regular and clear glass, aqua and darkAqua) at 31 values of `t`
(`macos-liquid-glass-tint.csv`, every input that moves); `lgshot -panel` windows (raw captures, never converted) of a 200 x 160 glass over gray 128 and of a
560 x 400 glass over the gray ramp `tone` and the step edges `kern2` at `t` = 0, 0.25, 0.5, 0.75 and 1, in both appearances and both styles; the kernel is the
step-edge fit of section 7 (`opacity` is the share of the Gaussian over the sharp step). 1x, the probe's own windows (the active look). Captures:
`~/src/captures/liquidglass/tint/`.

### 13.1 The inputs of `glassBackground` as functions of `t` (read)

Every input is piecewise linear in `t`; the ones not listed do not move.

| Look | Input | Value |
|---|---|---|
| Regular, both schemes | `inputBlurFillNormalOpacity` | `t` |
| Regular, dark | `inputBlurFillDarkenOpacity` | `0.675 + 0.45 t` up to `t` 0.5, then 0.9 |
| Regular, light | `inputBlurFillLightenOpacity` | the same |
| Regular, dark | `inputFaceColorMatrixFillColor` | rgb `0.125 \|1 - 2t\|`, alpha `max(0, t - 0.5)` |
| Regular, dark | `inputFaceColorMatrixMaxLuma`, `…SDR` | `0.45 - 0.2 t` up to `t` 0.5, then 0.35 |
| Regular, light | `inputFaceColorMatrixFillColor` alpha | `0.4 t` up to 0.5, then `0.6 t - 0.1` (0.5 at `t` 1) |
| Regular, dark | the backdrop layer `scale` | 0.5 up to `t` 0.40, 0.25 from 0.42 to 0.96, 0.125 from 0.98 |
| Regular, light | the backdrop layer `scale` | 0.5 up to `t` 0.32, 0.25 from 0.34 to 0.96, 0.125 from 0.98 |
| Clear, both schemes | `inputBlurOpacity0..3` | 0, 0, 0, 0 at `t` 0 (no blur); 1, 0.5, 0.5, 1 above |
| Clear | `inputBlurRadius` | `1.5 t` up to 0.5, then 0.75 |
| Clear | `inputBlurFillBlurRadius` | 0 up to `t` 0.5, then 8 |
| Clear | `inputBlurFillNormalOpacity` | `2 max(0, t - 0.5)` |

The lens, rim, shadow, refraction and every other input are the same at every `t`.

### 13.2 What the pixels do (measured)

Interior of the regular glass over gray 128, raw device value (clear is 154 at every `t`):

| `t` | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---|---|---|---|---|
| Regular, dark | 122 | 119 | 116 | 91 | 76 |
| Regular, light | 188 | 196 | 203 | 214 | 224 |

Over the ramp `tone` (input 30, 81, 132, 149 green levels): regular dark gives 55, 93, 121, 127 at `t` 0 and 44, 62, 74, 77 at 1; regular light gives
125, 160, 192, 202 at 0 and 195, 210, 223, 227 at 1. Clear gives 61, 109, 159, 175 at 0 and 61, 109, 157, 173 at 1: the clear tone does not follow the slider.

Gaussian sigma of the blur (px at 1x, 100 px in from the rim; 60 and 160 px differ by up to 1.5; `opacity` in brackets), and the contrast a white and black step keeps through the glass (levels, 223 with no glass):

| `t` | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---|---|---|---|---|
| Regular, dark: sigma | 6.5 (0.84) | 8.8 (0.80) | 10.2 (0.90) | 11.8 (0.92) | 15.5 (0.91) |
| Regular, dark: contrast | 104 | 94 | 85 | 57 | 36 |
| Regular, light: sigma | 8.2 (0.86) | 10.5 (0.87) | 13.2 (0.88) | 14.8 (0.91) | 17.5 (0.93) |
| Regular, light: contrast | 140 | 120 | 100 | 75 | 51 |
| Clear, both: sigma | none (0.5) | 4.5 (0.07) | 3.8 (0.18) | 14.5 (0.50) | 16.5 (0.97) |
| Clear, both: contrast | 223 | 223 | 223 | 214 | 184 |

So the slider takes the glass from the glass as designed (regular blur of 6.5 px dark and 8.2 px light, clear glass with no blur at all) to a frosted
one: at 1 the regular glass is a tinted panel of 15.5 px (dark) and 17.5 px (light) that keeps 16 % and 23 % of the contrast of a step, and the clear glass
a blur of 16.5 px with its tone unchanged (82 % of the contrast). The regular blur grows about linearly with `t` (0.41 of its range at 0.5 in dark), not with the
square, and the tone does not "hardly move": in light it has covered 42 % of its range at 0.5; in dark it moves late, 13 % at 0.5 and most of it after 0.5, where the fill alpha starts.

### 13.3 The model (2026-11-22)

`ctappearance::liquid_glass::tinted(set, tint, t, shorter)` is the active look at
slider position `t`; `render(set, tint, t, state, backdrop, shape)` is the CPU
reference. The tokens are `components.liquid-glass.tint.<regular|clear>.<light|dark>`
(`GlassTint`, `LIQUID_GLASS_TINT_*`): knots at `t` = 0, 0.5 and 1 for the inputs of
[13.1](#131-the-inputs-of-glassbackground-as-functions-of-t-read) that are linear in
`t`, the `t` at which the read scale halves (dark 0.41 and 0.97, light 0.33 and 0.97:
the middle of the reads that bracket them), and four nine-point tone tables.

What was decided from the pixels, and with what method (2026-11-22, the captures of
13.2 and numpy, no new capture):

- The blur fill is a Gaussian of sigma 16 px: clear at `t` = 1 has a normal opacity of
  1 and its main blur is 0.75 x 2 px, and it measures 16.5 px; regular dark and light
  at 1 measure 15.5 and 17.5. Sigma 12 and 32 fit the bars worse than 16 (rms of the
  difference to `t` = 0: 1.0 to 5.5 against 4 to 12).
- The read scale adds a box of 4 or 8 px; fixing it at 2 changes the width of a step
  by 1 px at most, so the read factor is kept as read.
- The darken and lighten opacity (0.675 to 0.9) is not read by the model: a scale of
  the nine-point table by that opacity left errors of 3 to 5 levels, and by the
  opacity times `1 - n` fitted dark but not light. The tone is fitted instead: for each
  scheme and `t` = 0.25, 0.5, 0.75 and 1 the residual of the face (with the fill and
  cap of that `t`) against the ramp row of the `tone` capture and gray 128 is a
  nine-point table; its difference to the table of `t` = 0 is the shift table, so the
  tone at 0 is the earlier model exactly. The end nodes copy their neighbour's shift
  (the ramp covers levels 26 to 230).
- The cap: the dark size rule `clamp(0.84 - 0.00375 m, 0.45, 0.6)` has its lower end
  moved by the slider (0.45 to 0.35), so the cap moves only where the rule is below
  its top (a 200 px shape moves, 30 to 56 px shapes read 0.6 at every `t`).
- The clear blur opacities become 1, 0.5, 0.5, 1 as soon as `t` > 0; the size rules of
  clear give 0 and are not applied then (read at 200 x 160 and 560 x 400 only).

Error (tests of `ctappearance::liquid_glass`): the tone over gray 128 and the ramp
(5 positions, both schemes, regular and clear; 200 samples) is within 2.25 levels
(regular dark gray 1.9 rms; the ramp 1.5 worst; light 1.0; clear 0.8) against the
captures. The step of a 40 px bar at 100 px from the rim of a 560 x 400 shape
through `render` (25 to 75 % width in pixels, and the contrast in levels; the
sigma of Gaussian is the width over 1.349):

| Look | `t` | Width model / capture | Contrast model / capture |
|---|---|---|---|
| Regular, dark | 0, 0.25, 0.5, 0.75, 1 | 6.1 / 6.5, 7.3 / 7.4, 9.3 / 11.1, 12.7 / 13.3, 17.5 / 17.9 | 114 / 104, 103 / 94, 93 / 85, 68 / 57, 45 / 36 |
| Regular, light | 0, 0.25, 0.5, 0.75, 1 | 7.8 / 8.2, 9.1 / 10.7, 11.4 / 13.3, 15.3 / 16.1, 22.4 / 19.6 | 149 / 140, 127 / 120, 105 / 99, 78 / 74, 54 / 51 |
| Clear, both | 0, 0.25, 0.5, 0.75, 1 | 1.0 / 1.0, 1.3 / 1.1, 1.7 / 1.3, 3.6 / 3.3, 21.5 / 20.2 | 223 / 223, 223 / 223, 223 / 223, 214 / 213, 185 / 183 |

The tolerance is 2.5 px of sigma (3.4 px of width; ADR 0095) and 0.08 on the contrast
as a share of its value at `t` = 0: the model keeps 5 to 11 levels more contrast than
the captures at every `t` (the bleed is not modelled, section 4), and the regular
blur at `t` = 0 is up to 2 px narrower than macOS (section 6.1), which the slider
inherits. What differs most: regular light at 1 (2.8 px wider), regular dark at 0.5
(1.8 px narrower).

### 13.4 Other surfaces, states and flags (read 2026-11-22)

Per-process reads of the layer tree (`-NSGlassTintAmount <t>` and, where noted,
`-reduceTransparency 1` or `-increaseContrast 1` on the probe's command line; no system
setting changed; no capture, so these are inputs, not pixels). The values at 0, 0.25,
0.5, 0.75 and 1 (0, 0.5, 1 for the key window) are in
[`macos-liquid-glass-tint-surfaces.csv`](macos-liquid-glass-tint-surfaces.csv): every input
of every `glassBackground` that moves, by source (`lgcfg-nonkey`, `lgmulti-key`,
`swiftglass`, `swiftwin`), surface, scheme and flags.

| Surface | Follows the slider? |
|---|---|
| `NSGlassEffectView` in a key window and in a non-key window (the inactive look) | yes, the same inputs as 13.1 |
| SwiftUI `glassEffect` (`.regular`, `.clear`, tints, `.interactive()`, containers, 30 to 300 px) and the glass buttons (`.glass`, `.glassProminent`) | yes, the same inputs; `.identity` does not move |
| The toolbar's button glass and the scroll-edge bar of a `NavigationSplitView` window (soft and hard) | yes, the same inputs as regular glass (their 36 px and 48 px bars) |
| The tab control's pill | only the face fill moves (the blur fill, the cap and the scale do not) |
| The window sidebar (variant 18, 144 x 500) | the blur fill opacity is 0.25 at 0, 0.5 at 0.5 and 1 at 1 (not `t`), the darken or lighten opacity as 13.1, the read scale is 0.5 up to 0.5 and 0.25 from 0.75; its fill does not move |
| The titlebar, toolbar and sidebar backdrops built from `gaussianBlur`, `colorMatrix`, `sdrNormalize`, `vibrantColorMatrix`, `variableBlur` | no input moves |
| Reduce Transparency, regular | the blur fill opacity is `t`; the fill alpha is a tent: dark `1.2 t` to 0.6 at 0.5, then 0.5 at 1 (black fill); light `1.6 t` to 0.8 at 0.5, then 0.5 (white); the cap follows dark as in 13.1; the read scale does not move |
| Reduce Transparency, clear | the blur fill follows 13.1; the main blur radius and opacities do not move |
| Increase Contrast, regular | as 13.1 except the fill: light 0.45 up to 0.75 and 0.5 at 1, dark does not move; the read scale is 0.25 from 0 (light) or from 0.25 (dark) |
| Increase Contrast, clear | as 13.1 |
| Both (key window) | light: the fill alpha is 0.45 at 0, 0.8 at 0.5 and 0.5 at 1, no scale move; dark: the fill does not move, the cap and the blur fill as 13.1, the scale is 0.125 at 0 and 0.5 and 0.25 at 1 |

The cap of dark regular glass does not move below shapes of 64 px (read 0.6 at 30 x 30,
65 x 24 and 74 x 56), moves at 200 x 160 and 300 x 200 (0.45 to 0.35), which the model
reproduces. `tinted` models the active look; the inactive, Reduce Transparency and
Increase Contrast looks keep their looks under the slider (their inputs are read, their
pixels were not captured).

### 13.5 Not measured

| Cell | What it needs |
|---|---|
| Whether the Dock, the menu bar, Control Center and the notifications follow the slider | the system value (a setting change): `~/src/probes/states.d/` batch `liquid-glass-tint-*` and `global-states.sh`, run by the user |
| The pixels of the inactive look, Reduce Transparency and Increase Contrast under the slider, and of the chrome glass (toolbar, sidebar, tab pill, scroll edge) inside a window | `lgshot` with the flags and a window that hosts them; only their inputs were read (13.4) |
| Scale 2 at other `t` | a HiDPI mode |
| The tint over colour backdrops and the rim and shadow at other `t` | the inputs of the rim and shadow do not move; colour captures were not taken |
| Why the scale thresholds differ between dark (0.41) and light (0.33) | the scale is probably chosen from the fill's luminance; one value at each end was read |
| The sizes between 64 and 104 px under the slider (cap), clear at sizes other than 200 x 160 and 560 x 400 (blur opacities), and the thresholds of the read scale at other sizes (the 540 x 36 toolbar bar is 0.5 up to 0.75 and 0.125 at 1) | reads at those sizes |
| The mixing order of the blur fill, the darken and lighten blend and the bleed | the filter's code; only its inputs and 1x pixels were available |
