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
| Application menu | Adaptive glass, two veil sets by scheme, ink follows the panel |
| Dock | Scheme-biased glass, still backdrop-dependent; scheme solid under RT |
| Control Center | Same family as the menu with small offsets, strong blur, rim |
| Notification | Lighter veil than the menu in both modes, rim |
| Normal window | Opaque semantic background |

The menu bar is the important exception. It is transparent but not frosted.
The menu panel below it is a separate adaptive material.

The menu panel carries two veil sets. Dark mode is a black veil capped
bright: `#a3a3a3` over white and `#2b2b2b` over black. Light mode is a white
veil capped near white: `#f2f2f2` over white and `#75` to `#7b` over black.
A split white/black backdrop switches the panel and its ink at the split
line in both modes. The panel responds to the content behind it and to the
appearance; the earlier "scheme-independent" claim was an artifact of a dark
wallpaper.

The Dock is dark in dark mode and light in light mode, and it still follows
the backdrop inside each scheme. Reduce Transparency replaces it with the
scheme solid: `#03` to `#0d` in dark mode and `#f1` to `#89` in light mode.

The static captures cannot prove **lensing**. A lens bends the image behind
the glass; a blur destroys its edges. The checker test shows the blur but does
not measure a displacement field. The `grid16` and boundary passes in
[`macos-blur.md`](macos-blur.md) closed this: a white/black boundary keeps
its midpoint within one pixel inside the Dock in both modes, and the grid
lines stay unshifted near the menu, Dock, Control Center and banner edges, so
no surface shows measurable refraction at this scale. Treat the materials as
adaptive blur and tint with specular edges, not as lensing glass.

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
menu. The dark veil set is in place and tuned; the missing half is the
**light veil set** for the menu and the scheme solid for the RT Dock.

## 4. Comparison with Malaysia

| Element | macOS default | Malaysia before | After |
|---|---|---|---|
| Menu bar | Clear | `translucent` tinted | `floating` clear |
| Menu panel | Two veil sets by scheme | fixed scheme glass | dark veil set only |
| Menu ink | Follows panel | follows scheme | follows the panel |
| Dock light glass | `#ebebeb` over white | `#ffffff66` | `#b5b5b548` |
| Dock light opaque | `#f1f1f1` over white | `#757575` | `#f1f1f1` |
| Dock dark glass | `#bebebe` over white, `#0d` over black | `#2727274e` | unchanged |
| Strong blur | Low-frequency gradient | `glass_blur_passes`, `soften` | pass count unverified |
| Specular rim | Light/dark bevel | `rim`, `rim_side`, `corner`, `glow` | model exists |
| Lens | Not proven by boundary tests in either mode | `lens` parameters | unverified |
| Accent tint | Prominent controls | `take_accent_chroma` | only some surfaces |
| Grouping | Nearby shapes render together | no group | new concept |
| Morphing | Shapes morph as one | no morph | new concept |
| Interactive glass | Press and hover response | no interactive mode | new concept |
| Reduce Transparency | Dock scheme solid; menu and CC unchanged | solid fallback | approximate |

The largest visual gap is the **light menu veil set**. The compositor draws
the dark matrix in both schemes today, so a light desktop still renders dark
panels. The adaptive ink is already in place and is not the gap.

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
2. **Add the light menu veil set.** The dark tokens reproduce the dark
   matrix; a light scheme needs its own veil and cap so the panel renders
   `#f2` over white and `#75` over black. The adaptive ink already follows
   the panel; do not add a wallpaper proxy.
3. **Make the RT Dock a scheme solid.** `#03` to `#0d` dark, `#f1` to `#89`
   light; menu and Control Center unchanged.
4. **Promote glass to roles.** Define at least `menu`, `dock`, `panel` and
   `card`, each with its own fill, blur, rim and tone. The menu and Dock are
   not the same material.
5. **Add a glass group.** A container owns the blur, the rim and the tone for
   its children, so nearby cards can merge.
6. **Add morphing between grouped shapes.** Animate the container's shape,
   not each child independently.
7. **Add an interactive state.** Press and hover change the glass slightly,
   without a full re-render of the backdrop.
8. **Keep the quadratic `glass_blur`.** It is already measured against the
   macOS slider. Do not replace it with a linear ramp.
9. **Verify `lens`.** If no surface uses it, delete it. If the Dock uses it,
   add a live edge test that proves the displacement; the boundary tests in
   [`macos-blur.md`](macos-blur.md) found none at 1x.

## 7. Open questions

- Does the macOS menu panel actually lens, or only blur and tint? The 1x
  boundary tests say blur and tint; 2x captures are the remaining check.
- Does the menu panel follow the window or the wallpaper behind it?
- Does the Dock lens at rest or only under magnification?
- How does a glass group decide when two shapes merge?
- What is the exact interaction between Increase Contrast and the rim?
- Does Reduce Transparency keep the rim or remove it?

## 8. Implementation prompt

```text
You are fixing the Liquid Glass model from measured values. Read this file
and macos-blur.md first. Work in ~/src/barrierkit and ~/src/malaysia,
main only: git pull --ff-only before editing and before each push, one
commit per session, English messages, no code comments.

Measured targets, dark / light:
- Menu: white #a3a3a3 / #f2f2f2, gray #92-94 / #d1d1d1, black #2b2b2b /
  #75-7b, checker #8b-8d / #bf-c6.
- Dock: white #bebebe / #ebebeb, gray #8b / #a6a6a6, black #0d-10 / #34,
  checker #6c / #91.
- RT Dock: scheme solid #03-0d / #f1-89; menu and CC unchanged.
- Ink: dark above 0.4 panel luma, white below.
- No lensing: the boundary midpoint never moves at 1x; do not build a
  lens pass.

Tasks:
1. barrierkit: add the light veil set for the menu; align Dock and RT
   targets; four-backdrop tests per surface per scheme in
   ctappearance::resolve within 8/255.
2. malaysia: after the rev bump, confirm the glass path renders the
   resolved values and the RT early-out stays.

Do not start group, morph or interactive work in this pass; they are
separate prompts.

Definition of done: ./dev.sh check passes in both repos; tests assert the
measured values; barrierkit main is pushed and malaysia pins the new rev.
Report changed tokens, model inputs and outputs, and commits.
```
