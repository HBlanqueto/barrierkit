# macOS keyboard focus ring

**Goal:** pin down the focus-ring width, colour and contrast so keyboard focus
looks native and stays accessible.
**Scope:** the keyboard focus indicator on a control or tab. It excludes the
text caret, the selection highlight and the pointer hover ring.
**Reader:** anyone who changes `interaction.focus_ring`, `FOCUS_RING` or the
focus colour in `ctappearance`.
**Input:** System Events focus on a Settings tab, the semantic colour dump and
the focus crop.
**Output:** measured values, a Malaysia comparison and ordered gaps.

## Steps

1. Read the limits in
   [`macos-observations.md`](macos-observations.md).
2. Use the width and colour table together.
3. Check the contrast target before changing the colour.
4. Do not use the focus ring as a pointer hover effect.
5. Do not copy an Apple focus-ring image.

---

## 1. Measured values

| Property | Value |
|---|---:|
| Width | 3 pt |
| Light colour | `#0067f4` at 49.8% |
| Dark colour | `#1aa9ff` at 49.8% |
| Shape | Follows the control's corner radius |
| Trigger | Keyboard focus |

The focus ring is a **3 pt translucent blue outline**. It is not a glow and
not a solid accent stroke. The alpha is about 50% in both appearances.

The ring was observed on a Settings tab through System Events. The crop is
`focus-crop.png` and the 4x crop is `focus-crop-4x.png`. Pixel sampling of
the crop was inconclusive because the crop coordinates did not line up with
the ring, so the colour values above come from `NSColor.keyboardFocusIndicator`,
not from the crop.

The exact offset from the control edge was not measured. The screenshots show
the ring outside the control, but the gap is not available as a number.

## 2. Contrast

The light ring is `#0067f4` at 49.8% over the window. The dark ring is
`#1aa9ff` at 49.8%. Because the alpha is low, the effective contrast depends
on the surface behind it:

- Over `#ffffff`, the light ring is a mid blue.
- Over `#1e1e1e`, the dark ring is a lighter blue.
- Over a coloured control, the result shifts toward the control colour.

The ring should keep at least **3:1** against the adjacent surface. That is the
WCAG 2.2 non-text contrast target for a focus indicator. BarrierKit follows
macOS by default and does not hold the translucent ring to 3:1; it keeps the
guarantee with Increase Contrast, where the ring is opaque (section 3).

## 3. Comparison with Malaysia

| Element | macOS (measured) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Width | 3 | 3 | `FOCUS_RING.width` | Matches |
| Light colour | `#0067f4` at 49.8% | `#0067f4` at 49.8% with the default accent; a custom accent at the same alpha | `FOCUS_RING_COLOR_LIGHT`, `resolve.rs` `focus_ring` | Matches |
| Dark colour | `#1aa9ff` at 49.8% | `#1aa9ff` at 49.8% with the default accent; a custom accent at the same alpha | `FOCUS_RING_COLOR_DARK`, `resolve.rs` `focus_ring` | Matches |
| Increase Contrast | Not measured; stronger or opaque expected | Opaque accent moved to 3:1 on the window | `resolve.rs` `focus_ring` | Done |
| Contrast target | 3:1 | 3:1 only with Increase Contrast | `resolve.rs` | Trade-off, see below |
| Shape | Control radius | Control radius | `interaction.rs` | Matches in principle |
| Offset | Not measured | `FOCUS_RING.width` outside | `interaction.rs` | Verify |
| Trigger | Keyboard | Keyboard | `interaction.rs` | Matches |

macOS draws a **translucent** ring and BarrierKit now does the same by
default. A translucent ring can fail 3:1 over a busy or light surface even if
its solid colour would pass. The decision is to follow macOS and keep the
accessibility guarantee on the Increase Contrast path, where the ring is the
opaque accent moved until it reaches 3:1 on the window. Without Increase
Contrast the ring is not held to 3:1.

## 4. Accessibility notes

- The ring must be visible on light, dark, accent and image backgrounds.
- It must not rely on colour alone. A 3 pt width and a shape change help.
- With Increase Contrast on, the ring should become stronger or opaque.
- With Reduce Motion on, the ring should not fade or pulse.
- The focus ring is separate from the selection highlight. A focused,
  selected row needs both, not one.

## 5. What not to copy

- Apple's focus-ring image or shader.
- The exact blue for a branded accent without checking contrast.
- A glow or shadow that changes the ring's measured width.

## 6. Prioritized changes

1. **Add a translucent focus colour pair.** Done: `FOCUS_RING_COLOR_*`.
   The default is not verified against 3:1; that is the accepted trade-off.
2. **Keep the 3 pt width.** It already matches.
3. **Add a focus-ring contrast test over four backgrounds.** Partly done:
   `resolve.rs` tests the Increase Contrast ring at 3:1 on the window for
   every accent swatch. The translucent ring has no contrast test and the
   four backgrounds are not covered.
4. **Measure the offset.** The ring's distance from the control edge is still
   unknown.
5. **Make Increase Contrast opaque.** Done.
6. **Do not reuse the ring for pointer focus.** Keep pointer hover and
   keyboard focus distinct.

## 7. Open questions

- What is the exact ring offset at 1x and 2x?
- Does the ring sit outside the control or inside its bounds?
- Does macOS draw two rings for a selected, focused tab?
- How does the ring change with Increase Contrast?
- Is the dark ring the same hue as the light ring or a different blue?
