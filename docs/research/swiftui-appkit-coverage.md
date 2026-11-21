# SwiftUI and AppKit coverage

**Goal:** list, family by family, what SwiftUI and AppKit offer for designing
macOS 27 apps and what BarrierKit already has, so sessions that work in
parallel pick work without overlap.
**Scope:** SwiftUI and AppKit as the macOS 27.2.1 SDK (Xcode 27.2) ships them,
against `bk` on `main`. It covers the look and behavior of controls,
containers, presentation, windows, materials, symbols and accessibility. It
does not cover data, networking, media, document or graphics frameworks.
**Snapshot:** `main` at `bb83b10`, 2026-11-20. Sessions that change a row
change it in the same commit.
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
`views-contrast`, `menu`, `toolbar` and `sidebar` (light and dark; it compares
the kit with the SwiftUI probe `sidebarui`). Sessions that add the scenes
`texts` and `lists` move rows from `implemented` to `measured`.

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
| 1. Buttons and button styles | 5 | 7 | 5 | 8 | 25 |
| 2. Toggles | 4 | 1 | 0 | 2 | 7 |
| 3. Pickers | 3 | 2 | 2 | 5 | 12 |
| 4. Steppers | 1 | 1 | 0 | 1 | 3 |
| 5. Sliders | 3 | 2 | 0 | 1 | 6 |
| 6. Gauges, progress and level indicators | 6 | 3 | 0 | 2 | 11 |
| 7. Text | 5 | 4 | 13 | 10 | 32 |
| 8. Date and time | 2 | 0 | 1 | 5 | 8 |
| 9. Color and font panels | 1 | 0 | 2 | 4 | 7 |
| 10. Image and path control | 0 | 3 | 1 | 0 | 4 |
| 11. Control sizes | 0 | 0 | 1 | 1 | 2 |
| 12. Containers and navigation | 4 | 10 | 9 | 8 | 31 |
| 13. Presentation | 0 | 6 | 6 | 2 | 14 |
| 14. Windows and chrome | 0 | 6 | 11 | 11 | 28 |
| 15. Materials and glass | 0 | 2 | 4 | 0 | 6 |
| 16. Symbols | 0 | 0 | 5 | 1 | 6 |
| 17. Accessibility | 0 | 2 | 7 | 1 | 10 |
| **All** | 34 | 49 | 67 | 62 | 212 |

To recount after a change, run this from the repository root:

```sh
awk -F'|' '/^## /{s=$0} /^\|/{for(i=4;i<=6&&i<=NF;i++)if(match($i,/^ *`(measured|implemented|partial|missing)`/)){w=substr($i,RSTART,RLENGTH);gsub(/[ `]/,"",w);n[s" "w]++;break}} END{for(k in n)print n[k],k}' docs/research/swiftui-appkit-coverage.md | sort -k2
```

## 1. Buttons and button styles

Kit: `Button`, `ButtonStyle`, `ControlSize` (`Small`, `Regular`, `Large`).
AppKit has 13 current bezel styles (`NSButtonCell.h`). Nine names are
deprecated: `shadowlessSquare` and `texturedSquare` point to `smallSquare`;
`rounded`, `regularSquare`, `texturedRounded`, `roundRect`, `recessed`,
`roundedDisclosure` and `inline` point to `push`, `flexiblePush`, `toolbar`,
`accessoryBarAction`, `accessoryBar`, `pushDisclosure` and `badge`.

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.bordered`, `.automatic` at regular, small, large | `.push` (`.automatic` since 14) | `Button`, `Button::size` | `measured` | bench `push-regular`, `push-small`, `push-large`, `push-disabled`, `push-pressed`; [Sizes][m-sizes]; [Increase Contrast][m-hc] | — |
| `.bordered` at mini and extra large | `.push` with `.mini`, `.extraLarge` | `ControlSize` has no `Mini` or `ExtraLarge` | `missing` | probe `push-mini` 46 × 16, `push-xl` 77 × 36 | P2 |
| `.borderedProminent` | `.push` with `keyEquivalent = "\r"` (the default button) | `ButtonStyle::bordered_prominent` | `measured` | bench `default-regular`, `default-disabled`, `default-pressed` | — |
| `Button(role: .destructive)` | `hasDestructiveAction` | `ButtonStyle::destructive` | `implemented` | probe `push-destructive` 64 × 24 | P2 |
| `.borderless` | `isBordered = false` | `ButtonStyle::borderless` | `implemented` | none | P2 |
| `.plain` | `isBordered = false` | — | `missing` | none; `borderless` is the nearest | P2 |
| `.link` | — | `Link` | `implemented` | none | P2 |
| `.glass` | `.glass` (macOS 26) | `ButtonStyle::glass` | `implemented` | probe `glass` 58 × 24; the compositor draws the glass, so fill and rim are not comparable ([Toolbar][m-toolbar]) | P2 |
| `.glassProminent` | `.glass`; no probe shows the prominent look | `ButtonStyle::glass_prominent` | `implemented` | none | P2 |
| `.accessoryBar`, `.accessoryBarAction` | `.accessoryBar`, `.accessoryBarAction` | — | `missing` | probe `accessory` 63 × 24 | P3 |
| `buttonBorderShape` (`.capsule`, `.circle`, `.roundedRectangle`) | — | `ButtonStyle.shape` | `partial`: only the `circular` preset; the others go through the style closure | none | P3 |
| `.buttonBorderShape(.circle)` | `.circular` | `ButtonStyle::circular` | `measured` | bench `circular`; [Round button][m-rings] | — |
| `HelpLink` | `.helpButton` | `HelpLink` | `measured` | bench `help` | — |
| `DisclosureGroup` triangle | `.disclosure` | `DisclosureGroup` | `measured` | bench `disclosure`; [Disclosure triangle][m-rings] | — |
| — | `.pushDisclosure` | — | `missing` | probe `push-disclosure` 24 × 24 | P3 |
| — | `.smallSquare` (the +/− bar under a list) | — | `missing` | probe `small-square` 39 × 22 | P3 |
| — | `.toolbar` (replaces the textured styles) | `IconButton` inside `ToolGroup` only | `missing` outside a toolbar | probe `toolbar-button` 42 × 23 | P3 |
| `.badge(_:)` | `.badge` (replaces `inline`): a count in a table row | `Badge` (red capsule counter) | `partial`: neither the SwiftUI badge nor the bezel was probed | none | P2 |
| — | `.flexiblePush` | — | `missing` | not probed | P3 |
| `Toggle` with `.button` | `NSButton` toggle types | `Button::toggled` | `implemented` | none | P2 |
| `Menu` with `.button`, `.borderlessButton` (`.borderedButton` is deprecated) | pull-down `NSPopUpButton`, `NSMenuToolbarItem` | `MenuButton`, `PopUpButton::pulls_down` | `partial`: the pull-down pop-up is measured (bench `pulldown`), the menu styles are not | probe `pulldown` 71 × 24 | P1 |
| `Menu(primaryAction:)` | `NSComboButton` | `ComboButton` | `implemented` | none | P3 |
| `ControlGroup` (`.automatic`, `.menu`, `.palette`, `.navigation`, `.compactMenu`) | `NSToolbarItemGroup`, `NSSegmentedControl` | `ControlGroup`, `ToolGroup` | `partial`: `ToolGroup` is in [Toolbar][m-toolbar]; the SwiftUI styles are not probed | none | P2 |
| `ShareLink` | `NSSharingServicePicker` | `ShareLink` | `partial`: copy, email and open only | none | P3 |
| `PasteButton`, `EditButton`, `SettingsLink` | — | — | `missing` | none | P3 |

## 2. Toggles

Kit: `Switch`, `Checkbox`, `RadioGroup`.

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.toggleStyle(.switch)` | `NSSwitch` | `Switch` | `measured` | bench `switch-off-regular`, `switch-on-regular`, `switch-on-disabled`, `switch-off-disabled`; [Rings][m-rings] (Switch); every size is 54 × 24 | — |
| `.toggleStyle(.checkbox)` at regular | `NSButton` `.switch` | `Checkbox` | `measured` | bench `check-off-regular`, `check-on-regular`, `check-mixed`, `check-on-disabled`, `check-off-disabled`, `check-off-pressed`, `check-on-pressed` | — |
| `.checkbox` at mini, small, large, extra large | `controlSize` | `Checkbox` takes no `ControlSize` | `missing` | probe `check-off-mini` 44 × 12, `-small` 52 × 14, `-large` 63 × 18, `-xl` 63 × 18; [Sizes][m-sizes] | P1 |
| `Picker` with `.radioGroup` at regular | `NSButton` `.radio` | `RadioGroup` | `measured` | bench `radio-off-regular`, `radio-on-regular`, `radio-off-disabled`, `radio-on-disabled`, `radio-off-pressed`, `radio-on-pressed` | — |
| `.radioGroup` at other sizes | `controlSize` | `RadioGroup` takes no `ControlSize` | `missing` | probe `radio-off-mini` 41 × 12 to `radio-off-xl` 59 × 18 | P2 |
| mixed state | `allowsMixedState` | `Checkbox::mixed` | `measured` | bench `check-mixed`; [Rings][m-rings] (Mixed checkbox) | — |
| `Toggle` row in a `Form` | — | `Row::toggle` | `implemented` | none | P1 |

## 3. Pickers

Kit: `SegmentedControl`, `PopUpButton`, `RadioGroup`, `ColorWell`, `ComboBox`.

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.segmented` at regular | `NSSegmentedControl` select one | `SegmentedControl` | `measured` | bench `segmented-regular`, `segmented-disabled`; [Combo box and segmented][m-rings] | — |
| `.segmented` at mini, small, large, extra large | `controlSize` | `SegmentedControl` takes no `ControlSize` | `missing` | probe `segmented-mini` 134 × 16, `-small` 146 × 20, `-large` 170 × 28, `-xl` 194 × 36 | P2 |
| `.segmented` with symbols | `NSSegmentedControl(images:)` | `SegmentedControl::segment` takes text only | `missing` | none | P2 |
| — | `segmentStyle` (`.rounded`, `.roundRect`, `.texturedSquare`, `.smallSquare`) | — | `missing` | not probed | P3 |
| — | select any, momentary | `ToggleGroup` (select any); momentary only in `ToolGroup` | `partial` | probe `segmented-any` 95 × 24, not compared | P2 |
| `.menu` at regular | `NSPopUpButton` | `PopUpButton` | `measured` | bench `popup-regular`, `popup-disabled`, `popup-pressed`, `pulldown` | — |
| `.menu` at mini, small, large, extra large | `controlSize` | `PopUpButton` takes no `ControlSize` | `missing` | probe `popup-mini` 62 × 16, `-small` 76 × 20, `-large` 97 × 28, `-xl` 111 × 36 | P2 |
| `.menu` sections and separators | `NSMenuItem.sectionHeader(title:)`, `.separator()` | `PopUpButton::section_header`, `PopUpButton::separator` | `implemented` | none | P2 |
| `.radioGroup` | `NSButton` `.radio` | `RadioGroup` | `measured` | see [Toggles](#2-toggles) | — |
| `.inline` | — | `RadioGroup` (nearest) | `partial`: no probe shows the SwiftUI look | none | P3 |
| `.palette` | — | `ColorWell` (swatch row) | `implemented` | none | P3 |
| `.tabs` (macOS 27) | — | — | `missing` | sdk `PickerStyle.tabs`, introduced macOS 27.0 | P2 |
| `.wheel`, `.navigationLink` | — | — | not on macOS | sdk marks both unavailable on macOS | — |
| `DatePicker`, `ColorPicker` | `NSDatePicker`, `NSColorWell` | see [8](#8-date-and-time), [9](#9-color-and-font-panels) | | | |

## 4. Steppers

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Stepper` at regular | `NSStepper` | `Stepper` | `measured` | bench `stepper-regular`, `stepper-disabled`; [Rings][m-rings] (Internal glyphs, Stepper divider) | — |
| `Stepper` at mini, small, large, extra large | `controlSize` | `Stepper` takes no `ControlSize` | `missing` | probe `stepper-mini` 13 × 16, `-small` 17 × 20, `-large` 23 × 28, `-xl` 30 × 36 | P2 |
| `Stepper` with label and value | `NSStepper` next to a text field | `Stepper` with `StepperStyle.value_position` | `implemented` | none | P2 |

## 5. Sliders

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Slider` at regular | `NSSlider` linear | `Slider` | `measured` | bench `slider-regular`, `slider-disabled`; [Open differences][m-open] | — |
| `Slider` with tick marks | `numberOfTickMarks`, `allowsTickMarkValuesOnly` | `Slider::number_of_tick_marks`, `Slider::stepped` | `measured` for 5 ticks in the default position | bench `slider-ticks` | — |
| `Slider` at mini, small, large, extra large | `controlSize` | `Slider` takes no `ControlSize` | `missing` | probe `slider-mini` 160 × 12, `-small` 160 × 14, `-large` 160 × 20; knobs 15.5, 17.5, 23.5 wide ([Open differences][m-open]) | P2 |
| — | `sliderType = .circular` | `CircularSlider` | `measured` | bench `slider-circular`; the metrics sentence that says the kit has no counterpart is out of date ([drift](#drift-and-open-questions)) | — |
| — | `isVertical` | `SliderStyle.axis` | `implemented` | none | P3 |
| `Slider` with minimum and maximum labels | `NSSliderCell` captions | `Slider` marks and captions | `implemented` | none | P3 |

## 6. Gauges, progress and level indicators

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `ProgressView(value:)` | `NSProgressIndicator` bar | `ProgressView` | `measured` | bench `progress-bar`; [Gauges][m-rings] (bar) | — |
| `ProgressView()` bar, indeterminate | `isIndeterminate` | `ProgressView` | `implemented` | none; the probe fixes `isIndeterminate = false` | P2 |
| `.progressViewStyle(.circular)` with a value | spinning, determinate | `ProgressRing` | `measured` at regular | bench `ring-regular`; [Rings][m-rings] | — |
| ring at mini and small | `controlSize` | `ProgressRing` takes no `ControlSize` | `missing` | probe `ring-mini` and `ring-small` 16 × 16; regular, large and extra large 32 × 32 | P3 |
| `ProgressView()` circular, indeterminate | spinning, indeterminate | `Spinner` | `implemented` | none | P1 |
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
| `.roundedBorder`, `.automatic` | `NSTextField` bezeled | `TextField` | `measured` | bench `textfield-regular`, `textfield-filled`; [Text fields][m-text] | — |
| `.plain` | `isBezeled = false` | `TextFieldStyle` fields, no preset | `partial` | none | P2 |
| `.squareBorder` | `bezelStyle = .squareBezel` | — | `missing` | probe `textfield-rounded` 140 × 24 only | P3 |
| `.bordered` (macOS 27) | — | — | `missing` | sdk `BorderedTextFieldStyle`, introduced macOS 27.0; not probed | P2 |
| `TextField` at mini and small | `controlSize` | `TextField` takes no `ControlSize` | `missing` | probe `textfield-mini` 140 × 19, `-small` 140 × 22 (with the font of that size) | P2 |
| `TextField(axis: .vertical)` | wrapping `NSTextField` | `TextField` is one line | `missing` | none | P2 |
| `.searchable` | `NSSearchField` | `SearchField` | `measured` | bench `search`, `search-filled` | — |
| `.searchable` in a toolbar | `NSSearchToolbarItem` | `ToolbarSearch` | `implemented` | probe `toolbar` has the item; not compared | P1 |
| `searchScopes`, `searchSuggestions`, `searchCompletion` | `searchMenuTemplate`, recents | `SearchField` tokens only | `missing` | none | P2 |
| `SecureField` | `NSSecureTextField` | `SecureField` | `measured` | bench `secure` | — |
| — | `NSTokenField` | `TokenField` | `measured` | bench `token` | — |
| — | `NSComboBox` | `ComboBox` | `measured` | bench `combo`, `combo-disabled` | — |
| `TextEditor` | `NSTextView` in `NSScrollView` | `TextEditor`, `TextView` | `implemented` | none | P1 |
| `TextEditorStyle` (`.automatic`, `.plain`) | `NSTextView` | `TextEditorStyle` fields, no preset | `partial` | none | P3 |
| `TextEditor` with `AttributedString`, `AttributedTextSelection` | rich `NSTextView` | `rich_text` (bold, italic, link, family, size) | `partial`: no underline, strike, color, paragraph style or list | none | P2 |
| `findNavigator` | `NSTextFinder` | `TextEditor` find bar | `implemented` | none | P2 |
| `writingToolsBehavior`, spelling, substitutions | `NSSpellChecker`, Writing Tools | `spelling` (Hunspell) | `partial`: spelling only | none | P3 |
| — | `NSRulerView`, `NSTextList`, `NSTextTable` | — | `missing` | none | P3 |

### Text styles

Read on macOS 27.2.1 with `NSFont.preferredFont(forTextStyle:)`: size, line
(ascent plus descent plus leading) and weight. The kit draws Inter or Adwaita
Sans, so glyph widths differ ([Limits](appkit-comparison.md#limits)).

| SwiftUI | AppKit | macOS 27.2.1 | Kit | Status | Pri |
|---|---|---|---|---|---|
| `.largeTitle` | `.largeTitle` | 26 / 32, regular | `LARGE_TITLE` 26 / 32, 700 | `partial`: weight | P1 |
| `.title` | `.title1` | 22 / 26, regular | `TITLE` 22 / 26, 650 | `partial`: weight | P1 |
| `.title2` | `.title2` | 17 / 22, regular | — | `missing` | P1 |
| `.title3` | `.title3` | 15 / 20, regular | `TITLE_2` 15 / 20, 600 | `partial`: weight | P2 |
| `.headline` | `.headline` | 13 / 16, bold | `HEADLINE` 13 / 18, 700 | `partial`: line | P1 |
| `.body` | `.body` | 13 / 16, regular | `BODY` 13 / 18, 400 | `partial`: line | P1 |
| `.callout` | `.callout` | 12 / 15, regular | `CALLOUT` 12 / 16, 400 | `partial`: line | P2 |
| `.subheadline` | `.subheadline` | 11 / 14, regular | `CAPTION` 11 / 14, 500 | `partial`: weight | P2 |
| `.footnote` | `.footnote` | 10 / 13, regular | — | `missing` | P2 |
| `.caption` | `.caption1` | 10 / 13, regular | — | `missing` | P2 |
| `.caption2` | `.caption2` | 10 / 13, regular | — | `missing` | P3 |
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
| `.stepperField` | `.textFieldAndStepper`, year, month, day | `DatePicker` | `measured` | bench `datepicker`; [Date picker][m-rings] | — |
| `.field` | `.textField` (no stepper) | — | `missing` | not probed | P3 |
| `.graphical` | `.clockAndCalendar`, year, month, day | `DatePicker::graphical` | `measured` | bench `calendar`, `calendar-focused`; [Graphical date picker][m-rings]; 8–13% of the pixels differ | — |
| `.compact` | — | — | `missing` | not probed | P2 |
| time parts | `.hourMinute`, `.hourMinuteSecond` | `DateElements::Time`, `DateAndTime` | `partial`: no seconds; the probe draws year, month and day only | none | P2 |
| — | analog clock of `.clockAndCalendar` | — | `missing` | not probed | P3 |
| — | `datePickerMode = .range` | — | `missing` | none | P3 |
| `MultiDatePicker` | — | — | `missing` | none | P3 |

## 9. Color and font panels

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `ColorPicker` well | `NSColorWell` default style | `ColorPicker` | `measured` | bench `colorwell`, `colorwell-blue`; [Color well][m-rings] | — |
| — | `NSColorWell` `.minimal`, `.expanded` | — | `missing` | not probed | P3 |
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
| — | `NSImageView` editable, `NSImageCell` frame styles | `ImageWell` | `implemented` | none | P3 |
| — | `NSPathControl` (`.standard`, `.popUp`, `.navigationBar`) | `PathControl` | `implemented` | none | P3 |

## 11. Control sizes

SwiftUI has `.controlSize(_:)` with `.mini`, `.small`, `.regular`, `.large` and
`.extraLarge` (since macOS 14). AppKit has `NSControl.ControlSize` with the
same five cases (`.extraLarge` since macOS 26). The kit has `ControlSize` with `Small`, `Regular` and `Large`, and only
`Button::size` takes it. Nothing passes a size down to the controls inside a
view, as `.controlSize` does. `NSView.prefersCompactControlSizeMetrics`
(macOS 26) was not probed.

Sizes in points, from the probe `controls` on macOS 27.2.1 (`aqua`, white
backdrop; width × height or height):

| Control | mini | small | regular | large | extra large | Kit |
|---|---|---|---|---|---|---|
| Push button | 16 | 20 | 24 | 28 | 36 | small, regular, large |
| Checkbox, radio | 12 | 14 | 16 | 18 | 18 | regular |
| Switch | 54 × 24 | 54 × 24 | 54 × 24 | 54 × 24 | 54 × 24 | regular |
| Slider | 12 | 14 | 16 | 20 | 20 | regular |
| Segmented control | 16 | 20 | 24 | 28 | 36 | regular |
| Pop-up button | 16 | 20 | 24 | 28 | 36 | regular |
| Stepper | 13 × 16 | 17 × 20 | 20 × 24 | 23 × 28 | 30 × 36 | regular |
| Text field (with the size's font) | 19 | 22 | 24 | 24 | 24 | regular |
| Search field, combo box (metrics) | 16 | 20 | 24 | 28 | not captured | regular |
| Ring | 16 | 16 | 32 | 32 | 32 | regular |

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `.controlSize(_:)` as an environment value | `controlSize` per control | `Button::size` per instance | `partial` | [Sizes][m-sizes] | P1 |
| `.mini`, `.extraLarge` | `.mini`, `.extraLarge` | — | `missing` | table above | P2 |

The size of each control is a row in its own family.

## 12. Containers and navigation

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `List` (`.automatic`, `.inset`, `.plain`, `.sidebar`, `.bordered`) | single-column `NSTableView` | — (`Table`, `OutlineView`, `Sidebar` cover parts) | `missing`: no list of free rows with selection and separators | none | P1 |
| `Table` with `.inset` | `NSTableView` `.inset` | `Table` | `measured` | bench `table`; [Tables][m-tables] | — |
| `Table` `.automatic`, `.bordered` | `.automatic`, `.fullWidth`, `.sourceList`, `.plain`; `rowSizeStyle` | `TableStyle` fields; no preset but the inset look | `missing` | not probed | P2 |
| `TableColumn` with views | cell views | `Table` shows text rows | `missing` | none | P1 |
| sort descriptors | `NSSortDescriptor` | `SortOrder`: one column | `partial` | [Table sorted column][m-rings] | P3 |
| `OutlineGroup` | `NSOutlineView` | `OutlineView`, `TreeNode` | `implemented` | none; the source list is the classic one ([Tables][m-tables]) | P2 |
| — | `NSBrowser` | `Browser` | `implemented` | none | P3 |
| `Form` `.grouped` | — | `Form`, `Section`, `Row` | `implemented` | none; no probe has a SwiftUI `Form` | P1 |
| `Form` `.columns` | `NSGridView` | — | `missing` | none | P2 |
| `LabeledContent` | — | `Row` | `implemented` | none | P2 |
| `GroupBox` | `NSBox` primary | `GroupBox` | `measured` | bench `box`; [Group box][m-rings] | — |
| `Divider` | `NSBox` separator | `Divider` | `measured` | bench `separator` | — |
| `DisclosureGroup` | disclosure triangle | `DisclosureGroup` | `partial`: the triangle is measured, not the indent or the animation | bench `disclosure` | P2 |
| `Label`, `LabelStyle` | — | `LabelStyle` for menu buttons only | `partial`: no general `Label` view | none | P2 |
| `ContentUnavailableView` | — | `ContentUnavailableView` | `implemented` | none | P2 |
| `TabView` (toolbar-like tabs) | `NSTabViewController` `.toolbar` | `TabView` | `partial`: measured in [Toolbar tabs][m-tabs], not in the bench | probe `tabs` | P1 |
| `TabView` classic | `NSTabView` (top, bottom, left, right, none), `.segmentedControlOnTop`, `.segmentedControlOnBottom` | — | `missing` | probe `views` captures `NSTabView` | P2 |
| `TabView` `.sidebarAdaptable`, `.tabBarOnly`, `Tab`, `TabSection` | — | — | `missing` | sdk | P3 |
| `NavigationSplitView` (`.automatic`, `.balanced`, `.prominentDetail`) | `NSSplitViewController`, `NSSplitViewItem` (sidebar, content list, inspector) | `SplitView` | `partial`: two columns and an inspector; no content-list column; the sidebar is in [14](#14-windows-and-chrome) | none | P1 |
| `NavigationStack`, `NavigationLink`, `navigationDestination` | — | `History`, `NavigationButtons`, `Row::page` | `partial`: no path or destination API | none | P2 |
| `HSplitView`, `VSplitView` | `NSSplitView` (`.thin`, `.thick`) | `SplitPanes` | `implemented` | none | P2 |
| `inspector(isPresented:)` | `NSSplitViewItem` inspector | `SplitView::inspector` | `implemented` | none | P2 |
| `ScrollView` legacy bar | `NSScroller` legacy | `ScrollView` | `measured` | bench `scroll-legacy`; [Menus, scrollers][m-menus] | — |
| `ScrollView` overlay bar | `NSScroller` overlay | `ScrollView` | `partial`: the overlay bar is not compared | probe `views` has `scroll-overlay` | P1 |
| `scrollEdgeEffectStyle` | `NSScrollPocket` | `TopGlass`, `ScrollView` edge | `partial` | [Sidebar][m-sidebar] | P2 |
| `Grid`, `GridRow` | `NSGridView` | — | `missing` | none | P2 |
| `LazyVGrid`, `LazyHGrid` | `NSCollectionView` | `CollectionView` | `implemented` | none | P3 |
| — | `NSPredicateEditor`, `NSRuleEditor` | `RuleEditor` | `implemented` | none | P3 |
| `HStack`, `VStack` | `NSStackView` | `HStack`, `VStack` | `implemented` | none | — |
| `ZStack`, `ViewThatFits`, `Layout` | — | Freya's `rect` | `partial` | none | P3 |
| `reorderable()` (macOS 27), swipe actions | drag in `NSTableView` | — | `missing` | sdk | P3 |

## 13. Presentation

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Menu` content, pop-up menu | `NSMenu` | `MenuView`, `MenuItem` | `partial`: measured in light and dark, not in Increase Contrast; item icons not measured | bench `menu` scene; [Menus][m-menus] | P1 |
| `.contextMenu` | `NSMenu.popUpContextMenu` | `ViewModifiers::context_menu` | `implemented` | same look as the menu | P1 |
| `Commands`, `CommandMenu` | main menu | `MainMenu`, menu bar menu | `implemented` | [`macos-menu-bar.md`](macos-menu-bar.md) | P1 |
| `MenuBarExtra` | `NSStatusItem` | `status_item` | `implemented` | none | P2 |
| `.popover` | `NSPopover` | `Popover` | `implemented` | none | P1 |
| `.sheet` | `beginSheet` | `Sheet` | `partial`: light and dark only; corners, shadow and edge are the compositor's | [Sheets][m-sheets] | P1 |
| `.alert` | `NSAlert` (`.warning`, `.informational`, `.critical`) | `Alert` | `partial`: the geometry comes from the view tree, not pixels; no icon, accessory view, help button or text field | [Alerts][m-alerts] | P1 |
| `.confirmationDialog` | — | — | `missing`: `Alert` is the nearest | none | P2 |
| `.fileImporter`, `.fileExporter` | `NSOpenPanel`, `NSSavePanel` | `portal::open_files`, `portal::save_file` | `partial`: the desktop's dialog, not the macOS look | none | P3 |
| `.help` | `toolTip` | `ViewModifiers::help` | `implemented` | none | P2 |
| — | `NSPrintPanel`, `NSPageLayout` | — | `missing` | none | P3 |
| `UNUserNotificationCenter` | banners | `notifications::post` | `implemented` | [`macos-notifications.md`](macos-notifications.md) | P2 |
| `.draggable`, `.dropDestination` | `NSDraggingSource`, `NSDraggingDestination` | `DragSource`, `ImageWell` drop | `partial`: drag out; drop only on the image well | none | P3 |
| `.presentationBackground`, `.presentationDetents` | — | `SheetStyle.background` | `partial`: detents are iOS | none | P3 |

## 14. Windows and chrome

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| window corners | compositor | `Bar`, `WindowCorner` | `partial`: the alpha reaches the edge 15 rows down in every style; the kit's radius was not compared | [Window corners][m-tabs] | P1 |
| traffic lights | `standardWindowButton(_:)` | `TrafficLights` | `partial`: geometry and glyphs measured ([windows](macos-windows.md), §2–3); not in the bench; hover, inactive and full-screen states not compared | [Bar and lights][m-sidebar] | P1 |
| `.windowStyle(.titleBar)` title bar | `NSWindow` titled, 32 high | `Titlebar` | `partial` | [`macos-windows.md`](macos-windows.md) §2 | P1 |
| `navigationTitle`, `navigationSubtitle` | `title`, `subtitle`, `titleVisibility` | `Titlebar` title only | `partial`: no subtitle | none | P1 |
| `.windowToolbarStyle(.unified)` items | `.unified`, `NSToolbarItem` | `IconButton`, `ToolGroup`, `NavigationButtons` | `partial`: measured in light and dark; glyphs other than the chevron and the plus not drawn; a lone `IconButton` has no platter | bench `toolbar`; [Toolbar][m-toolbar] | P1 |
| `.windowToolbarStyle(.unifiedCompact)` | `.unifiedCompact` | `Bar::UnifiedCompact`, `Titlebar` | `partial`: 24 high platters, a pair of 63 × 24 within 1 of AppKit's; the 22 between the lights and the first platter is not drawn | bench `toolbar`; [What the kit does][m-toolbar] | P2 |
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
| title bar line | `NSTitlebarSeparatorStyle` | `TitlebarStyle` hairline | `partial` | none | P2 |
| `NavigationSplitView` sidebar | `NSSplitViewItem(sidebarWithViewController:)` | `Sidebar`, `SidebarItem` | `partial`: flat, full height and within 1 of macOS 27's measures; the toggle is not in the sidebar and the glyph is a tile; no Increase Contrast | bench `sidebar`; [What the kit does][m-sidebar] | P1 |
| full screen | `toggleFullScreen` | `FullscreenBar` | `implemented` | none | P2 |
| `WindowGroup`, `Window`, `Settings` | `NSWindow`, `NSWindowController` | `Application`, `SettingsWindow` | `implemented` | none | P1 |
| `UtilityWindow` | `NSPanel` | `UtilityWindow`, `PanelBehavior` | `implemented` | none | P3 |
| — | window tabs | `WindowTabs` | `implemented` | none | P3 |
| `DocumentGroup`, `AlertScene` | `NSDocument` | — | `missing` | none | P3 |

## 15. Materials and glass

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Material` (`.ultraThin` to `.ultraThick`, `.bar`) | `NSVisualEffectView.Material`: 14 semantic materials | `Material`, `Thickness` | `partial`: composites measured for the Dock, menu and Control Center; the 14 semantic materials are not mapped | [`macos-materials.md`](macos-materials.md) §2–3 | P2 |
| `.glassEffect(_:in:)` (`.regular`, `.clear`) | `NSGlassEffectView` | `GlassContainer`, `WindowGlass` | `partial`: the kit asks the compositor for the glass | [`macos-liquid-glass.md`](macos-liquid-glass.md) | P1 |
| `GlassEffectContainer`, `glassEffectID` | `NSGlassEffectContainerView` | `GlassContainer` | `implemented` | none | P2 |
| `backgroundExtensionEffect` | `NSBackgroundExtensionView` | `SplitView::background_extension` | `implemented` | none | P3 |
| `scrollEdgeEffectStyle` | `NSScrollPocket` | `TopGlass` | `partial` | [Sidebar][m-sidebar] | P2 |
| vibrancy of `.secondary` over a material | `NSVisualEffectView` vibrant labels | `Role` colors | `partial` | none | P3 |

## 16. Symbols

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| `Image(systemName:)` | `NSImage(systemSymbolName:)` | `Image::system`, `Symbol` | `partial`: 65 symbols against the 9649 of this Mac (`name_availability.plist`) | none | P1 |
| toolbar glyphs | SF Symbols at 16 pt | `IconButton` symbols | `partial`: chevrons and plus match; `ellipsis`, `sidebar.left`, `magnifyingglass` do not | [Toolbar][m-toolbar] | P1 |
| `.symbolRenderingMode` (monochrome, hierarchical, palette, multicolor) | `NSImage.SymbolConfiguration` | `TemplateRenderingMode` (original, template) | `partial` | none | P3 |
| `.imageScale`, symbol weight | `SymbolConfiguration` | `ImageStyle` size and stroke | `partial` | none | P2 |
| `.symbolVariant`, `.symbolEffect`, variable value | `NSSymbolEffect` | — | `missing` | none | P3 |
| menu item icons | `NSMenuItem.image` | `MenuIcon` | `partial`: `NSMenuItem.image` draws nothing in a pop-up menu on this macOS | [Menus][m-menus] | P2 |

## 17. Accessibility

| SwiftUI | AppKit | Kit | Status | Measure | Pri |
|---|---|---|---|---|---|
| Increase Contrast | `accessibilityDisplayShouldIncreaseContrast` | `ContrastPalette`, `Resolved` | `partial`: controls and tables measured; menus, sheets, toolbar, sidebar and materials not | [Increase Contrast][m-hc] | P1 |
| Reduce Transparency | `accessibilityDisplayShouldReduceTransparency` | `Resolved::reduce_transparency`; menus keep glass with it | `partial`: `NSColor` ignores it; the Dock solids are measured, other materials are not | [`macos-materials.md`](macos-materials.md) §4 | P1 |
| Reduce Motion (`accessibilityReduceMotion`) | `accessibilityDisplayShouldReduceMotion` | `use_reduced_motion`, `motion_ms` | `implemented`: the kit jumps; what macOS substitutes was not measured | none | P2 |
| Differentiate Without Color | `accessibilityDisplayShouldDifferentiateWithoutColor` | `Resolved::differentiate_without_color`, read by `TrafficLights` only | `partial` | none | P3 |
| Invert Colors | `accessibilityDisplayShouldInvertColors` | `ctappearance` setting; no control reads it | `partial` | none | P3 |
| VoiceOver | `NSAccessibility` | `platform/atspi.rs` (AT-SPI) | `implemented`: no audit per control | none | P2 |
| focus ring, full keyboard access | `focusRingType` | `ViewModifiers::focus_ring` | `partial` | [`macos-focus-ring.md`](macos-focus-ring.md); bench `calendar-focused` | P2 |
| `DynamicTypeSize` (macOS 12) | — | `Resolved::text_size` scales `Text` and `SidebarItem` | `partial`: other controls do not scale; `preferredFont(forTextStyle:)` returns fixed sizes ([Text styles](#text-styles)); whether SwiftUI text on macOS 27 reacts to the setting was not probed | none | P2 |

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

1. **P1** Sizes for checkbox, radio, pop-up, segmented, stepper, slider and
   text field at mini, small and large, and a `.control_size` modifier that
   reaches the controls inside a view ([11](#11-control-sizes)).
2. **P1** Probe and compare the indeterminate `Spinner` and a SwiftUI `Gauge`
   ([6](#6-gauges-progress-and-level-indicators)).
3. **P2** `Mini` and `ExtraLarge` in `ControlSize` for `Button`:
   16 and 36 high ([1](#1-buttons-and-button-styles)).
4. **P2** Compare the buttons that are only `implemented`: `destructive`,
   `borderless`, `glass`, `glass_prominent`, the toggle button and the menu
   styles; add `.plain`.
5. **P2** Segmented control: symbols, the four segment styles, momentary
   tracking, and `segmented-any` in the bench ([3](#3-pickers)).
6. **P2** Probe SwiftUI `Picker` with `.inline`, `.palette` and `.tabs`.
7. **P3** The other bezels: `smallSquare`, `pushDisclosure`, `accessoryBar`,
   `accessoryBarAction`, `flexiblePush`, `toolbar`, `badge`.
8. **P3** Ring at mini and small, rating image, tick positions, vertical
   slider.

### B. Text, dates, color and panels

1. **P1** Text styles: add `title2` (17 / 22), `footnote`, `caption1` and
   `caption2` (10 / 13); compare weights and lines with `preferredFont`
   and SwiftUI `Font` ([Text styles](#text-styles)).
2. **P1** Probe and compare `TextEditor` and `NSTextView`: insets, caret,
   selection, placeholder, find bar ([7](#7-text)).
3. **P1** Text field at mini (19) and small (22); `.plain`, `.squareBorder`
   and the new `.bordered`; `TextField(axis: .vertical)`.
4. **P2** Date picker: `.compact`, time parts with seconds, `.field`
   without stepper, `MultiDatePicker`, range mode ([8](#8-date-and-time)).
5. **P2** Color panel: the other seven modes, alpha, eyedropper, `.minimal`
   and `.expanded` wells; probe the SwiftUI `ColorPicker` popover
   ([9](#9-color-and-font-panels)).
6. **P2** Search: scopes, suggestions and the recents menu.
7. **P2** Rich text: underline, strike, color, paragraph style, lists.
8. **P3** Font panel, rulers, text lists and tables; image scaling modes;
   probe `NSPathControl` ([10](#10-image-and-path-control)).

### C. Lists, tables, forms, containers and presentation

1. **P1** A `List` view of free rows with selection, separators and the
   `.inset`, `.plain` and `.sidebar` looks; probe SwiftUI `List`, `Form`,
   `GroupBox` and `DisclosureGroup`, which no probe has
   ([12](#12-containers-and-navigation)).
2. **P1** `Table`: cells with icons and controls, row size styles, the other
   table styles, the overlay scroller.
3. **P1** `Form` with `.columns`, compared with a probe of a macOS 27
   Settings window.
4. **P1** Alert: icon, accessory view, help button, text field; popover probe;
   sheet and menu in Increase Contrast ([13](#13-presentation)).
5. **P1** `TabView`: classic top tabs and segmented-on-top; the toolbar tabs
   as a bench entry.
6. **P2** A three-column `NavigationSplitView`; a path and destination API for
   `NavigationStack`; a general `Label`; `Grid`; confirmation dialog;
   tooltip and `NSSplitView` divider probes.
7. **P3** Reordering, swipe actions, collection layouts, drop targets.

### D. Window and toolbar

1. **P1** Unified toolbar: prominent item, glyphs other than the chevron and
   the plus, search item, offset of a lone chevron, group widths with labels,
   Increase Contrast capture ([14](#14-windows-and-chrome)).
2. **P1** Sidebar: toggle position, bare accent glyph instead of the tile,
   scrolling under the bar, Increase Contrast capture ([Sidebar][m-sidebar]).
3. **P1** Traffic lights, title bar, title with subtitle and window corner
   as bench entries.
4. **P1** Materials and Reduce Transparency: map the 14 semantic materials;
   Increase Contrast for menu, sheet and toolbar ([15](#15-materials-and-glass),
   [17](#17-accessibility)).
5. **P1** Symbols: more than 65, with weight and scale ([16](#16-symbols)).
6. **P2** `unifiedCompact` and `expanded` platters (24 and 28 high); toolbar
   labels, spacer, tracking separator, title bar accessory.
7. **P3** The [macOS 27 APIs](#18-new-in-macos-27), toolbar customization,
   document scenes.

### Probes to write first

A row stays `implemented` until a probe captures the AppKit or SwiftUI side.
No probe of `~/src/probes` renders these:

| Probe | Items | Area |
|---|---|---|
| SwiftUI `Form`, `List`, `GroupBox`, `DisclosureGroup`, `LabeledContent` in a macOS 27 window | row heights, section cards, separators, selection | C |
| SwiftUI `Gauge` in each style, `ProgressView` indeterminate, `Spinner` | sizes and colors | A |
| `NSTextView` in `NSScrollView` and SwiftUI `TextEditor` | insets, caret, selection, placeholder, find bar | B |
| `NSColorPanel` and the SwiftUI `ColorPicker` popover | modes, sizes, alpha | B |
| `NSPopover` and SwiftUI `.popover`, `.confirmationDialog` | size, arrow, corner | C |
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

[m-sizes]: macos-control-metrics.md#sizes-width--height-points
[m-open]: macos-control-metrics.md#open-differences
[m-text]: macos-control-metrics.md#text-fields
[m-hc]: macos-control-metrics.md#reduce-transparency-and-increase-contrast
[m-rings]: macos-control-metrics.md#rings-level-indicators-and-boxes
[m-tables]: macos-control-metrics.md#tables-and-source-lists
[m-alerts]: macos-control-metrics.md#alerts
[m-sheets]: macos-control-metrics.md#sheets
[m-menus]: macos-control-metrics.md#menus-scrollers-and-inactive-selection
[m-sidebar]: macos-control-metrics.md#sidebar-and-window-macos-27
[m-toolbar]: macos-control-metrics.md#toolbar-items-and-navigation-buttons
[m-tabs]: macos-control-metrics.md#toolbar-tabs-and-window-corners
