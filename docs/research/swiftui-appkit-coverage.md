# SwiftUI and AppKit coverage

**Goal:** list, family by family, what SwiftUI and AppKit offer for designing
macOS 27 apps and what BarrierKit already has, so sessions that work in
parallel pick work without overlap.
**Scope:** SwiftUI and AppKit as the macOS 27.2.1 SDK (Xcode 27.2) ships them,
against `bk` on `main`. It covers the look and behavior of controls,
containers, presentation, windows, materials, symbols and accessibility. It
does not cover data, networking, media, document or graphics frameworks.
**Snapshot:** `main` at `bb83b10`, 2026-11-20. This matrix is the one source of
each API's status: sessions that change a row change it in the same commit.
**Reader:** an engineer or agent who changes a control, a view or a token in
the kit.
**Input:** [the glossary](../barrierkit/20-glossary.md), the bench scene
`crates/bk/tests/kit/appkit.rs`, the probes in `~/src/probes`, the SDK in
Xcode and [`macos-control-metrics.md`](macos-control-metrics.md).
**Output:** one row per API with its status, its evidence and its priority,
and a backlog per area at the end.

## Steps

1. Find the family in the [contents](#contents).
2. Read the row: the SwiftUI and AppKit names, the kit name, the
   [status](#statuses) and where the measure is.
3. To pick work, go to the [backlog](#backlog) and take one item of your area.
4. When a commit changes what a row says, change the row in that commit
   (`AGENTS.md`, «Documentación», rule 3).
5. To check a row, rerun its evidence:
   [`appkit-comparison.md`](appkit-comparison.md) says how.

## Contents

1. [Buttons and button styles](#1-buttons-and-button-styles)
2. [Toggles](#2-toggles)
3. [Pickers](#3-pickers)
4. [Steppers](#4-steppers)
5. [Sliders](#5-sliders)
6. [Gauges, progress and level indicators](#6-gauges-progress-and-level-indicators)
7. [Text](#7-text)
8. [Date and time](#8-date-and-time)
9. [Color and font panels](#9-color-and-font-panels)
10. [Image and path control](#10-image-and-path-control)
11. [Control sizes](#11-control-sizes)
12. [Containers and navigation](#12-containers-and-navigation)
13. [Presentation](#13-presentation)
14. [Windows and chrome](#14-windows-and-chrome)
15. [Materials and glass](#15-materials-and-glass)
16. [Symbols](#16-symbols)
17. [Accessibility](#17-accessibility)
18. [New in macOS 27](#18-new-in-macos-27)
19. [Backlog](#backlog)
20. [Drift and open questions](#drift-and-open-questions)

## How to read the tables

Each row has six columns.

| Column | Meaning |
|---|---|
| SwiftUI | The public API or style, as the SDK lists it. |
| AppKit | The class, bezel, style or property. `—` means AppKit has none. |
| Kit | The public name from the [glossary](../barrierkit/20-glossary.md). `—` means the kit has none. |
| Status | One of the four [statuses](#statuses). |
| Measure | Where the numbers or captures are. |
| Pri | [Priority](#priorities) of the gap, `—` when there is none. |

### Statuses

| Status | Meaning |
|---|---|
| `measured` | The kit draws it, the bench compares it with AppKit in light, dark and Increase Contrast, and its numbers are in the metrics. |
| `implemented` | The kit draws it. Nobody has compared it with AppKit or SwiftUI. |
| `partial` | The kit draws part of it. The note says what is missing: sizes, styles, states or an appearance. A row measured in light and dark only is `partial`. |
| `missing` | The kit has nothing. |

A row names one size or one style, so a control that is measured at regular
size and missing at mini has two rows.

### Priorities

| Pri | Meaning |
|---|---|
| P1 | Appears in nearly every macOS app, and a gap shows at first glance. |
| P2 | Appears in many apps, or finishes a family. The kit has a workaround. |
| P3 | Rare, deprecated or tied to Apple hardware. |

### Evidence

- **bench `name`:** an entry of `crates/bk/tests/kit/appkit.rs` that
  `scripts/appkit-compare.py` compares with the probe capture of the same name.
  The `controls` and `views` scenes run in light, dark and Increase
  Contrast; `menu` and `toolbar` run in light and dark. The Increase Contrast
  runs turn Reduce Transparency on too, as the probe captures do.
- **probe `name`:** an item in a capture of `~/src/probes`, with its size.
  Nothing compares it with the kit.
- **metrics:** a section of [`macos-control-metrics.md`](macos-control-metrics.md).
- **sdk:** the SDK header or `.swiftinterface` that lists the API.

The bench of this snapshot has the scenes `controls`, `contrast`, `views`,
`views-contrast`, `menu`, `toolbar`, `sidebar` and `texts` and `texts-contrast`
(text, date, color, image and path controls, in light, dark and Increase Contrast; `sidebar`
compares the kit with the SwiftUI probe `sidebarui`).

### Where the API lists come from

```sh
SDK=$(xcrun --show-sdk-path)
grep -n 'NSBezelStyle' "$SDK/System/Library/Frameworks/AppKit.framework/Headers/NSButtonCell.h"
grep -A3 'extension SwiftUI.PrimitiveButtonStyle where Self ==' \
  "$SDK/System/Library/Frameworks/SwiftUI.framework/Modules/SwiftUI.swiftmodule/arm64e-apple-macos.swiftinterface"
```

The SwiftUI interface holds the controls and scenes. The view basics
(`Material`, `controlSize`, `glassEffect`) live in `SwiftUICore`, which this
document reads through the style lists and the probes.

## Summary

Rows per status at the snapshot. A row is one API, style or size.

| Family | measured | implemented | partial | missing | Rows |
|---|---|---|---|---|---|
| 1. Buttons and button styles | 14 | 3 | 10 | 1 | 28 |
| 2. Toggles | 6 | 1 | 0 | 0 | 7 |
| 3. Pickers | 5 | 2 | 2 | 3 | 12 |
| 4. Steppers | 2 | 1 | 0 | 0 | 3 |
| 5. Sliders | 4 | 2 | 0 | 0 | 6 |
| 6. Gauges, progress and level indicators | 6 | 2 | 1 | 2 | 11 |
| 7. Text | 7 | 11 | 8 | 6 | 32 |
| 8. Date and time | 3 | 0 | 1 | 4 | 8 |
| 9. Color and font panels | 2 | 0 | 2 | 3 | 7 |
| 10. Image and path control | 0 | 2 | 2 | 0 | 4 |
| 11. Control sizes | 2 | 2 | 2 | 0 | 6 |
| 12. Containers and navigation | 13 | 6 | 15 | 3 | 37 |
| 13. Presentation | 1 | 3 | 9 | 1 | 14 |
| 14. Windows and chrome | 5 | 4 | 9 | 13 | 31 |
| 15. Materials and glass | 0 | 1 | 8 | 1 | 10 |
| 16. Symbols | 2 | 0 | 3 | 1 | 6 |
| 17. Accessibility | 1 | 1 | 10 | 1 | 13 |
| **All** | 73 | 41 | 82 | 39 | 235 |

To recount after a change, run this from the repository root:

```sh
awk -F'|' '/^## /{s=$0} /^\|/{for(i=4;i<=6&&i<=NF;i++)if(match($i,/^ *`(measured|implemented|partial|missing)`/)){w=substr($i,RSTART,RLENGTH);gsub(/[ `]/,"",w);n[s" "w]++;break}} END{for(k in n)print n[k],k}' docs/research/swiftui-appkit-coverage.md | sort -k2
```

## 1. Buttons and button styles

Kit: `Button`, `ButtonStyle`, `ControlSize` (`Mini`, `Small`, `Regular`, `Large`, `ExtraLarge`).
AppKit has 13 current bezel styles (`NSButtonCell.h`). Nine names are
deprecated: `shadowlessSquare` and `texturedSquare` point to `smallSquare`;
`rounded`, `regularSquare`, `texturedRounded`, `roundRect`, `recessed`,
`roundedDisclosure` and `inline` point to `push`, `flexiblePush`, `toolbar`,
`accessoryBarAction`, `accessoryBar`, `pushDisclosure` and `badge`.

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.bordered`, `.automatic` at regular, small, large | `.push` (`.automatic` since 14) | `Button`, `Button::size` | `measured` | bench `push-regular`, `push-small`, `push-large`, `push-disabled`, `push-pressed`; [Sizes][m-sizes]; [Increase Contrast][m-hc] | — |
| `.bordered` at mini, small, large and extra large | `.push` with `.mini`, `.small`, `.large`, `.extraLarge` | `Button::size` with `ControlSize::Mini` and `ExtraLarge` | `measured` | bench family `buttons`: `push-mini` to `push-xl`, `default-*`, `push-disabled-*`, `push-pressed-*`; [Control sizes][m-control-sizes] | — |
| `.bordered` at mini in SwiftUI | `.push` with `.mini` | `ControlSize::Mini` is AppKit's 16 | `partial`: the SwiftUI mini button is 46 × 13 | bench family `swiftui` `bordered-mini`; [Control sizes][m-control-sizes] | P3 |
| `.borderedProminent` | `.push` with `keyEquivalent = "\r"` (the default button) | `ButtonStyle::bordered_prominent` | `measured` | bench `default-regular`, `default-disabled`, `default-pressed` | — |
| `Button(role: .destructive)` | `hasDestructiveAction` | `ButtonStyle::destructive` | `partial`: SwiftUI and AppKit draw the plain bordered button, the kit preset is red (for alerts) | probe `push-destructive` 64 × 24; bench family `swiftui` `destructive` | P3 |
| `.borderless` | `isBordered = false` | `ButtonStyle::borderless` | `partial`: SwiftUI draws only the label (41 × 16); the kit keeps the 24 high hit area | bench family `swiftui`; [Control sizes][m-control-sizes] | P2 |
| `.plain` | `isBordered = false` | `ButtonStyle::plain` | `partial`: label only, 8% differ (label color); bench family `swiftui` `plain` | [Control sizes][m-control-sizes] | P3 |
| `.link` | — | `ButtonStyle::link`, `Link` | `partial`: accent label, 8% differ; bench family `swiftui` `link` | [Control sizes][m-control-sizes] | P3 |
| `.glass` | `.glass` (macOS 26) | `ButtonStyle::glass` | `implemented` | probe `glass` 58 × 24; the compositor draws the glass, so fill and rim are not comparable ([Toolbar][m-toolbar]) | P2 |
| `.glassProminent` | `.glass`; no probe shows the prominent look | `ButtonStyle::glass_prominent` | `implemented` | none | P2 |
| `.accessoryBar`, `.accessoryBarAction` | `.accessoryBar`, `.accessoryBarAction` | `ButtonStyle::tengen()`: they draw the push button | `measured` | bench `bezel-accessory-bar`, `bezel-accessory-bar-action` (family `buttons`) | — |
| `buttonBorderShape` (`.capsule`, `.circle`, `.roundedRectangle`) | — | `ButtonStyle.shape` | `partial`: only the `circular` preset; the others go through the style closure | none | P3 |
| `.buttonBorderShape(.circle)` | `.circular` | `ButtonStyle::circular` | `measured` | bench `circular`; [Round button][m-rings] | — |
| `HelpLink` | `.helpButton` | `HelpLink` | `measured` | bench `help` | — |
| `DisclosureGroup` triangle | `.disclosure` | `DisclosureGroup` | `measured` | bench `disclosure`; [Disclosure triangle][m-rings] | — |
| — | `.pushDisclosure`, `.roundedDisclosure` | `ButtonStyle::push_disclosure` | `measured` | bench `bezel-push-disclosure`, `bezel-rounded-disclosure`, `-disabled`, `-pressed` (family `buttons`) | — |
| — | `.smallSquare` (the +/− bar under a list), `.shadowlessSquare` | `ButtonStyle::small_square` | `measured`: a bitmap in AppKit, 11% to 18% differ | bench `bezel-small-square`, `-disabled`, `-pressed`; [Control sizes][m-control-sizes] | — |
| — | `.toolbar`, `.texturedRounded` | `ButtonStyle::textured_rounded`; `IconButton` inside `ToolGroup` | `measured`: 4% in light, 13% under Increase Contrast | bench `bezel-toolbar`, `bezel-textured-rounded`; [Control sizes][m-control-sizes] | — |
| — | `.texturedSquare` | `ButtonStyle::textured_square` | `measured`: a bitmap in AppKit, 11% to 15% differ | bench `bezel-textured-square`, `-disabled`, `-pressed` | — |
| — | `.inline` | `ButtonStyle::inline` | `measured` | bench `bezel-inline`, `-disabled`, `-pressed` | — |
| `.badge(_:)` | `.badge` (replaces `inline`): a count in a table row | `Badge` (red capsule counter) | `partial`: neither the SwiftUI badge nor the bezel was probed | none | P2 |
| — | `.flexiblePush`, `.roundRect`, `.recessed` | `ButtonStyle::tengen()`: they draw the push button | `measured` | bench `bezel-flexible-push`, `bezel-round-rect`, `bezel-recessed` | — |
| `Toggle` with `.button` | `NSButton` toggle types | `Button::toggled` | `measured` | bench family `swiftui` `togglebutton-off-*`, `togglebutton-on-*` | — |
| `Menu` with `.button`, `.borderlessButton` (`.borderedButton` is deprecated) | pull-down `NSPopUpButton`, `NSMenuToolbarItem` | `MenuButton`, `PopUpButton::pulls_down` | `partial`: the pull-down pop-up is measured (bench `pulldown`), the menu styles are not | probe `pulldown` 71 × 24 | P1 |
| `Menu(primaryAction:)` | `NSComboButton` | `ComboButton` | `implemented` | none | P3 |
| `ControlGroup` (`.automatic`, `.menu`, `.palette`, `.navigation`, `.compactMenu`) | `NSToolbarItemGroup`, `NSSegmentedControl` | `ControlGroup`, `ToolGroup` | `partial`: `ToolGroup` is in [Toolbar][m-toolbar]; the SwiftUI styles are not probed | none | P2 |
| `ShareLink` | `NSSharingServicePicker` | `ShareLink` | `partial`: copy, email and open only | none | P3 |
| `PasteButton`, `EditButton`, `SettingsLink` | — | — | `missing` | none | P3 |

## 2. Toggles

Kit: `Switch`, `Checkbox`, `RadioGroup`.

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.toggleStyle(.switch)` | `NSSwitch` | `Switch` | `measured` | bench `switch-off-regular`, `switch-on-regular`, `switch-on-disabled`, `switch-off-disabled`; [Rings][m-rings] (Switch); every size is 54 × 24 in a layout and drawn 36 × 16 to 80 × 36 (bench family `buttons` `switch-*-mini` to `-xl`, family `swiftui`) | — |
| `.toggleStyle(.checkbox)` at regular | `NSButton` `.switch` | `Checkbox` | `measured` | bench `check-off-regular`, `check-on-regular`, `check-mixed`, `check-on-disabled`, `check-off-disabled`, `check-off-pressed`, `check-on-pressed` | — |
| `.checkbox` at mini, small, large, extra large | `controlSize` | `Checkbox::size` | `measured` | bench family `buttons` `check-off-*`, `check-on-*`, `check-mixed-*`, `-disabled`, `-pressed`; the large mark is a soft bitmap in AppKit; [Control sizes][m-control-sizes] | — |
| `Picker` with `.radioGroup` at regular | `NSButton` `.radio` | `RadioGroup` | `measured` | bench `radio-off-regular`, `radio-on-regular`, `radio-off-disabled`, `radio-on-disabled`, `radio-off-pressed`, `radio-on-pressed` | — |
| `.radioGroup` at other sizes | `controlSize` | `RadioGroup::size` | `measured` | bench family `buttons` `radio-off-*`, `radio-on-*`, `-disabled`, `-pressed` | — |
| mixed state | `allowsMixedState` | `Checkbox::mixed` | `measured` | bench `check-mixed`; [Rings][m-rings] (Mixed checkbox) | — |
| `Toggle` row in a `Form` | — | `Row::toggle` | `implemented` | none | P1 |

## 3. Pickers

Kit: `SegmentedControl`, `PopUpButton`, `RadioGroup`, `ColorWell`, `ComboBox`.

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.segmented` at regular | `NSSegmentedControl` select one | `SegmentedControl` | `measured`; as wide as its segments (82, 90, 98, 106, 122 for «One», «Two»), not the width it is given | bench `segmented-regular`, `segmented-disabled`; [Combo box and segmented][m-rings] | — |
| `.segmented` at mini, small, large, extra large | `controlSize` | `SegmentedControl::size` | `measured` | bench family `buttons` `segmented-mini` to `segmented-xl`; SwiftUI segments are equal width, AppKit's are fit to their text | — |
| `.segmented` with symbols | `NSSegmentedControl(images:)` | `SegmentedControl::segment` takes text only | `missing` | none | P2 |
| — | `segmentStyle` (`.rounded`, `.roundRect`, `.texturedSquare`, `.smallSquare`) | — | `missing` | not probed | P3 |
| — | select any, momentary | `ToggleGroup` (select any); momentary only in `ToolGroup` | `partial` | probe `segmented-any` 95 × 24, not compared | P2 |
| `.menu` at regular | `NSPopUpButton` | `PopUpButton` | `measured` | bench `popup-regular`, `popup-disabled`, `popup-pressed`, `pulldown` | — |
| `.menu` at mini, small, large, extra large | `controlSize` | `PopUpButton::size` | `measured` | bench family `buttons` `popup-*`, `pulldown-*`, `-disabled`, `-pressed` | — |
| `.menu` sections and separators | `NSMenuItem.sectionHeader(title:)`, `.separator()` | `PopUpButton::section_header`, `PopUpButton::separator` | `implemented` | none | P2 |
| `.radioGroup` | `NSButton` `.radio` | `RadioGroup` | `measured` | see [Toggles](#2-toggles) | — |
| `.inline` | — | `RadioGroup` (nearest) | `partial`: no probe shows the SwiftUI look | none | P3 |
| `.palette` | — | `ColorWell` (swatch row) | `implemented` | none | P3 |
| `.tabs` (macOS 27) | — | — | `missing` | sdk `PickerStyle.tabs`, introduced macOS 27.0 | P2 |
| `.wheel`, `.navigationLink` | — | — | not on macOS | sdk marks both unavailable on macOS | — |

The date and color pickers are in [8](#8-date-and-time) and
[9](#9-color-and-font-panels).

## 4. Steppers

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Stepper` at regular | `NSStepper` | `Stepper` | `measured` | bench `stepper-regular`, `stepper-disabled`; [Rings][m-rings] (Internal glyphs, Stepper divider) | — |
| `Stepper` at mini, small, large, extra large | `controlSize` | `Stepper::size` | `measured`: the bezel differs by 1%, the chevrons by 5% to 10%; the arrows are 16, 20, 24, 28 and 36 high as SwiftUI's | bench family `buttons` `stepper-mini` to `stepper-xl`, `-disabled` | — |
| `Stepper` with label and value | `NSStepper` next to a text field | `Stepper` with `StepperStyle.value_position` | `implemented` | none | P2 |

## 5. Sliders

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Slider` at regular | `NSSlider` linear | `Slider` | `measured` | bench `slider-regular`, `slider-disabled`; [Open differences][m-open] | — |
| `Slider` with tick marks | `numberOfTickMarks`, `allowsTickMarkValuesOnly` | `Slider::number_of_tick_marks`, `Slider::stepped` | `measured` for 5 ticks in the default position | bench `slider-ticks` | — |
| `Slider` at mini, small, large, extra large | `controlSize` | `Slider::size` | `measured` | bench family `buttons` `slider-mini` to `slider-xl`, `-disabled` | — |
| — | `sliderType = .circular` | `CircularSlider` | `measured` | bench `slider-circular`; the metrics sentence that says the kit has no counterpart is out of date ([drift](#drift-and-open-questions)) | — |
| — | `isVertical` | `SliderStyle.axis` | `implemented` | none | P3 |
| `Slider` with minimum and maximum labels | `NSSliderCell` captions | `Slider` marks and captions | `implemented` | none | P3 |

## 6. Gauges, progress and level indicators

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `ProgressView(value:)` | `NSProgressIndicator` bar | `ProgressView` | `measured`; its layout box is 12 high at mini and small and 20 from regular, as in SwiftUI | bench `progress-bar`; test `swiftui_defaults`; [Gauges][m-rings] (bar) | — |
| `ProgressView()` bar, indeterminate | `isIndeterminate` | `ProgressView` | `implemented` | none; the probe fixes `isIndeterminate = false` | P2 |
| `.progressViewStyle(.circular)` with a value | spinning, determinate | `ProgressRing` | `measured` at regular | bench `ring-regular`; [Rings][m-rings] | — |
| ring at mini and small | `controlSize` | `ProgressRing` takes no `ControlSize` | `missing` | probe `ring-mini` and `ring-small` 16 × 16; regular, large and extra large 32 × 32 | P3 |
| `ProgressView()` circular, indeterminate | spinning, indeterminate | `Spinner` | `partial`: 10, 16 and 32 square by control size as SwiftUI frames it; the drawing is not compared | test `swiftui_defaults` | P1 |
| `Gauge` (`.linearCapacity`, `.accessoryLinear`, `.accessoryLinearCapacity`, `.accessoryCircular`, `.accessoryCircularCapacity`, `.circular`, `.linear`) | — | `Gauge`, `CapacityIndicator` | `implemented` | none: `gauges.swift` draws AppKit controls only, no SwiftUI `Gauge` | P2 |
| — | `.continuousCapacity` | `CapacityIndicator` | `measured` | bench `level`, `level-warn`, `level-crit`; [Level indicator][m-rings] | — |
| — | `.discreteCapacity` | `CapacityIndicator` discrete | `measured` | bench `discrete-5`, `discrete-10` | — |
| — | `.rating` | `RatingIndicator` | `measured` | bench `rating-3`; [Rating][m-rings] | — |
| — | `.relevancy` | `RelevancyIndicator` | `measured` | bench `relevancy-3` | — |
| — | tick marks, custom rating image, `placeholderVisibility` | — | `missing` | none | P3 |

## 7. Text

Kit: `TextField`, `SearchField`, `SecureField`, `TokenField`, `ComboBox`,
`TextEditor`, `TextView`, `Text`.

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.roundedBorder`, `.automatic` | `NSTextField` bezeled | `TextField` | `measured` | bench family `texts` `tf-empty`, `tf-filled`, `tf-square`, `tf-rounded`; [Text fields][m-text] | — |
| `.plain` | `isBezeled = false` | `TextFieldStyle::plain` | `measured` | bench `tf-plain`; [Text, date, color and image][m-texts] | — |
| `.squareBorder` | `bezelStyle = .squareBezel` | — | `missing` | `.squareBezel` draws like the default; `isBordered` without `isBezeled` is a 1 pt black square, 140 × 20 (probe `tf-bordered`) | P3 |
| `.bordered` (macOS 27) | — | — | `missing` | sdk `BorderedTextFieldStyle`, introduced macOS 27.0; not probed | P2 |
| `TextField` at mini and small | `controlSize` | `TextField::size` | `measured` | bench `tf-mini`, `tf-small`, `tf-large`, `tf-xl`; 19, 22, 24, 24, 24 | — |
| `TextField(axis: .vertical)` | wrapping `NSTextField` | `TextField` is one line | `missing` | probe `tf-multiline` 140 × 56 (three lines); not drawn | P2 |
| `.searchable` | `NSSearchField` | `SearchField` | `measured` | bench `search`, `search-text`, `search-mini` to `search-xl`, `search-disabled-text`; [Text, date, color and image][m-texts] | — |
| `.searchable` in a toolbar | `NSSearchToolbarItem` | `ToolbarSearch` | `implemented` | probe `toolbar` has the item; not compared | P1 |
| `searchScopes`, `searchSuggestions`, `searchCompletion` | `searchMenuTemplate`, recents | `SearchField` tokens only | `missing` | probe `search-menu`: the magnifier gets a 5 wide chevron and the text starts 7 later | P2 |
| `SecureField` | `NSSecureTextField` | `SecureField` | `measured` | bench `secure`, `secure-empty`; `.disabled` | — |
| — | `NSTokenField` | `TokenField` | `measured` | bench `token` | — |
| — | `NSComboBox` | `ComboBox` | `measured` | bench `combo`, `combo-empty`, `combo-mini` to `combo-xl`, `combo-disabled` | — |
| `TextEditor` | `NSTextView` in `NSScrollView` | `TextEditor`, `TextView` | `partial`: the field and the 13 pt text are measured; caret, selection, placeholder and the find bar are not | bench `textview-system`; [Text, date, color and image][m-texts] | P1 |
| `TextEditorStyle` (`.automatic`, `.plain`) | `NSTextView` | `TextEditorStyle` fields, no preset | `partial` | none | P3 |
| `TextEditor` with `AttributedString`, `AttributedTextSelection` | rich `NSTextView` | `rich_text` (bold, italic, underline, strikethrough, link, family, size) | `partial`: no color, paragraph style or list; strikethrough has no editor shortcut | none | P2 |
| `findNavigator` | `NSTextFinder` | `TextEditor` find bar | `implemented` | none | P2 |
| `writingToolsBehavior`, spelling, substitutions | `NSSpellChecker`, Writing Tools | `spelling` (Hunspell) | `partial`: spelling only | none | P3 |
| — | `NSRulerView`, `NSTextList`, `NSTextTable` | — | `missing` | none | P3 |

### Text styles

Read on macOS 27.2.1 with `NSFont.preferredFont(forTextStyle:)`: size, line
(ascent plus descent plus leading) and weight. The kit draws Inter or Adwaita
Sans, so glyph widths differ ([Limits](appkit-comparison.md#limits)).

| SwiftUI | AppKit | macOS 27.2.1 | Kit | Status | Pri |
|---|---|---|---|---|---|
| `.largeTitle` | `.largeTitle` | 26 / 32, regular | `LARGE_TITLE` 26 / 31, 700 | `partial`: weight | P1 |
| `.title` | `.title1` | 22 / 26, regular | `TITLE` 22 / 26, 650 | `partial`: weight | P1 |
| `.title2` | `.title2` | 17 / 22, regular | — | `missing` | P1 |
| `.title3` | `.title3` | 15 / 20, regular | `TITLE_3` 15 / 19, 400 (`TITLE_2` 15 / 19, 600 stays) | `implemented` | — |
| `.headline` | `.headline` | 13 / 16, bold | `HEADLINE` 13 / 16, 700 | `implemented` | — |
| `.body` | `.body` | 13 / 16, regular | `BODY` 13 / 16, 400 | `implemented` | — |
| `.callout` | `.callout` | 12 / 15, regular | `CALLOUT` 12 / 15, 400 | `implemented` | — |
| `.subheadline` | `.subheadline` | 11 / 14, regular | `SUBHEADLINE` 11 / 14, 400 (`CAPTION` 11 / 14, 500 stays) | `implemented` | — |
| `.footnote` | `.footnote` | 10 / 13, regular | `FOOTNOTE` 10 / 13, 400 | `implemented` | — |
| `.caption` | `.caption1` | 10 / 13, regular | `FOOTNOTE` 10 / 13, 400 | `implemented` | — |
| `.caption2` | `.caption2` | 10 / 13, medium | `CAPTION_2` 10 / 13, 500 | `implemented` | — |
| `.monospaced()` | `monospacedSystemFont` | not read | `MONO` 13 / 16, 400 | `implemented` | P3 |

`NSFont.systemFontSize(for:)` is 9 at mini, 11 at small and 13 at regular,
large and extra large. The probe `controls` uses it for the text fields.

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Font.system(_:design:weight:)`, `.fontWeight`, `.fontWidth` | `NSFont.systemFont(ofSize:weight:)` | `ctdesign::Font` (family, size, line height, weight) | `partial`: no `.rounded` or `.serif` design, no width | none | P3 |
| SF Pro | SF Pro | Inter or Adwaita Sans | `partial`: the glyph weight and widths differ | [Limits](appkit-comparison.md#limits) | P2 |

## 8. Date and time

Kit: `DatePicker`, `DateElements` (`Date`, `Time`, `DateAndTime`).

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.stepperField` | `.textFieldAndStepper`, year, month, day | `DatePicker` | `measured` | bench `datepicker`, `datepicker-mini` to `datepicker-xl`, `datepicker-disabled`; [Text, date, color and image][m-texts] | — |
| `.field` | `.textField` (no stepper) | `DatePicker::field` | `measured` | bench `date-field`; 80 × 23 with the border outside | — |
| `.graphical` | `.clockAndCalendar`, year, month, day | `DatePicker::graphical` | `measured` | bench `calendar`, `calendar-focused`; [Graphical date picker][m-rings]; the card frame follows AppKit (`DATE_PICKER.card_*`) | — |
| `.compact` | — | — | `missing` | not probed | P2 |
| time parts | `.hourMinute`, `.hourMinuteSecond` | `DateElements::Time`, `DateAndTime` | `partial`: no seconds | bench `time`, `datetime` (9.6 to 18.4% of the pixels, the comma and the digits are wider in SF Pro) | P2 |
| — | analog clock of `.clockAndCalendar` | — | `missing` | not probed | P3 |
| — | `datePickerMode = .range` | — | `missing` | none | P3 |
| `MultiDatePicker` | — | — | `missing` | none | P3 |

## 9. Color and font panels

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `ColorPicker` well | `NSColorWell` default style | `ColorPicker` | `measured` | bench `colorwell`, `colorwell-blue`, `well-default`, `well-default-disabled`; [Color well][m-rings] | — |
| — | `NSColorWell` `.minimal`, `.expanded` | `ColorPickerStyle::minimal`, `expanded` | `measured` | bench `well-minimal`, `well-expanded` and their disabled looks; [Text, date, color and image][m-texts] | — |
| `ColorPicker` panel | `NSColorPanel`: gray, RGB, CMYK, HSB, palette, list, wheel, crayon | `ColorPicker`, `ColorPanel::show`: an HSB popover | `partial`: one of eight modes | none | P2 |
| `ColorPicker(supportsOpacity:)` | alpha of `NSColorPanel` | — | `missing`: the kit's colors are opaque in the panel | none | P2 |
| — | eyedropper, swatch history | — | `missing` | none | P3 |
| — | `NSFontPanel`, `NSFontManager` | `FontPicker` (family and size) | `partial`: no typeface collections or effects | none | P3 |
| — | toolbar items `.showColors`, `.showFonts` | — | `missing` | none | P3 |

## 10. Image and path control

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Image` from data | `NSImageView` | `Image::data`, `Image::rgba` | `partial`: no scaling or alignment modes | none | P2 |
| `AsyncImage` | — | `use_artwork` | `implemented` | none | P3 |
| — | `NSImageView` editable, `NSImageCell` frame styles | `ImageWell` | `implemented` | probe `imagewell-none` to `imagewell-button` and their empty looks: `.none` draws nothing, the rest are bevels; the kit well is its own rounded design (70% of the pixels differ) | P3 |
| — | `NSPathControl` (`.standard`, `.popUp`, `.navigationBar`) | `PathControl` | `partial`: Finder icons and collapsed names are not drawn | probe `path-standard`, `path-popup`, five sizes; bench `path-standard` 15.7% of the pixels, 2% of the box | P3 |

## 11. Control sizes

SwiftUI has `.controlSize(_:)` with `.mini`, `.small`, `.regular`, `.large` and
`.extraLarge` (since macOS 14). AppKit has `NSControl.ControlSize` with the
same five cases (`.extraLarge` since macOS 26). The kit has `ControlSize` with all five, and
`Button`, `PopUpButton`, `SegmentedControl`, `Checkbox`, `RadioGroup`, `Switch`, `Slider`,
`Stepper` and the text, search and combo fields take it through `.size(..)`. `.control_size(ControlSize)` on a
view sizes the controls inside it, like `.controlSize`, and a control's own `.size(..)` wins. `NSView.prefersCompactControlSizeMetrics`
(macOS 26) was not probed.

Sizes in points, from the probe `controls` on macOS 27.2.1 (`aqua`, white
backdrop; width × height or height):

| Control | mini | small | regular | large | extra large | Kit |
|---|---|---|---|---|---|---|
| Push button | 16 | 20 | 24 | 28 | 36 | all five, measured |
| Checkbox, radio | 12 | 14 | 16 | 18 | 18 | all five, measured |
| Switch | 54 × 24 | 54 × 24 | 54 × 24 | 54 × 24 | 54 × 24 | all five, measured (drawn 36 × 16 to 80 × 36 in the 54 × 24 frame) |
| Slider | 12 | 14 | 16 | 20 | 20 | all five, measured |
| Segmented control | 16 | 20 | 24 | 28 | 36 | all five, measured |
| Pop-up button | 16 | 20 | 24 | 28 | 36 | all five, measured |
| Stepper | 13 × 16 | 17 × 20 | 20 × 24 | 23 × 28 | 30 × 36 | all five, measured (the mini bezel is drawn 13 × 20) |
| Text field (with the size's font) | 19 | 22 | 24 | 24 | 24 | all five, measured |
| Search field, combo box | 16 | 20 | 24 | 28 | 36 | all five, measured |
| Date picker (stepper) | 9 × 13 | 11 × 16 | 13 × 20 | 22 × 28 | 29 × 36 | all five, measured (the field is 78 × 21, taller at large and extra large) |
| Ring | 16 | 16 | 32 | 32 | 32 | regular |

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.controlSize(_:)` as an environment value | `controlSize` per control | `StyleModifiers::control_size` (a style scope), `.size(ControlSize)` per instance | `measured` | bench family `buttons`: `push-*`, `check-*`, `switch-*` are built through `.control_size`; [Control sizes][m-control-sizes] | — |
| `.mini`, `.extraLarge` | `.mini`, `.extraLarge` | `ControlSize::Mini`, `ExtraLarge` | `measured` | bench family `buttons`; [Control sizes][m-control-sizes] | — |
| `.disabled(_:)` on a view, `\.isEnabled` | `NSControl.isEnabled` | `EnvironmentModifiers::disabled`, `use_is_enabled`; the 19 controls with `.disabled` follow it | `implemented`: a disabled container draws its controls as their own flag does (kit test `ambient`, light and dark); `MenuButton`, `ComboButton`, `HelpLink`, `Link` and `ToggleGroup` go inert and are not dimmed (not measured); lists, tables, tabs and the other views do not follow it | kit test `ambient`; [ADR 0087](../decisions.md) | P2 |
| `.environment(_:_:)`, `@Environment` | — | `EnvironmentModifiers::environment`, `use_environment` | `implemented`: readers render again when the value changes; popups and sheets read nothing from the view that opened them | kit test `ambient` | — |
| `.keyboardShortcut` on a button | `NSButton.keyEquivalent` | `Button::keyboard_shortcut` | `partial`: the key calls `on_press` while the button is enabled; a menu item with the same key wins; no default-button look, no `defaultAction` or `cancelAction` constants | kit test `focus_by_name` | P2 |
| `FocusState`, `.focused(_:equals:)`, `.defaultFocus` | `makeFirstResponder`, `initialFirstResponder` | `FocusState`, `use_focus_state`, `EnvironmentModifiers::focused`, `default_focus` | `partial`: moves and reads focus by name over the text inputs and the controls built on `use_interaction`; no `focusScope`, `focusedValue` or `isFocused` | kit test `focus_by_name` | P2 |

The size of each control is a row in its own family.

## 12. Containers and navigation

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `List` (`.automatic`, `.inset`, `.plain`, `.sidebar`, `.bordered`) | single-column `NSTableView` | `List`, `List::section`, `List::row` for view rows (`.sidebar` is `Sidebar`) | `measured`: sections 16% of the pixels (the pinned first header); rows with views draw like text rows, their height and insets are not measured | bench `lists`; [Lists][m-lists] | — |
| `Table` with `.inset` | `NSTableView` `.inset` | `Table` | `measured` | bench `table`; [Tables][m-tables] | — |
| `Table` `.automatic`, `.bordered` | `.automatic`, `.fullWidth`, `.sourceList`, `.plain`; `rowSizeStyle` | `TableStyle::full_width`, `plain`, `source_list`, `row_size` | `measured`: 5 to 11% of the pixels, a source list apart (AppKit draws the sidebar material) | bench `tables`; [Classic tables][m-classic] | — |
| `TableColumn` with views | cell views | `Table::cell`, `Table::icons` before the first column | `implemented`: a view cell draws like a text cell (kit test `list_rows`); icon cells measured, a control cell is not | bench `tables` | P2 |
| sort descriptors | `NSSortDescriptor` | `SortOrder`: one column | `partial` | [Table sorted column][m-rings] | P3 |
| `OutlineGroup` | `NSOutlineView` | `OutlineView`, `TreeNode`, `OutlineViewStyle::classic`, `full_width` | `measured`: inset and full width 5 to 6%; the source list outline is not drawn | bench `tables`; [Classic tables][m-classic] | P2 |
| — | `NSBrowser` | `Browser` | `partial`: fixed column width, no keyboard, no column titles | none | P3 |
| `Form` `.grouped` | — | `Form`, `Section`, `Row` | `measured`: heading, card (at most 704 wide, centered), row heights, the lighter separator, the card color and the borderless text field row are SwiftUI's; per element in the twin `form`; the button row has no kit row | test `swiftui_defaults`; twin `form`; probe `formtwin` | P1 |
| `Picker` (menu), `Toggle` (checkbox), `Picker` (radio group), `Button`, `.searchable`'s field outside a `Form` | `NSPopUpButton`, `NSButton`, `NSSearchField` | `PopUpButton`, `Checkbox`, `RadioGroup`, `Button`, `SearchField` | `measured` by the twin `standalone` (a `VStack` of 20 spacing in a 760 × 480 window, SwiftUI with an `NSSearchField` hosted): positions and sizes equal the reference to the pixel except that everything from the radio group down is 1 px high (the kit's checkbox frame is 16, as AppKit's `NSButton` measures in [control metrics](macos-control-metrics.md); SwiftUI's `Toggle` frame is 17, so the kit keeps the AppKit value and `Checkbox::style` can set the label's `line_height`); 14 to 16 % of the pop-up's pixels differ, the checkbox 23 to 34, the radio group 20 to 26, the button 19, the search field 19 (all text; the dark checkbox also has the vibrant accent outside a `Form`, which the kit draws opaque); the `Button` row of a `Form` is 3 % | twin `standalone`, `controls` (button row); probes `standalone`, `controlstwin` | P2 |
| `Picker`, `Stepper`, `Slider`, `DatePicker`, `ProgressView`, `Gauge`, `Toggle` in a `Form` | `NSPopUpButton`, `NSSegmentedControl`, `NSStepper`, `NSSlider`, `NSDatePicker`, `NSProgressIndicator`, `NSLevelIndicator`, `NSButton` | the kit controls in a `Row` | `measured` by the twin `controls` (SwiftUI `Form` `.grouped`, 760 × 480; row heights and every control position equal the reference): the pop-up 7 to 8% of its pixels differ, the stepper 9, the slider 8, the progress 1, the capacity gauge 4, the checkbox 6 to 9, the date field 22 to 24 (the text), the segmented control 11; the Form's button row (`Row::button`) is 3 %; a control of a `Form` row draws the accent vibrantly (dark: the token lifted by (18, -1, 0), SwiftUI's (24,125,255) against the opaque (6,126,255); the Form switch fell from 23 to 7 %); the segments are 2 px wider than SwiftUI's because the kit's font measures "Columns" wider (the width is the widest title plus the padding; SwiftUI's is 78 on SF); SwiftUI's `Gauge` draws the kit's `CapacityIndicator`, the kit's `Gauge` is the circular style | twin `controls`; probe `controlstwin` | P2 |
| `Form` `.columns` | `NSGridView` | `FormStyle::columns` | `measured`: 11% of the pixels differ | bench `lists`; [Forms][m-lists] | — |
| `LabeledContent` | — | `Row` | `implemented` | none | P2 |
| `GroupBox` | `NSBox` primary | `GroupBox` | `measured` | bench `box`; [Group box][m-rings] | — |
| `Divider` | `NSBox` separator | `Divider` | `measured` | bench `separator` | — |
| `DisclosureGroup` | disclosure triangle | `DisclosureGroup` | `partial`: the triangle is measured, not the indent or the animation | bench `disclosure` | P2 |
| `Label`, `LabelStyle` | — | `Label::new(title, Symbol)`; `LabelStyle` for menu buttons only | `partial`: the icon box (16 × 16.5), the 6 gap and the accent icon in a `List` are SwiftUI's; no `Label(title:icon:)` with views, no style protocol | golden `label`; test `swiftui_defaults`; twin `tasks` | P2 |
| `ContentUnavailableView` | — | `ContentUnavailableView` with `.description` and `.actions` | `partial`: the symbol line (44), the gaps (20, 6, 19), the fonts (15 bold and 15 regular, line 19), the colors and the centering are SwiftUI's; SwiftUI starts the content 1.5, 7.5 and −2 pt above the middle for title only, with description and with actions, the kit centers it; the symbol ink follows the symbol redraw (37.3 against 38.3 for `doc.text`) | test `swiftui_defaults`; probe `contentunavailable` | P2 |
| `TabView` (toolbar-like tabs) | `NSTabViewController` `.toolbar` | `TabView` with `TabViewStyle::toolbar()` | `partial`: measured in [Toolbar tabs][m-tabs], not in the bench | probe `tabs` | P1 |
| `TabView` classic | `NSTabView` (top, bottom, left, right, none), `.segmentedControlOnTop`, `.segmentedControlOnBottom` | `TabView` with `TabType` `Top`, `Bottom` and the three without tabs | `measured`: 3.7 to 4.9% of the pixels for the strips, 1.5 to 2.3% without tabs; no left or right tabs | bench `containers` (`tabs-*`); [Tab views][m-tabviews] | — |
| `TabView` `.sidebarAdaptable`, `.tabBarOnly`, `Tab`, `TabSection` | — | — | `missing` | sdk | P3 |
| `NavigationSplitView` (`.automatic`, `.balanced`, `.prominentDetail`) | `NSSplitViewController`, `NSSplitViewItem` (sidebar, content list, inspector) | `SplitView` | `partial`: two columns and an inspector; no content-list column; the sidebar is in [14](#14-windows-and-chrome) | none | P1 |
| `NavigationStack`, `NavigationLink`, `navigationDestination` | — | `NavigationStack`, `NavigationLink`, `NavigationPath`, `use_navigation_path`; `History`, `NavigationButtons` for browser-like panes | `implemented`: a typed path with push and pop, the destination closure per route ([11. Views](../barrierkit/11-views.md#navigationstack)) | recipe `agents/recipes/navigation_stack.rs` | — |
| `HSplitView`, `VSplitView` | `NSSplitView` (`.thin`, `.thick`) | `SplitPanes` | `partial`: no `.thick` divider, no keyboard on the divider | none | P2 |
| `inspector(isPresented:)` | `NSSplitViewItem` inspector | `SplitView::inspector` | `implemented` | none | P2 |
| `ScrollView` legacy bar | `NSScroller` legacy | `ScrollView`, `Table`, `List`, `OutlineView` | `measured` in the four appearances | bench `scrollers`; [Scrollers][m-scrollers] | — |
| `ScrollView` elasticity and deceleration | `NSScrollView.verticalScrollElasticity`, `UIScrollView.bounces`, `decelerationRate` | `ScrollView`, `List` | `measured` on a Catalyst scroll view: stop distance `157.5 v^(4/3)`, a quartic ease-out path, a rubber band of 0.07 and a critically damped return of 12 rad/s; four flicks within 0.5 to 1.8 % of the distance; the touch path, a real trackpad and Weather are not measured; `List` through `ScrollView` and `Table` through its rows take the same element; `refreshable` pulls at 80 and holds at 60 (UIKit's, three pulls, the pull band is not fitted) | tests `scroll`, `scroll_physics`; [UIKit animations](uikit-animations.md#6-scroll) | P1 |
| `ScrollView` overlay bar | `NSScroller` overlay | `ScrollView`, `Table`, `List`, `OutlineView` | `partial`: the thumb is measured (6 wide, per appearance), not compared in a scene | probe `scrollers` (`overlay`); [Scrollers][m-scrollers] | P2 |
| `scrollEdgeEffectStyle` | `NSScrollPocket` | `TopGlass`, `ScrollView` edge | `partial` | [Sidebar][m-sidebar] | P2 |
| `Grid`, `GridRow`, `gridCellColumns`, `gridColumnAlignment` | `NSGridView`, `NSGridRow`, `NSGridCell` | `Grid`, `GridRow`, `GridModifiers` | `partial`: rows, columns, spans and alignments; content-sized cells only, no `gridCellAnchor`, `gridCellUnsizedAxes` or hidden rows, and a parent that aligns places it by the tallest column ([ADR 0089](../decisions.md)) | probe `layout` (L5 to L10) | P2 |
| `Spacer` | `NSLayoutGuide` | `Spacer` | `implemented`: the minimum length and the sharing of space are the numbers SwiftUI gives (probe `layout`, L1 to L3) | probe `layout` | — |
| `frame(width:height:alignment:)`, `frame(maxWidth:)` | — | `StyleModifiers::frame`, `Frame` | `implemented`: the nine alignments place the view as SwiftUI does (probe `layout`, L4) | probe `layout` | — |
| `HStack(alignment: .firstTextBaseline)`, `GridRow(alignment: .firstTextBaseline)` | `NSStackView.alignment` `.firstBaseline`, `NSGridRow.rowAlignment` | — | `missing`: torin cannot state a baseline ([ADR 0089](../decisions.md)) | probe `layout` (L7) | P3 |
| `LazyVGrid`, `LazyHGrid` | `NSCollectionView` | `CollectionView` | `partial`: one vertical wrap of fixed-width tiles, not lazy, no `GridItem`, no sections, no keyboard | none | P3 |
| — | `NSPredicateEditor`, `NSRuleEditor` | `RuleEditor` | `partial`: `RuleKind` has Text, Number and Choice only | none | P3 |
| `HStack`, `VStack` | `NSStackView` | `HStack`, `VStack` | `partial`: wrap a `rect()`; no distribution, gravity or hiding priority; the `HStack` gap is 8 and the `VStack` gap follows the neighbours as SwiftUI's (a text, a symbol, a label, a checkbox, other controls); a label over a label is 0, not -1; a child added with `children` counts as a control | test `swiftui_defaults`; probe `spacing` | — |
| `ZStack`, `ViewThatFits`, `Layout` | — | Freya's `rect` | `partial` | none | P3 |
| `reorderable()` (macOS 27), swipe actions | drag in `NSTableView` | — | `missing` | sdk | P3 |

## 13. Presentation

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Menu` content, pop-up menu | `NSMenu` | `MenuView`, `MenuItem` | `partial`: measured in light and dark, not in Increase Contrast; item icons not measured | bench `menu` scene; [Menus][m-menus] | P1 |
| `.contextMenu` | `NSMenu.popUpContextMenu` | `ViewModifiers::context_menu` | `implemented` | same look as the menu | P1 |
| `Commands`, `CommandMenu`, `CommandGroup`, `TextEditingCommands` | main menu | `Commands`, `CommandGroup`, `MainMenu`, menu bar menu | `partial`: the App, File, Edit, Window and Help groups with the measured titles and Ctrl shortcuts; the Cut, Copy, Paste, Delete, Select All, Find and Help items start disabled until an app handles them (no responder chain, ADR 0085); no About, View, Format, sidebar, toolbar or inspector commands, no `commandsRemoved` | [`macos-standard-menus.md`](macos-standard-menus.md) | P1 |
| `MenuBarExtra` | `NSStatusItem` | `status_item` | `partial`: a static item (icon name, title, menu) | none | P2 |
| `.popover` | `NSPopover` | `Popover` | `partial`: `Popover::show` returns `()`, so no close, behavior or edge; the corner is measured (20, continuous; the compositor draws it) | [Corners][m-corners] | P1 |
| `.sheet` | `beginSheet` | `Sheet` | `partial`: light and dark only; the corner is measured (16 API, 16.8 silhouette, continuous; an `NSAlert` frame is 26) and the kit draws a `Sheet` at 16.8 and an `Alert` at 26, the shadow and edge are the compositor's | [Sheets][m-sheets], [Corners][m-corners] | P1 |
| `.alert` | `NSAlert` (`.warning`, `.informational`, `.critical`) | `Alert` | `partial`: the geometry comes from the view tree, not pixels; icon, `.critical`, accessory view, help button and stacked buttons are implemented and measured in four appearances, `.warning` is `.informational` | [Alerts][m-alerts] | P1 |
| `.confirmationDialog` | — | `Alert::add_button` | `implemented`: an alert whose three buttons stack; no layout of its own | [Alerts][m-alerts] | — |
| `.fileImporter`, `.fileExporter` | `NSOpenPanel`, `NSSavePanel` | `portal::open_files`, `portal::save_file` | `partial`: the desktop's dialog, not the macOS look | none | P3 |
| `.help` | `toolTip` | `ViewModifiers::help` | `measured`: delay, place, font, padding, corner (5.2), fill, ink and wrapping in light, dark and Increase Contrast | [Tooltips][m-tooltips], [Corners][m-corners] | — |
| — | `NSPrintPanel`, `NSPageLayout` | — | `missing` | none | P3 |
| `UNUserNotificationCenter` | banners | `notifications::post` | `implemented` | [`macos-notifications.md`](macos-notifications.md) | P2 |
| `.draggable`, `.dropDestination` | `NSDraggingSource`, `NSDraggingDestination` | `DragSource`, `ImageWell` drop | `partial`: drag out; drop only on the image well | none | P3 |
| `.presentationBackground`, `.presentationDetents` | — | `SheetStyle.background` | `partial`: detents are iOS | none | P3 |

## 14. Windows and chrome

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `RoundedRectangle(cornerRadius:style:)`, `.continuous` | `CALayer.cornerCurve = .continuous`, `cornerRadius` | `Shape::RoundedRectangle`, `CornerStyle`, `style::continuous_corner` | `measured`: the smoothing is 0.745 at radius 8 to 40 and the kit draws it (`CORNER_SMOOTHING`, ADR 0101); push, pop-up, segmented, text field, combo box, checkbox, group box, tab view, sidebar item, rows, cards and sheets are continuous, the circular and help buttons, the progress bar and what nobody measured are circular; scale 2 and the tail clamp near a capsule are not measured | [Corners][m-corners] section 12 | P1 |
| `ConcentricRectangle`, `.containerShape`, `.concentric(minimum:)` | `NSViewCornerRadius.containerConcentric`, `NSView.cornerConfiguration` | `ctappearance::corner::concentric`, `style::concentric` | `partial`: the rule is measured (AppKit and SwiftUI agree to 0.01); the kit uses it for the color well and the image well, no view API; measured cell-drawn nests (tab pill) keep their measurement | [Corners][m-corners] sections 7 and 12 | P3 |
| window corners | compositor | `Bar`, `WindowCorner`, `CORNER_WINDOW` | `measured`: 16 (API) and 16.8 (silhouette, continuous) in every style, light and dark, active and inactive; full screen 0; `WindowCorner` was 16, 20, 26, 15 and is 16 for all; the kit's `WindowGlass` hole is the 16.8 silhouette; scale 2 not measured | [Window corners][m-tabs], [Corners][m-corners] | P1 |
| traffic lights | `standardWindowButton(_:)` | `TrafficLights` | `measured`: geometry, glyphs, gradient, ring, halo and every state in light, dark and Increase Contrast, at 1x; the utility panel's 10 pt lights are not drawn and the full-screen lights are compared in light only | bench `lights`; [Traffic lights][m-lights] | P1 |
| `.windowStyle(.titleBar)` title bar | `NSWindow` titled, 32 high | `Titlebar` | `partial` | [`macos-windows.md`](macos-windows.md) §2 | P1 |
| `navigationTitle`, `navigationSubtitle` | `title`, `subtitle`, `titleVisibility` | `Titlebar` title only | `partial`: no subtitle | none | P1 |
| `.windowToolbarStyle(.unified)` items | `.unified`, `NSToolbarItem` | `IconButton`, `ToolGroup`, `NavigationButtons` | `measured` on SwiftUI twins in light and dark: the bar is 52 high with the title at the start when it has items, a pair is a 76 × 38 capsule with no divider, one item is a 38 circle, and the compose, ellipsis and sidebar glyphs have the ink boxes of SF to the pixel; capsule 9 to 21% of its pixels, the toggle 20%; the glass shadow and the toggle ring are not drawn | bench `swiftui_twin` (`notes`, `tasks`), `toolbar`; [Toolbar][m-toolbar] | P1 |
| `.windowToolbarStyle(.unifiedCompact)` | `.unifiedCompact` | `Bar::UnifiedCompact`, `Titlebar` | `partial`: 24 high platters, a pair of 63 × 24 within 1 of AppKit's; the 22 between the lights and the first platter is not drawn; `Titlebar::toolbar_style(UnifiedCompact)` asks for it | bench `toolbar`; [What the kit does][m-toolbar] | P2 |
| `.windowToolbarStyle(.expanded)` | `.expanded` | — | `missing`: numbers measured (28 high platters under a 32 title row) | [Toolbar][m-toolbar] | P2 |
| settings toolbar tabs | `.preference` | `TabView` | `partial` | [Toolbar tabs][m-tabs] | P1 |
| `ToolbarItem(placement:)` | `NSToolbarItem` | `Titlebar` left and right slots | `partial`: no placement set | none | P2 |
| `ToolbarItemGroup` | `NSToolbarItemGroup` | `ToolGroup` | `partial`: pair and N buttons measured; labels, select-one and momentary widths not drawn | [Groups][m-toolbar] | P1 |
| prominent item | `NSToolbarItem.style = .prominent` | — | `missing`: a 36 accent circle is measured | [Toolbar][m-toolbar] | P1 |
| `ToolbarSpacer` | `.space`, `.flexibleSpace` | — | `missing` | [Toolbar][m-toolbar] | P2 |
| item labels | `NSToolbar.displayMode` | — | `missing`: only icon-only was measured | none | P2 |
| `.badge` on an item | `NSToolbarItem.badge` | — | `missing` | none | P3 |
| `ToolbarItemVisibilityPriority` (macOS 26.1) | which items overflow first | — | `missing` | sdk | P3 |
| `ToolbarTitleMenu` | — | — | `missing` | none | P3 |
| `.searchable` | `NSSearchToolbarItem` | `ToolbarSearch` | `implemented` | none | P1 |
| overflow | `NSToolbar` overflow chevron | `ToolbarOverflow` | `implemented` | none | P2 |
| — | `NSTrackingSeparatorToolbarItem` | — | `missing` | none | P2 |
| — | `NSTitlebarAccessoryViewController` | — | `missing` | none | P2 |
| `.toolbar(id:)` customization | `allowsUserCustomization` | — | `missing` | none | P3 |
| title bar line | `NSTitlebarSeparatorStyle` | `TitlebarStyle` hairline | `partial`: none at rest, 0% of the pixels of the bar's edge differ in both twins and both appearances; the line that appears when content scrolls under the bar is not implemented | bench `swiftui_twin` | P2 |
| `NavigationSplitView` sidebar | `NSSplitViewItem(sidebarWithViewController:)` | `Sidebar`, `SidebarItem` | `measured` on the notes twin in light and dark: the background 0% of the pixels differ, the rows 6 to 7% (text), the selected row 10%; flat, full height and within 1 of macOS 27's measures; the bare glyph (accent, 17 px clock) is the default and the tile is `symbol_tile`; no Increase Contrast | bench `swiftui_twin` (`notes`), `sidebar`; [What the kit does][m-sidebar] | P1 |
| full screen | `toggleFullScreen` | `FullscreenBar` | `implemented` | none | P2 |
| `WindowGroup`, `Window`, `Settings`, `openWindow`, `dismissWindow` | `NSWindow`, `NSWindowController` | `WindowGroup`, `Window`, `Settings`, `use_open_window`, `use_dismiss_window`, `SettingsWindow` | `partial`: ids, one window per value, fronting a window that is open, and a Window menu from the kit's own list; no `openSettings` or `SettingsLink`, no launch behavior, no restore of the open windows, no position (xdg-shell) | [`macos-standard-menus.md`](macos-standard-menus.md) | P1 |
| `UtilityWindow` | `NSPanel` | `UtilityWindow`, `PanelBehavior` | `implemented` | none | P3 |
| — | window tabs | — | `absent`: no tab bar, no tab group | none | P3 |
| `DocumentGroup`, `AlertScene` | `NSDocument` | — | `missing` | none | P3 |
| `windowShouldClose`, `applicationShouldTerminate`, `application(_:open:)` | `NSWindowDelegate`, `NSApplicationDelegate` | — | `missing`: a non-goal until an app asks ([ADR 0085](../decisions.md)); a close request closes the window at once, except a sheet's, and the last window ends the process | [Swift syntax in Rust](swift-syntax-in-rust.md) | — |
| a command to the focused view | `NSResponder` chain, `validateMenuItem` | `Action`, `use_menu_action` | `missing`: every handler of the window gets every id, and a shortcut works only through an enabled menu item; a non-goal until an app asks ([ADR 0085](../decisions.md)) | [Swift syntax in Rust](swift-syntax-in-rust.md) | — |

## 15. Materials and glass

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Material` (`.ultraThin` to `.ultraThick`, `.bar`) | `NSVisualEffectView.Material`: 14 semantic materials | `Material`, `Thickness`; `ctappearance::material` | `partial`: the 14 semantic materials are measured in both blending modes, light and dark, active, inactive, with Reduce Transparency and Increase Contrast, at 1x, with tokens and a reference function (2026-11-22); the six SwiftUI thicknesses are measured active and `withinWindow` (sigma 59.2); the kit's `Thickness` is not mapped to them and scale 2 is not measured | [`macos-materials.md`](macos-materials.md) §10 | P2 |
| `NavigationSplitView` sidebar material | `NSVisualEffectView.Material.sidebar`, `behindWindow` | `Sidebar`, `WindowGlass` | `partial`: with version 10 the kit sends `set_backdrop` (`behind_window`, `sidebar`, the panel's corner) and leaves the panel transparent; with versions 6 to 9 it sends `set_glass`; the look of one against the other was not compared and none ran end to end ([ADR 0100](../decisions.md)) | [`macos-materials.md`](macos-materials.md) §10 | P1 |
| (no public API) the desktop shell's glass: menu panel, Control Center, Dock, notifications | — | `ShellGlass`, `ctappearance::shell_glass` | `partial`: the menu bar is clear (identity to 0 levels); the app menu, banner, Spotlight, Apps, notification card, Control Center tile and backing panel, Dock and Mission Control's Spaces bar are measured in dark at scale 1 (tone, chroma gain, kernel, edges, how the blur samples beyond them); light, Reduce Transparency, Increase Contrast, scale 2, the banner's kernel and the lock screen are not | [Desktop surfaces](macos-blur.md#14-desktop-surfaces-as-a-pipeline-measured-2026-11-22) | P1 |
| `.glassEffect(_:in:)` (`.regular`, `.clear`) | `NSGlassEffectView` | `GlassContainer`, `WindowGlass`; `ctappearance::liquid_glass` | `partial`: the kit asks the compositor for the glass; the stack is read and measured at 1x (2026-11-22): half-resolution backdrop, lens, blur by depth, tone, rim and shadow, tokens per state and a reference function with its error table; the compositor does not serve it yet and scale 2 is not measured; real popovers and menus are this glass | [`macos-liquid-glass.md`](macos-liquid-glass.md) | P1 |
| `GlassEffectContainer`, `glassEffectID` | `NSGlassEffectContainerView` | `GlassContainer` | `implemented`: the union of children is a smooth minimum of 0.8 of the spacing in the reference ([§7](macos-liquid-glass.md#7-interaction-merging-and-morphing)); the kit's own morph is not compared | [`macos-liquid-glass.md`](macos-liquid-glass.md) §7 | P2 |
| `Glass.tint(_:)`, `Glass.interactive(_:)`, `Glass.identity`, `glassEffectUnion`, `glassEffectTransition` | `NSGlassEffectView.tintColor`, `effectIsInteractive` (27.0) | — | `missing`: the tint is an alpha blend over the glass (measured), `interactive` has no static effect at rest and its hovered and pressed look is an affine map of the glass colour (forced state; a real pointer did not trigger it), the union is one element, `identity` is variant 14 | [`macos-liquid-glass.md`](macos-liquid-glass.md) §6–7 | P2 |
| `backgroundExtensionEffect` | `NSBackgroundExtensionView` | `SplitView::background_extension` | `partial`: stretches the first pixel column where Apple mirrors, and only under a docked sidebar | none | P3 |
| `scrollEdgeEffectStyle` | `NSScrollPocket` | `TopGlass` | `partial`: the protocol has `set_scroll_edge` (version 10, [ADR 0091](../decisions.md)); with a compositor that offers version 10 the kit declares the edge on a chrome sub-surface (`soft` for `TopGlass` and `ProgressiveGlass`, `hard` for `Glass`, a leading edge for the rail) and draws no blur ([ADR 0098](../decisions.md), [ADR 0100](../decisions.md), tested against a fake compositor, not run end to end); with any other it draws the opaque solid; the edge's own values are not measured and the glass of a full-screen bar no longer fades | [Sidebar][m-sidebar] | P2 |
| `.background(.material)` over scrolling content | `NSVisualEffectView`, `withinWindow` | `Material` | `partial`: with version 10 the kit sends `set_backdrop` (`within_window`, `MaterialStyle::material`, the corner radius) from a chrome sub-surface; the material it sends (`HudWindow`) was not compared with the kit's own look; one material per chrome; not run end to end | [Design 7.3](blur-backdrop-design.md#73-k3-the-slices-and-what-they-need) | P2 |
| vibrancy of `.secondary` over a material | `NSVisualEffectView` vibrant labels | `Role` colors | `partial` | none | P3 |

## 16. Symbols

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Image(systemName:)` | `NSImage(systemSymbolName:)` | `Image::system`, `Symbol` | `partial`: 151 symbols against the 2386 public names that resolve here; the 83 drawn on 2026-11-20 and 2026-11-21 and the 36 older ones redrawn on 2026-11-22 match SF at 16 pt (ink box within 0.55 in all but the 4 wider than the grid, mass within 10% in 88 of 119 in the lab); 29 older ones are not redrawn (5 sized into measured controls, 24 close to SF or small) | [Icons][m-icons] | P1 |
| toolbar glyphs | SF Symbols at 16 pt | `IconButton` symbols | `partial`: chevrons and plus match; `ellipsis`, `sidebar.left`, `magnifyingglass` do not | [Toolbar][m-toolbar] | P1 |
| `.symbolRenderingMode` (monochrome, hierarchical, palette, multicolor) | `NSImage.SymbolConfiguration` | `Image::symbol_rendering_mode`: monochrome and hierarchical (alpha 1, 0.5, 0.25 measured; 30 of the 36 symbols SF dims); `TemplateRenderingMode` (original, template) | `partial`: palette and multicolor missing | [Icons][m-icons] | P3 |
| `.imageScale`, symbol weight | `SymbolConfiguration` | `Image::image_scale`, `Image::font_weight` (nine weights, three scales, measured with `ImageRenderer`) | `measured`: on one image; the environment values that reach a subtree and the controls' own symbols do not follow | [Icons][m-icons] | P2 |
| `.symbolVariant`, `.symbolEffect`, variable value | `NSSymbolEffect` | — | `missing` | none | P3 |
| menu item icons | `NSMenuItem.image` | `MenuIcon` | `measured`: a SwiftUI menu draws them with `.labelStyle(.titleAndIcon)`, `NSMenuItem.image` draws nothing in a pop-up menu; column, gap, size and color match; AppKit sets 47 of 58 standard items' images, listed in [Icons][m-icons] | [Menus][m-menus] | P2 |

## 17. Accessibility

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| Increase Contrast | `accessibilityDisplayShouldIncreaseContrast` | `ContrastPalette`, `Resolved` | `partial`: controls and tables measured; menus, sheets, toolbar, sidebar and materials not | [Increase Contrast][m-hc] | P1 |
| Reduce Transparency | `accessibilityDisplayShouldReduceTransparency` | `Resolved::reduce_transparency`; menus keep glass with it | `partial`: `NSColor` ignores it; the Dock solids are measured, other materials are not | [`macos-materials.md`](macos-materials.md) §4 | P1 |
| Reduce Motion (`accessibilityReduceMotion`) | `accessibilityDisplayShouldReduceMotion` | `use_reduced_motion`, `motion_ms` | `implemented`: the kit jumps; what macOS substitutes was not measured | none | P2 |
| Differentiate Without Color | `accessibilityDisplayShouldDifferentiateWithoutColor` | `Resolved::differentiate_without_color`, read by `TrafficLights` only | `partial` | none | P3 |
| Invert Colors | `accessibilityDisplayShouldInvertColors` | `ctappearance` setting; no control reads it | `partial` | none | P3 |
| VoiceOver | `NSAccessibility` | `platform/atspi.rs` (AT-SPI) | `partial`: popups and tooltips publish no tree, the window name is empty, text has no ranges, only Focus and Click are served; no audit per control | none | P2 |
| focus ring, full keyboard access | `focusRingType` | `ViewModifiers::focus_ring` | `partial` | [`macos-focus-ring.md`](macos-focus-ring.md); bench `calendar-focused` | P2 |
| accent color: `Color.accentColor`, `.tint`, `AccentColor` | `NSColor.controlAccentColor`, `AppleAccentColor` | `ACCENTS`, `Resolved::accent_colors`, config `accent`, `accent_color`, `wallpaper_accent` | `measured`: the eight accents and Multicolor in light, dark and Increase Contrast; a color without a name follows a fitted rule | [Accent and selection](macos-control-metrics.md#accent-and-selection); bench `accents` | — |
| accent picker | System Settings › Appearance › Color | `ColorWell::accents` | `partial`: light and dark match at 1x; Increase Contrast not captured | [Accent picker](macos-control-metrics.md#the-picker-of-system-settings); bench `picker` | P3 |
| text highlight color | `selectedTextBackgroundColor`, `AppleHighlightColor` | `AccentColors::selected_text`, the theme's `text_highlight` | `partial`: follows the accent; a separate highlight color is not read | [Accent and selection](macos-control-metrics.md#accent-and-selection) | P3 |
| `DynamicTypeSize` (macOS 12) | — | `Resolved::text_size` scales `Text`, `SidebarItem` and the text and symbol of `Button` (its height stays) | `partial`: other controls do not scale; `preferredFont(forTextStyle:)` returns fixed sizes ([Text styles](#text-styles)); whether SwiftUI text on macOS 27 reacts to the setting was not probed | none | P2 |

### Motion

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Animation` (`.smooth`, `.snappy`, `.bouncy`, `.spring`, `.easeInOut`) | `NSAnimationContext` | `Spring`, `ease_out`, `ease_in_out` | `partial`: the window, Dock and menu timings were observed, some durations are unverified | [`macos-animations.md`](macos-animations.md) | P2 |
| `.transition`, `matchedGeometryEffect`, `PhaseAnimator`, `KeyframeAnimator` | — | — | `missing` | none | P3 |

## 18. New in macOS 27

APIs whose availability starts at macOS 27 in the SwiftUI and SwiftUICore
interfaces of this SDK (`@available(macOS 27.…)` or `anyAppleOS 27.…`). The
kit has none of them. `ToolbarItemVisibilityPriority` is macOS 26.1 and is not
listed.

| API | Since | What it is | Pri |
|---|---|---|---|
| `PickerStyle.tabs` | 27.0 | A picker drawn as tabs | P2 |
| `TextFieldStyle.bordered`, `TextInputBorderShape` | 27.0 | A bordered text field and the shape of its border | P2 |
| `ToolbarMinimizationBehavior` | 27.0 | How a toolbar shrinks | P3 |
| `ToolbarVerticalCompressionBehavior`, `ToolbarVerticalBehavior`, `ToolbarItemAxisBehavior` | 27.1 | How a toolbar and its items behave along the vertical axis | P3 |
| `ToolbarTitleAlignment` | 27.2 | Where the toolbar title sits | P3 |
| `ArrangementView`, `arrangementViewStyle` (automatic, split, overlay) | 27.1 | A container that arranges a primary and a secondary view | P2 |
| `ContentMarginGuide`, `ReservedRegion` | 27.1 | Layout guides for content margins and reserved regions | P3 |
| `reorderable()`, `reorderContainer`, `reorderDestination` | 27.0 | Drag to reorder in a collection | P3 |
| `Document`, `DocumentReader`, `DocumentWriter` | 27.0 | A new document model | P3 |
| `TabRole.prominent`, `TabSectionExpansion` | 27.0 | Tab roles and section expansion | P3 |
| `PresentationPlacement` | 27.0 | Where a presentation appears | P3 |
| `AsyncImage(request:)` | 27.0 | An image from a `URLRequest` | P3 |
| `EnvironmentValues.systemPrefersReducedResourceUsage` | 27.0 | The system asks apps to use fewer resources | P3 |

## Backlog

Ordered by priority inside each area. An item names the rows it closes. The
first four areas are the sessions that run now.

### A. Buttons and selection controls

1. **P1** ~~Sizes of the controls and a `.control_size` modifier that reaches the
   controls inside a view~~ done ([11](#11-control-sizes)).
2. **P1** Probe and compare the indeterminate `Spinner` and a SwiftUI `Gauge`
   ([6](#6-gauges-progress-and-level-indicators)).
3. **P2** ~~`Mini` and `ExtraLarge` in `ControlSize` for `Button`~~ done.
4. **P2** Compare the buttons that are only `implemented`: `glass`,
   `glass_prominent` and the menu styles. `destructive`, `borderless`,
   `plain`, `link` and the toggle button are compared (family `swiftui`).
5. **P2** Segmented control: symbols, the four segment styles, momentary
   tracking, and `segmented-any` in the bench ([3](#3-pickers)).
6. **P2** Probe SwiftUI `Picker` with `.inline`, `.palette` and `.tabs`.
7. **P3** The `badge` bezel; the other AppKit bezels are done (family `buttons`).
8. **P3** Ring at mini and small, rating image, tick positions, vertical
   slider.

### B. Text, dates, color and panels

1. **P1** Text styles: `title2` (17 / 22) is left; the kit's `TITLE_2` is
   15 / 20 and the titlebar uses it. ~~`footnote`, `caption1` and `caption2`~~
   done as `FOOTNOTE`, `CAPTION_2` (and `TITLE_3`, `SUBHEADLINE`): numbers
   only, no bench ([Text styles](#text-styles)).
2. **P1** `TextEditor` and `NSTextView`: the field and the text are measured
   (`textview-system`). Caret, selection, placeholder and find bar are left
   ([7](#7-text)); a hosted SwiftUI `TextEditor` draws 12 pt Helvetica until
   the app sets a font.
3. **P1** ~~Text field at mini (19) and small (22); `.plain`~~ done.
   `.squareBorder`, the new `.bordered` and `TextField(axis: .vertical)` are left.
4. **P2** Date picker: `.compact`, time parts with seconds, `MultiDatePicker`,
   range mode, the analog clock ([8](#8-date-and-time)). ~~`.field` without
   stepper~~ done.
5. **P2** Color panel: the other seven modes, alpha, eyedropper; probe the
   SwiftUI `ColorPicker` popover ([9](#9-color-and-font-panels)). ~~`.minimal`
   and `.expanded` wells~~ done.
6. **P2** Search: scopes, suggestions and the recents menu.
7. **P2** Rich text: underline, strike, color, paragraph style, lists.
8. **P3** Font panel, rulers, text lists and tables; image scaling modes;
   probe `NSPathControl` ([10](#10-image-and-path-control)).

### C. Lists, tables, forms, containers and presentation

1. **P1** ~~A `List` view of free rows with selection and separators~~ done
   ([ADR 0086](../decisions.md)); `.sidebar` is `Sidebar`. The rows with views are
   not measured: probe a SwiftUI `List` row with an avatar, two lines and a badge
   (height, insets, where the separator starts), and `GroupBox` and
   `DisclosureGroup`, which no probe has ([12](#12-containers-and-navigation)).
2. **P1** ~~`Table`: cells with controls~~ done (icons, row size styles, the
   other table styles and the scrollers were already done); a control cell is
   not measured.
3. **P1** `Form` with `.columns`, compared with a probe of a macOS 27
   Settings window.
4. **P1** Alert: icon, accessory view, help button, text field; popover probe;
   sheet and menu in Increase Contrast ([13](#13-presentation)).
5. **P1** `TabView`: the toolbar tabs as a bench entry (the classic tabs are
   done).
6. **P2** A three-column `NavigationSplitView`; a path and destination API for
   `NavigationStack`; confirmation dialog;
   tooltip and `NSSplitView` divider probes.
7. **P3** Reordering, swipe actions, collection layouts, drop targets.

### D. Window and toolbar

1. **P1** Unified toolbar: prominent item, glyphs other than the chevron and
   the plus, search item, offset of a lone chevron, group widths with labels,
   Increase Contrast capture ([14](#14-windows-and-chrome)).
2. **P1** Sidebar: scrolling under the bar (the bare glyph is done), Increase Contrast capture ([Sidebar][m-sidebar]).
3. **P1** Traffic lights, title bar, title with subtitle and window corner
   as bench entries.
4. **P1** Materials and Reduce Transparency: map the 14 semantic materials;
   Increase Contrast for menu, sheet and toolbar ([15](#15-materials-and-glass),
   [17](#17-accessibility)).
5. **P1** Symbols: more than 124, with weight, scale and the ones wider than the grid ([16](#16-symbols)).
6. **P2** `unifiedCompact` and `expanded` platters (24 and 28 high); toolbar
   labels, spacer, tracking separator, title bar accessory.
7. **P3** The [macOS 27 APIs](#18-new-in-macos-27), toolbar customization,
   document scenes.
8. **P2** Standard menus still missing: About, View, Format, the sidebar,
   toolbar and inspector commands, Close All (an alternate), `commandsRemoved`,
   and the Fill, Center and Move & Resize items of the Window menu, which the
   probe did not show ([14](#14-windows-and-chrome)).

### Probes to write first

A row stays `implemented` until a probe captures the AppKit or SwiftUI side.
No probe of `~/src/probes` renders these:

| Probe | Items | Area |
|---|---|---|
| SwiftUI `Form`, `List`, `GroupBox`, `DisclosureGroup`, `LabeledContent` in a macOS 27 window | row heights, section cards, separators, selection | C |
| SwiftUI `Gauge` in each style, `ProgressView` indeterminate, `Spinner` | sizes and colors | A |
| `NSTextView` in `NSScrollView` and SwiftUI `TextEditor` | insets, caret, selection, placeholder, find bar | B |
| `NSColorPanel` and the SwiftUI `ColorPicker` popover | modes, sizes, alpha | B |
| `NSPopover` and SwiftUI `.popover`, `.confirmationDialog` | size, arrow (the corner is measured: [Corners][m-corners]) | C |
| `NSToolbar` display modes and `NSToolbarItem` prominent, badge, spacer | widths, labels | D |
| `NSWindow` subtitle, title bar accessory and tracking separator | heights, positions | D |
| `NSSplitView` thin divider, `NSPathControl`, `NSImageView` | widths, heights | C |

## Drift and open questions

- [`macos-control-metrics.md`](macos-control-metrics.md) says the circular
  `NSSlider` has no kit counterpart. `CircularSlider` exists and the bench
  compares `slider-circular`.
- The same file gives a text field 24 high at every size. The probe, which
  also sets the font of each size, captures 19 at mini and 22 at small.
- [`appkit-comparison.md`](appkit-comparison.md) lists the scenes of the bench
  without the ones the parallel sessions are adding.
- `~/src/probes/swiftui.swift` renders SwiftUI `Button`, `Toggle`, `Picker`,
  `TextField`, `Slider` and `ProgressView`, but `~/src/captures` holds no
  capture of it. Every comparison but the sidebar is against AppKit.
- Not probed on this Mac: SwiftUI `Form`, `List`, `GroupBox`,
  `DisclosureGroup`, `Gauge`, `Menu` styles, `TextEditor`, `.popover`,
  `.confirmationDialog`, `NSSplitView`, `NSPathControl`, `NSImageView`,
  `NSColorPanel`, `NSFontPanel`, toolbar display modes and window subtitles.
- The availability of SwiftUI styles comes from the `@available` lines of the
  interface; a style with `macOS, unavailable` is listed as not on macOS.
- The matrix keeps no syntax-level rows. `Binding` is not in the kit and is a
  non-goal until an app asks ([ADR 0085](../decisions.md)); the Swift
  constructs and their kit idioms are in
  [SwiftUI to the kit](../barrierkit/16-swiftui-to-the-kit.md).

[m-sizes]: macos-control-metrics.md#sizes-width--height-points
[m-control-sizes]: macos-control-metrics.md#control-sizes
[m-open]: macos-control-metrics.md#open-differences
[m-text]: macos-control-metrics.md#text-fields
[m-texts]: macos-control-metrics.md#text-date-color-and-image-controls
[m-hc]: macos-control-metrics.md#reduce-transparency-and-increase-contrast
[m-rings]: macos-control-metrics.md#rings-level-indicators-and-boxes
[m-tables]: macos-control-metrics.md#tables-and-source-lists
[m-alerts]: macos-control-metrics.md#alerts
[m-tooltips]: macos-control-metrics.md#tooltips
[m-lists]: macos-control-metrics.md#lists-tables-and-outlines-swiftui
[m-sheets]: macos-control-metrics.md#sheets
[m-menus]: macos-control-metrics.md#menus-scrollers-and-inactive-selection
[m-sidebar]: macos-control-metrics.md#sidebar-and-window-macos-27
[m-lights]: macos-control-metrics.md#traffic-lights-macos-27
[m-toolbar]: macos-control-metrics.md#toolbar-items-and-navigation-buttons
[m-icons]: macos-icons.md
[m-tabs]: macos-control-metrics.md#toolbar-tabs-and-window-corners
[m-corners]: macos-corners.md
[m-tabviews]: macos-control-metrics.md#tab-views
[m-scrollers]: macos-control-metrics.md#scrollers
[m-classic]: macos-control-metrics.md#classic-tables-and-outlines
