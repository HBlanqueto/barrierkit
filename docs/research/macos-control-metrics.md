# macOS control metrics

Measured on macOS 27.2.1 with a Swift script: `fittingSize` of each control per `ControlSize`, and the `NSColor` semantic colors resolved in the `aqua` and `darkAqua` appearances through `performAsCurrentDrawingAppearance`.

## Sizes (width × height, points)

| Control | regular | small | mini | large |
|---|---|---|---|---|
| Push button | 24 | 20 | 16 | 28 |
| Checkbox | 16 | 14 | 12 | 18 |
| Radio button | 16 | 14 | 12 | 18 |
| Switch | 54 × 24 | 54 × 24 | 54 × 24 | 54 × 24 |
| Slider | 16 | 14 | 12 | 20 |
| Pop-up button | 24 | 20 | 16 | 28 |
| Combo box | 24 | 20 | 16 | 28 |
| Segmented control | 24 | 20 | 16 | 28 |
| Text field | 24 | 24 | 24 | 24 |
| Search field | 24 | 20 | 16 | 28 |
| Stepper | 20 × 24 | 17 × 20 | 13 × 16 | 23 × 28 |
| Scroller | 17 | | | |

Heights agree with `components.json` for button, checkbox, radio, switch, pop-up, segmented, text field and search field.

## Open differences

- `stepper.height` follows the measured 24; the compact kit golden needs regenerating on Linux.
- The regular slider is 16 high with a 19.5 × 16 knob and a 6 high track starting 5 from the top, in light and dark; `slider.height` and `slider.knob-width` follow it. Mini, small and large knobs are 15.5, 17.5 and 23.5 wide.

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
- Mixed checkbox: the box is the checked one with a 6.5 × 1.8 dash centered (`CHECKBOX.mixed_mark`, `mixed_stroke`); `Checkbox::mixed(true)`.
- Disclosure triangle (`NSButton` disclosure, measured over white and black): an opaque `c0c0c0` in light and `808080` in dark, `000000` at 74% and `ffffff` at 68% under Increase Contrast; 5.5 × 8.8 ink, stroke about 1.35 (`DISCLOSURE.triangle`, `triangle_stroke`, `DISCLOSURE_TRIANGLE`).
- Table sorted column: the title is bold (weight 700 in Inter matches the mass) and the arrow is a 7 × 4 chevron of secondary-label ink, stroke about 1.05, at the right end of the header cell (7 beyond its content, 0.5 below the title's center): `TABLE.sorted_weight`, `sort_arrow*`.
- Graphical date picker (`clockAndCalendar`, `es_ES`, 139 × 148, `DatePicker::graphical`): the month and year are bold 13 at 3.5 with ◀ ● ▶ 6 × 7 at 100.5, 115.5 and 129.5; the weekdays are bold 10 secondary, right-aligned in 18.5 columns that end at 19.75 + 18.5 × n, over a 1 high line at 37; the days are 11, in 18 high rows from 38.5, right-aligned, with the days of other months in tertiary ink; the chosen day has a 19 × 16 highlight (radius 4.5) that ends 3.25 past its column, `dcdcdc` or `464646` until the picker has focus and then the selection color. The card is opaque white in light with a 0.5 `ededed` border and a darker line along the bottom; in dark it is a 9.4% gray; under Increase Contrast it has a 1 border (`595959` light, `c0c0c0` dark). `kit-hc` and the pressed scene (`calendar-focused`) compare it at 8–13%.
- Increase Contrast gauges: ring arc `0057ad` and `127be7`, ring track 14.9%, level fills `008932`, `a16a00`, `e9152d` in light and `4ad968`, `fedf43`, `ff6165` in dark, level track 50% and 30%, border 79% black or `b2b2b2` at 38%.
- Level indicator fill: a rounded rectangle of radius 2 at both ends, not clipped square at the right.
- Combo box button fill is black or white at 9% (`combo_button`), the segment divider 23% (`segment_divider`) and 14 high, and a segmented control is `fit`: each segment is its text plus 12 on each side, the chosen one in the toggle accent.
- Gauges (`~/src/probes/gauges.swift`): the ring arc has round ends and sweeps 3.6° per percent from the top, drawn in `0a82ff` in light and `1769e6` in dark (`accent_ring`) over a track of black 5% or white 10%; the bar is 8 high in a 20 high frame (6 above and below), its fill is the accent and its track black 6% or white 8%; the capacity level is 16.5 high with a 1 wide border (black 19% in light, gray `696969` at 22% in dark), a track of black 7% or white 10%, and greens, yellows and reds of `34c759`, `ffcc00`, `ff383c` in light and `30d158`, `ffd600`, `ff4245` in dark.

## Tables and source lists

Measured with `~/src/probes/views.swift` (an inset-style `NSTableView`, a source-list `NSOutlineView`, an `NSTabView` and two `NSScroller`, at 4x in the four appearances).

- Table (two columns of 150, no scroller): header 28 high, header text in the label color at 11 regular and vertically centered (cap height from y=10 to 18); a 2 px line from y=27 to 29 under it (`separatorColor`) and a 1 by 16 separator at the right edge of every column (black 10% in light, white 16% in dark; black 65% and white 30% with Increase Contrast) from y=6 to 22; rows 24 high starting 5 below the header, inset 10 on both sides, with 7 corners on the selection and the stripes. The first column's text starts 10 from the column and 8 in the header; every later column adds 2 (AppKit places those cells 6 and 8 from the column edge, plus 2 inside the text field).
- Selection `selectedContentBackgroundColor` (`0064e1`, `0059d1`, `0040dd` and `1774da` with Increase Contrast); stripes black 4.3% in light and white 4.7% in dark.
- The source list is the classic one: 24 high rows, uppercase 11 semibold group headers 19 high, 16 folder icons. The kit's `Sidebar` is the floating glass sidebar of macOS 26 (32 high rows), so it is not compared with it. The selected source-list row renders black in `cacheDisplay`; its colors are the `unemphasizedSelectedContentBackgroundColor` role (`dcdcdc`, `464646`).

## Alerts

Read from the view hierarchy of an `NSAlert` (informational, two buttons) with `~/src/probes/alerts.swift`; its pixels are not captured because the window is hosted by SwiftUI and its glass and shadow come from the compositor, not from the app. 260 wide; the 64 icon at 20 from the top and the left; title 13 bold and message 13 regular, both in the label color, each line 16 high, 20 from the sides, 10 between them; buttons large (28 high), two of 110 with 8 between, 16 from the sides and 16 from the bottom, 16 under the message; Cancel on the left, the default button on the right. The kit draws no icon and leaves the glass and the shadow to the compositor.

## Sheets

A titled window with an empty 430 × 280 sheet (`~/src/probes/sheets.swift`, over black, white and a pattern; the sheet is composited by the window server, so the capture is a screen region): the sheet is centered in the window's frame, 116 from the top of a 512 high window, and its fill is the window background, `ffffff` in light and `1e1e1e` in dark. The parent window behind it is not blurred: it is veiled linearly by a gray at 60%, `aaaaaa` in light (black becomes `666666`, white `cccccc`) and `141414` in dark (`0c0c0c` and `727272`). The kit's `Palette::scrim` now holds these two veils, and `SheetStyle.background` is the window background (it was the raised one, `191919` in dark). The sheet's corner profile, shadow and edge are the compositor's: at 1x the corner reaches the straight edge about 21 rows down in light, about the 15 of the windows.

## Menus, scrollers and inactive selection

- Menu (a real `NSMenu` over white and black with `~/src/probes/menu.swift`, screenshots converted to sRGB with `sips -m`; the unconverted display profile turns blue into indigo): the highlighted row is `controlAccentColor` (`007aff`) opaque in light and dark, 24 high, inset 5 from the panel, corner 8; the panel is 5 padded, items 24, a separator 11, panel corner about 14. The glass, its alpha (about 0.41 light and 0.56 dark over white) and the shadow belong to the compositor. `selectedContentBackgroundColor` (`0064e1`) is for table and list rows, not menus.
- Legacy scroller (17 wide strip, from the real window): a capsule track and thumb 11 wide with 3 on each side, 3 from the ends of the strip; track black 5% (white 5% in dark), thumb black 52% over it in light and white 57% in dark; the thumb is never shorter than 20.
- Unemphasized selection (table in a window that is not key): `dcdcdc` in light and `464646` in dark, with the text in the label color, not white.
- Menu width has no minimum: an `NSMenu` is the widest title, plus the widest shortcut, plus a 24 gap, plus 12 and 5 on each side (one `A` item is 42 wide, the four-item test menu 135). The kit's 220 minimum for shell menus is gone; the popover menu keeps its own.
- Menu marks and arrows (a real `NSMenu` with checked, mixed and submenu items; the display is 1x, so these are sums of pixel coverage, good to about 0.5): the check, the mixed dash and the submenu chevron all use the label color, as the title does, and are heavy strokes of about 1.7. The check is the same shape as the checkbox's, 9 across and 9 down with its center 17.5 from the panel edge; the dash is 8.2 long, its right edge level with the check's; the chevron is 3.6 wide and 7.4 high, its ink ending 20 from the panel edge. A menu with any mark puts its titles 30 from the panel edge, 13 further than without, and a submenu row is 31 wider than a plain one (a shortcut adds 24 plus its own width). `NSMenuItem.image` draws nothing in a pop-up menu on this macOS, so item icons could not be measured.
- Menu text: titles use the label color (`26` darkest in light), shortcuts and disabled items the tertiary label (about 25%: black in light, white in dark). The kit drew shortcuts at the secondary level.

## Toolbar (unified, icon only)

Read from the view tree of an `NSToolbar` window with `~/src/probes/toolbar.swift` on macOS 27.2.1: the title bar is 52 high; the traffic lights are 14 wide at x=19, 42 and 65 and y=19; each platter is a glass capsule 36 high, 8 from the top and bottom, with 8 between platters; a single bordered item is 47 wide, a two-item selection group 74 (a 34 slot twice and 3 on each side, the selected slot a 30 high capsule), the search item 239. The kit's `SETTINGS.toolbar_height` (52), `toolgroup-h` (36) and the 8 gap agree; its `ToolGroup` is a row of 36 slots with separators, not the selection group.

## Toolbar tabs and window corners

- Settings-style tabs (`NSTabViewController` with `tabStyle = .toolbar`, `~/src/probes/tabs.swift`): the title bar is 88 (32 plus a 56 toolbar), each tab is a platter 48 high whose `NSGlassEffectView.cornerRadius` is 8, as wide as its title plus 13 and never under 55, with 1 between tabs; the strip is centered and has no box of its own. The selected tab shows its icon and title in the accent, the others in the secondary label. The toolbar's single and group platters report `cornerRadius` 18, a capsule.
- Window corner: the alpha of a window capture (`-o`) reaches the full edge 15 rows below the top in the plain titled window, in the toolbar-tab window and in the unified toolbar window alike, so no style got the 26 the kit assumes for `unified`. Whether the kit's nominal radius plus `WINDOW_CORNER.expansion` draws that extent has not been compared; it is the compositor that draws the window shape.
