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

## Tables and source lists

Measured with `~/src/probes/views.swift` (an inset-style `NSTableView`, a source-list `NSOutlineView`, an `NSTabView` and two `NSScroller`, at 4x in the four appearances).

- Table (two columns of 150, no scroller): header 28 high, header text in the label color at 11 regular and vertically centered (cap height from y=10 to 18); a 2 px line from y=27 to 29 under it (`separatorColor`) and a 1 by 16 separator at the right edge of every column (black 10% in light, white 16% in dark; black 65% and white 30% with Increase Contrast) from y=6 to 22; rows 24 high starting 5 below the header, inset 10 on both sides, with 7 corners on the selection and the stripes. The first column's text starts 10 from the column and 8 in the header; every later column adds 2 (AppKit places those cells 6 and 8 from the column edge, plus 2 inside the text field).
- Selection `selectedContentBackgroundColor` (`0064e1`, `0059d1`, `0040dd` and `1774da` with Increase Contrast); stripes black 4.3% in light and white 4.7% in dark.
- The source list is the classic one: 24 high rows, uppercase 11 semibold group headers 19 high, 16 folder icons. The kit's `Sidebar` is the floating glass sidebar of macOS 26 (32 high rows), so it is not compared with it. The selected source-list row renders black in `cacheDisplay`; its colors are the `unemphasizedSelectedContentBackgroundColor` role (`dcdcdc`, `464646`).

## Alerts

Read from the view hierarchy of an `NSAlert` (informational, two buttons) with `~/src/probes/alerts.swift`; its pixels are not captured because the window is hosted by SwiftUI and its glass and shadow come from the compositor, not from the app. 260 wide; the 64 icon at 20 from the top and the left; title 13 bold and message 13 regular, both in the label color, each line 16 high, 20 from the sides, 10 between them; buttons large (28 high), two of 110 with 8 between, 16 from the sides and 16 from the bottom, 16 under the message; Cancel on the left, the default button on the right. The kit draws no icon and leaves the glass and the shadow to the compositor.

## Menus, scrollers and inactive selection

- Menu (a real `NSMenu` over white and black with `~/src/probes/menu.swift`, screenshots converted to sRGB with `sips -m`; the unconverted display profile turns blue into indigo): the highlighted row is `controlAccentColor` (`007aff`) opaque in light and dark, 24 high, inset 5 from the panel, corner 8; the panel is 5 padded, items 24, a separator 11, panel corner about 14. The glass, its alpha (about 0.41 light and 0.56 dark over white) and the shadow belong to the compositor. `selectedContentBackgroundColor` (`0064e1`) is for table and list rows, not menus.
- Legacy scroller (17 wide strip, from the real window): a capsule track and thumb 11 wide with 3 on each side, 3 from the ends of the strip; track black 5% (white 5% in dark), thumb black 52% over it in light and white 57% in dark; the thumb is never shorter than 20.
- Unemphasized selection (table in a window that is not key): `dcdcdc` in light and `464646` in dark, with the text in the label color, not white.
- Menu width has no minimum: an `NSMenu` is the widest title, plus the widest shortcut, plus a 24 gap, plus 12 and 5 on each side (one `A` item is 42 wide, the four-item test menu 135). The kit's 220 minimum for shell menus is gone; the popover menu keeps its own.
- Menu text: titles use the label color (`26` darkest in light), shortcuts and disabled items the tertiary label (about 25%: black in light, white in dark). The kit drew shortcuts at the secondary level.

## Toolbar (unified, icon only)

Read from the view tree of an `NSToolbar` window with `~/src/probes/toolbar.swift` on macOS 27.2.1: the title bar is 52 high; the traffic lights are 14 wide at x=19, 42 and 65 and y=19; each platter is a glass capsule 36 high, 8 from the top and bottom, with 8 between platters; a single bordered item is 47 wide, a two-item selection group 74 (a 34 slot twice and 3 on each side, the selected slot a 30 high capsule), the search item 239. The kit's `SETTINGS.toolbar_height` (52), `toolgroup-h` (36) and the 8 gap agree; its `ToolGroup` is a row of 36 slots with separators, not the selection group.

