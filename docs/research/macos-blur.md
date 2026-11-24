# macOS blur and Liquid Glass per element

**Goal:** classify every macOS shell surface by its material and give measured
values for its blur, tint, border, shadow and lensing in both appearances, so
a compositor can tell Liquid Glass from a general blur and can use one value
per mode instead of guessing.
**Scope:** the menu bar, the application menu panel, the Dock, Control Center,
Notification Center, Spotlight, notification banners, Mission Control,
Launchpad and one window's chrome. It ends at the classification table and the
implementation prompt.
**Reader:** anyone who changes `ctappearance` glass, the compositor blur path
or a Malaysia shell surface.
**Input:** the probes in `~/src/probes`, the four controlled backdrops, the
Reduce Transparency switch and dark and light mode measured under the same
wallpaper.
**Output:** this matrix, the per-mode model constants, the Liquid Glass
verdict, the code gaps and an implementation prompt. Since 2026-11-22 also
the per-surface pipeline of [section 14](#14-desktop-surfaces-as-a-pipeline-measured-2026-11-22).

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the material model in
   [`macos-materials.md`](macos-materials.md) and the glass system in
   [`macos-liquid-glass.md`](macos-liquid-glass.md).
3. Use a value only with its backdrop and its appearance; never lift one
   colour alone.
4. Treat an element as Liquid Glass only with lensing, specular or adaptive
   ink evidence; a blur plus a tint is general translucency.
5. Check the corrections section before trusting a value from an earlier
   research file.
6. Read section 14 for the measured pipeline of each desktop surface (tone,
   chroma and kernel); sections 3 to 5 and 10 are superseded by it.

---

## 1. Method

All values come from pixel samples of 2560x1440, scale 1.0 captures. The
backdrops were white, gray (`#a3a3a3` as rendered), black, checker (8 px
cells) and a white/black boundary (`edgeh`, `edgev`). The boundary backdrop
is the refraction test: a refracting surface moves the transition midpoint; a
pure blur only softens it.

Dark and light mode were measured back to back under the same wallpaper, and
the wallpaper was sampled before each pass so a wallpaper change cannot
contaminate the comparison. The 2026-11-19 pass at 1920x1080 was recorded
under a darker wallpaper; where the two passes disagree the wallpaper is the
likely cause and the disagreement is listed in section 9.

There is no 2x capture on this host, so one-pixel rims are not comparable at
Retina density. The lock screen was not captured. This host has no desktop
widgets; Notification Center only exposes its clock pill.

The materials of windows (`NSVisualEffectView`, in-window blur) are measured
apart, from raw pixels and the layer tree, in
[`macos-materials.md`](macos-materials.md#10-materials-in-windows-measured-2026-11-22)
(2026-11-22). This file keeps the desktop surfaces.

## 2. The material classes observed

| Class | Surfaces |
|---|---|
| Clear | menu bar, Spaces bar, Mission Control background |
| Scheme-adaptive glass | application menu panel, Control Center, Spotlight field, notification banner, Notification Center, Dock |
| Scheme solid | Dock under Reduce Transparency |
| Wallpaper blur | Launchpad background |
| Translucent chrome | window toolbar and sidebar |
| Opaque | window content |

## 3. The measured matrix, dark mode

**Corrected 2026-11-21 ([section 9.1](#91-2026-11-21-mid-tones-read-with-sample-were-too-light)):** every mid-tone in this table was read with `sample pixel`, which lightens mid-tones by up to 17/255. The cells stay as recorded; section 9.1 gives each one as the raw capture value.

| Element | White | Gray | Black | Checker | RT | Ink |
|---|---|---|---|---|---|---|
| Menu bar | sharp white | sharp gray | sharp black | sharp 8 px cells | unchanged | white glyphs |
| Application menu | `#a3a3a3` | `#92` to `#94` | `#2b2b2b` | `#8b` to `#8d` | unchanged: `#a3` over white | follows the panel |
| Control Center | `#a3` to `#b1` | `#a3a3a3` | `#262626` | `#7f7f7f` | `#b1` over white | follows the panel |
| Dock | `#bebebe` | `#8b8b8b` | `#0d` to `#10` | `#6c6c6c` | solid `#03` to `#0d` | n/a |
| Notification Center clock pill | `#bcbcbc` | not measured | `#5c5c5c` | `#9f9f9f` | not measured | n/a |
| Notification banner | not measured | not measured | not measured | `#86` to `#8c` | not measured | white |
| Spotlight field | not measured | not measured | not measured | `#8d8d8d` | not measured | follows the panel |
| Finder window title/toolbar/sidebar/content | n/a | n/a | n/a | n/a | n/a | `#282828` / `#2f2f2f` / `#454f57` / `#282828` |

## 4. The measured matrix, light mode

**Corrected 2026-11-21 ([section 9.1](#91-2026-11-21-mid-tones-read-with-sample-were-too-light)):** every mid-tone in this table was read with `sample pixel`, which lightens mid-tones by up to 17/255. The cells stay as recorded; section 9.1 gives each one as the raw capture value.

| Element | White | Gray | Black | Checker | RT | Ink |
|---|---|---|---|---|---|---|
| Menu bar | sharp white | sharp gray | sharp black | sharp 8 px cells | unchanged | dark glyphs |
| Application menu | `#f2f2f2` | `#d1d1d1` | `#75` to `#7b` | `#bf` to `#c6` | unchanged: `#f2` over white | follows the panel: dark on `#75` |
| Control Center | `#fbfbfb` | `#c7c7c7` | `#666666` | `#bababa` | not measured | follows the panel |
| Dock | `#ebebeb` | `#a6a6a6` | `#343434` | `#919191` | solid `#f1` to `#89` | n/a |
| Notification Center clock pill | `#ededed` | not measured | `#9e9e9e` | `#d2d2d2` | not measured | n/a |
| Notification banner | not measured | not measured | not measured | `#b3b3b3` | not measured | dark |
| Finder window title/toolbar/sidebar/content | n/a | n/a | n/a | n/a | n/a | `#fafafa` / `#dbdbdb` / `#efefef` / `#f5f5f5` |

The menu bar stays a sharp 8 px pattern in both modes: clear, not blurred.
The Mission Control background over the wallpaper is pixel-identical to the
desktop on this host: macOS 27.2 does not dim or blur it.

## 5. The model fit: one veil per surface per mode

**Corrected 2026-11-21 ([section 9.1](#91-2026-11-21-mid-tones-read-with-sample-were-too-light)):** every mid-tone in this table was read with `sample pixel`, which lightens mid-tones by up to 17/255. The cells stay as recorded; section 9.1 gives each one as the raw capture value.

Treating each glass as a white or black veil plus a brightness cap fits every
measured value within a few units. A veil is `behind.mix(color, alpha)`; the
cap clamps the resulting luminance. These are the target constants for the
compositor:

| Surface | Mode | Veil | Cap | Fit check |
|---|---|---|---|---|
| Application menu | dark | white `a=0.2` | `0.64` | white `0.64` vs `#a3` 0.64; black `0.17` vs `#2b` 0.17 |
| Application menu | light | white `a≈0.45` | `≈0.95` | black `0.45` vs `#75` 0.46; gray `0.80` vs `#d1` 0.82 |
| Control Center | dark | white `a≈0.15` | `0.64` | black `0.15` vs `#26` 0.149; checker `0.57` vs `#7f` 0.50 |
| Control Center | light | white `a≈0.4` | `0.98` | black `0.40` vs `#66` 0.40; gray `0.78` vs `#c7` 0.78 |
| Dock | dark | black `a≈0.15` | `0.75` | white `0.85` vs `#be` 0.75; black `0.85` vs `#0d` 0.05 |
| Dock | light | white `a≈0.4` | `0.92` | black `0.40` vs `#34` 0.20; checker `0.70` vs `#91` 0.57 |
| NC clock pill | dark | white `a≈0.35` | `0.74` | black `0.35` vs `#5c` 0.36; white `0.74` vs `#bc` 0.74 |
| NC clock pill | light | white `a≈0.6` | `0.93` | black `0.60` vs `#9e` 0.62; checker `0.80` vs `#d2` 0.82 |
| Banner | dark | white `a≈0.45` | `0.55` | checker `0.52` vs `#86` 0.53 |
| Banner | light | white `a≈0.6` | `0.72` | checker `0.70` vs `#b3` 0.70 |

The Dock veil is a black veil in dark mode and a white veil in light mode,
which is why the two columns do not share a curve. Reduce Transparency
replaces the Dock glass with the scheme solid: `#03` to `#0d` in dark mode,
`#f1` to `#89` in light mode; the menu and Control Center stay unchanged.

## 6. Blur strength per element

The checker is 8 px cells. A surface that shows a flat colour has destroyed
that frequency; a surface that shows a faint dither has not.

- **Menu panel:** flat with a residual dither of about 3/255. Strong blur.
- **Control Center:** flat, same family as the menu.
- **Spotlight and banner:** flat, the same family.
- **Notification Center clock pill:** flattens the checker; its tone curve is
  lighter than the menu in both modes.
- **Dock:** flat, stronger than the menu at the same backdrop.
- **Window toolbar and sidebar:** checker fully flattened.
- **Launchpad:** the wallpaper detail is gone and the luminance lifts about
  36/255 (measured on the 2026-11-19 pass; the 2026-11-20 pass could not
  reopen Launchpad).

## 7. Borders, shadows and edge zones

- The **menu panel** edge is 1 px, light over light backdrops and dark at the
  corners; its shadow reaches about 30 px.
- The **Dock** top edge is a 1 px light line over a 1 to 2 px dark line,
  followed by the glass; its bottom ramp is up to 13 px over a bright
  backdrop. Both modes share the structure; the line values follow the mode
  (`#88` light mode, `#74` dark mode at the top line).
- The **Control Center** bottom edge is a 2 px light line with a shadow that
  fades over about 30 px on white.
- A **window** has a 1 px dark outline plus a 1 px light bevel and a 10 to
  15 px shadow.
- The **menu corner radius** stays about 6.

### 7.1 The boundary test for refraction

A white/black boundary at a known position was measured inside each surface
in both modes. A refracting edge would move the midpoint; a pure blur only
softens it.

| Surface | Boundary | Sample point | Midpoint | Displacement |
|---|---|---|---|---|
| Dock dark | horizontal at y=1355 | x=1300 | y≈1355 | 0 px |
| Dock light | horizontal at y=1355 | x=1300 | y≈1354.5 | within 1 px |
| Dock dark, 2026-11-19 | horizontal at y=1030 | gaps at x=250, 316, 1200, 1500, 1730 | y=1030 | 0 px |
| Dock dark, 2026-11-19 | vertical at x=250 | y=1000, 1060 | x=250 | 0 px |
| Banner dark, 2026-11-19 | horizontal at y=80 | x=1700 | y=80 to 81 | within 1 px |
| NC dark, 2026-11-19 | horizontal at y=120 | x=1700 | y=120 | within 1 px |

The 2026-11-20 light banner run is contaminated by the banner's own bottom
shadow and is not listed. No surface moved the boundary at 1x in either mode.
The blur is the same material from the rim inward; the only edge-specific
rendering is the rim itself.

## 8. Liquid Glass or general blur

The boundary tests are the verdict for both modes. No surface shifted a
white/black boundary, neither in the center nor next to its borders. There is
no measurable lensing or refraction at 1x in dark or light mode.

What the surfaces do have is the rest of the Liquid Glass description: a
specular 1 px edge, a strong blur and adaptive ink. The honest classification
on this host is **adaptive blur and tint materials with specular edges,
without proven lensing**. Do not call any of them lensing glass in the code
or the docs.

## 9. Corrections to the 2026-11-19 research

Measured under the same wallpaper, the two appearances disagree with three
claims of the 1920x1080 pass:

- **The menu panel is scheme-dependent.** Dark mode caps the panel at
  `#a3a3a3` and floors it at `#2b2b2b`; light mode renders `#f2f2f2` over
  white and `#75` to `#7b` over black. The earlier "scheme-independent" claim
  was recorded under a dark wallpaper, which the light mode evidently tracks
  when the wallpaper is dark. There are two veil sets, not one.
- **The Dock is scheme-dependent, not wallpaper-only.** Same wallpaper, dark
  mode `#bebebe` over white versus light mode `#ebebeb` over white. The Dock
  follows the appearance like the code already assumes; the wallpaper claim
  was an artifact of the dark wallpaper pass.
- **Reduce Transparency makes the Dock a scheme solid.** `#03` to `#0d` in
  dark mode, `#f1` to `#89` in light mode. The menu and Control Center stay
  unchanged in both modes.
- **Mission Control has no background material on this host.** Unchanged.
- **Notification Center carries a light veil even in dark mode** (`#5c` over
  black, `#bc` over white), lighter than the menu in both modes.

The Dock, menu and Notification Center values of the 2026-11-19 pass
(`#8d` menu over white, `#7f` Dock over checker) belong to the darker
wallpaper state of that day; treat them as wallpaper variants, not as
constants.

### 9.1 2026-11-21: mid-tones read with `sample` were too light

Every mid-tone of sections 3 to 5 was read with `~/src/probes/sample pixel|avg`.
That tool returned the raw byte value lightened as if it were Generic RGB
gamma 1.8. Black and white are unchanged, which hid it. The cells in sections 3
to 5 stay as recorded and are superseded by the raw values below.

**Cause (controlled test, no screen needed).** A synthetic 0 to 255 gray ramp
saved with no profile, with Apple's sRGB profile, and with `cICP` 9/13, 1/13,
9/1 and 12/13 reads identically through `sample`: 32 gives `#2b`, 64 `#51`, 128
`#92`, 192 `#cb`. `NSBitmapImageRep` reports the right profile (`colorSpace`
`sRGB IEC61966-2.1`, or `Rec. ITU-R BT.2020-1; sRGB Gamma` for a capture) and so
does its `CGImage`. But `colorAt(x:y:)` returns an `NSColor` in `Generic RGB`
whose components are the raw bytes (`0.5020` for 128), and
`usingColorSpace(.sRGB)` then converts from gamma 1.8: `srgb_encode((v/255)^1.8)`.
It is not the profile, not the `cICP` chunk and not rounding. The capture's
display profile has an exact sRGB curve (its 1024-entry curve equals the sRGB
decode at 32, 64, 96, 128, 146, 163, 192 and 224), `sips -m` and LittleCMS leave
grays unchanged, and PIL on the PNG bytes reads 146 where `sample` read `#a3`
(`d3-white-menu.png`, pixel 150, 200). A raw gray value in a `screencapture` PNG
already is its sRGB value.

**Law.** `doc = srgb_encode((raw / 255)^1.8)` and
`raw = 255 * srgb_decode(doc / 255)^(1 / 1.8)`. Checked against raw pixels on
the dark Apple menu (146, 130, 32, 122 read `#a3`, `#94`, `#2b`, `#8d`) and the
dark Dock (176 reads `#be`). `~/src/probes/sample` now reads the bytes
(`getPixel`); its older outputs are the shifted ones.

**Every cell of sections 3 to 5 as the raw value** (derived with the law and
±1; the cells with a capture were checked against raw pixels):

| Surface | Mode | White | Gray | Black | Checker | Other |
|---|---|---|---|---|---|---|
| Application menu | dark | 146 (was `#a3`) | 129 (`#93`) | 32 (`#2b`) | 121 (`#8c`) | |
| Application menu | light | 239 (`#f2`) | 199 (`#d1`) | 98 to 104 (`#75` to `#7b`) | 178 to 186 (`#bf` to `#c6`) | |
| Control Center | dark | 146 to 162 (`#a3` to `#b1`) | 146 (`#a3`) | 29 (`#26`) | 108 (`#7f`) | |
| Control Center | light | 250 (`#fb`) | 187 (`#c7`) | 83 (`#66`) | 172 (`#ba`) | |
| Dock | dark | 176 (`#be`) | 120 (`#8b`) | 12 to 14 (`#0d` to `#10`) | 89 (`#6c`) | RT solid 5 to 12 (`#03` to `#0d`) |
| Dock | light | 230 (`#eb`) | 149 (`#a6`) | 39 (`#34`) | 127 (`#91`) | RT solid 118 to 237 (`#89` to `#f1`) |
| NC clock pill | dark | 174 (`#bc`) | not measured | 74 (`#5c`) | 142 (`#9f`) | |
| NC clock pill | light | 233 (`#ed`) | not measured | 140 (`#9e`) | 200 (`#d2`) | |
| Banner | dark | not measured | not measured | not measured | 115 to 121 (`#86` to `#8c`) | |
| Banner | light | not measured | not measured | not measured | 164 (`#b3`) | |
| Spotlight field | dark | not measured | not measured | not measured | 122 (`#8d`) | |

The gray backdrop is `NSColor(calibratedWhite: 0.5)`, which renders as raw 146
(the "`#a3a3a3` as rendered" of section 1 was this raw 146 shifted). The Finder
chrome cells (`#282828`, `#2f2f2f`, `#454f57`) were probably read the same way
and are raw 30, 35 and 53 if so; they were not re-read here. The window-chrome
values the kit takes from the appkit-compare bench read raw pixels and are not
affected.

**What depended on the shifted figures.**

- `crates/ctdesign/tengen/components.json`:
  `components.shell.menu.glass-veil-light|dark`, `glass-lift-light|dark`,
  `glass-cap-light|dark`, `glass-chroma` (fitted to the menu cells);
  `components.shell.dock.tone-light|tone-dark` (`lift`, `cap`, `outline`,
  `gain`) and the rim luminances `rim-light`, `rim-dark`, `divider-light`,
  `divider-dark` (Dock cells and rim lines);
  `components.shell.notifications.glass-veil-light` (banner `#b3`).
- `crates/ctdesign/tengen/tokens.json`, in every theme: `dock-bar`,
  `dock-bar-opaque` (the Reduce Transparency solids `#f1` and `#0a`),
  `dock-bar-frosted`, `dock-outline`.
- `crates/ctappearance/src/resolve.rs` tests: `MENU_PANELS_DARK`,
  `MENU_PANELS_LIGHT`, `DOCK_PANELS_DARK`, `DOCK_PANELS_LIGHT`,
  `BANNER_OVER_THE_CHECKER_DARK|LIGHT`, `MACOS_GRAY` (`0xa3` stands for the raw
  146 backdrop, so the kit's gray input is also off) and the Reduce Transparency
  solids. They pass because the model was fitted to the same shifted figures;
  the model renders those surfaces up to 17/255 lighter than macOS.
- `MENU_DARK_INK_ABOVE` (`components.shell.menu.dark-ink-above`, 0.4): chosen
  between the shifted panel lumas 0.46 and 0.17. The raw light panel over
  black is 98 to 104, luma 0.38 to 0.41, so the threshold sits on the panel it
  must classify.
- Malaysia, through `ctappearance::Resolved`:
  `crates/malaysia-compositor/src/blur.rs` `material` (`Resolved::dock`,
  `notification_glass`, `menu_glass`) and
  `crates/malaysia-compositor/src/focus_notice.rs` (`ctappearance::menu::panel_over`
  of `menu_glass`, the panel luma that picks the ink).

These stay in place and keep working until the pipeline of section 13 replaces
them; they are superseded, not removed (ADR 0094).

## 10. Code gaps against the current kit and compositor

Superseded on 2026-11-22: the veils, lifts and caps below were fitted to the
shifted numbers of [section 9.1](#91-2026-11-21-mid-tones-read-with-sample-were-too-light).
They stay in `ctdesign` and `ctappearance` until Malaysia moves to
`ShellGlass` ([ADR 0094](../decisions.md)); [section 14](#14-desktop-surfaces-as-a-pipeline-measured-2026-11-22)
has the measured replacement.

Closed on 2026-11-20 in `ctdesign` and `ctappearance`; the tests in
`ctappearance::resolve` hold every value below within 8/255.

- **The earlier claim here was wrong.** The old menu tokens (`#ffffff33`, lift
  `-0.032`, cap `0.6434`) reproduced the `#8d` of the 2026-11-19 pass, not the
  current `#a3`: white was 22/255 off. The caps of section 5 are in sRGB gamma
  units and the token `cap` is OKLab lightness, so the two are not pasted
  across.
- **The menu has two sets** in `components.shell.menu`, picked by
  `resolve::menu_glass` from the scheme. Dark and dark tint: `#ffffff41`, lift
  `-0.084`, cap `0.706`; over white, gray, black and the checker they give
  `#a0`, `#98`, `#2c` and `#87`. Light: `#ffffff76`, lift `0.012`, cap
  `0.968`; they give `#f2`, `#d1`, `#7a` and `#bf`. Chroma and spread are
  shared.
- `ctappearance::menu::dark_ink_over` at `0.4` works for both matrices: the
  light panel over black is `0.48` (dark ink) and the dark panel over black is
  `0.17` (light ink).
- **The Dock** keeps its light tone (`#e9`, `#a8`, `#33`, `#8f`). The dark tone
  is now lift `0.032` and cap `0.822` (`#be`, `#87`, `#13`, `#6e`).
- **The notification banner** rides the menu glass in dark (checker `#87`) and
  has its own light veil, `NOTIFICATION_GLASS_VEIL_LIGHT` `#ffffff5e`, with
  the light menu lift and cap (checker `#b3`).
- **Reduce Transparency:** the Dock is the scheme solid (`#0a` dark, `#f1`
  light) over any backdrop, and the menu glass keeps its veil and tone.
  Section 3 gives `#03` to `#0d` for the dark solid and section 5 gives `#f1`
  to `#89` for the light one; one solid cannot be `#f1` over white and `#89`
  over black, so the light solid stays `#f1` until a capture settles it.
- **bk menus under Reduce Transparency:** `theme::over_glass` keeps the menu
  clear under Reduce Transparency alone and returns the solid palette only
  with Increase Contrast, which was not measured. The compositor's blur and
  glass gate is a separate change.
- Not built: Control Center, Spotlight and the Notification Center clock pill
  (no consumer), the Dock blur strength and the removal of the lens pass. No
  lensing pass is justified by the measurements.

## 11. Open questions

- What are the rim and blur values at 2x? A sub-pixel refraction zone could
  exist at 1x without being measurable; Retina captures are needed.
  **Not measured (2026-11-22):** the host display runs at scale 1 and a Retina
  capture needs a scaled display mode, a system setting
  ([section 14.9](#149-not-measured-and-why)).
- Does the light mode track a dark wallpaper and revert to the dark matrix?
  The 2026-11-19 pass suggests it; verify with the current light wallpaper in
  dark mode and a dark wallpaper in light mode.
  **Not measured (2026-11-22):** it needs the Light appearance. The probe draws
  its own backdrop, so the wallpaper never reaches a surface that blurs.
- Does Notification Center change under Reduce Transparency?
  **Not measured:** it needs the system switch (Reduce transparency); the cards
  were measured in dark ([section 14.4](#144-the-numbers-dark)).
- Does the Launchpad lift depend on the wallpaper luminance?
  **Answered (2026-11-22):** the Launchpad of macOS 27 is Apps, a floating glass panel
  (864 x 606 px at 848, 431) with the glass of the application menu: it blurs the
  backdrop under it, dims nothing else and lifts nothing over the whole screen.
- Is the compact Control Center panel the default or a host state?
  **Answered (2026-11-22):** on this host the panel opens as a stack of glass
  tiles (Wi-Fi, Bluetooth, AirDrop, now playing, the display and sound sliders)
  over a dimming backing panel; the 2026-11-20 "compact" panel was an expanded
  Bluetooth module, which wears the menu glass.

## 12. Evidence

The captures are local and not committed:

- `~/src/captures/d3-{white,gray,black,checker}-menu.png` and
  `d3-*-cc.png`: dark mode matrix under the current wallpaper.
- `~/src/captures/l2-{white,gray,black,checker}-menu.png`,
  `l2-*-cc.png`, `l3-white-cc.png`, `l3-black-cc.png`: light mode matrix.
- `~/src/captures/d3-dock-eh1355.png`, `l3-dock-eh1355.png`: the boundary
  refraction tests in both modes.
- `~/src/captures/d2-rt-*.png`, `l3-rt-*.png`: Reduce Transparency passes.
- `~/src/captures/d2-{white,black,checker}-nc.png`, `l3-*-nc.png`:
  Notification Center clock pill.
- `~/src/captures/d2-checker-banner.png`, `l3-checker-banner.png`,
  `d2-checker-spotlight.png`: banner and Spotlight.
- `~/src/captures/d2-finder.png`, `l3-finder.png`: window chrome.
- `~/src/captures/l3-wallcheck.png`, `d3-wallcheck.png`: the wallpaper
  controls proving the two passes share one wallpaper.
- `~/src/captures/defaults-2026-11-20-modes/`: the backup before this pass.
- `~/src/captures/blur-*.png`: the 2026-11-19 pass at 1920x1080.
- `~/src/captures/shellblur/dark/<surface>-<name>.png` (2026-11-22, raw
  captures; `<surface>` is `bar`, `appmenu`, `cc`, `nc`, `dock`, `banner`, `spotlight`, `apps` or `mc`; `<name>` is
  `g<level>`, `c<r>-<g>-<b>`, `checker<cell>`, `stripes<period>`, `ev|evi<d>`,
  `eh|ehi<d>`, `kg<level>`, `sl<lo>-<hi>`, `bw|bb<side>`), `<surface>-state.json`,
  `<surface>-captures.jsonl` and `results.json`; probes `backdrop`, `keys`,
  `shellstate`, `shellblur.sh`, `shellblur-all.sh`, `shellblur-more.sh`,
  `shellblur-kernels.sh`, `shellblur-queue.sh` and the analysis `shellblur.py`,
  `shellblur-fit.py`, `shellblur-chroma.py`, `shellblur-crossval.py`,
  `shellblur-pyramid.py`, `shellblur-kawase.py`, `shellblur-edge.py`,
  `shellblur-results.py`, `shellblur-samples.py` in `~/src/probes`.

## 13. Implementation prompt

Hand this to one subagent per task and keep the sequence.

```text
You are fixing the macOS glass model from measured values. Read this file
first, then the touched crates. Work in ~/src/barrierkit and
~/src/malaysia, main only: git pull --ff-only before editing and before
each push, one commit per session, English messages, no code comments.

Measured targets, dark mode / light mode (backdrop behind the panel):
- Application menu: white #a3a3a3 / #f2f2f2, gray #92-94 / #d1d1d1,
  black #2b2b2b / #75-7b, checker #8b-8d / #bf-c6.
- Control Center: white #a3-b1 / #fbfbfb, gray #a3 / #c7c7c7,
  black #262626 / #666666, checker #7f / #bababa.
- Dock: white #bebebe / #ebebeb, gray #8b / #a6a6a6, black #0d-10 / #34,
  checker #6c / #91. RT dock: solid #03-0d / #f1-89; menu and CC unchanged
  under RT.
- Ink: dark ink on any panel above 0.4 luma, light ink below. The light
  panel over black is 0.46, the dark panel over black is 0.17.

Task 1, barrierkit: add the light veil set for the menu in
crates/ctdesign/tengen/components.json and resolve.rs so a light scheme
renders #75-7b over black and #f2 over white. The existing dark
tokens reproduce the old `#8d`, not `#a3`: refit them too.

Task 2, barrierkit: make the Dock and Reduce Transparency targets match
section 5 of this file. The Dock already picks by scheme; replace the
light values with the measured ones and make the RT dock a scheme solid.

Task 3, malaysia: verify the compositor feeds the same values in
crates/malaysia-compositor/src/blur.rs and focus_notice.rs after the
barrierkit rev bump (Cargo.toml, Cargo.lock, flake.nix, flake.lock with
nix flake update).

Tests: a four-backdrop test per surface per scheme in resolve.rs asserting
the eight target colours within 8/255. A pure logic test for the ink
threshold: 0.46 takes dark ink, 0.17 takes light ink.

Definition of done: ./dev.sh check passes in both repos; the tests assert
the measured values; barrierkit main is pushed and malaysia pins the new
rev; the report lists the changed tokens, the model inputs and outputs,
and the commits.
```

---

## 14. Desktop surfaces as a pipeline, measured 2026-11-22

**Goal:** give the compositor one function per desktop surface that reproduces
what macOS 27.2.1 draws over any backdrop, with the error stated, and say how
macOS composes and configures each blur.
**Scope:** the menu bar, the application menu panel, notification banners and
cards, Spotlight, Apps (the Launchpad of macOS 27), Control Center (tiles and
backing panel), the Dock and Mission Control's Spaces bar, in dark mode, Reduce
Transparency off, Increase Contrast off, scale 1. Section 14.9 lists what was
not measured.
**Reader:** whoever changes `ctappearance::shell_glass`, the compositor's glass
pass or a Malaysia shell surface.
**Input:** the probes of [section 14.1](#141-method), the captures of
[section 12](#12-evidence) and the system resources of [section 14.8](#148-configuration-and-provenance).
**Output:** the pipeline, its numbers per surface, the kernel, the edges, the
error of Malaysia's blurs, the provenance of each value and the parity
procedure.

### 14.1 Method

1. **State.** `shellstate` records the appearance (dark), Reduce Transparency
   (off), Increase Contrast (off), the scale (1) and whether the session is
   locked. No system setting was changed: the Dock, the appearance and the
   wallpaper are the user's. The session was locked until 06:59 and again from 07:55
   to 10:35 of 2026-11-22; a locked session captures only the wallpaper, so the
   drivers refuse to run.
2. **Backdrop.** `backdrop live:<file>` is an ordinary window at level 3 that
   fills the screen and redraws the mode a file names: `gray:<n>`,
   `rgb:<r>,<g>,<b>`, `checker:<cell>`, `stripes:<period>`,
   `edgev|edgevi|edgevl:<x>`, `boxat:<cx>:<cy>:<w>:<h>:<inside>:<outside>`. One
   process serves a whole series under a surface that stays open, so the surface
   is the same pixel for pixel across the series.
3. **Surface.** `shellblur.sh` opens it (`mouse` on the menu-bar item, `keys` for
   Spotlight, a notification for the banner, `open -a Apps`, `open -a "Mission
   Control"`), checks that it is open (a probe pixel differs from the backdrop,
   three tries), captures the screen after each swap and closes it with Escape. It
   leaves 10 s after it ends and changes
   nothing.
4. **Reading.** Every value is a raw byte of the PNG (PIL; `sample` only since the
   fix of 2026-11-22, [section 9.1](#91-2026-11-21-mid-tones-read-with-sample-were-too-light)).
   The input of a sample is the bare backdrop in the same capture (a 60x60 patch
   away from every surface): it equals the drawn level (16, 32, 64, 96, 128, 160,
   192, 224, 255 read 16, 32, 64, 96, 128, 160, 192, 224, 255, std 0.5). The
   display profile has an exact sRGB curve, so a raw gray is its sRGB gray. Colours
   are read as device values, the capture's own bytes (BT.2020 primaries).
5. **Series per surface.** Twelve gray levels, nine colours, checkers of 2, 4, 8
   and 16 px, stripes of period 8, 16, 32 and 64 px and white|black edges in both
   polarities; then, in a second pass that captures its own gray levels and finds
   its own geometry (the Notification Center changes between passes), ten held-out
   gray levels, 24 more colours, low-contrast steps (0|64, 48|80, 112|144, 192|255),
   centred boxes of 8 to 256 px (white on black and black on white) and two boxes
   that are exactly the surface (`rimb` black over white, `rimw` white over black).
6. **Windows.** The value of a surface is the mean of a 24x8 window that is flat
   (within 1 level of its plateau) in the gray 128 capture and lies inside the
   glass, found by `shellblur-fit.py geom` from where a white and a black backdrop
   come out less than 200 levels apart.

Probes, drivers and fits are in `~/src/probes` (`README.txt`); the raw captures
are in `~/src/captures/shellblur/dark` and stay outside the repository.

### 14.2 The menu bar is clear

The bar rows (y 4 to 26) equal the rows 32 px below in every capture of the
series, to 0 levels: gray 0 to 255, checkers of 2, 4, 8 and 16 px and stripes of 8
to 64 px (700 to 1000 px wide). The bar has no blur, no veil and no tint over any
backdrop. This confirms [section 4](#4-the-measured-matrix-light-mode)'s "clear"
with the error bound it lacked (0 of 255); light mode was not measured.

### 14.3 The pipeline

Two families, both over device values (the bytes of the capture, 0 to 255).

**Glass** (application menu panel, banner, Spotlight, Apps panel, notification
card, Control Center tile and backing panel, Dock):

```text
blurred = (1 - w) G(sigma) * b + w G(halo_sigma) * b
Y       = 0.2126 R + 0.7152 G + 0.0722 B   of the blurred pixel
out_c   = tone(Y) + s(Y) (blurred_c - Y)    for each channel c
```

`tone` is the gray response (12 levels, linear between them), `s(Y)` the gain of
the colour difference (five knots at Y = 0, 64, 128, 192, 255). The luma has the
weights of Rec. 709 and is taken over device values: with the BT.2020 weights the
luma law of the nine first colours of the menu misses by 4.2 levels instead of 0.9,
and over values converted to extended sRGB it misses by 28 (red 48 against 76, green
113 against 141).

**Spaces bar** (Mission Control) is the layer stack of the in-window materials
([`macos-materials.md`](macos-materials.md#102-the-pipeline)): a Gaussian of sigma
27.5 that sees only its own region, a saturation of 1.6 (`Y + 1.6 (c - Y)`, clamped), a
veil of gray 46 at alpha 0.4 and a cap of 169 per channel. Over 21 samples (grays and
colours) the worst channel misses by 0.66 level; a blue backdrop (59, 28, 243) comes
out (57, 27, 169): the blue is capped, as no luma tone would do.

Checks, glass:

- **Held-out grays.** Ten levels not in the table: the linear interpolation misses
  by 1.0 level (menu, Control Center) and 0.9 (Dock) at most.
- **Held-out colours.** The `s(Y)` knots were fitted on nine colours and tested on 23
  to 24 others: the worst channel misses by 1.7 (menu), 0.9 (Dock) and 2.3 (backing
  panel) levels. For the tile the nine-colour fit gives 8.1 (its training set holds
  `rgb:0,0,255`, whose blue clips at 255 and bends `s(Y)` near Y = 48) and the table
  fitted on all 33 colours misses by 1.3 at most. A channel that reaches 255 or 0 is
  clipped on screen and is not compared.
- **Mixing is in encoded values.** The 8 px checker comes out where the model puts
  the mean of black and white, 127.5: menu 122 (model 122), tile 111 (110), backing
  panel 115 (115). A mixing space of exponent 0.4 to 2.2 on the encoded values fits
  the Dock's steps worse than 1.0 (0.7 to 3.9 levels against 0.5) and its 0|255 edge
  worse too (0.5 to 2.2 against 0.4).
- **No noise.** A flat window of the menu, banner, Spotlight and Apps has std 0.00;
  the only grain is the capture's own (std 0.5 on the bare backdrop).
- **Alpha.** Every capture is opaque. A glass surface shows the same pixel over a
  backdrop whatever is under the backdrop: the glass has no transparency of its own
  beyond the backdrop it blurs.

[`ctappearance::shell_glass::over`](../barrierkit/04-appearance-and-tokens.md#shell-glass)
is the glass function (the Spaces bar uses `ctappearance::material::over`); the tests
reproduce every gray and colour sample of the table within 2 levels (1 level the
Spaces bar).

### 14.4 The numbers, dark

Raw output over the four classic backdrops (gray is the raw 146 that
`NSColor(calibratedWhite: 0.5)` renders):

| Surface | White 255 | Gray 146 | Black 0 | Checker 8 px |
|---|---|---|---|---|
| Application menu panel, banner, Spotlight, Apps panel | 146 | 129 | 32 | 122 |
| Notification card | 145 | 116 | 32 | 108 |
| Control Center tile | 186 | 121 | 32 | 111 |
| Control Center backing panel | 230 | 131 | 0 | 115 |
| Dock | 175 | 119 | 13 | 96 (the 8 px cells are not flattened) |
| Spaces bar | 169 | 106 | 18 | 95 |

`tone`, raw output at each input level:

| Surface | 0 | 16 | 32 | 48 | 64 | 96 | 128 | 160 | 192 | 224 | 240 | 255 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Menu panel, banner, Spotlight, Apps (the four agree to 0.5 level) | 32 | 44 | 57 | 70 | 82 | 104 | 122 | 135 | 143 | 147 | 147 | 146 |
| Notification card | 32 | 42 | 52 | 62 | 72 | 92 | 108 | 122 | 133 | 141 | 143 | 145 |
| Control Center tile | 32 | 41 | 51 | 61 | 71 | 90 | 110 | 129 | 149 | 168 | 178 | 186 |
| Control Center backing panel | 0 | 14 | 29 | 43 | 58 | 87 | 115 | 144 | 173 | 202 | 217 | 230 |
| Dock | 13 | 26 | 39 | 52 | 64 | 87 | 108 | 128 | 145 | 161 | 169 | 175 |
| Spaces bar | 18 | 28 | 38 | 47 | 57 | 76 | 95 | 114 | 134 | 153 | 162 | 169 |

The menu panel is a smooth shoulder, not a veil with a hard cap (veil and cap
misses by 6.3 levels). The tile is a line (31.7 + 0.609 in, veil and cap fit it with
0.4), the backing panel a pure dimming (0.903 in: a black veil of alpha 0.097) and the
Spaces bar a line capped at 169 (18.4 + 0.600 in). The banner, Spotlight and Apps are
the same glass as the application menu; the notification card is another variant.

`s(Y)`, the gain of the colour difference (glass family):

| Surface | s(0) | s(64) | s(128) | s(192) | s(255) | worst channel, all colours |
|---|---|---|---|---|---|---|
| Menu panel | 1.19 | 1.15 | 1.02 | 0.86 | 0.69 | 1.7 |
| Banner | 1.20 | 1.15 | 1.02 | 0.87 | 0.69 | 1.2 |
| Spotlight | 1.20 | 1.15 | 1.02 | 0.87 | 0.69 | 1.2 |
| Apps panel | 1.20 | 1.15 | 1.02 | 0.86 | 0.68 | 1.0 |
| Notification card | 1.01 | 0.92 | 0.87 | 0.77 | 0.67 | 1.1 |
| Control Center tile | 1.09 | 1.09 | 1.08 | 1.08 | 1.08 | 1.3 |
| Control Center backing panel | 0.91 | 0.90 | 0.90 | 0.90 | 0.91 | 0.8 |
| Dock | 1.23 | 1.17 | 1.11 | 1.05 | 0.99 | 1.0 |

A saturated primary leaves the menu panel nearly unchanged (device red 208, 74, 34
comes out 224, 79, 36) while white goes to 146: the glass caps the luma, not the
channels. The notification card's table comes from the nine colours of its run only.

### 14.5 The kernel

The checkers bound the blur from below (a Gaussian under 5.5 px leaves the 8 px
cells visible); steps, boxes and stripes give its width. The half-way point of a
low-contrast step (0|64) sits where the edge is: **0.0 px** (menu) and **-0.1 px**
(tile), so nothing is displaced at the centre of a surface: no refraction, as in
[section 7.1](#71-the-boundary-test-for-refraction). Stripes and checkers do not
distinguish a Gaussian from a Kawase; steps and boxes do (a Dual Kawase fits the
menu's step 0.002 worse than a Gaussian and has no halo).

Fitted with the tone after the blur, mixing in encoded values and one or two Gaussians
(`shellblur-pyramid.py <dir> <surface> two`), from the second pass:

| Surface | core sigma | halo sigma | halo weight | rms, steps and boxes | rms, stripes |
|---|---|---|---|---|---|
| Application menu panel | 9.0 px | 128 px | 0.20 | 4.2 levels | 7.5 |
| Spotlight | 10.5 px | 64 px | 0.20 | 4.0 | not measured |
| Apps panel | 8.0 px | 64 px | 0.25 | 4.8 | 9.3 |
| Notification card | 9.0 px | 64 px | 0.20 | 4.6 | 6.9 |
| Banner | not measured (the banner vanished during the second pass) | | | | |
| Control Center tile | 3.5 px | none | 0 | 0.9 | 6.9 |
| Control Center backing panel | about 1 px (checker lock-in, 0.7 to 1.2) | none | 0 | not fitted | not measured |
| Dock | 2.0 px (0.75 px for a 0\|255 edge) | none | 0 | 0.5 | not measured |
| Spaces bar | 27.5 px | none | 0 | 1.1 | 0.3 to 0.8 |

What the fits leave out:

- The glass of the menu family shows more contrast in a black box of 16 or 32 px (28
  and 17 levels off in the menu, 47 and 18 in Apps) and in fine stripes (period 32:
  15 off) than blur then tone gives, and the white boxes of the same size are within 3
  to 7. The error repeats between the two passes. The Dock's 0|255 edge is sharper (0.75
  px) than its low-contrast steps and boxes (2.0): the width depends on the contrast,
  which no linear blur followed by a point tone gives. The glass has several blurs
  (below), so the model states its error and keeps blur then tone.
- The Control Center tile fits every step within 0.5 level and every box of 32 px or
  more exactly, misses by 3 to 3.5 on the 16 px boxes and by 7 on the 8 px boxes: the
  kernel is not a Gaussian at the smallest scale.
- The Dock's boxes and steps were taken in its 9 px strip above the icons: the strip
  is flat but sits 6 to 15 px from the rim.

The menu family's two components agree with the configuration of the glass filter read
in a window ([`macos-materials.md`](macos-materials.md#12-glass-tiles-observed-2026-11-22)
and its probe of the blur radius: the backdrop of a glass is scaled by 0.5 and a radius
is 0.99 sigma on it): a `inputBlurRadius` 5 is 9.9 px on screen and a bleed radius of 42
is 83 px. A kernel with those two sigmas and a weight of 0.2 or 0.25 misses the
captures by 4.6 to 6.3 levels (menu 5.7, Spotlight 4.6, Apps 6.3, card 6.2), 1 to 2
levels more than the free fit: the captures decide, the configuration explains them.
The same menu tone is also the regular dark glass tile of a window
(over gray 0, 32, 64, 128, 192, 224, 255 it shows 42, 59, 84, 123, 144, 147, 151; the
panel 32, 57, 82, 122, 143, 147, 146): within 2 levels from 32 to 224, 10 apart over
black and 5 over white. The shell's panel is the same filter family with other inputs.

### 14.6 Edges: outline, shadow and what a blur sees beyond them

- **The blur does not see beyond the edge.** `rimb` (a black box that is exactly the
  surface over white) and `rimw` (white over black) show what the glass next to its
  own rim samples. Hypotheses: the whole kernel sees the screen; the core is
  normalised to the region and the halo sees the screen; both are normalised.
  Control Center tile, over the black and white boxes: 10.9 and 11.4 levels for the
  first hypothesis, 0.0 and 2.0 for the others (measured 32 and 187 flat to the rim):
  normalised. Spaces bar: 54 against 3.0 and 1.0: normalised. Dock: the black box
  stays at 12 to 13 to the rim, as the normalised models say (12) and not the first
  (34 at the rim). Application menu, black box: 23.3 (left rim) and 14.1 (right)
  for the first, 30.3 and 29.4 for the third, **6.6 and 6.0** for the second: the
  core is normalised to the region and the halo sees the screen (the black box
  shows 60 to 64 near its rim, where the normalised core would give 32); the white
  box is within 1.0 to 4.6 of all three. The Apps panel agrees with the menu (56 to 58 near the rim of
  the black box, 51 to 56 modelled); the Notification Center's union of cards was not
  a usable box.
  So the blur is the `inputNormalizeEdges` of the in-window materials
  ([`macos-materials.md`](macos-materials.md#14-the-blur-algorithm-and-malaysias-dual-kawase-2026-11-22))
  for the core and the tiles, and the wide bleed samples outside.
- **Outline and shadow**, over white. The pixel line at the rim (inside to
  outside), then the black alpha of the shadow at d px:

| Surface | rim pixels | alpha at d = 1, 2, 5, 10, 20, 40 |
|---|---|---|
| Application menu panel | 143, 143, 145, **66** | 0.114, 0.106, 0.090, 0.063, 0.027, 0 |
| Spotlight pill | 158, **90, 22** | 0.263, 0.251, 0.216, 0.169, 0.082, 0.016 |
| Apps panel | 146, 148, 158, **8** | 0.329, 0.314, 0.275, 0.208, 0.106, 0.012 |
| Notification card | 143, 143, 149, **116** | 0.208, 0.204, 0.184, 0.157, 0.106, not measured |
| Control Center tile | 186, 186, 189, **152** | on the backing panel: 0.098 flat |
| Spaces bar (bottom edge) | 169, 169, 169, **198** | 0.133, 0.125, 0.114, 0.090, 0.051, 0.004 |
| Dock (left end) | 176, 176, 176, **104** | 0 |

  The menu panel's shadow is also an exponential, 0.134 exp(-d / 12.3) (rms over
  white and gray). The Control Center tiles sit on a backing panel that dims what is
  behind by 0.903 and blurs it by about 1 px: the strip beyond them (x 2549 to
  2559, y 60 to 500) is flat at 0.903 of the backdrop.

### 14.7 Malaysia's blurs against the measured kernels

Malaysia blurs the Dock with a Gaussian of sigma 3 (`dock_soften.frag`, `SIGMA` 3
times the `soften` of 1.0), menus, popups and notifications with sigma 6 (`soften`
2.0) and everything behind a window with a Dual Kawase of 3 passes at offset 3.0
(sigma 17.2 by its second moment, 16.8 by the profile of
[`macos-materials.md`](macos-materials.md#14-the-blur-algorithm-and-malaysias-dual-kawase-2026-11-22)).
Applied to every step and box of the second pass, with the measured tone after the
blur, the mean rms and the worst item in levels:

| Surface | Malaysia's blur for it | Dock sigma 3 | Popup sigma 6 | Wallpaper Dual Kawase | Measured model |
|---|---|---|---|---|---|
| Application menu panel | popup | 25.7 / 88 | **17.4 / 48** | 15.2 / 54 | 4.2 / 29 |
| Spotlight | not checked | 27.3 / 107 | 18.6 / 70 | **9.9 / 54** | 4.6 / 26 |
| Apps panel | none (no Apps) | 21.2 / 83 | 13.8 / 41 | 15.0 / 57 | 5.9 / 50 |
| Notification card | popup | 21.9 / 79 | **13.1 / 44** | 12.9 / 51 | 4.6 / 38 |
| Control Center tile | none (no Control Center) | 1.9 / 13 | 9.4 / 53 | 34.7 / 125 | 0.9 / 4 |
| Dock | Dock | **4.2 / 46** | 15.5 / 112 | 41.1 / 147 | 0.5 / 2 |
| Spaces bar | none (no overview bar) | 55.2 / 144 | 43.7 / 121 | 17.5 / 52 | 1.1 / 3 |

None of Malaysia's blurs is within 2 levels (the tolerance of
[section 14.10](#1410-parity-check)) of the surface it draws, except the Dock's
sigma of 3 against a Control Center tile that Malaysia does not have. The Dock's
own error is 4.2 on average and 46 at worst (62 at its 0|255 edge); the menu's and the
notification card's, 13 to 17 on average and 44 to 48 at worst; the Spaces bar's
(a window overview Malaysia draws without a bar) 17.5.

What reproduces them: a region-normalised Gaussian at reduced resolution, as macOS
builds it. For the glass family, a core of sigma 9 (`inputBlurRadius` 5 on a backdrop
scaled by 0.5) plus a bleed of sigma 64 to 128 (radius 42) at weight 0.2 to 0.25 that
is *not* normalised, then the tone and chroma tables; for the tile and the Dock a single
sigma of 3.5 and 2.0 (and 0.75 for the Dock's high-contrast edges); for the Spaces bar
sigma 27.5 (radius 30 at scale 0.125). The in-window study measured that a Gaussian
on a backdrop downsampled by 4 equals the Dual Kawase's cost for the wide sigmas
([`macos-materials.md`](macos-materials.md#14-the-blur-algorithm-and-malaysias-dual-kawase-2026-11-22));
the narrow sigmas (0.75 to 3.5) are a separable Gaussian of 5 to 25 taps at full or half
resolution. Cost on a GPU was not measured here. A Dual Kawase of 1 to 3 passes with a
small offset reproduces a single narrow sigma (sigma 3.5 is 3 passes
at offset 0.37) but has no halo and does not normalise edges.

### 14.8 Configuration and provenance

| Value | Source | Where |
|---|---|---|
| Pipeline, tone, chroma, kernel, edges of each surface | measured: captures over the backdrops of section 14.1 | 14.3 to 14.6 |
| A glass's blur radius 5, fill radius 8, bleed radius 42, backdrop scale 0.5 | read: layer tree of an `NSGlassEffectView` in the probe's own process (runtime observation, not API), by the in-window study | [`macos-materials.md`](macos-materials.md#12-glass-tiles-observed-2026-11-22), 14.3 |
| Radius is 0.99 sigma on the scaled backdrop | read: throwaway probe with `CABackdropLayer`, by the in-window study | `macos-materials.md` section 14.3 |
| Saturation 1.6 of the Spaces bar | measured; equal to the `saturation` of `dockDark.materialrecipe` and `modulesBackground.materialrecipe` (read) | below |
| Dock recipe: `blurRadius` 30, `backdropScale` 0.25, `saturation` 1.6, `luminanceAmount` 0.5, `luminanceValues` [0.29, -0.2, 0.375, 0.65] | read: `CoreMaterial.framework/Versions/A/Resources/dockDark.materialrecipe` (macOS 27.2.1) | |
| Control Center modules: `colorMatrix` (m11 0.924, m12 -0.175, m13 -0.018, m15 -0.061; m21 -0.052, m22 0.801, m23 -0.018; m31 -0.052, m32 -0.175, m33 0.958) | read: `modules.materialrecipe`; its gray gain is 0.731 against 0.609 measured on the tile | |
| Control Center backing recipe: `blurRadius` 25, `saturation` 1.6, `luminanceAmount` 0.4, `zoom` 0.04 | read: `modulesBackground.materialrecipe` | |
| Notification platters: `blurRadius` 30, `backdropScale` 0.25, `saturation` 1.4 (dark), `luminanceAmount` 0.4 | read: `plattersDark.materialrecipe` | |

The recipes are the old CoreMaterial data; the captures do not follow them (the Dock is
0.75 to 2 px, not radius 30 at scale 0.25; the tile's gray gain is 0.609, not 0.731), so
they are listed for the record and the capture wins. The Dock, Control Center,
Notification Center, Spotlight and System UI Server bundles ship no material resource.
Nothing of Apple's was copied: the numbers carry their resource and key.

### 14.9 Not measured, and why

| Cell | Why | Exact setting needed |
|---|---|---|
| Light mode, every surface | the system appearance is global | System Settings › Appearance › Appearance = Light (`AppleInterfaceStyle` removed) |
| Reduce Transparency | system switch; a per-process flag changes no desktop surface | System Settings › Accessibility › Display › Reduce transparency (`com.apple.universalaccess reduceTransparency`) |
| Increase Contrast | same | Accessibility › Display › Increase contrast (`increaseContrast`) |
| Scale 2 (Retina) | the display runs a 1x mode | System Settings › Displays › Resolution = a scaled HiDPI mode (VG27WQ3B, "Looks like 1280 x 720") |
| Dock position, size, magnification, auto-hide | `com.apple.dock` preferences | Desktop & Dock › Size, Magnification, Position on screen, Automatically hide (`tilesize`, `largesize`, `magnification`, `orientation`, `autohide`) |
| Banner kernel | the banner vanished before its second pass | none: repeat when a notification stays |
| Notification Center cards under Reduce Transparency, widgets | needs the switch; the host has no widgets | Reduce transparency; Notification Center › Edit Widgets |
| Stage Manager strip | the feature is off | Control Center › Stage Manager = on, or Desktop & Dock › Stage Manager |
| Lock screen and login window | locking the only session blocks the host; a capture of a locked session shows only the wallpaper | none reachable |
| Inactive and focused states | the desktop surfaces have none | none |
| Control Center's expanded modules | not opened in this pass | none: open one module |
| GPU cost of the algorithms of 14.7 | no GPU timing was taken | a run on a GPU |

### 14.10 Parity check

To compare macOS with the model or the compositor over the same backdrop:

1. Draw the backdrop with `backdrop`, or show the same level or pattern in a client of
   the compositor, and open the surface.
2. Capture the screen (`screencapture -x` on macOS, the compositor's capture elsewhere)
   and read the bytes: raw, no colour conversion.
3. For each series (grays, colours, a step, a box, `rimb`, `rimw`) compare the flat
   window of the surface with `over(glass, bare backdrop)`, where the bare backdrop is
   read from the same capture.
4. Accept a gray if it is within 2 levels, a colour channel within 2 (1 for the Spaces
   bar), a step or box centre within the rms of [section 14.5](#145-the-kernel) and a
   surface outline within 1 px.

The 2 levels are the sum of the 1.0 level that linear interpolation of the tone misses
at most and the 0.5 level of the quantisation of an 8-bit capture, with 0.5 for the noise
of the bare patch (std 0.5); the in-window materials use 1.5
([`macos-materials.md`](macos-materials.md#107-parity-check)) because their fit is a
closed form.
