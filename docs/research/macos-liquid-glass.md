# macOS Liquid Glass

**Goal:** describe the glass material as a system, not as one blur token, and
show which parts Malaysia already models and which parts are missing.
**Scope:** the glass used by the menu bar, the menu panel, the Dock, Control
Center and notification cards, plus tint, adaptive ink and accessibility.
**Reader:** anyone who changes `ctappearance` glass, `DockSurface`,
`menu_glass` or the compositor's blur path.
**Input:** the default backdrop matrix, the semantic colour dump and the
existing glass model in `ctappearance`.
**Output:** observed behaviour, a Malaysia comparison and ordered gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the material model in
   [`macos-materials.md`](macos-materials.md).
3. Read the default matrix in
   [`macos-defaults.md`](macos-defaults.md).
4. Read the measured per-element matrix and the lens verdict in
   [`macos-blur.md`](macos-blur.md).
5. Treat "glass" as a system of blur, tint, rim, lens, tone and grouping.
6. Do not reduce Liquid Glass to a single opacity or blur radius.
7. Do not copy Apple shaders, textures or SF Symbols.

---

## 1. What Liquid Glass is

Apple describes Liquid Glass as a material that:

- Has **no inherent colour**; it takes colour from the content behind it.
- **Lenses** or refracts that content instead of only blurring it.
- Carries a **specular rim** that changes with the light and the backdrop.
- **Adapts** its light or dark ink to what is underneath.
- Can be **tinted** with the accent, especially for prominent controls.
- Can be **interactive**, responding to press and hover.
- Can be **grouped** so nearby shapes render and morph as one.
- Is modified by Reduce Transparency and Increase Contrast.

The public descriptions are summarised in
[`styling-apple.md`](styling-apple.md). This file adds the observations from
the live desktop.

## 2. What was observed

The captured desktop does **not** make every surface glass:

| Surface | Observation |
|---|---|
| Menu bar | Clear, no blur, scheme ink |
| Application menu | Adaptive light/dark glass, ink follows the panel |
| Dock | Scheme-biased glass, still backdrop-dependent |
| Control Center | Dark glass, strong blur, rim |
| Notification | Dark glass, rim |
| Normal window | Opaque semantic background |

The menu bar is the important exception. It is transparent but not frosted.
The menu panel below it is a separate adaptive material.

The menu panel is the clearest new finding. In dark mode it is light over a
white or gray backdrop and dark over a black backdrop. A split white/black
backdrop switched the panel and its ink at the split line. The panel is
therefore not fixed by the colour scheme; it responds to the content behind
it.

The Dock is different. It is dark in dark mode and light in light mode, but it
still follows the backdrop inside each scheme. Reduce Transparency reduces
the effect but does not remove it: the light Dock is `#f1f1f1` over white and
`#888888` over black.

The static captures cannot prove **lensing**. A lens bends the image behind
the glass; a blur destroys its edges. The checker test shows the blur but does
not measure a displacement field. The `grid16` pass in
[`macos-blur.md`](macos-blur.md) closed this: the grid lines stay unshifted
near the menu, Dock, Control Center and banner edges, so no surface shows
measurable refraction at this scale. Treat the materials as adaptive blur and
tint with specular edges, not as lensing glass.

## 3. The existing Malaysia model

Malaysia already has more than a blur token:

- `glass_blur` is a 0 to 1 setting.
- `liquid_glass(glass_blur) = glass_blur * glass_blur`, measured on the
  macOS 26 Liquid Glass slider: halfway, the blur has grown a quarter of the
  way and the tint has hardly moved.
- `GLASS_BLUR_SPREAD = 4.0` and `GLASS_BLUR_FULL = 0.6` widen the blur from
  its resting value to the full setting.
- `glass_soften(rest, amount)` interpolates between the resting blur and the
  spread.
- `DockSurface` carries `glass`, `outline`, `rim`, `divider`, `corner`,
  `rim_side`, `glow`, `lens`, `tone` and `soften`.
- `menu_glass` reuses the Dock frame but replaces the fill, removes the side
  rim, flattens the lens and adds a dark tone cap and chroma.
- `glass_chroma`, `glass_lightness`, `glass_rim_lift`,
  `glass_blur_passes`, `glass_edge` and `glass_shadow` describe the
  wallpaper interaction.
- `reduce_transparency` switches the Dock and menu to solid surfaces.

The `DockTone` shader mixes the blurred backdrop with a tone curve. It can
lift a dark backdrop and cap a bright one, which explains the measured dark
menu. It does not choose a **light panel with dark ink**; that is the missing
half of the adaptive model.

## 4. Comparison with Malaysia

| Element | macOS default | Malaysia before | After |
|---|---|---|---|
| Menu bar | Clear | `translucent` tinted | `floating` clear |
| Menu panel | Adaptive light/dark | fixed scheme glass | adaptive model |
| Menu ink | Follows panel | follows scheme | still scheme-based |
| Dock light glass | `#eaeaea` over white | `#ffffff66` | `#b5b5b548` |
| Dock light opaque | `#f1f1f1` over white | `#757575` | `#f1f1f1` |
| Dock dark glass | `#0d0d0d` over black | `#2727274e` | unchanged |
| Strong blur | Low-frequency gradient | `glass_blur_passes`, `soften` | pass count unverified |
| Specular rim | Light/dark bevel | `rim`, `rim_side`, `corner`, `glow` | model exists |
| Lens | Not proven by static capture | `lens` parameters | unverified |
| Accent tint | Prominent controls | `take_accent_chroma` | only some surfaces |
| Grouping | Nearby shapes render together | no group | new concept |
| Morphing | Shapes morph as one | no morph | new concept |
| Interactive glass | Press and hover response | no interactive mode | new concept |
| Reduce Transparency | Frosted/opaque substitution | solid fallback | approximate |

The largest visual gap is the **menu ink**. The compositor can draw a light
panel over a bright window, but the shell keeps the scheme's white text. The
largest conceptual gap is the **group and morph** behaviour: Malaysia has
individual glass surfaces, not a container that manages several shapes.

## 5. What not to copy

- Apple's lens shader, specular texture or glass rendering code.
- Apple's SF Symbols or application icons.
- A screenshot of a glass surface as a texture.
- Apple's exact HIG wording in user-visible text.
- The names `regular`, `clear` and `identity` as public API without checking
  the glossary rules.

## 6. Prioritized changes

1. **Keep the clear menu bar.** The default is now `floating`, with blur only
   under the open title pill. This matches the measured bar.
2. **Add a light menu ink variant.** It must follow the panel, not the colour
   scheme. The shell needs a luminance input from the compositor or a
   wallpaper proxy. Without it the light panel keeps white text.
3. **Promote glass to roles.** Define at least `menu`, `dock`, `panel` and
   `card`, each with its own fill, blur, rim and tone. The menu and Dock are
   not the same material.
4. **Add a glass group.** A container owns the blur, the rim and the tone for
   its children, so nearby cards can merge.
5. **Add morphing between grouped shapes.** Animate the container's shape,
   not each child independently.
6. **Add an interactive state.** Press and hover change the glass slightly,
   without a full re-render of the backdrop.
7. **Keep the quadratic `glass_blur`.** It is already measured against the
   macOS slider. Do not replace it with a linear ramp.
8. **Verify `lens`.** If no surface uses it, delete it. If the Dock uses it,
   add a live edge test that proves the displacement.

## 7. Open questions

- Does the macOS menu panel actually lens, or only blur and tint?
- Does the menu panel follow the window or the wallpaper behind it?
- Does the Dock lens at rest or only under magnification?
- How does a glass group decide when two shapes merge?
- What is the exact interaction between Increase Contrast and the rim?
- Does Reduce Transparency keep the rim or remove it?
