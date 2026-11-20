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
- The regular slider reports 16, `slider.height` is 20 with a 20 × 16 knob, which matches the large size's track area. Not yet compared pixel by pixel.

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
