# macOS control metrics

Measured on macOS 27.2.1 with a Swift script: `fittingSize` of each control per `ControlSize`, and the `NSColor` semantic colors resolved in the `aqua` and `darkAqua` appearances through `performAsCurrentDrawingAppearance`.

## Sizes (width × height, points)

| Control | regular | small | mini | large | extra large |
|---|---|---|---|---|---|
| Push button | 24 | 20 | 16 | 28 | 36 |
| Checkbox | 16 | 14 | 12 | 18 | 18 |
| Radio button | 16 | 14 | 12 | 18 | 18 |
| Switch | 54 × 24 | 54 × 24 | 54 × 24 | 54 × 24 | 54 × 24 |
| Slider | 16 | 14 | 12 | 20 | 20 |
| Pop-up button | 24 | 20 | 16 | 28 | 36 |
| Combo box | 24 | 20 | 16 | 28 | 36 |
| Segmented control | 24 | 20 | 16 | 28 | 36 |
| Text field | 24 | 24 | 24 | 24 | 24 |
| Search field | 24 | 20 | 16 | 28 | 36 |
| Stepper | 20 × 24 | 17 × 20 | 13 × 16 | 23 × 28 | 30 × 36 |
| Scroller | 17 | | | | |

Heights agree with `components.json` for button, checkbox, radio, switch, pop-up, segmented, text field and search field.

## Open differences

- `stepper.height` follows the measured 24; the compact kit golden needs regenerating on Linux.
- The regular slider is 16 high with a 19.5 × 16 knob and a 6 high track starting 5 from the top, in light and dark; `slider.height` and `slider.knob-width` follow it. Mini, small and large knobs are 15.5, 17.5 and 23.5 wide.

## Control sizes

Read with `~/src/probes/buttons.swift` (`buttons-capture.sh` writes `~/src/captures/buttons`: 4x and 2x captures in aqua, dark aqua and both Increase Contrast looks over white and black; `BUTTONS_INACTIVE=1` draws the window as inactive) and `~/src/probes/sizes.swift` (the layout size of every bezel at every size). The bench is `scripts/appkit-render.sh buttons` and `scripts/appkit-compare.py controls --family buttons`; its `%bezel` column leaves out the box of the label ink on both sides, so it measures the bezel and the glyphs, not Inter against SF Pro.

- **Push, pop-up, pull-down, segmented (mini, small, regular, large, extra large).** Heights 16, 20, 24, 28, 36. Corner radius 4, 5, 6, then a capsule (14 and 18). Label 9, 11, 13, 13, 13. The label's ink starts 9.0, 10.75, 13.0, 15.0, 19.0 from the left of a push button, so its padding is 9.05, 10.45, 12.35, 14.35, 18.35; the kit's is 0.35 less (`BUTTON.mini`, `small`, `large`, `xl`) because Inter's labels are wider. A default button has the same boxes in the accent.
- **Pop-up arrows.** The up-down pair is 6 × 9 at mini and small, 7.25 × 10.5 at regular and 9 × 13 at large and extra large; its ink ends 7, 9, 9.75, 11, 14 from the right edge. "Option" gives widths 62, 76, 89, 97, 111; a pull-down "Pull" 49, 60, 71, 79, 93 (`BUTTON.popup`).
- **Segmented.** One, Two, Three give 134, 146, 158, 170, 194 wide; a segment is its label plus 12.4, 12.2, 12, 14, 18 on each side (`SEGMENTED.mini`…).
- **Checkbox and radio.** The box is 12, 14, 16, 18, 18 with a corner of 3.6, 4.6, 5.65, 6.7, 6.7. The checkmark's ink is 7.5 × 7.0, 8.75 × 8.0, 9.75 × 9.25, 11 × 11, 11 × 11, so the mark grows with the box; the radio dot is 4, 5, 5, 5, 5. The label's ink starts 4.0, 4.75, 6.75, 6.75, 6.75 from the right edge of the box. An unchecked box has no outline in any look (4x and 2x, and the 1x screen): the kit drew a hairline inside it and no longer does. At large the mark is drawn from a bitmap and is softer than a vector one.
- **Switch.** The layout frame stays 54 × 24 at every size and the control is drawn scaled around its center: the track is 36 × 16, 44 × 20, 54 × 24, 64 × 28, 80 × 36, the knob 21 × 14, 26 × 16, 32 × 20, 38 × 24, 47 × 30, with 1, 2, 2, 2, 3 between the knob and the track. The kit does the same (`SwitchStyle::frame_size`).
- **Slider.** Heights 12, 14, 16, 20, 20; the track is 4, 4, 6, 6, 6 thick and centered; the knob is 15.5, 17.5, 19.5, 23.5, 23.5 wide.
- **Stepper.** Frames 13 × 16, 17 × 20, 20 × 24, 23 × 28, 30 × 36; the corner is 4, 5, 6, 7, 9; the chevrons' ink is 7.75, 9, 11.25, 11.25, 11.25 wide, so the extra large one keeps the regular chevron in a larger bezel. The mini bezel is drawn 13 × 20 inside its 16 high frame.
- **Button bezels** (`NSButton.BezelStyle`, regular, "Button" label; `buttons` probe, the colors solved over white and black). `push`, `flexiblePush`, `roundRect`, `recessed`, `accessoryBar` and `accessoryBarAction` draw the same 24 high push bezel (black or white 8%, 18% under Increase Contrast); `helpButton` is a 24 circle in that fill and `circular` a 17 circle; `pushDisclosure` and `roundedDisclosure` a 24 square of radius 6 in it with a chevron 12.5 wide. `smallSquare` is a 47 × 21 rectangle (padding 3.35) with an opaque gradient from `fafafa` to `f5f5f5`, a `b4b4b4` border and a black label in light, white 15% to 12% and a 24% border in dark; pressed it is `bdbdbd` (white 24% in dark), disabled it is half as opaque. `texturedRounded` and `toolbar` are a 55 × 20 capsule (padding 7.35) with no fill and a 1 ring of black or white 5% (47% under Increase Contrast); pressed it fills with 10% (opaque `cbcbcb` under Increase Contrast light, white 36% dark). `texturedSquare` is 49 × 20 (padding 4.35, radius about 4.5) with a 90% white gradient `fdfdfd` to `f5f5f5` in light and white 5% in dark; pressed `e4e4e4`. `inline` is a 45 × 18 capsule with a 5% fill (15% under Increase Contrast) and an 11 semibold label. `shadowlessSquare` is a 51 × 22 variant of `smallSquare`; `badge` looks like `inline`. The legacy bezels are bitmaps, so a 4x capture shows their edges soft.
- **SwiftUI** (`~/src/probes/swiftuibuttons.swift`, `NSHostingView` per item, `.controlSize` mini to extra large). `.bordered`, `.borderedProminent` and `Button(role:)` are the push button at 65 × 24 (and 55 × 20, 69 × 28, 77 × 36), except the mini one, which is 46 × 13 where AppKit's is 46 × 16; `.automatic` and `.buttonBorderShape(.capsule)`, `.roundedRectangle` and `.circle` keep that frame (the circle is 23 × 21). `.borderless`, `.plain` and `.link` draw only the label, 41 × 16 at regular, 35 × 14 small and 30 × 11 mini (the same 41 × 16 at large and extra large). A destructive role draws the plain bordered button: no red, as with AppKit's `hasDestructiveAction`. A disabled `.borderedProminent` is the accent at 70% (`4fa3ff` over white), where AppKit's disabled default button is gray. A `Toggle` row is its label plus the control (the switch row is 103 × 24 at every size); the switch itself is drawn as AppKit's 36 × 16 to 80 × 36 and is clipped by its 54 × 24 frame, the checkbox box is 12, 14, 16, 18, 18 with the label 1.75 nearer than AppKit's, and the `.button` toggle is the push button filled with the toggle accent (`0078f9`) when on. A `Picker` `.segmented` gives equal segments (126, 153, 180, 192, 216 wide for One, Two, Three) where AppKit's are fit to their text, `.menu` is 58, 71, 84, 92, 106 wide for "One", and `.radioGroup` and `.inline` are a column of three 16 high radios (57 × 62 at regular). Slider and stepper match AppKit at every size.
- **Bench results** (`appkit-compare.py controls --family buttons`, share of pixels that differ outside the label ink, light, dark and Increase Contrast, over white and black; the SwiftUI family is in `--family swiftui`). Push, default, pull-down, help, round, disclosure and the five push-like bezels: 3% or less. Pop-up: 3%, and 9% when disabled in the small sizes. Segmented: 1%, and 6.6% disabled, because AppKit draws the translucent selected segment over the window and the kit draws it over the group fill. Check and radio: 5% or less, 6.6% for the large disabled mark and 6% for the mini radio; the large mark is a soft bitmap in AppKit. Switch: 4% or less. Slider: 3% or less. Stepper: the bezel 1% (8% disabled in light), but the chevrons differ by 5% to 10% of the box: AppKit draws them from a bitmap whose legs are cut flat at the divider, the kit draws round-capped strokes. Push disclosure: 2%. Inline: 2%, and 14% disabled in dark over black, where the label color differs. Textured rounded: 4% in light and 13% under Increase Contrast, where the ring is centered on the edge and about 1.5 wide. Small square and textured square: 11% to 15%, and 18% for the disabled and pressed states; they are bitmaps in AppKit. Not compared: the focused look (the 4x capture draws no focus ring), the inactive window (the scene cannot make the kit window inactive) and the glass bezels (the compositor draws them).
- **Disabled and pressed** states keep the regular colors at every size (`DisabledPalette`, `PressedPalette`): the bench differs by the same share as the normal state.

## Semantic colors (RGBA)

| Role | Light | Dark |
|---|---|---|
| separator | `0000001a` | `ffffff1f` |
| placeholder text | `0000007f` | `ffffff8c` |
| tertiary label | `00000042` | `ffffff3f` |
| quaternary label | `00000019` | `ffffff19` |
| header text | `000000d8` | `ffffffff` |
| selected content background | `0064e1` | `0059d1` |
| unemphasized selection | `dcdcdc` | `464646` |
| selected text background | `b3d7ff` | `3f638b` |
| keyboard focus indicator | `0067f47f` | `1aa9ff7f` |
| grid | `e6e6e6` | `1a1a1a` |
| control background, text background | `ffffff` | `1e1e1e` |
| alternating rows | `ffffff`, `f4f5f5` | `1e1e1e`, `ffffff0c` |
| under-page background | `f6f6f6` | `282828` |

A command-line process has no window context, so window-level colors here are not authoritative; the window background in `tokens.json` stays as measured earlier.

## Accent and selection

`controlAccentColor` is `007aff` in both appearances; `selectedContentBackgroundColor` is `0064e1` in light and `0059d1` in dark, and the menu and list selection use it. Read per `AppleAccentColor` in a process-local argument domain (`-AppleAccentColor <n>`), so no system setting changed:

| Accent | controlAccent light / dark | selectedContentBackground light / dark |
|---|---|---|
| blue (default) | `007aff` / `007aff` | `0064e1` / `0059d1` |
| red | `e0383e` / `ff5257` | `c4262b` / `d13539` |
| orange | `f7821b` / `f7821b` | `d96b0a` / `c96003` |
| yellow | `ffc726` / `ffc600` | `e1ac15` / `d19e00` |
| green | `62ba46` / `62ba46` | `4da033` / `43932a` |
| purple | `953d96` / `a550a7` | `7d2a7e` / `803482` |
| pink | `f74f9e` / `f74f9e` | `d93b86` / `c93379` |

`ACCENT_SWATCHES` holds the light `controlAccentColor` of each except graphite, which was not measured; the dark values of red, yellow and purple differ and have no slot yet.

## Text fields

The fill is `textBackgroundColor` (`fffffff9` light, `161616fa` dark) over 24, with a 1 px border outside the frame: black at 15% in light, white at 9% in dark. The kit draws that border inside the 24.

## Reduce Transparency and Increase Contrast

Read with `NSWorkspace` flags overridden for the probe process only (`-reduceTransparency 1`, `-increaseContrast 1` on the command line), so no system setting changed. `NSColor` is not affected by Reduce Transparency: every role above is identical with it on. Increase Contrast changes them:

| Role | Light | Dark |
|---|---|---|
| label | `000000ff` | `ffffffff` |
| secondary label | `000000c1` | `ffffffb2` |
| tertiary label | `0000008e` | `ffffff7f` |
| quaternary label | `00000059` | `ffffff4c` |
| separator | `00000033` | `ffffff33` |
| grid | `a5a5a5` | `4d4d4d` |
| control accent, selected content | `0040dd` | `1774da` |
| keyboard focus indicator (opaque) | `0022af` | `469af5` |
| control color | `ffffff` | `00000000` |
| window, control, text background | `ffffff` | `202020` |
| disabled control text | `000000c1` | `ffffff7f` |
| placeholder text | `000000c1` | `ffffffb2` |
| system fill | `00000033` | `ffffff33` |

Controls rendered with the same flags (`~/src/probes/controls.swift` on the high-contrast appearances) add: push, pop-up, segmented and stepper bezel `0000002d` light and `ffffff2d` dark; off fill of check, radio, switch and slider `00000059` and `ffffff4c`; disabled bezel `00000017` and `ffffff17`; text field border `00000045` and `ffffff30`, dark text background `0d0d0dfa`; default button and progress fill `0040dd` and `1774da`; the fill of a checkbox, radio, switch or slider that is on `003bca` and `287edd`. `ContrastPalette` holds these, and the kit matches them within 8% of the pixels.

Materials under Reduce Transparency need screenshots with the system switch on and are in [macos-materials.md](macos-materials.md).

## Rings, level indicators and boxes

- Determinate ring (`NSProgressIndicator` spinning, regular): 32 wide, arc from the top clockwise with flat ends, stroke 5 (radius 11 to 16), the full circle behind it at black 6% in light; the arc is `0a82ff` in light and `1769e6` in dark.
- Continuous capacity indicator: green is `systemGreen` (`34c759` light, `30d158` dark; yellow `ffcc00` and `ffd600`, red `ff383c` and `ff4245`), the track black 7% in light and white 10% in dark.
- Box (`NSBox`): fill black or white at 3.1%.
- Combo box: the field is the text field of 24, with a gray 20 high button inside.

- Check and radio marks (regular, 4x capture of the white mark on the accent fill): the check is a 7.4 wide and 7.2 high stroke of 2.1, its short leg going 2.7 right and 3.4 down and its long leg 4.9 right and 7.2 up, its box 3.0 to 12.75 across and 3.75 to 13.0 down in the 16 square; the radio dot is 5 across, centered. The kit drew a 3.8 mark and a 6 dot.

- Internal glyphs (4x captures of the glyph over its control, fitted with the kit's own renderer in `appkit_glyph_lab`; AppKit rasterizes them softly, so the fit uses the ink box and, where the edge is sharp, the ink mass): pop-up up-down pair, two chevrons each 8 wide and 5 high with 1 between and 11 over all, stroke 1.25; combo box chevron 7.5 × 4.5, stroke 1.5; stepper chevrons 12 × 7 at 3.0 and 15.0 from the top of its 24 high bezel with a 13.5 wide line at 12.0, stroke 1.6; the stepper of a date picker 13 wide and 20 high, centered on the 21 high field, chevrons 7.75 × 4.5, 7.5 from the field; help `?` 7.5 × 9.25, stroke about 1.8; search magnifier 13.25 square, stroke about 1.1. The kit's chevron paths were centered 0.5 off the box and are now centered.
- Search field: the magnifier is secondary label colored and 13.25 square (stroke 1.1, its center 13.4 from the left and 0.6 below the middle); the clear button is a disc 13 across, 5 from the right edge, black 54% in light and white 58% in dark, with a cross of 5.5 cut out of it in the field's background (stroke 1.07). The fields draw their border 1 wide outside their 24 high box.
- Switch: no outline on the off track; the knob shadow is about 40% of the kit's old one. A disabled switch is `0074f3` (light) or `0b80ff` (dark) at 70%, solved from captures over white and black; under Increase Contrast it stays the contrast toggle accent at 70%, because the capture over black shows no track to solve from.
- Date picker (probe: `datePickerElements = .yearMonthDay`, `en_GB`, UTC): the field is 78 wide and 21 high, 3 below the top of its 24 high frame, with a 7.5 gap to the stepper; its fill is the text background at 85% (`ffffffd9` light, `161616d9` dark, `0d0d0dd9` under Increase Contrast dark) and its border is black 8% light, white 4.6% dark, black 15% and white 9.8% under Increase Contrast; the `/` and `:` are drawn in the label color; the text starts 4.5 from the left edge. The kit's text is narrower (Inter digits), so its padding is 5.375 to keep the 78 wide box. The 3 pt drop is the probe frame's alignment, so the bench scene wraps the kit field in a 3 top margin.
- Stepper divider: 1 high, 12.0 from the top of the 24 high bezel (0.5 below its center), 14 wide; black 12% light, white 12% dark, 70% under Increase Contrast.
- Group box (`NSBox`, 11 pt title): a 38 high rounded box (radius 12, black or white 3.1%, no border) from 12 to 50 inside a 50 high frame; the title row is 12 high, its text is 11 pt and starts 10.75 from the left, overlapping the top of the box.
- Ring: the center line of the arc ends 1° before `value × 360°` (`PROGRESS.ring_trim`); the level indicator is 16 high, not 16.5 (the soft bottom edge only looked taller).
- Disabled controls use fixed alphas, not a dimmed whole: check and radio off black or white 5.1%, on 7.5% with a tertiary-label mark, off switch 5.1% after its 70% (its token is 7.1%), slider fill 5.1% over a 9.8% track (14.5% in all; under Increase Contrast 17.3% light over a 34.9% track and 14.9% dark over a 29.8% one), pop-up and stepper bezel 3.9%, stepper chevrons about 19%, segmented selected segment `0071eb` at 72.8% with a white 50% label, combo button 3.9%. Text and search fields look the same disabled when empty. `DisabledPalette` holds them, with the Increase Contrast set measured the same way (switch opacity 74% there).
- Pressed: push and pop-up bezel black or white 16.9% (46% under Increase Contrast), default button `006fe6` light and `1988ff` dark, check or radio off 18.8%, on `006ce1` and `1e8bff`; there is no outline while pressed and no scale (`BUTTON.pressed_scale` is 1). `PressedPalette` holds them.
- Color well: a 48 × 24 capsule bezel (black or white 9.8%) around a 42 × 18 swatch 3 in, with a 1 ring that is the color moved by 0.125 per channel, darker in light and lighter in dark.
- Round button (`NSButton` circular): a 17 circle with a 10 wide plus; the kit's `ButtonStyle::circular`.
- Secure field: the dots are 5.75 wide, 8 apart, the first 7.25 from the left and 0.4 below the center; the border is outside like the text field's.
- Token field: the field is 24 high with an outside border; the placeholder shows only while there are no tokens; the first token starts 10 from the left, 5 from the top of its 24 high line (the label sits 1 above the center of the token); tokens are 15 high rectangles of radius 2, 56 wide for "One" and "Two" (the kit's Inter text is 3 wider, so its padding is 15.5 instead of 17), 7 apart, filled `aad7ff` light and `0f4064` dark (`b3cdfb` and `27465f` under Increase Contrast).
- Level indicator (continuous and discrete): the filled part has a border 0.87 times the level color (0.35 light and 0.87 dark under Increase Contrast) and is cut square, with no border at its end unless the bar is full; the empty part has the track inside a `level_border` that sits outside it, over the backdrop. The cells touch and are 12 wide in the discrete one. AppKit draws these edges from a 1 px bitmap that it resamples with a sharpening filter, so at 2x the border goes into the fill over four pixels: the border color `b`, `b + 0.115 (f - b)`, `f - 0.115 (f - b)` and the fill `f` (`LEVEL_INDICATOR.edge_softness`); the kit draws them as three rings of half a hairline. The outer edges are crisp at the top, left and right; the corners fit rings of radius 2.09, 1.72, 0.65 and 0.05 (`LEVEL_RING_RADII`, circular). The bottom edge of a filled cell has its last row at 89% and a faint row under it at 11%; an empty cell's last row is at 78% with nothing under it (`LEVEL_BOTTOM`). Freya quantizes vertical coverage to quarters of a pixel, so the kit draws those two as pixel-aligned lines with opacity.
- Rating: filled stars 12 apart, `secondary label` ink for the value and `system fill` for the rest, no outline. The star is a regular pentagram (inner radius 0.382 of the outer, which is 6.03 pt, tips sharp) whose center sits 0.93 below the middle of its 12 pt cell; AppKit draws it from a 2x bitmap, so a 4x capture is soft at the edges where the kit's vector star is crisp. Relevancy (read at 2x): a 12 high track (black or white 10.2%, `RELEVANCY_TRACK`) with a fill over it (black 27.8% and white 33.3%) and 1 pt lines every 2 drawn over that fill (black 25.9% and white 27.5%); under Increase Contrast the track is the gauge's (50.2% and 30.2%), the fill black 53.3% and white 52.9%, the lines black 38.8% and white 29.8%. `RelevancyPalette` holds the fill and the lines.
- The circular `NSSlider` is a 24 gray circle with a short dark dash for the value; the kit has no counterpart.
- Mixed checkbox: the box is the checked one with a 6.45 × 2 dash centered, whose four pixel rows at 2x sit on the pixel grid (`CHECKBOX.mixed_mark`, `mixed_stroke`; the SVG rasterizer only keeps the rows aligned for marks of an even size); `Checkbox::mixed(true)`.
- Disclosure triangle (`NSButton` disclosure, measured over white and black): an opaque `c0c0c0` in light and `808080` in dark, `000000` at 74% and `ffffff` at 68% under Increase Contrast; 5.5 × 8.8 ink, stroke about 1.35. The kit has no `NSButton` disclosure triangle; its `DisclosureGroup` follows SwiftUI's (below) and keeps this color (`DISCLOSURE_TRIANGLE`).
- Table sorted column: the title is bold (weight 700 in Inter matches the mass) and the arrow is a 7 × 4 chevron of secondary-label ink, stroke about 1.05, at the right end of the header cell (7 beyond its content, 0.5 below the title's center): `TABLE.sorted_weight`, `sort_arrow*`.
- Graphical date picker (`clockAndCalendar`, `es_ES`, 139 × 148, `DatePicker::graphical`): the month and year are bold 13 at 3.5 with ◀ ● ▶ 6 × 7 at 100.5, 115.5 and 129.5; the weekdays are bold 10 secondary, right-aligned in 18.5 columns that end at 19.75 + 18.5 × n, over a 1 high line at 37; the days are 11, in 18 high rows from 38.5, right-aligned, with the days of other months in tertiary ink; the chosen day has a 19 × 16 highlight (radius 4.5) that ends 3.25 past its column, `dcdcdc` or `464646` until the picker has focus and then the selection color. The card is opaque white in light with a 0.5 `ededed` border and a darker line along the bottom; in dark it is a 9.4% gray; under Increase Contrast it has a 1 border (`595959` light, `c0c0c0` dark). `kit-hc` and the pressed scene (`calendar-focused`) compare it at 8–13%. The border is outside the fill: the fill (opaque white in light) spans 0.9 to 137.5 across and 1.0 to 147.5 down inside the 139 × 148 frame, and the 0.5 border, a translucent black, sits outside it (over black it vanishes); the card's ink box is 137.5 × 147.5 from 0.5. The kit draws that box with the border inside (`DATE_PICKER.card_*` is 0.5, 0.5, 1 and 0.5 from the frame), which only differs over a backdrop darker than the card. The line under the weekdays spans the card, 0.5 to 138.0.
- Increase Contrast gauges: ring arc `0057ad` and `127be7`, ring track 14.9%, level fills `008932`, `a16a00`, `e9152d` in light and `4ad968`, `fedf43`, `ff6165` in dark, level track 50% and 30%, border 79% black or `b2b2b2` at 38%.
- Level indicator fill: a rounded rectangle of radius 2 at both ends, not clipped square at the right.
- Combo box button fill is black or white at 9% (`combo_button`), the segment divider 23% (`segment_divider`) and 14 high, and a segmented control is `fit`: each segment is its text plus 12 on each side, the chosen one in the toggle accent.
- Gauges (`~/src/probes/gauges.swift`): the ring arc has round ends and sweeps 3.6° per percent from the top, drawn in `0a82ff` in light and `1769e6` in dark (`accent_ring`) over a track of black 5% or white 10%; the bar is 8 high in a 20 high frame (6 above and below), its fill is the accent and its track black 6% or white 8%; the capacity level is 16.5 high with a 1 wide border (black 19% in light, gray `696969` at 22% in dark), a track of black 7% or white 10%, and greens, yellows and reds of `34c759`, `ffcc00`, `ff383c` in light and `30d158`, `ffd600`, `ff4245` in dark.

## Text, date, color and image controls

Read with `~/src/probes/texts.swift` (`~/src/captures/texts`, 1600 wide, flow layout, `TEXTS_INACTIVE=1` for an inactive window; after the main capture each focusable item is made first responder and captured as `<prefix>-focus-<name>@<scale>x.png`) and `~/src/probes/fonts.swift`. The kit scene is `crates/bk/tests/kit/appkit_texts.rs`; `scripts/appkit-compare.py controls --family texts` and `focused` print the tables.

- Heights from `fittingSize` and the ink of the 4x captures (the border is a 1 pt line outside the frame, so the ink is 2 more): text field 19 mini, 22 small, 24 regular, large and extra large (the font stays 13 above small: 9, 11, 13); search field and combo box 16, 20, 24, 28 and 36; date picker field 21 with the stepper (24 high frame, 4 above and 3 below the field in `alignmentRectInsets`), and 24, 28, 36 for large and extra large.
- Search field by size (4x ink, text 9, 11, 13, 13, 13): the text starts 16.5, 18.5, 24.75, 28.75 and 32.75 from the left; the magnifier is 9.25, 11.25 and 13.25 (large and extra large) square, its ink starting 5.0, 5.0, 6.75, 8.75 and 10.75 from the left and 0.5 above, 0.4 above, then 0.6 below the middle; the clear disc is 9, 11 and then 13 across, 3, 4, 5, 7 and 11 from the right edge. `SEARCH_FIELD` holds them.
- Combo box by size: the arrow button is 16 × 11, 20 × 16, 24 × 20, 23 × 21 and 29 × 27 (mini to extra large), 3, 3, 4, 4 and 5 from the right edge, with radii of about 3, 4, 5, 5 and 7, and the chevron ink is 6.5 × 4, 7.5 × 5, 7.5 × 5, 9 × 6 and 9 × 6 (`COMBO_BOX`).
- Date picker by size (the date text stays 13): the field is 78 wide at every size and 21 high at mini, small and regular, 28 at large and 36 at extra large with its 1 border outside (30 and 38 of ink); the stepper is arrows only, 9 × 13, 11 × 16 and 13 × 20 (mini to regular) 6, 7 and 7.5 from the field, and a gray 22 × 28 or 29 × 36 pill 4 from the field at large and extra large. The date and the time are separated by `, `. `.textField` (no stepper) is the field with its border outside, 80 × 23.
- A disabled text, search or secure field keeps its bezel and fill; only the ink changes, to the tertiary label (about 26% over the fill, `TertiaryLabel`). The placeholder of an empty field does not change.
- `NSTextField` `bezelStyle` `.squareBezel` and `.roundedBezel` look like the default. `isBordered` without `isBezeled` draws a square 1 pt black line (the frame is 20 high); no bezel and no background draws only the text (16 high). A read-only field keeps the bezel.
- An empty `NSImageView` with `imageFrameStyle` `.none` draws nothing; the other four styles are the 1990s bevels (`grayBezel` has a black top and left line and a gray bottom and right one).
- `NSSearchField` with a `searchMenuTemplate`: the magnifier gets a 5 wide chevron, so the glyph spans 7.75 to 26.0 and the text starts 31.5 instead of 24.5; the clear button is the same.
- `NSTextView` in a bezel-bordered `NSScrollView`: a square 1 pt light gray line (no rounding) around the `textBackgroundColor` fill, text 13 pt starting 5 from the edge. `cacheDisplay` draws the selection and caret but not the focus ring or the scrollers.
- `NSColor.systemBlue` is `0088ff` in light and `0091ff` in dark, so the dark bench scene passes the dark value (`system_blue` in the scene); it was a bench artifact, not a probe one.

### Type styles

`NSFont.preferredFont(forTextStyle:)` and SwiftUI `Font` agree (`~/src/captures/fonts.json`, SF Pro, point size / line height from `NSLayoutManager`): `largeTitle` 26 / 32 regular, `title` 22 / 26, `title2` 17 / 22, `title3` 15 / 20, `headline` 13 / 16 bold, `subheadline` 11 / 14, `body` 13 / 16, `callout` 12 / 15, `footnote` 10 / 13, `caption` 10 / 13 and `caption2` 10 / 13 medium. A text field's line is 16 high at 13 pt (14 at 11 pt, 11 at 9 pt). The kit's own scale (`ctdesign::font`) has heavier weights and 18 high lines for body.

## Tables and source lists

Measured with `~/src/probes/views.swift` (an inset-style `NSTableView`, a source-list `NSOutlineView`, an `NSTabView` and two `NSScroller`, at 4x in the four appearances).

- Table (two columns of 150, no scroller): header 28 high, header text in the label color at 11 regular and vertically centered (cap height from y=10 to 18); a 2 px line from y=27 to 29 under it (`separatorColor`) and a 1 by 16 separator at the right edge of every column (black 10% in light, white 16% in dark; black 65% and white 30% with Increase Contrast) from y=6 to 22; rows 24 high starting 5 below the header, inset 10 on both sides, with 7 corners on the selection and the stripes. The first column's text starts 10 from the column and 8 in the header; every later column adds 2 (AppKit places those cells 6 and 8 from the column edge, plus 2 inside the text field).
- Selection `selectedContentBackgroundColor` (`0064e1`, `0059d1`, `0040dd` and `1774da` with Increase Contrast); stripes black 4.3% in light and white 4.7% in dark.
- The source list is the classic one: 24 high rows, uppercase 11 semibold group headers 19 high, 16 folder icons. The kit's `Sidebar` follows the sidebar of macOS 27, measured in [Sidebar](#sidebar-and-window-macos-27), not this source list. The selected source-list row renders black in `cacheDisplay`; its colors are the `unemphasizedSelectedContentBackgroundColor` role (`dcdcdc`, `464646`).

## Lists, tables and outlines (SwiftUI)

Read with `~/src/probes/lists.swift`: SwiftUI `List` in the `.inset`, `.plain`, `.bordered` and striped styles, a `Table` sorted by name and an outline with `DisclosureGroup`, in one hosted view of 1160 × 690 captured at 4x with `cacheDisplay` (opaque views, so no screen capture is needed) in `aqua`, `darkAqua` and the two Increase Contrast appearances, once with nothing focused and once with each list made first responder. `List` and `Table` are `NSTableView`s whose rows are 24 high in every style but `.sidebar`. The window background is `ffffff` light, `1e1e1e` dark, `ffffff` and `202020` under Increase Contrast. The `scripts/appkit-compare.py lists` bench compares each item with the kit.

- **Selection.** A pill of the row's height whose corners are continuous with radius 7 (the leftmost pixel column reaches full coverage 10 below the top, the 1.53 expansion of `WINDOW_CORNER`). While the table is not first responder it is `dcdcdc` light and `464646` dark (`unemphasizedSelectedContentBackgroundColor`, also under Increase Contrast) with the label color; focused it is `selectedContentBackgroundColor` with white text. The icon of an outline row stays the accent while unfocused.
- **Inset** (`.automatic` and `.inset` are the same): rows 10 in from each side and the first row 10 below the top, text 16 from the edge (6 inside the pill), a 1 high line under every row from the start of its text to 16 from the end, `e5e5e5` (black 10%). A selected row and its neighbors draw no line, and the last row none. `alternatesRowBackgrounds` adds `f4f5f5` (black 4.3%) rows at the odd indices, continues them below the last row, and drops the lines. Increase Contrast stripes are black 7.8% and white 9.9% (`TABLE.zebra_contrast`).
- **Plain:** rows from edge to edge, first row at the top, text 8 from the edge, square selection, lines from the text to the right edge.
- **Bordered:** a 1 `e5e5e5` border inside the frame, rows from edge to edge inside it, text 7 from the outer edge, square selection, lines from 7 to 7 from the right edge.
- **Table:** header 28, rows 24 from 5 below the line under it, inset 10, text 16 from the edge in the first column and 8.5 from the second column's edge. The header has a 1 separator at the leading edge as well as after each column (`TABLE.header_separator`). A table whose rows do not fill it keeps striping below the last row. With a scroller drawn the rows keep 17 of their width free, so a table without overflow is compared; the kit's scrollers overlay the content (see Scroll below).
- **Outline** (`List(children:)`, or `DisclosureGroup` rows in a `List`): the disclosure chevron is the SF symbol (`838383` light, `969696` dark: the secondary label, not the `c0c0c0` triangle of a standalone `DisclosureGroup`), 8.5 × 5.25 when open and 5 × 8.25 when closed, its ink centered at 18.75 and 23.4 (open) or 19.5 and 22.1 (closed) in the row; the folder icon is 15 wide at 26 and the title at 47. A child is indented 13, inside the selection. The line under a row starts at the title. Lines are not drawn in the last top-level node.
- The kit's remaining difference in these items is 4 to 9% of the pixels in the region, all of it glyph shapes: Inter against SF Pro, the folder symbol, and the 0.5 to 1 lower baseline of Inter in a 24 high row.

## Alerts

Read from the view hierarchy of an `NSAlert` (informational, two buttons) with `~/src/probes/alerts.swift`; its pixels are not captured because the window is hosted by SwiftUI and its glass and shadow come from the compositor, not from the app. 260 wide; the 64 icon at 20 from the top and the left; title 13 bold and message 13 regular, both in the label color, each line 16 high, 20 from the sides, 10 between them; buttons large (28 high), two of 110 with 8 between, 16 from the sides and 16 from the bottom, 16 under the message; Cancel on the left, the default button on the right. The icon sits above the text, not beside it: the title starts 16 under it, at the same 20 from the left. `Alert::icon` draws the 64 square the app gives it, with the title 16 under it (`DIALOG_ALERT_ICON`, `DIALOG_ALERT_ICON_GAP`); the kit leaves the glass and the shadow to the compositor.

## Sheets

A titled window with an empty 430 × 280 sheet (`~/src/probes/sheets.swift`, over black, white and a pattern; the sheet is composited by the window server, so the capture is a screen region): the sheet is centered in the window's frame, 116 from the top of a 512 high window, and its fill is the window background, `ffffff` in light and `1e1e1e` in dark. The parent window behind it is not blurred: it is veiled linearly by a gray at 60%, `aaaaaa` in light (black becomes `666666`, white `cccccc`) and `141414` in dark (`0c0c0c` and `727272`). The kit's `Palette::scrim` now holds these two veils, and `SheetStyle.background` is the window background (it was the raised one, `191919` in dark). The sheet's corner profile, shadow and edge are the compositor's: at 1x the corner reaches the straight edge about 21 rows down in light, about the 15 of the windows.

## Menus, scrollers and inactive selection

- Menu (a real `NSMenu` over white and black with `~/src/probes/menu.swift`, screenshots converted to sRGB with `sips -m`; the unconverted display profile turns blue into indigo): the highlighted row is `controlAccentColor` (`007aff`) opaque in light and dark, 24 high, inset 5 from the panel, corner 8; the panel is 5 padded, items 24, a separator 11, panel corner about 14. The glass, its alpha (about 0.41 light and 0.56 dark over white) and the shadow belong to the compositor. `selectedContentBackgroundColor` (`0064e1`) is for table and list rows, not menus.
- Legacy scroller (17 wide strip, from the real window): a capsule track and thumb 11 wide with 3 on each side, 3 from the ends of the strip; track black 5% (white 5% in dark), thumb black 52% over it in light and white 57% in dark; the thumb is never shorter than 20.
- Unemphasized selection (table in a window that is not key): `dcdcdc` in light and `464646` in dark, with the text in the label color, not white.
- Menu width has no minimum: an `NSMenu` is the widest title, plus the widest shortcut, plus a 24 gap, plus 12 and 5 on each side (one `A` item is 42 wide, the four-item test menu 135). The kit's 220 minimum for shell menus is gone; the popover menu keeps its own.
- Menu marks and arrows (a real `NSMenu` with checked, mixed and submenu items; the display is 1x, so these are sums of pixel coverage, good to about 0.5): the check, the mixed dash and the submenu chevron all use the label color, as the title does, and are heavy strokes of about 1.7. The check is the same shape as the checkbox's, 9 across and 9 down with its center 17.5 from the panel edge; the dash is 8.2 long, its right edge level with the check's; the chevron is 3.6 wide and 7.4 high, its ink ending 20 from the panel edge. A menu with any mark puts its titles 30 from the panel edge, 13 further than without, and a submenu row is 31 wider than a plain one (a shortcut adds 24 plus its own width). `NSMenuItem.image` draws nothing in a pop-up menu on this macOS, so item icons could not be measured.
- Menu text: titles use the label color (`26` darkest in light), shortcuts and disabled items the tertiary label (about 25%: black in light, white in dark). The kit drew shortcuts at the secondary level.

## Sidebar and window (macOS 27)

Read with `~/src/probes/sidebar` (an `NSSplitViewController` whose first item is `NSSplitViewItem(sidebarWithViewController:)` holding a `.sourceList` table) and `~/src/probes/sidebarui` (a SwiftUI `NavigationSplitView` with a `List` sidebar) on macOS 27.2.1, display 1x, `.fullSizeContentView` window of 900 × 560 with a unified toolbar. Both run through `appbundle.sh`, which makes the window key and takes the screenshot; colors are the screenshot converted to sRGB. Frames come from the view tree, so they are exact; the glass is the compositor's.

- **No floating panel.** The sidebar is flat and full height: an `NSGlassEffectView` from x = 0 to its width and from the top of the window to the bottom, `cornerRadius` 0, with no inset, no rim, no shadow of its own and no line at its edge. The edge is the change of color: sidebar `ededed` and content `ffffff` in light, sidebar `262626` and content `1e1e1e` in dark. (The probe's window has no wallpaper behind it, so these are the glass over the window material.)
- **Width.** 200 in AppKit (the `minimumThickness` set by the probe) and 220 in SwiftUI (the width asked for); a user drags it between its minimum and maximum. Content starts at that x; the window title sits 20 to its right (x = 220 and 240).
- **Bar and lights.** The unified toolbar is 52 high over the sidebar too, and the traffic lights stay where a toolbar window puts them, 14 across at x = 19, 42 and 65 and y = 19, inside the sidebar. The sidebar's list starts below the bar, at y = 52, and scrolls under it (`NSScrollPocket`).
- **Sidebar toggle.** A 36 glass circle at y = 8. AppKit's `toggleSidebar` item sits after the lights, at x = 96; SwiftUI puts it at the right end of the sidebar, 8.5 from its edge (x = 175.5 of 220).
- **Rows (SwiftUI `List`).** An item is 32 high; a section header 19 high, with 13 between the last item of a section and the next header. The cell is inset 16 on both sides (188 wide in 220) and the header 14. The icon's ink starts at x = 21 and the text at x = 46; an item's text is 13 pt in the label color and a header's 11 pt semibold in the secondary label color (about `a0a0a0` in light); icons are accent colored.
- **Selection.** A 200 × 32 pill inset 10 from each side, corner radius about 6 (at 1x the first row starts 4 pixels in and the edge is straight from the fifth), filled with the accent: `0270f5` in light and `007aff` in dark after conversion to sRGB, with white text and icon.
- **Rows (AppKit source list).** `rowSizeStyle` default gives 24 high rows and 19 high headers, 183 wide in a 200 sidebar.
- **Bar.** In dark the bar over the content is `262626`, the sidebar's color, not the content's `1e1e1e`; in light it is `ffffff`. A 1 high line sits right under the bar, in the content (y = 52): `e5e5e5` in light, `2e2e2e` in dark.

### What the kit does

`scripts/appkit-render.sh sidebar` draws the SwiftUI probe's window and `scripts/appkit-compare.py sidebar light|dark` compares it with `~/src/captures/sidebar/swiftui-*-srgb.png`. The kit's sidebar is flat and full height, 220 wide, with the colors, the 52 bar, the lights at (19, 19), 32 high rows, 19 high headers 13 apart, the 10 inset selection and the line under the bar all within 1 of AppKit's, in light and dark. What differs: the kit's rows carry a 20 square tile where AppKit draws a bare accent glyph (so the icon's ink is 2 to 3 wider), the toolbar buttons that are not in a `ToolGroup` have no 36 platter in this scene, and the list does not scroll under the bar. The sidebar toggle sits in the sidebar like SwiftUI's, at x = 176 to 211 against 175 to 213 with AppKit's shadow, and the title starts at 240 or 241 in both.

## Toolbar items and navigation buttons

Read with `~/src/probes/navbuttons` on macOS 27.2.1, display 1x, `.fullSizeContentView` windows, icon-only toolbars. Frames come from the view tree; glyph ink comes from `cacheDisplay` at 4x of each hosted button (the compositor draws the glass, so fills, rims and shadows are not here). `navbuttons.sh` runs the probe as an app bundle so the window is key: a bare binary leaves it inactive and every item draws gray.

Three traps in the probe, each of which gave a wrong number before:

- An item with no target and action is not a button but a plain image view: its platter is `glyph + 32` wide (47 for the plus). The old "single bordered item is 47 wide" came from that path. Every real button, enabled or disabled, is a 36 circle in the unified style.
- A disabled button needs `autovalidates = false` and `isEnabled = false`; with an action the toolbar enables it again.
- The momentary group of a toolbar is sized by its labels even in icon-only mode (below).

### What fixes each size

1. **`NSWindow.toolbarStyle`** fixes the bar and the platter height. The platter is a glass capsule (`NSGlassEffectView`, radius half its height) inside an `NSToolbarPlatterView`.

   | Style | Bar | Platter | Radius | Platter top | First platter at x | Last platter ends |
   |---|---|---|---|---|---|---|
   | `unified` | 52 | 36 | 18 | 8 | 96 | 8 from the right |
   | `unifiedCompact` | 40 | 24 | 12 | 8 | 94 | 12 from the right |
   | `expanded` | 68 (32 title row, 36 toolbar row) | 28 | 14 | 32 (top of the toolbar row) | 12 | 11 from the right |

   The traffic lights end at 79 in `unified` (x = 19, 42, 65) and at 72 in `unifiedCompact` (12, 35, 58); `expanded` puts them in the 32 title row and the toolbar under it. Platters are 8 apart in every style.

2. **The kind of item** fixes the width. Adjacent bordered buttons share one platter and draw no divider. A flexible space or an unbordered item ends the platter.

   | Platter | `unified` | `unifiedCompact` | `expanded` |
   |---|---|---|---|
   | One button | 36 | 26.5 | 28.5 |
   | Two buttons (a back and forward pair) | 73 | 63 | 65 |
   | N buttons | 36 N + 1 for N ≥ 2: 73, 109, 145, 181, 217 | | |
   | Two segments, select one | 74 | 66 | 66 |
## Forms and groups (SwiftUI)

Read with the same `lists` probe (`Form` grouped and columns, `GroupBox`, `DisclosureGroup` open and closed), light, dark and Increase Contrast.

- **Form, `.grouped`:** the form is inset 20 on every side; a section's title (bold 13) takes a 16 high line from the top, with its text 10 in from the card and 8 above it; cards are `f7f7f7` (black 3.1%) with continuous corners of radius 12.5 (the leftmost column reaches full coverage 15.5 below the top), 10 apart, rows 36 high with a 1 hairline between them from 10 to 10 inside the card, labels 10 in. A `Toggle` is a switch of 36 × 16 (the kit's mini), 10 from the right edge. A `TextField` is borderless text in the label color, a `LabeledContent` value is in the secondary label color. The kit stacks a control under its label below 300 wide. In dark the card is black or white 3.1% (`252525` over `1e1e1e`); the kit's card is the shared content background (`2a2a2a`), five levels lighter, because a window with a tinted backdrop must look the same in light and dark and the 3.1% fill follows the scheme.
- **Form, `.columns`:** no cards. Labels are right-aligned in a column as wide as the widest label, the controls start 9.5 after it; a checkbox row starts at the control column. The kit has no counterpart.
- **GroupBox:** the title is 11 pt, its ink 10 in (the text box 9.5), in a 17 high row above the box; the box is the content plus 5 on every side, `f7f7f7`, continuous corners of radius 12. `NSBox` differs in the 12 high title row and the 10.75 inset.
- **DisclosureGroup** (standalone): the chevron is `c1c1c1`, 7.75 × 4.5 open (center 3.9 and 13.25) and 5 × 7.5 closed (center 4.5 and 12.25), the title starts 12 from the left, and the children are not indented.
- The kit's difference in these items is 3 to 9% of the pixels, text apart from the missing `.columns` form.
- `NSColorWell` styles (`systemBlue`, all 48 × 24 at every `controlSize`): `.default` is the 48 × 24 bezel and the 42 × 18 swatch; `.minimal` is the capsule swatch alone with its darker ring; `.expanded` is a 47 wide swatch of radius 5 and, 1 after it, a 24 square button of radius 6 (black 9.8%) with a 16 color wheel, red at 3 o'clock, white at the center (72 wide in all). A disabled well is the same at 50% opacity, the button and the wheel included.

## Containers: split views and scrollers

Read with `~/src/probes/containers.swift` (`NSSplitView` thin, thick and pane-splitter dividers in both orientations, `HSplitView` and `VSplitView`, `NSTabView` in its styles and `NSScrollView` with each border and both scroller styles) captured at 4x with `cacheDisplay`. The scroll views, the tab strip and the thick dividers are drawn partly by the window server, so `cacheDisplay` shows black or no strip there; the bench compares the dividers. `scripts/appkit-compare.py lists <mode> --family containers`.

- **Thin divider** (`NSSplitView.dividerStyle = .thin`, and `HSplitView`, `VSplitView`): 1 thick, in the separator color of every appearance: `e5e5e5` and `393939`, and `cccccc` and `4d4d4d` under Increase Contrast. The panes touch it; an even split of 160 puts the line at 80 (left pane 80, right pane 79) and one of 100 at 49 (AppKit rounds to the pixel; the kit puts it at 79.5 and 49.5). The kit's `SplitPanes` divider was 3 thick; it is 1 with a 5 wide grab area (`SPLIT_PANES.divider`, `grab`), at 0.6 to 2% of the pixels.
- **Thick and pane-splitter dividers** are 10 thick: a `e5e5e5` band with a soft top edge and, on the pane splitter, a dot; not drawn by the kit.
- **Legacy scroller:** the kit's `ScrollView` keeps the 17 wide strip (11 capsule plus 3 on each side) out of the content while the bar is always shown, as `NSScrollView` does; before it overlaid the content.
- **Not measured:** the glass tab strip of `NSTabView` and `TabView` needs a screen capture, and `NSScrollView` borders are not in the kit.

   | Two segments, momentary, labels "Back" and "Forward" | 87 | 79 | 83 |
## Forms and groups (SwiftUI)

Read with the same `lists` probe (`Form` grouped and columns, `GroupBox`, `DisclosureGroup` open and closed), light, dark and Increase Contrast.

- **Form, `.grouped`:** the form is inset 20 on every side; a section's title (bold 13) takes a 16 high line from the top, with its text 10 in from the card and 8 above it; cards are `f7f7f7` (black 3.1%) with continuous corners of radius 12.5 (the leftmost column reaches full coverage 15.5 below the top), 10 apart, rows 36 high with a 1 hairline between them from 10 to 10 inside the card, labels 10 in. A `Toggle` is a switch of 36 × 16 (the kit's mini), 10 from the right edge. A `TextField` is borderless text in the label color, a `LabeledContent` value is in the secondary label color. The kit stacks a control under its label below 300 wide. In dark the card is black or white 3.1% (`252525` over `1e1e1e`); the kit's card is the shared content background (`2a2a2a`), five levels lighter, because a window with a tinted backdrop must look the same in light and dark and the 3.1% fill follows the scheme.
- **Form, `.columns`:** no cards. Labels are right-aligned in a column as wide as the widest label, the controls start 9.5 after it; a checkbox row starts at the control column. The kit has no counterpart.
- **GroupBox:** the title is 11 pt, its ink 10 in (the text box 9.5), in a 17 high row above the box; the box is the content plus 5 on every side, `f7f7f7`, continuous corners of radius 12. `NSBox` differs in the 12 high title row and the 10.75 inset.
- **DisclosureGroup** (standalone): the chevron is `c1c1c1`, 7.75 × 4.5 open (center 3.9 and 13.25) and 5 × 7.5 closed (center 4.5 and 12.25), the title starts 12 from the left, and the children are not indented.
- The kit's difference in these items is 3 to 9% of the pixels, text apart from the missing `.columns` form.

   | Three segments, select one or any | 111 to 123, by glyph width | | |

   The center of the glyphs of a pair are 36.9 apart in `unified` (18.1 and 55.0 from the left of its 73) and 38.9 apart in the other two (12.1 and 51.0 of 63; 13.1 and 52.0 of 65). A segment is at least 34 wide with 3 on each end; a momentary group takes its width from the label text, 74 with one-letter labels, 87 with "Back" and "Forward", about 118 with `chevron.left` and `chevron.right`. A select-one group does not.

3. **`isNavigational`** moves the item to the leading edge, after the sidebar toggle, in a platter of its own, whatever its place in the list. A pair of them is the back and forward control of Safari or Finder.

4. **The glyph** does not follow the platter. Every toolbar symbol is the SF Symbol at 16 pt, 1.23 times the 13 pt default, in the three styles: a chevron's ink is 8.75 × 14.75, so it fills 14.75 of the 24 high compact platter. Ink at 10% of the peak, in points:

   | Symbol | Ink | Symbol | Ink |
   |---|---|---|---|
   | `chevron.left` | 8.75 × 14.75 | `plus` | 14.25 × 14.25 |
   | `chevron.right` | 8.5 × 14.75 | `sidebar.left` | 20.25 × 16.0 |
   | `ellipsis` | 15.75 × 4.0 | `magnifyingglass` | 16.5 × 17.25 |

   A chevron's centerline is 7.2 × 13.2 with a stroke of 1.5; a plus has arms of 12.8 and the same stroke. A lone chevron sits 0.9 off the center of its circle, toward the open side (the left one at 17.1, the right one at 19.0 of 36); in a pair the glyphs are at 18.1 and 19.0 of their 36 and 37 wide halves.

5. **The state** sets the color: the label color at 84.7% (black in light, white in dark), 25.5% when disabled, white on the accent for a prominent item (a 36 accent circle; its check is 15.25 × 14.5).

### Groups and dividers

A momentary group draws a divider between its segments: 1 wide, black or white at 22.4%, 20 high in `unified` (8 from the top of the platter), 14 in `unifiedCompact` and 18 in `expanded` (5 from the top). A select-one group draws no divider; its selected segment is a capsule inset 4 in `unified` (28 high) and 2 in the other two (20 and 24 high). The pair of navigational buttons has no divider at all: a row of the platter at its middle reads the same on both sides of the seam.

### What the kit does

A toolbar item is a glass capsule whose height follows the bar the `Titlebar` draws: `ToolGroup` and `IconButton` read it from the bar (`TOOLGROUP.height` 36 in the unified bar, `TOOLGROUP.compact_height` 24 in the compact one) and, outside a `Titlebar`, a `ToolGroup` is the unified one.

- **Glyph.** `IconButton` in a toolbar has a 21 symbol at stroke 1.6 (`ICON_BUTTON.toolbar_icon`, `toolbar_stroke`), the same in both bars as in AppKit. The chevron's ink is 8.5 × 15.0 and the plus 13.5, against 8.75 × 14.75 and 14.25, and their mass differs by 0.8% and 1.2% (`scripts/appkit-compare.py toolbar`).
- **Unified bar.** An item is 36 × 36 and `NavigationButtons` is 73 × 36 with an invisible 1 seam; its glyph boxes are within 0.25 of AppKit's.
- **Compact bar.** An item is 26 × 24 (`TOOLGROUP.height` less twice `compact_inset`, by `compact_height`) and a group's slots are 36 wide with 5 of the first and last one reaching past the platter, so a pair is 63 × 24 like AppKit's. The glyph boxes of the pair are within 1 of AppKit's: its chevrons sit 0.9 nearer the ends than the kit's, which centers them in their slot.
- **Divider.** `ToolGroup` draws a 1 × 20 divider at 22.4% (14 high in the compact bar; `TOOLGROUP.separator_h`, `compact_separator_h`, `toolgroup-separator`), the momentary group's. A group of three is 110 wide in the unified bar, one more than AppKit's.

Not drawn yet: the `expanded` style (28 high platters under a 32 title row); the platter of a lone `IconButton` (AppKit draws every item on a glass capsule; the kit draws one only in a `ToolGroup`); the 22 between the traffic lights and the first platter of the compact bar (the kit's left zone starts at the lights' end, 72, and AppKit's at 94); the labels that widen a momentary group; the offset of a lone chevron; and the SF symbols other than the chevron and the plus (the kit's `sidebar.left` is 16.0 × 13.5 against 20.25 × 16.0).

## Toolbar tabs and window corners

- Settings-style tabs (`NSTabViewController` with `tabStyle = .toolbar`, `~/src/probes/tabs.swift`): the title bar is 88 (32 plus a 56 toolbar), each tab is a platter 48 high whose `NSGlassEffectView.cornerRadius` is 8, as wide as its title plus 13 and never under 55, with 1 between tabs; the strip is centered and has no box of its own. The selected tab shows its icon and title in the accent, the others in the secondary label. The toolbar's single and group platters report `cornerRadius` 18, a capsule.
- Window corner: the alpha of a window capture (`-o`) reaches the full edge 15 rows below the top in the plain titled window, in the toolbar-tab window and in the unified toolbar window alike, so no style got the 26 the kit assumes for `unified`. Whether the kit's nominal radius plus `WINDOW_CORNER.expansion` draws that extent has not been compared; it is the compositor that draws the window shape.
