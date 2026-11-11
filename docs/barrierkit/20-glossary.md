# 20. Glossary

**Goal:** find the BarrierKit name of a concept you know from SwiftUI or
AppKit, and name new items the same way.
**Scope:** every public item of `bk`, `ctdesign` and
`ctappearance`, with its SwiftUI and AppKit equivalent and the chapter that
documents it. Methods are in the chapters, not here.
**Reader:** anyone who writes an app with the kit, adds an item to it, or is an
AI agent ([`agents/prompt.md`](../../agents/prompt.md)).
**Input:** a name. **Output:** its counterpart and the chapter to read.

## Steps

1. Search this page for the SwiftUI or AppKit name (for example `Toggle`,
   `NSPopUpButton`).
2. Read the BarrierKit name in the first column and follow the chapter link.
3. To add or rename a public item, follow [How names are chosen](#how-names-are-chosen)
   and add its row here in the same commit (`AGENTS.md`, «Nombres y glosario»).

## How names are chosen

1. **SwiftUI first.** When SwiftUI has the concept, use its name: `Text`,
   `VStack`, `Button`, `TabView`.
2. **Then AppKit.** When only AppKit has it, use the AppKit name without the
   `NS` prefix: `ColorWell`, `PathControl`, `TokenField`.
3. **Then plain English.** Otherwise, one or two common words that say what
   the item is, not how it is built: `Palette`, `Sidebar`.
4. **Rust case.** Types `UpperCamelCase`; functions and methods `snake_case`
   (`foregroundStyle` → `foreground_style`); constants `SCREAMING_SNAKE_CASE`
   (`.title` → `TITLE`).
5. **Fixed shapes.** A constructor is `Type::new(required)`. Options are
   chained methods named after what they set (`.font`, `.size`). Events are
   `on_<verb>` (`on_press`, `on_change`). Booleans read as adjectives
   (`disabled`, `focused`). Hooks are `use_<thing>`.
6. **Taken names.** When Freya already uses a name (`App`, `label`, `rect`),
   take the next clear one (`Application`) and record it here.

## Renamed items

| Before | Now | Date |
|---|---|---|
| `vow_design::TEXT` | `ctdesign::BODY` | 2026-11-07 |
| `vow_design::DISPLAY` | `ctdesign::LARGE_TITLE` | 2026-11-07 |
| `vow_design::TITLE_1` | `ctdesign::TITLE` | 2026-11-07 |
| crate `barrier-kit` (`barrier_kit`) | crate `bk`, BarrierKit | 2026-11-07 |
| crate `vow-design` (`vow_design`), then `ct-design` (`ct_design`) | crate `ctdesign`, CTDesign | 2026-11-07 |
| `ctappearance::write_dominant` | `ctappearance::write_wallpaper_state` | 2026-11-08 |
| crate `vow-appearance` (`vow_appearance`), then `ct-appearance` (`ct_appearance`) | crate `ctappearance`, CTAppearance | 2026-11-07 |
| `styles::Theme::slider` | `styles::Theme::style` (generic: `\|s: SliderStyle\|`) | 2026-11-07 |
| `modifiers::StyleModifiers::slider_style` | `modifiers::StyleModifiers::control_style` (generic) | 2026-11-07 |
| enum `controls::ButtonStyle` | struct `styles::ButtonStyle` | 2026-11-07 |
| `controls::Button::style(ButtonStyle)` | `controls::Button::style(\|s\| ..)`, the closure of every control | 2026-11-07 |
| `ButtonStyle::Secondary` | `ButtonStyle::tengen()`, the default (SwiftUI `.bordered`) | 2026-11-07 |
| `ButtonStyle::Primary` | `ButtonStyle::bordered_prominent` | 2026-11-07 |
| `ButtonStyle::Glass` | `ButtonStyle::glass` | 2026-11-07 |
| `ButtonStyle::GlassProminent` | `ButtonStyle::glass_prominent` | 2026-11-07 |
| `ButtonStyle::Borderless` | `ButtonStyle::borderless` | 2026-11-07 |
| enum `level::LevelStyle` | struct `styles::LevelSliderStyle` | 2026-11-07 |
| `level::LevelSlider::style(LevelStyle)` | `level::LevelSlider::style(\|s\| ..)` | 2026-11-07 |
| `LevelStyle::Regular` | `LevelSliderStyle::tengen()`, the default | 2026-11-07 |
| `WallpaperSelection::pinned`, `pinned_for`, `set_pinned` (an image index) | `appearance`, `appearance_for`, `set_appearance` (`WallpaperAppearance`) | 2026-11-11 |
| `LevelStyle::Fat(Symbol)` | `LevelSliderStyle::fat(symbol)` | 2026-11-07 |
| `rich_text::TextStyle` | `rich_text::RunStyle`; `styles::TextStyle` is the style of `Text` | 2026-11-07 |
| `chrome::ChromeStyle` | `styles::WindowChromeStyle`, with the look of the rail | 2026-11-07 |
| `chrome::WindowChrome::new(style, child)` | `chrome::WindowChrome::new(child).style(\|s\| ..)` | 2026-11-07 |
| `shell::SpaceInfo`, `Toplevels::space` | `Toplevels::focus_bar` (`FocusBar`); the `space` event keeps only `focus_bar` (Infinity v7) | 2026-11-08 |
| `platform::Bar::Compact`, `WindowCorner::compact`, Infinity `bar` entry `compact` | `Bar::UnifiedCompact`, `WindowCorner::unified_compact`, `unified_compact` | 2026-11-08 |
| `ctdesign::LOCK_TOP_MARGIN` (72 px) | `ctdesign::LOCK_TOP_PERCENT`, a share of the screen height | 2026-11-11 |
| `ctdesign::LOCK_FIELD_FILL_OPAQUE` (the field with «Reduce transparency») | `ctdesign::LOCK_AVATAR_FILL`; the field is always `LOCK_FIELD_FILL` | 2026-11-11 |
| The 22 views, modifiers and names listed in [ADR 0063](../decisions/0063-views-as-types-and-swift-names.md#decision) (`divider()`, `empty_state`, `use_tooltip`, `PreferencesWindow`, `AppMenu`, `ProgressBar`…) | `Divider`, `ContentUnavailableView`, `.help`, `SettingsWindow`, `MainMenu`, `ProgressView`… | 2026-11-07 |
| `shell::pixmap_image(pixmap, size)` | `symbol::Image::rgba(width, height, rgba).size(size)` | 2026-11-11 |

## How to read the tables

- **BarrierKit:** the path inside `bk` (for example
  `views::form::Form`), or the full path for the other crates.
- **SwiftUI / AppKit:** the Apple equivalent; `≈` marks a partial match, `—`
  means there is none.
- **Chapter:** where the item is documented.

## Entry points

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `prelude` | module | `import SwiftUI` | `import AppKit` | Re-exports the entry points, controls, Freya's prelude and the tokens | [2](02-first-app.md#the-prelude) |
| `elements::Text` | struct | `Text` | `NSTextField(labelWithString:)` | Text with `BODY` and the label color by default; wraps | [2](02-first-app.md#text-and-stacks) |
| `elements::Text::line_limit` | method | `.lineLimit(_:)` | `NSTextField.maximumNumberOfLines` | Shows at most `n` lines | [2](02-first-app.md#text-and-stacks) |
| `elements::VStack` | struct | `VStack` | `NSStackView` (vertical) | Column of children, centered, `SPACE_3` apart | [2](02-first-app.md#text-and-stacks) |
| `elements::HStack` | struct | `HStack` | `NSStackView` (horizontal) | Row of children, centered, `SPACE_3` apart | [2](02-first-app.md#text-and-stacks) |

## Style and palette

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `style::Palette` | struct | — | ≈ `NSColor` (semantic colors) | Semantic colors of the current context: labels, backgrounds, accent, separators | [4](04-appearance-and-tokens.md#the-palette) |
| `style::text` | fn | ≈ `Text` | ≈ `NSTextField(labelWithString:)` | One-line label with a type token and a color | [4](04-appearance-and-tokens.md#text-shapes-and-colors) |
| `style::continuous_corner` | fn | `RoundedCornerStyle.continuous` | — | Corner radius with Apple's continuous curvature | [4](04-appearance-and-tokens.md#text-shapes-and-colors) |
| `style::panel_edges` | fn | — | — | Card framed with a panel window's inner and outer lines (glass in dark) | [4](04-appearance-and-tokens.md#text-shapes-and-colors) |
| `theme::color` | fn | — | — | Converts a Tengen `Rgba` to Freya's `Color` | [4](04-appearance-and-tokens.md#text-shapes-and-colors) |
| `theme::resolved` | fn | ≈ `EnvironmentValues` | ≈ `NSApplication.effectiveAppearance` | Resolved appearance at this point of the tree | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::use_appearance` | fn | ≈ `.environment(_:_:)` | ≈ `NSAppearanceCustomization.appearance` | Hook: changes the appearance for a component and its children | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::use_shell_theme` | fn | — | — | Hook: Freya component theme from the resolved shell colors | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::for_scheme` | fn | — | — | Freya theme for the default light or dark appearance | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::SurfaceKind` | enum | — | — | `Toplevel` or `LayerShell`: which kind of surface a color is for | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::surface_background` | fn | — | — | Background color of a window or a shell surface | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::menu_palette` | fn | — | — | Menu colors over a given surface, lifted when tinted | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::popover_palette` | fn | — | — | Menu palette with pop-up menu metrics | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::apply_popover_metrics` | fn | — | — | Replaces a menu palette's metrics with `MENU_POPOVER` | [4](04-appearance-and-tokens.md#theme-functions) |

## Styles and themes

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `styles` | module | — | — | Value types of the styling system, `Theme`, `StyleScope` and the control styles | [Styles](../../agents/styles.md) |
| `styles::ShapeStyle` | enum | `ShapeStyle` | ≈ `NSColor` | Paint of a part: a role, the accent, a fixed color, a gradient, the backdrop, a material or nothing | [Styles](../../agents/styles.md#value-types) |
| `styles::Role` | enum | ≈ semantic `Color` (`.primary`, `.secondary`) | ≈ `NSColor.labelColor`… | One color of the `Palette` (all but `accent`), resolved late | [Styles](../../agents/styles.md#value-types) |
| `styles::Gradient` | struct | `LinearGradient` | `NSGradient` | Linear gradient: an angle and stops of `ShapeStyle` | [Styles](../../agents/styles.md#value-types) |
| `styles::Thickness` | enum | `Material` (`.ultraThin`…`.ultraThick`) | `NSVisualEffectView.Material` | How translucent and blurred a material is; values in `MATERIAL` | [Styles](../../agents/styles.md#value-types) |
| `styles::StateValue` | struct | — | — | A value with optional hovered, pressed, focused and disabled values | [Styles](../../agents/styles.md#value-types) |
| `styles::ControlState` | struct | ≈ `ButtonStyle.Configuration` (`isPressed`) | ≈ `NSControl.isHighlighted`, `isEnabled` | Hovered, pressed, focused and disabled flags of a control | [Styles](../../agents/styles.md#value-types) |
| `styles::Shape` | enum | `Rectangle`, `RoundedRectangle`, `UnevenRoundedRectangle`, `Capsule`, `Circle` | ≈ `NSBezierPath` | Outline of a part | [Styles](../../agents/styles.md#value-types) |
| `styles::CornerStyle` | enum | `RoundedCornerStyle` | — | `Continuous` or `Circular` corners | [Styles](../../agents/styles.md#value-types) |
| `styles::ShadowStyle` | struct | `ShadowStyle` (`.drop`, `.inner`) | `NSShadow` | One shadow: color, x, y, blur, spread, inner | [Styles](../../agents/styles.md#value-types) |
| `styles::Animation` | enum | `Animation` | ≈ `NSAnimationContext` | How a change moves: `None`, `Ease`, `Spring`, `Custom` | [Styles](../../agents/styles.md#value-types) |
| `styles::ColorEffect` | type | — | ≈ `NSColor.withSystemEffect(_:)` | `fn(Rgba, ControlState) -> Rgba` applied to every paint of a control | [Styles](../../agents/styles.md#value-types) |
| `styles::system_effect` | fn | — | ≈ `NSColor.SystemEffect` | Tengen's color effect: returns the color unchanged | [Styles](../../agents/styles.md#value-types) |
| `styles::Axis` | enum | `Axis` | `NSUserInterfaceLayoutOrientation` | `Horizontal` or `Vertical` | [Styles](../../agents/styles.md#value-types) |
| `styles::Edge` | enum | `Edge` | `NSRectEdge` | `Top`, `Bottom`, `Leading`, `Trailing` | [Styles](../../agents/styles.md#value-types) |
| `styles::LayoutDirection` | enum | `LayoutDirection` | `NSUserInterfaceLayoutDirection` | `LeftToRight` or `RightToLeft` | [Styles](../../agents/styles.md#value-types) |
| `styles::ControlStyle` | trait | ≈ the style protocols (`ButtonStyle`, `ToggleStyle`) | ≈ `NSCell` | Implemented by every control style: `tengen()` and the accessibility rules of its control | [Styles](../../agents/styles.md#which-controls-take-a-style) |
| `styles::SliderStyle` | struct | ≈ `ButtonStyle` | ≈ `NSSliderCell` | Every visual property of `Slider`; `tengen()` builds the default from the tokens | [Styles](../../agents/styles.md#the-fields-of-a-style) |
| `styles::ButtonStyle` | struct | `ButtonStyle`, `PrimitiveButtonStyle` | ≈ `NSButtonCell`, `NSButton.BezelStyle` | Every visual property of `Button`; `tengen()` is `.bordered` | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::bordered_prominent` | method | `PrimitiveButtonStyle.borderedProminent` | ≈ `NSButton.keyEquivalent` `"\r"` | Preset: accent fill, `on_accent` label | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::destructive` | method | `Button(role: .destructive)` | `NSButton.hasDestructiveAction` | Preset: red label on a faint red fill | [Styles](../../crates/bk/src/styles/button.rs) |
| `views::feedback::Alert::destructive` | method | `Button(role: .destructive)` in `.alert` | `NSAlert.hasDestructiveAction` | The primary button is destructive | [11](11-views.md#alert) |
| `styles::ButtonStyle::glass` | method | `PrimitiveButtonStyle.glass` | ≈ `NSButton.BezelStyle.glass` | Preset: raised capsule with a hairline rim | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::glass_prominent` | method | `PrimitiveButtonStyle.glassProminent` | — | Preset: accent capsule with a hairline rim | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::borderless` | method | `PrimitiveButtonStyle.borderless` | `NSButton.isBordered` `false` | Preset: no background until hovered | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::SwitchStyle` | struct | `SwitchToggleStyle` | ≈ `NSSwitch` | Every visual property of `Switch`, with the label position | [Styles](../../crates/bk/src/styles/switch.rs) |
| `styles::CheckboxStyle` | struct | `CheckboxToggleStyle` | ≈ `NSButtonCell` | Every visual property of `Checkbox`, with the label position | [Styles](../../crates/bk/src/styles/checkbox.rs) |
| `styles::SegmentedControlStyle` | struct | `SegmentedPickerStyle` | ≈ `NSSegmentedCell` | Every visual property of `SegmentedControl`, with its axis | [Styles](../../crates/bk/src/styles/segmented_control.rs) |
| `styles::ToolGroupStyle` | struct | — | — | Every visual property of `ToolGroup`, with its axis | [Styles](../../crates/bk/src/styles/tool_group.rs) |
| `styles::TextFieldStyle` | struct | `TextFieldStyle` | ≈ `NSTextFieldCell` | Every visual property of `TextField` | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `styles::SearchFieldStyle` | struct | — | ≈ `NSSearchFieldCell` | Every visual property of `SearchField` | [Styles](../../crates/bk/src/styles/search_field.rs) |
| `styles::SecureFieldStyle` | struct | ≈ `TextFieldStyle` | ≈ `NSSecureTextFieldCell` | Every visual property of `SecureField` | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `styles::LevelSliderStyle` | struct | — | ≈ `NSSliderCell` | Every visual property of `LevelSlider`, with the layout direction | [Styles](../../crates/bk/src/styles/level_slider.rs) |
| `styles::LevelSliderStyle::fat` | method | — | — | Preset: the 22 tall Control Center bar without a knob, with a symbol | [Styles](../../crates/bk/src/styles/level_slider.rs) |
| `styles::RadioGroupStyle` | struct | `RadioGroupPickerStyle` | ≈ `NSButtonCell` (radio) | Every visual property of `RadioGroup`, with the axis and the label position | [Styles](../../crates/bk/src/styles/radio_group.rs) |
| `styles::ColorWellStyle` | struct | ≈ `PalettePickerStyle` | ≈ `NSColorWell.Style` | Every visual property of `ColorWell`, with the axis and wrapping of the swatches | [Styles](../../crates/bk/src/styles/color_well.rs) |
| `styles::ChoiceCardsStyle` | struct | — | — | Every visual property of `ChoiceCards`, with the axis and wrapping of the cards | [Styles](../../crates/bk/src/styles/choice_cards.rs) |
| `styles::IconButtonStyle` | struct | ≈ `BorderlessButtonStyle` | ≈ `NSButtonCell` | Every visual property of `IconButton` and `ToolbarOverflow` | [Styles](../../crates/bk/src/styles/icon_button.rs) |
| `styles::StepperStyle` | struct | — | ≈ `NSStepperCell` | Every visual property of `Stepper` and of the arrows of `DatePicker`, with the value position | [Styles](../../crates/bk/src/styles/stepper.rs) |
| `styles::ToggleGroupStyle` | struct | — | ≈ `NSSegmentedCell` | Every visual property of `ToggleGroup`, with its axis | [Styles](../../crates/bk/src/styles/toggle_group.rs) |
| `styles::ToolbarSearchStyle` | struct | — | ≈ `NSSearchToolbarItem` | Width of `ToolbarSearch`; its field and button take their own styles | [Styles](../../crates/bk/src/styles/toolbar_search.rs) |
| `styles::MenuButtonStyle` | struct | `ButtonMenuStyle` | ≈ `NSPopUpButtonCell` (pull-down) | Every visual property of `MenuButton` and `ShareLink` | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `styles::ComboButtonStyle` | struct | ≈ `MenuStyle` | `NSComboButton.Style` | Every visual property of `ComboButton` | [Styles](../../crates/bk/src/styles/combo_button.rs) |
| `styles::HelpLinkStyle` | struct | — | ≈ `NSButton.BezelStyle.helpButton` | Every visual property of `HelpLink` | [Styles](../../crates/bk/src/styles/help_link.rs) |
| `styles::ControlGroupStyle` | struct | `ControlGroupStyle` | — | Every visual property of `ControlGroup`: background and separators | [Styles](../../crates/bk/src/styles/control_group.rs) |
| `styles::PopUpButtonStyle` | struct | `MenuPickerStyle` | ≈ `NSPopUpButtonCell` | Every visual property of `PopUpButton`: label, indicator and chevron | [Styles](../../crates/bk/src/styles/popup_button.rs) |
| `styles::ComboBoxStyle` | struct | — | ≈ `NSComboBoxCell` | The arrow button of `ComboBox`; its field takes `TextFieldStyle` | [Styles](../../crates/bk/src/styles/combo_box.rs) |
| `styles::TokenFieldStyle` | struct | — | ≈ `NSTokenFieldCell` | Every visual property of `TokenField` and of the tokens of `SearchField` | [Styles](../../crates/bk/src/styles/token_field.rs) |
| `styles::DatePickerStyle` | struct | `DatePickerStyle` | ≈ `NSDatePickerCell` | Every visual property of `DatePicker` and its calendar; its arrows take `StepperStyle` | [Styles](../../crates/bk/src/styles/date_picker.rs) |
| `styles::PathControlStyle` | struct | — | ≈ `NSPathControlItem` | Every visual property of `PathControl` | [Styles](../../crates/bk/src/styles/path_control.rs) |
| `styles::ImageWellStyle` | struct | — | ≈ `NSImageCell` | Every visual property of `ImageWell`, with its drop border | [Styles](../../crates/bk/src/styles/image_well.rs) |
| `styles::ColorPickerStyle` | struct | — | ≈ `NSColorWell.Style` | Every visual property of `ColorPicker` and its panel but the color | [Styles](../../crates/bk/src/styles/color_picker.rs) |
| `styles::ProgressViewStyle` | struct | `LinearProgressViewStyle` | ≈ `NSProgressIndicator.Style` | Every visual property of `ProgressView`, with its sweep | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `styles::ProgressRingStyle` | struct | `CircularProgressViewStyle` | ≈ `NSProgressIndicator.Style` | Every visual property of `ProgressRing` | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `styles::SpinnerStyle` | struct | ≈ `CircularProgressViewStyle` | ≈ `NSProgressIndicator.Style` | Every visual property of `Spinner`, with its turn | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `styles::CapacityIndicatorStyle` | struct | ≈ `GaugeStyle` | ≈ `NSLevelIndicatorCell` | Every visual property of `CapacityIndicator` | [Styles](../../crates/bk/src/styles/capacity_indicator.rs) |
| `styles::RelevancyIndicatorStyle` | struct | — | ≈ `NSLevelIndicatorCell` | Every visual property of `RelevancyIndicator` | [Styles](../../crates/bk/src/styles/relevancy_indicator.rs) |
| `styles::RatingIndicatorStyle` | struct | — | ≈ `NSLevelIndicatorCell` | Every visual property of `RatingIndicator` | [Styles](../../crates/bk/src/styles/rating_indicator.rs) |
| `styles::GaugeStyle` | struct | `GaugeStyle` | — | Every visual property of `Gauge` | [Styles](../../crates/bk/src/styles/gauge.rs) |
| `styles::MediaButtonStyle` | struct | ≈ `ButtonStyle` | — | Every visual property of `MediaButton` | [Styles](../../crates/bk/src/styles/media_button.rs) |
| `styles::RoundButtonStyle` | struct | ≈ `ButtonStyle` | — | Every visual property of `RoundButton` | [Styles](../../crates/bk/src/styles/round_button.rs) |
| `styles::LinkStyle` | struct | `LinkButtonStyle` | — | Every visual property of `Link` | [Styles](../../crates/bk/src/styles/link.rs) |
| `styles::TextStyle` | struct | ≈ `.font(_:)` and `.foregroundStyle(_:)` in the environment | ≈ `NSTextField.font`, `textColor` | Default font and paint of every `Text` | [Styles](../../crates/bk/src/styles/text.rs) |
| `styles::BadgeStyle` | struct | ≈ `.badgeProminence(_:)` | — | Every visual property of `Badge` | [Styles](../../crates/bk/src/styles/badge.rs) |
| `styles::DividerStyle` | struct | — | ≈ `NSBox` (separator) | Thickness and paint of `Divider` | [Styles](../../crates/bk/src/styles/divider.rs) |
| `styles::AvatarStyle` | struct | — | — | Every visual property of `Avatar` | [Styles](../../crates/bk/src/styles/avatar.rs) |
| `styles::ImageStyle` | struct | ≈ `.imageScale(_:)`, `.foregroundStyle(_:)` | — | Default size and paint of every `Image` | [Styles](../../crates/bk/src/styles/image.rs) |
| `styles::MaterialStyle` | struct | `Material` | ≈ `NSVisualEffectView.Material` | Tint, blur and rim of `Material` | [Styles](../../crates/bk/src/styles/material.rs) |
| `styles::TopGlassStyle` | struct | ≈ `.scrollEdgeEffectStyle(_:for:)` | — | Tint and glass metrics of `TopGlass` | [Styles](../../crates/bk/src/styles/top_glass.rs) |
| `styles::GlassContainerStyle` | struct | ≈ `.glassEffect(_:in:)` | ≈ `NSGlassEffectView` | Tint, solid color, rim, blur and morph of `GlassContainer` | [Styles](../../crates/bk/src/styles/glass_container.rs) |
| `styles::BackdropStyle` | struct | — | — | Angles, opacity and fade of the gradient layers of `Backdrop` | [Styles](../../crates/bk/src/styles/backdrop.rs) |
| `styles::ScrollViewStyle` | struct | ≈ `.scrollIndicators(_:axes:)` | ≈ `NSScroller.KnobStyle` | Every visual property of the bar, edge fade and top line of `ScrollView` | [Styles](../../crates/bk/src/styles/scroll_view.rs) |
| `styles::TextEditorStyle` | struct | `TextEditorStyle` | ≈ `NSTextView` (`backgroundColor`, `textColor`) | Every visual property of `TextEditor` | [Styles](../../crates/bk/src/styles/text_editor.rs) |
| `styles::RuleEditorStyle` | struct | — | ≈ `NSRuleEditor` | Gaps, indent and header text of `RuleEditor` | [Styles](../../crates/bk/src/styles/rule_editor.rs) |
| `styles::PrintPreviewStyle` | struct | — | — | Gaps, page background, shadows and page numbers of `PrintPreview` | [Styles](../../crates/bk/src/styles/print_preview.rs) |
| `styles::ChartStyle` | struct | ≈ `.chartForegroundStyleScale(_:)`, `.chartPlotStyle(content:)` | — | Every visual property of `Chart`, `SectorChart`, `HeatMap`, `StreamChart` and `Chart3D` | [Styles](../../crates/bk/src/styles/chart.rs) |
| `styles::TitlebarStyle` | struct | ≈ `WindowToolbarStyle` | ≈ `NSWindow.ToolbarStyle`, `NSTitlebarSeparatorStyle` | Every visual property of `Titlebar`: heights, paddings, background, title and hairline | [Styles](../../crates/bk/src/styles/titlebar.rs) |
| `styles::FullscreenBarStyle` | struct | — | — | Tint, line and metrics of the full-screen bars; theme and container only | [Styles](../../crates/bk/src/styles/fullscreen_bar.rs) |
| `styles::TrafficLightsStyle` | struct | — | ≈ `NSWindow.standardWindowButton(_:)` | Size, gap, shape and the fill, rim and glyph of each traffic light per state | [Styles](../../crates/bk/src/styles/traffic_lights.rs) |
| `styles::WindowChromeStyle` | struct | ≈ `WindowStyle` | — | Layout, reveal and resting edge of `WindowChrome`, and every visual property of its rail and pill | [Styles](../../crates/bk/src/styles/window_chrome.rs) |
| `styles::SidebarStyle` | struct | ≈ `SidebarListStyle` | ≈ `NSTableView.Style.sourceList` | Paddings, lights position, shape, background and borders of `Sidebar`; not `ctappearance::SidebarStyle` | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `styles::SidebarItemStyle` | struct | ≈ `.listItemTint(_:)` | ≈ `NSTableCellView` | Every visual property of `SidebarItem`, selected and not | [Styles](../../crates/bk/src/styles/sidebar_item.rs) |
| `styles::SidebarSectionStyle` | struct | — | — | Every visual property of `SidebarSection` and its fold | [Styles](../../crates/bk/src/styles/sidebar_section.rs) |
| `styles::SidebarAccountStyle` | struct | — | — | Every visual property of `SidebarAccount` but its avatar | [Styles](../../crates/bk/src/styles/sidebar_account.rs) |
| `styles::SymbolTileStyle` | struct | — | — | Size, shape, paints and border of `SymbolTile` | [Styles](../../crates/bk/src/styles/symbol_tile.rs) |
| `styles::SplitViewStyle` | struct | ≈ `NavigationSplitViewStyle` | ≈ `NSSplitView.DividerStyle` | Inset, grip, background, scrim, inspector line and animations of `SplitView` | [Styles](../../crates/bk/src/styles/split_view.rs) |
| `styles::SplitPanesStyle` | struct | — | ≈ `NSSplitView.DividerStyle` | Width and paint of the dividers of `SplitPanes` | [Styles](../../crates/bk/src/styles/split_panes.rs) |
| `styles::FloatingPaneStyle` | struct | — | ≈ `NSPanel` | Every visual property of `FloatingPane` | [Styles](../../crates/bk/src/styles/floating_pane.rs) |
| `styles::FormStyle` | struct | `FormStyle` (`.grouped`) | — | Every visual property of `Form`, its sections and rows | [Styles](../../crates/bk/src/styles/form.rs) |
| `styles::GroupBoxStyle` | struct | `GroupBoxStyle` | ≈ `NSBox` | Every visual property of `GroupBox` | [Styles](../../crates/bk/src/styles/group_box.rs) |
| `styles::DisclosureGroupStyle` | struct | `DisclosureGroupStyle` | ≈ `NSButton.BezelStyle.disclosure` | Every visual property of `DisclosureGroup` | [Styles](../../crates/bk/src/styles/disclosure_group.rs) |
| `styles::TableStyle` | struct | `TableStyle` | ≈ `NSTableView.Style`, `NSTableHeaderCell` | Every visual property of `Table`, its header cells and rows | [Styles](../../crates/bk/src/styles/table.rs) |
| `styles::CollectionViewStyle` | struct | — | ≈ `NSCollectionViewItem` | Every visual property of `CollectionView` and its tiles | [Styles](../../crates/bk/src/styles/collection_view.rs) |
| `styles::OutlineViewStyle` | struct | ≈ `ListStyle` | ≈ `NSOutlineView`, `NSBrowser` | Every visual property of `OutlineView` and `Browser` | [Styles](../../crates/bk/src/styles/outline_view.rs) |
| `styles::TabViewStyle` | struct | `TabViewStyle` | ≈ `NSTabView` | Every visual property of `TabView` and its tabs | [Styles](../../crates/bk/src/styles/tab_view.rs) |
| `styles::WindowTabsStyle` | struct | — | ≈ `NSWindowTabGroup` | Every visual property of `WindowTabs` and its tabs | [Styles](../../crates/bk/src/styles/window_tabs.rs) |
| `styles::TextViewStyle` | struct | ≈ `TextEditorStyle` | ≈ `NSTextView` (`textColor`, `insertionPointColor`) | Paint of the text and caret of `TextView` | [Styles](../../crates/bk/src/styles/text_view.rs) |
| `styles::SheetStyle` | struct | ≈ `presentationBackground(_:)` | ≈ `NSWindow` sheet | Every visual property of `Sheet` and of the sheet under an `Alert` | [Styles](../../crates/bk/src/styles/sheet.rs) |
| `styles::AlertStyle` | struct | — | ≈ `NSAlert` | Every visual property of `Alert` but its sheet, buttons and checkbox | [Styles](../../crates/bk/src/styles/alert.rs) |
| `styles::ContentUnavailableViewStyle` | struct | — | — | Every visual property of `ContentUnavailableView` | [Styles](../../crates/bk/src/styles/content_unavailable_view.rs) |
| `styles::SettingsWindowStyle` | struct | — | — | Gaps of `SettingsWindow` and its search popover | [Styles](../../crates/bk/src/styles/settings_window.rs) |
| `styles::AboutViewStyle` | struct | — | — | Every visual property of `AboutView` | [Styles](../../crates/bk/src/styles/about_view.rs) |
| `styles::MenuStyle` | type | ≈ `MenuStyle` | ≈ `NSMenu` drawing | `ctdesign::MenuPalette`: the colors and metrics of menus, pop-up menus, popovers and tooltips | [Styles](../../agents/styles.md#menustyle) |
| `styles::VideoViewStyle` | struct | — | — | The clip shape of `VideoView` (feature `video`) | [Styles](../../crates/bk/src/styles/video_view.rs) |
| `styles::Theme` | struct | ≈ styles set in the environment | ≈ `NSAppearance` | Style overrides for the whole app; `Theme::tengen()` changes nothing | [Styles](../../agents/styles.md#levels) |
| `styles::Theme::style` | method | ≈ `.buttonStyle(_:)` on the scene | — | Adds an override of the control whose style the closure takes | [Styles](../../agents/styles.md#levels) |
| `styles::StyleScope` | struct | ≈ a view after `.buttonStyle(_:)` | — | Component returned by `.control_style`: its overrides reach the controls inside | [Styles](../../agents/styles.md#levels) |
| `modifiers::StyleModifiers` | trait | `View` modifiers | — | Style modifiers for every view | [Styles](../../agents/styles.md#levels) |
| `modifiers::StyleModifiers::control_style` | method | ≈ `.buttonStyle(_:)`, `.toggleStyle(_:)` | — | Changes the style of every control of that style inside the view | [Styles](../../agents/styles.md#levels) |
| `modifiers::ViewModifiers::foreground_style` | method | `.foregroundStyle(_:)` | ≈ `NSTextField.textColor` | Text color of `Text`, `label` or `paragraph` from a `ShapeStyle` | [Styles](../../agents/styles.md#paint-on-views) |
| `modifiers::ViewModifiers::background_style` | method | `.background(_:ignoresSafeAreaEdges:)`; Freya already uses `background` | ≈ `NSView.layer.backgroundColor` | Background of a view from a `ShapeStyle`, with blur for a material | [Styles](../../agents/styles.md#paint-on-views) |
| `modifiers::ViewModifiers::shadows` | method | ≈ `.shadow(color:radius:x:y:)`; Freya already uses `shadow` | ≈ `NSView.shadow` | Adds a list of `ShadowStyle` to a view | [Styles](../../agents/styles.md#paint-on-views) |
| `controls::Slider::style` | method | ≈ `.buttonStyle(_:)` on one view | ≈ `NSSlider.trackFillColor` | Changes the style of one slider | [Styles](../../agents/styles.md#levels) |
| `controls::Button::style` | method | `.buttonStyle(_:)` on one view | ≈ `NSButton.bezelStyle` | Changes the style of one button; takes a preset or a closure | [Styles](../../crates/bk/src/styles/button.rs) |
| `controls::Switch::style` | method | `.toggleStyle(.switch)` on one view | — | Changes the style of one switch | [Styles](../../crates/bk/src/styles/switch.rs) |
| `controls::Checkbox::style` | method | `.toggleStyle(.checkbox)` on one view | — | Changes the style of one checkbox | [Styles](../../crates/bk/src/styles/checkbox.rs) |
| `controls::SegmentedControl::style` | method | `.pickerStyle(.segmented)` on one view | ≈ `NSSegmentedControl.segmentStyle` | Changes the style of one segmented control | [Styles](../../crates/bk/src/styles/segmented_control.rs) |
| `controls::ToolGroup::style` | method | — | — | Changes the style of one tool group | [Styles](../../crates/bk/src/styles/tool_group.rs) |
| `controls::TextField::style` | method | `.textFieldStyle(_:)` on one view | ≈ `NSTextField.bezelStyle` | Changes the style of one text field | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `controls::SearchField::style` | method | — | — | Changes the style of one search field | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `controls::SecureField::style` | method | `.textFieldStyle(_:)` on one view | — | Changes the style of one password field | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `level::LevelSlider::style` | method | — | — | Changes the style of one level slider | [Styles](../../crates/bk/src/styles/level_slider.rs) |
| `choices::RadioGroup::style` | method | `.pickerStyle(.radioGroup)` on one view | — | Changes the style of one radio group | [Styles](../../crates/bk/src/styles/radio_group.rs) |
| `choices::ColorWell::style` | method | `.pickerStyle(.palette)` on one view | ≈ `NSColorWell.colorWellStyle` | Changes the style of one color well | [Styles](../../crates/bk/src/styles/color_well.rs) |
| `choices::ChoiceCards::style` | method | — | — | Changes the style of one set of choice cards | [Styles](../../crates/bk/src/styles/color_well.rs) |
| `compact::IconButton::style` | method | `.buttonStyle(_:)` on one view | ≈ `NSButton.bezelStyle` | Changes the style of one icon button | [Styles](../../crates/bk/src/styles/icon_button.rs) |
| `compact::Stepper::style` | method | — | — | Changes the style of one stepper | [Styles](../../crates/bk/src/styles/stepper.rs) |
| `compact::ToggleGroup::style` | method | — | — | Changes the style of one toggle group | [Styles](../../crates/bk/src/styles/toggle_group.rs) |
| `compact::ToolbarSearch::style` | method | — | — | Changes the width of one toolbar search | [Styles](../../crates/bk/src/styles/toolbar_search.rs) |
| `compact::ToolbarOverflow::style` | method | `.buttonStyle(_:)` on one view | — | Changes the `IconButtonStyle` of one overflow button | [Styles](../../crates/bk/src/styles/icon_button.rs) |
| `buttons::MenuButton::style` | method | `.menuStyle(_:)` on one view | — | Changes the style of one menu button | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `buttons::ComboButton::style` | method | `.menuStyle(_:)` on one view | ≈ `NSComboButton.style` | Changes the style of one combo button | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `buttons::HelpLink::style` | method | — | — | Changes the style of one help button | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `buttons::ShareLink::style` | method | `.menuStyle(_:)` on one view | — | Changes the `MenuButtonStyle` of one share link | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `buttons::ControlGroup::style` | method | `.controlGroupStyle(_:)` on one view | — | Changes the style of one control group | [Styles](../../crates/bk/src/styles/control_group.rs) |
| `popup_button::PopUpButton::style` | method | `.pickerStyle(.menu)` on one view | — | Changes the style of one pop-up button | [Styles](../../crates/bk/src/styles/popup_button.rs) |
| `popup_button::PopUpButton::disabled` | method | `.disabled(_:)` | `isEnabled` | Dims the pop-up button and stops it opening | [5](05-controls.md#choices) |
| `combo_box::ComboBox::style` | method | — | — | Changes the style of the arrow button of one combo box | [Styles](../../crates/bk/src/styles/combo_box.rs) |
| `token_field::TokenField::style` | method | — | — | Changes the style of one token field | [Styles](../../crates/bk/src/styles/token_field.rs) |
| `date_picker::DatePicker::style` | method | `.datePickerStyle(_:)` on one view | ≈ `NSDatePicker.datePickerStyle` | Changes the style of one date picker | [Styles](../../crates/bk/src/styles/date_picker.rs) |
| `path_control::PathControl::style` | method | — | — | Changes the style of one path control | [Styles](../../crates/bk/src/styles/path_control.rs) |
| `image_well::ImageWell::style` | method | — | — | Changes the style of one image well | [Styles](../../crates/bk/src/styles/image_well.rs) |
| `color_picker::ColorPicker::style` | method | — | ≈ `NSColorWell.colorWellStyle` | Changes the style of one color picker | [Styles](../../crates/bk/src/styles/color_picker.rs) |
| `font_panel::FontPicker::style` | method | `.buttonStyle(_:)` on one view | — | Changes the `ButtonStyle` of one font picker | [Styles](../../crates/bk/src/styles/color_picker.rs) |
| `progress::ProgressView::style` | method | `.progressViewStyle(_:)` on one view | ≈ `NSProgressIndicator.Style` | Changes the style of one progress bar | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `progress::ProgressRing::style` | method | `.progressViewStyle(.circular)` on one view | — | Changes the style of one progress ring | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `progress::Spinner::style` | method | `.progressViewStyle(.circular)` on one view | — | Changes the style of one spinner | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `indicator::CapacityIndicator::style` | method | `.gaugeStyle(_:)` on one view | ≈ `NSLevelIndicator.levelIndicatorStyle` | Changes the style of one capacity indicator | [Styles](../../crates/bk/src/styles/capacity_indicator.rs) |
| `indicator::RelevancyIndicator::style` | method | — | ≈ `NSLevelIndicator.levelIndicatorStyle` | Changes the style of one relevancy indicator | [Styles](../../crates/bk/src/styles/relevancy_indicator.rs) |
| `indicator::RatingIndicator::style` | method | — | ≈ `NSLevelIndicator.levelIndicatorStyle` | Changes the style of one rating indicator | [Styles](../../crates/bk/src/styles/relevancy_indicator.rs) |
| `indicator::Gauge::style` | method | `.gaugeStyle(_:)` on one view | — | Changes the style of one gauge | [Styles](../../crates/bk/src/styles/relevancy_indicator.rs) |
| `media::MediaButton::style` | method | `.buttonStyle(_:)` on one view | — | Changes the style of one media button | [Styles](../../crates/bk/src/styles/media_button.rs) |
| `media::RoundButton::style` | method | `.buttonStyle(_:)` on one view | — | Changes the style of one round button | [Styles](../../crates/bk/src/styles/media_button.rs) |
| `adornments::Link::style` | method | `.buttonStyle(.link)` on one view | — | Changes the style of one link | [Styles](../../crates/bk/src/styles/link.rs) |
| `elements::Text::style` | method | ≈ `.font(_:)` on one view | — | Builds one text again with its style changed | [Styles](../../crates/bk/src/styles/text.rs) |
| `adornments::Badge::style` | method | — | — | Builds one badge again with its style changed | [Styles](../../crates/bk/src/styles/text.rs) |
| `adornments::Divider::style` | method | — | — | Builds one divider again with its style changed | [Styles](../../crates/bk/src/styles/text.rs) |
| `adornments::Avatar::style` | method | — | — | Builds one avatar again with its style changed | [Styles](../../crates/bk/src/styles/text.rs) |
| `symbol::Image::style` | method | ≈ `.imageScale(_:)` on one view | — | Changes the style of one image | [Styles](../../crates/bk/src/styles/text.rs) |
| `media::Material::style` | method | — | — | Changes the style of one material | [Styles](../../crates/bk/src/styles/material.rs) |
| `media::TopGlass::style` | method | ≈ `.scrollEdgeEffectStyle(_:for:)` on one view | — | Changes the style of one top glass | [Styles](../../crates/bk/src/styles/material.rs) |
| `glass_container::GlassContainer::style` | method | — | — | Changes the style of one glass container | [Styles](../../crates/bk/src/styles/material.rs) |
| `backdrop::Backdrop::style` | method | — | — | Changes the style of one backdrop | [Styles](../../crates/bk/src/styles/backdrop.rs) |
| `controls::ScrollView::style` | method | ≈ `.scrollIndicators(_:axes:)` on one view | — | Changes the style of one scroll view | [Styles](../../crates/bk/src/styles/scroll_view.rs) |
| `text_editor::TextEditor::style` | method | `.textEditorStyle(_:)` on one view | — | Changes the style of one text editor | [Styles](../../crates/bk/src/styles/text_editor.rs) |
| `rule_editor::RuleEditor::style` | method | — | — | Changes the style of one rule editor | [Styles](../../crates/bk/src/styles/rule_editor.rs) |
| `print::PrintPreview::style` | method | — | — | Changes the style of one print preview | [Styles](../../crates/bk/src/styles/print_preview.rs) |
| `charts::Chart::style`, `SectorChart::style`, `HeatMap::style`, `StreamChart::style`, `Chart3D::style` | method | ≈ `.chartForegroundStyleScale(_:)` on one chart | — | Changes the `ChartStyle` of one chart | [Styles](../../crates/bk/src/styles/chart.rs) |
| `titlebar::Titlebar::style` | method | — | — | Changes the style of one title bar | [Styles](../../crates/bk/src/styles/titlebar.rs) |
| `traffic::TrafficLights::style` | method | — | — | Changes the style of one set of traffic lights | [Styles](../../crates/bk/src/styles/traffic_lights.rs) |
| `chrome::WindowChrome::style` | method | — | — | Sets the layout and look of the window chrome | [Styles](../../crates/bk/src/styles/window_chrome.rs) |
| `sidebar::Sidebar::style` | method | — | — | Changes the style of one sidebar | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `sidebar::SidebarItem::style` | method | — | — | Changes the style of one sidebar row | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `sidebar::SidebarSection::style` | method | — | — | Changes the style of one sidebar section | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `sidebar::SidebarAccount::style` | method | — | — | Changes the style of one account row | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `sidebar::SymbolTile::style` | method | — | — | Builds the tile again with a style override | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `views::split::SplitView::style` | method | — | — | Changes the style of one split view | [Styles](../../crates/bk/src/styles/split_view.rs) |
| `views::split_panes::SplitPanes::style` | method | — | — | Changes the style of one set of panes | [Styles](../../crates/bk/src/styles/split_view.rs) |
| `views::floating_pane::FloatingPane::style` | method | — | — | Changes the style of one floating pane; the `style` field holds it | [Styles](../../crates/bk/src/styles/split_view.rs) |
| `views::form::Form::style` | method | `formStyle(_:)` | — | Changes the style of one form | [Styles](../../crates/bk/src/styles/form.rs) |
| `views::group::GroupBox::style` | method | `groupBoxStyle(_:)` | — | Changes the style of one group box | [Styles](../../crates/bk/src/styles/form.rs) |
| `views::group::DisclosureGroup::style` | method | `disclosureGroupStyle(_:)` | — | Changes the style of one disclosure group | [Styles](../../crates/bk/src/styles/form.rs) |
| `views::table::Table::style` | method | `tableStyle(_:)` | ≈ `NSTableView.style` | Changes the style of one table | [Styles](../../crates/bk/src/styles/table.rs) |
| `views::collection::CollectionView::style` | method | — | — | Changes the style of one collection view | [Styles](../../crates/bk/src/styles/table.rs) |
| `views::tree::OutlineView::style` | method | ≈ `listStyle(_:)` | — | Changes the style of one outline view | [Styles](../../crates/bk/src/styles/table.rs) |
| `views::tree::Browser::style` | method | — | — | Changes the style of one browser | [Styles](../../crates/bk/src/styles/table.rs) |
| `views::tabs::TabView::style` | method | `tabViewStyle(_:)` | — | Changes the style of one tab view | [Styles](../../crates/bk/src/styles/tab_view.rs) |
| `views::window_tabs::WindowTabs::style` | method | — | — | Changes the style of one tab bar | [Styles](../../crates/bk/src/styles/tab_view.rs) |
| `views::text_view::TextView::style` | method | ≈ `textEditorStyle(_:)` | — | Changes the style of one text view | [Styles](../../crates/bk/src/styles/text_view.rs) |
| `views::feedback::Sheet::style` | method | ≈ `presentationBackground(_:)` | — | Builds the sheet again with a style; call it right after `new` | [Styles](../../crates/bk/src/styles/sheet.rs) |
| `views::feedback::Alert::style` | method | — | — | Changes the style of one alert | [Styles](../../crates/bk/src/styles/sheet.rs) |
| `views::feedback::ContentUnavailableView::style` | method | — | — | Builds the view again with a style; call it right after `new` | [Styles](../../crates/bk/src/styles/sheet.rs) |
| `patterns::settings::SettingsWindow::navigation_destination` | method | `navigationDestination(item:)` | — | Opens a pane by id on request | [12](12-patterns.md#settingswindow) |
| `patterns::settings::SettingsWindow::style` | method | — | — | Changes the style of one preferences window | [Styles](../../crates/bk/src/styles/settings_window.rs) |
| `patterns::about::AboutView::style` | method | — | — | Changes the style of one About view | [Styles](../../crates/bk/src/styles/settings_window.rs) |
| `menu::MenuView::style` | method | — | — | Changes the style of one menu and its submenus | [7](07-menus-and-popovers.md#style) |
| `video::VideoView::style` | method | — | — | Changes the style of one video view | [Styles](../../crates/bk/src/styles/video_view.rs) |
| `app::Application::theme` | method | ≈ `.environment(_:_:)` on the scene | ≈ `NSApp.appearance` | Gives the app its `Theme` | [Styles](../../agents/styles.md#levels) |

## Symbols and fonts

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `symbol::Symbol` | enum | ≈ `Image(systemName:)` names | ≈ `NSImage(systemSymbolName:accessibilityDescription:)` names | Name of a built-in line icon; 61 variants | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image` | struct | `Image(systemName:)` | `NSImage(systemSymbolName:accessibilityDescription:)` | A symbol as a view; `BODY` size and label color by default | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image::data` | fn | `Image(nsImage:)` | `NSImage(data:)` | An image view from SVG, PNG or JPEG bytes | [4](04-appearance-and-tokens.md#images-and-template-rendering) |
| `symbol::Image::rgba` | fn | `Image(nsImage:)` | ≈ `NSBitmapImageRep` | An image view from straight-alpha RGBA pixels | [4](04-appearance-and-tokens.md#images-and-template-rendering) |
| `symbol::TemplateRenderingMode` | enum | `Image.TemplateRenderingMode` | ≈ `NSImage.isTemplate` | `Original` keeps the colors; `Template` paints the shape in the foreground color | [4](04-appearance-and-tokens.md#images-and-template-rendering) |
| `register_font` | fn | — | — | Adds a typeface from font file bytes; `false` if not a font | [4](04-appearance-and-tokens.md#fonts) |

## Helpers

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `init_tracing` | fn | — | — | Installs logging to stderr by `RUST_LOG`, and Tracy with `tracy` | [4](04-appearance-and-tokens.md#helpers) |
| `ProbeExt` | trait | `.accessibilityIdentifier(_:)` | `accessibilityIdentifier()` | `probe(id)`: sets a `rect`'s accessibility id for tests | [4](04-appearance-and-tokens.md#helpers) |

## Appearance settings

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `ctappearance::AppearanceConfig` | struct | — | — | The user's `appearance.domain`: mode, accent, tints, contrast, motion | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::ColorScheme` | enum | ≈ `ColorScheme` | ≈ `NSAppearance` | `Auto`, `Light`, `Dark`, `DarkTint`; SwiftUI has only light and dark | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::WindowColorMode` | enum | — | — | Window tint source: `Neutral` or `Accent` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::WindowTintScope` | enum | — | — | Tint reaches `Windows` only or the `Desktop` bars too | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::TintStrength` | enum | — | — | `Subtle`, `Normal`, `Strong`, `Intense`; `factor()` from `TINT_STRENGTH` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::TintParts` | struct | — | — | Which parts take the tint: sidebars, window, groups, controls, menus | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::SidebarStyle` | enum | — | — | `Island` or `Flat` sidebar | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::TrafficLightStyle` | enum | — | — | `Auto`, `Standard`, `Accent` traffic-light colors | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::TrafficLightShape` | struct | — | — | The lights' corner radius from 0 to `ROUND` (2026-11-09: was the enum `Square`/`Round`); `new(radius)`, `radius()`, `corner_radius(diameter)` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::Toolkit` | enum | — | — | `BarrierKit`, `Gtk`, `Qt`: the kinds of app in Accessibility › Display › Text size | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::TextSizes` | struct | — | ≈ the per-app list of Accessibility › Display › Text size | An optional text size per `Toolkit`; `get_mut(toolkit)` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::AppearanceConfig::text_size_for` | method | — | — | The toolkit's own text size, or the preferred `text_size` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::MenuBarSize` | enum | — | — | Menu bar text `Default` or `Large` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::BlurBehind` | enum | — | ≈ `NSVisualEffectView.BlendingMode` | What the compositor's background blur shows: `Mixed` (by surface), `Wallpaper` or `Windows` (malaysia ADR 0071) | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::ColorFilter` | enum | — | ≈ Accessibility › Display › Color filters | `Grayscale`, `RedGreen`, `GreenRed`, `BlueYellow` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::AppearanceConfig::transparency_reduced` | method | — | ≈ `NSWorkspace.accessibilityDisplayShouldReduceTransparency` | True with `reduce_transparency` or `contrast` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::ScrollBars` | enum | ≈ `ScrollIndicatorVisibility` | ≈ `NSScroller.Style` | Show scroll bars `Automatic` (by mouse or touchpad), `WhenScrolling` or `Always` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::ScrollBarClick` | enum | — | ≈ `AppleScrollerPagingBehavior` | A click on the track goes to the `NextPage` or the `ClickedSpot` | [4](04-appearance-and-tokens.md#the-appearance-file) |

## Appearance files and watch

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `ctappearance::config_dir` | fn | — | — | `$MALAYSIA_CONFIG_DIR` or `$XDG_CONFIG_HOME/malaysia` | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::data_dir` | fn | — | — | `$MALAYSIA_DATA_DIR` or `$XDG_DATA_HOME/malaysia` | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::appearance_path` | fn | — | — | Path of `appearance.domain` | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::wallpaper_state_path` | fn | — | — | Path of `wallpaper.state`, the wallpaper's dominant color and grid | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::read_appearance` | fn | — | — | Reads the appearance file; `None` without file, `Err` on bad TOML | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::load_appearance` | fn | — | — | Reads the appearance file, or the default on any error | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::write_appearance` | fn | — | — | Writes the config atomically, keeping other keys and comments | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::load_dominant` | fn | — | — | Reads the wallpaper's dominant color | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::write_wallpaper_state` | fn | — | — | Writes or removes the wallpaper's dominant color and grid | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::WallpaperState` | struct | — | — | Contents of `wallpaper.state`: `dominant`, `grid`, `grids` per desktop, `outputs`, `shown` and `shuffled_at` | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::wallpaper_selection_path` | fn | — | — | Path of `wallpaper-selection.state`, the wallpaper each desktop shows | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::read_wallpaper_selection` | fn | — | — | Reads the selection; the default without a file, `Err` on bad TOML | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::write_wallpaper_selection` | fn | — | — | Writes the selection atomically | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperSelection` | struct | — | ≈ `NSWorkspace.desktopImageURL(for:)` and its options | `wallpaper-selection.state`: sources, scales, crops, dynamic appearances and paused animations per desktop and output, and `shuffle` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperSelection::show_on_all_desktops` | method | — | ≈ «Show on all Spaces» | Makes a desktop's wallpaper, framing, appearance and pause the shared ones | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperScale` | enum | — | ≈ `NSImageScaling` | `Fill`, `Fit`, `Stretch`, `Center`, `Custom` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::ALL_DESKTOPS` | constant | — | — | Key `"default"`: the wallpaper and crop of every desktop | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::desktop_key` | fn | — | — | Key of a desktop in the wallpaper files: its number, or `ALL_DESKTOPS` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperSource` | enum | — | — | What a desktop shows: `BuiltIn`, `File`, `ShuffleColors`, `ShuffleCollection`; `parse` and `value` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperAppearance` | enum | — | ≈ `ColorScheme` | `Light` or `Dark`: the still a dynamic wallpaper keeps | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperShuffle` | enum | — | ≈ «Change picture» | How often a shuffle changes: `Login`, `Hourly`, `Daily` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperKind` | enum | — | — | `Image`, `Video` or `Shader`, from the file extension | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::wallpapers_in` | fn | — | — | Wallpaper files of a folder, sorted | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::system_wallpapers_dir` | fn | — | — | The first `$XDG_DATA_DIRS/wallpapers` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::custom_colors_dir` | fn | — | — | `data_dir()/wallpapers/colors`, the user's own color wallpapers | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::wallpaper_colors` | fn | — | — | The wallpaper colors: the system's `solid-colors`, then the user's | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperCollection` | struct | — | ≈ a Photos album as wallpaper | `name`, `folders` and `images`; `wallpapers()` lists them once | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperConfig` | struct | — | — | Contents of `wallpaper.domain`: `collections` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::wallpaper_config_path` | fn | — | — | Path of `wallpaper.domain` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::read_wallpaper_config` | fn | — | — | Reads the collections; the default without a file | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::write_wallpaper_config` | fn | — | — | Writes the collections atomically | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::file_uri`, `file_path` | fn | — | ≈ `g_filename_to_uri` | A path as a `file://` URI escaped like GLib, and back | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::thumbnail_path` | fn | — | — | The x-large thumbnail of a file in the freedesktop thumbnail cache | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::frame_path` | fn | — | — | The light or dark still of a dynamic wallpaper, in `~/.cache/malaysia/wallpaper-frames` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::thumbnail_is_fresh` | fn | — | — | Whether a thumbnail's `Thumb::MTime` matches its file | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::thumbnail_image_size` | fn | — | — | The original image size a thumbnail records in `Thumb::Image::Width` and `Height` | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::modified_seconds` | fn | — | — | A file's modification time in seconds | [4](04-appearance-and-tokens.md#the-wallpaper-files) |
| `ctappearance::WallpaperOutput` | struct | — | ≈ `NSScreen` | An output the wallpaper draws on: `name`, and `width` and `height` in pixels | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::load_wallpaper_state` | fn | — | — | Reads `wallpaper.state`, or the default | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::xdg::cache_home` | fn | — | — | `$XDG_CACHE_HOME` or `~/.cache` | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::tinting::WallpaperGrid` | struct | — | — | Average colors of the wallpaper; `from_rgba8`, `behind(output, window)` | [4](04-appearance-and-tokens.md#wallpaper-behind-a-window) |
| `ctappearance::watch::AppearanceWatch` | struct | — | — | inotify watch on the config and data folders; `drain()` | [4](04-appearance-and-tokens.md#watching) |
| `ctappearance::watch::AppearanceEvent` | enum | — | — | `Appearance`, `Wallpaper`, `WallpaperSelection`, `Defaults`; `restyles()` | [4](04-appearance-and-tokens.md#watching) |

## Appearance resolution

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `ctappearance::resolve` | fn | — | — | Turns the config and wallpaper color into concrete colors | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::Resolved` | struct | ≈ `EnvironmentValues` | — | Resolved appearance: accent, focus ring, link, tints, surfaces, palettes | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::Resolved::behind_window` | method | — | ≈ desktop tinting of `NSVisualEffectView.Material.sidebar` | Mixes the wallpaper behind a window into its sidebar | [4](04-appearance-and-tokens.md#wallpaper-behind-a-window) |
| `ctappearance::Surfaces` | struct | — | — | Tinted window, sunken, surface, raised, control, hover and line colors | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::SidebarSurface` | struct | — | — | Sidebar style, background, rim, inner line and palette | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::ToolgroupSurface` | struct | — | — | Toolbar group background and palette | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::palette` | fn | — | — | Base `ctdesign::Palette` of a color scheme | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::lights_for` | fn | — | — | The three traffic-light colors of a style | [4](04-appearance-and-tokens.md#resolution) |

## Window behavior

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `ctappearance::menubar::MenubarConfig` | struct | — | — | `menubar.domain`: `app_menus` the `show_*` status menus, `style` and `bar_height` | [4](04-appearance-and-tokens.md#menu-bar) |
| `ctappearance::menubar::MenubarConfig::height` | method | — | — | Menu bar height in logical pixels for `bar_height` | [4](04-appearance-and-tokens.md#menu-bar) |
| `ctappearance::menubar::colors_over` | function | — | — | Ink and glass of a translucent or floating menu bar over a wallpaper color: dark over light, light over dark | [4](04-appearance-and-tokens.md#menu-bar) |
| `ctappearance::menubar::MenubarStyle` | enum | — | — | `Solid`, `Translucent`, `Floating`: how the menu bar draws its background | [4](04-appearance-and-tokens.md#menu-bar) |
| `ctappearance::menubar::MenubarHeight` | enum | — | — | `Slim`, `Wide`: the menu bar height | [4](04-appearance-and-tokens.md#menu-bar) |
| `ctappearance::window::WindowBehavior` | struct | — | — | `window.domain`: titlebar double click and green button actions | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::read_window_behavior` | fn | — | — | Reads `window.domain`, or the default | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::DoubleClickAction` | enum | — | — | Titlebar double click: `Fill`, `Minimize`, `Zoom`, `None` | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::GreenButtonAction` | enum | — | — | Green light: `Focus`, `Zoom`, `None` | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::TitlebarAction` | enum | — | — | What to do: `Focus`, `Maximize`, `Minimize`, `None` | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::green_action` | fn | — | — | Green-button action for a window, fixed width or not | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::double_click_action` | fn | — | — | Double-click action for a window, zoomable or not | [4](04-appearance-and-tokens.md#window-behavior) |

## Traffic-light geometry

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `ctappearance::traffic::inset` | fn | — | — | Distance of the lights from the edges of a bar | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::button_rect` | fn | — | ≈ `NSWindow.standardWindowButton(_:)` frame | Rectangle of one light | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::group_rect` | fn | — | — | Rectangle of the three lights | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::hit` | fn | — | — | The light under a point | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::visual` | fn | — | — | Fill, rim and glyph of each light for an input | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::glyph_svg` | fn | — | — | SVG of a light's glyph | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::TrafficInput` | struct | — | — | Focus, group hover, pressed light and zoom support | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::ButtonVisual` | struct | — | — | Fill, rim and optional glyph color of one light | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::TrafficPointer` | struct | — | — | Pointer state machine for hover, arm and click of the lights | [4](04-appearance-and-tokens.md#traffic-light-geometry) |

## Backdrop colors

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `ctappearance::backdrop::Backdrop` | struct | — | — | Three gradient layers and a base color derived from an image | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::ImagePalette` | struct | — | — | Three representative colors and the most vivid one of an image | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::Oklab` | struct | — | — | A color in the OKLab space | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::palette_from_pixels` | fn | — | — | Image palette from RGBA8 pixels, by k-means in OKLab | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::dominant_from_pixels` | fn | — | — | Dominant chromatic color of RGBA8 pixels | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::ink_on` | fn | — | — | Light or dark ink with more contrast on a background | [4](04-appearance-and-tokens.md#backdrop-colors) |

## Language and directories

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `ctappearance::language::read_language` | fn | — | — | The `language` key of a `.domain` file | [4](04-appearance-and-tokens.md#language) |
| `ctappearance::language::system_language` | fn | — | — | The language chosen in `language.domain` | [4](04-appearance-and-tokens.md#language) |
| `ctappearance::language::requested_languages` | fn | — | — | App, system or POSIX-locale languages as tags, in order | [4](04-appearance-and-tokens.md#language) |
| `ctappearance::xdg::config_home` | fn | — | — | `$XDG_CONFIG_HOME` or `~/.config` | [4](04-appearance-and-tokens.md#directories) |
| `ctappearance::ini::value`, `ini::set` | fn | — | — | Read, set or remove a key of an INI file (GTK `settings.ini`, qt5ct and qt6ct) | [4](04-appearance-and-tokens.md#directories) |
| `ctappearance::xdg::data_home` | fn | — | — | `$XDG_DATA_HOME` or `~/.local/share` | [4](04-appearance-and-tokens.md#directories) |
| `ctappearance::xdg::data_dirs` | fn | — | — | `$XDG_DATA_DIRS` or `/usr/local/share` and `/usr/share` | [4](04-appearance-and-tokens.md#directories) |
| `ctappearance::xdg::user_dir` | fn | — | — | An XDG user directory such as `DOWNLOAD` from `user-dirs.dirs` | [4](04-appearance-and-tokens.md#directories) |

## Design token types

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `ctdesign::Rgba` | struct | ≈ `Color` | ≈ `NSColor` | sRGB color, channels 0–1, with mixing and contrast helpers | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Font` | struct | ≈ `Font.TextStyle` | ≈ `NSFont.TextStyle` | Type token: family, size, line height, weight | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Font::scaled` | method | — | — | The font with size and line height times a factor | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctdesign::Palette` | struct | — | — | Base colors of one theme: inks, lines, accent, titlebar | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::FocusPair` | struct | — | — | One color for a focused window, one for unfocused | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::PanelLine` | struct | — | — | Inner line of a focused panel: sides, edges and glow | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::TrafficRim` | struct | — | — | Rim color of a lit and an idle traffic light | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::ShellPalette` | struct | — | — | Menu bar and Dock colors | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::MenuPalette` | struct | — | — | Menu colors and metrics | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::SidebarPalette` | struct | — | — | Sidebar item, selection, section and line inks | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::ToolgroupPalette` | struct | — | — | Toolbar group separator and rim | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::ShadowLayer` | struct | ≈ `.shadow(color:radius:x:y:)` | ≈ `NSShadow` | One shadow: blur, offset, opacity, color | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::ShadowStyle` | struct | — | — | Ambient and contact shadows, scaled when unfocused | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::MaterialThickness` | struct | ≈ `Material` | ≈ `NSVisualEffectView.Material` | Opacity and blur of one material thickness | [4](04-appearance-and-tokens.md#materials) |
| `ctdesign::WindowCorner` | struct | — | — | Window corner radii and continuous-curve shape | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::WindowStyle` | struct | — | — | Corner radius and superellipse exponent of a dialog or pop-up | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::WindowResize` | struct | — | — | Width of the resize band at edges and corners | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Titlebar` | struct | — | — | Server-side titlebar thickness, lights and font | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::TitlebarButtons` | struct | — | — | Order, diameter, spacing and radius of the traffic lights | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Button` | struct | — | — | One traffic light: kind and color | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::ButtonKind` | enum | — | `NSWindow.ButtonType` | `Close`, `Minimize`, `Maximize` | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::TrafficGlyphs` | struct | — | — | Geometry of the close, minimize, zoom and plus glyphs | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Segment` | struct | — | — | A line from point `a` to point `b` | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Triangle` | struct | — | — | Three points of a filled glyph triangle | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::DoubleClick` | struct | — | ≈ `NSEvent.doubleClickInterval` | Double-click interval and pointer slop | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::SettingsStrength` | struct | — | — | Tint-strength scale of the settings app | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::SettingsCheck` | struct | — | — | Check list spacing of the settings app | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::SettingsWallpaper` | struct | — | — | Wallpaper pane of the settings app: preview, tiles, color dots, custom framing | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::SettingsMetrics` | struct | — | — | Sizes of the settings window, sidebar, search and previews | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ButtonMetrics` | struct | — | — | Push button sizes, radius, font and pressed scale | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::IconButtonMetrics` | struct | — | — | Icon button size, radius and icon size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SwitchMetrics` | struct | — | — | Switch track, knob and transition | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::CheckboxMetrics` | struct | — | — | Checkbox size, radius and mark | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SliderMetrics` | struct | — | — | Slider height, track and knob | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SliderFatMetrics` | struct | — | — | Wide level slider height, minimum fill and icon position | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SegmentedMetrics` | struct | — | — | Segmented control padding, segments and font | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::StepperMetrics` | struct | — | — | Stepper size, radius and chevrons | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TextFieldMetrics` | struct | — | — | Text field height, padding and radius | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TokenFieldMetrics` | struct | — | — | Token height, padding, gap and opacity | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SpellingMetrics` | struct | — | — | Size of the misspelling dot | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::ComboBoxMetrics` | struct | — | — | Combo box button and chevron sizes | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::PathControlMetrics` | struct | — | — | Path control height, chevrons and icons | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::DatePickerMetrics` | struct | — | — | Calendar size, header, weekday row and radii | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::ImageWellMetrics` | struct | — | — | Image well size, radius, inset and drop border | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::LevelIndicatorMetrics` | struct | — | — | Level indicator bars, stars and relevancy sizes | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::GaugeMetrics` | struct | — | — | Gauge size, stroke, arc and marker | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::ProgressMetrics` | struct | — | — | Progress bar, ring and spinner sizes and timing | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::BadgeMetrics` | struct | — | — | Badge height and padding | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::FocusRingMetrics` | struct | — | — | Focus ring width | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SymbolMetrics` | struct | — | — | Symbol grid and stroke width | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TooltipMetrics` | struct | — | — | Tooltip delay, padding, radius and pointer gap | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::PopoverMetrics` | struct | — | — | Popover radius, arrow and padding | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::MenuPopoverMetrics` | struct | — | — | Pop-up menu item, padding, check and swatch grid sizes | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::DragMetrics` | struct | — | — | Pointer travel before a drag starts | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TitlebarRailMetrics` | struct | — | — | Titlebar rail widths, pill and reveal zone | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::FullscreenBarMetrics` | struct | — | — | Content offset and band line of a full-screen bar | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::FullscreenGlassMetrics` | struct | — | — | Height, blur, layers, tint and mask of full-screen glass | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::SidebarMetrics` | struct | — | — | Sidebar width, island, items and sections | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ToolgroupMetrics` | struct | — | — | Toolbar group height, padding and separators | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::SplitViewMetrics` | struct | — | — | Sidebar limits, collapse width, inspector, grip and key step | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::SplitPanesMetrics` | struct | — | — | Divider width and minimum pane size | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::FloatingPaneMetrics` | struct | — | — | Floating pane title, outline, resize band and shadow | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::TableMetrics` | struct | — | — | Table rows, header, zebra and column sizes | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::CollectionMetrics` | struct | — | — | Collection tile, icon, gap, radius and selection | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ListGroupMetrics` | struct | — | — | Form group radius, rows, separators and headers | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::GroupBoxMetrics` | struct | — | — | Group box radius, padding, label and fill | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::DisclosureMetrics` | struct | — | — | Disclosure indent, rows, chevron and icon | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ToolbarTabsMetrics` | struct | — | — | Toolbar tab container and tab sizes | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ScrollBarMetrics` | struct | — | — | Scroll bar thumb, track, opacity and hide delay | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::BreakpointMetrics` | struct | — | — | Medium and wide window widths for adaptive layouts | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::FontPanelMetrics` | struct | — | — | Font panel size list and size column | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::PrintMetrics` | struct | — | — | Print resolution and page margins | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ChartMetrics` | struct | — | — | Chart strokes, bars, sectors, axes, zoom and 3D view | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::MediaMetrics` | struct | — | — | Media buttons, glass material, inks and scrims | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::MaterialMetrics` | struct | — | — | One `MaterialThickness` per `styles::Thickness` | [4](04-appearance-and-tokens.md#materials) |
| `ctdesign::BackdropMetrics` | struct | — | — | Backdrop gradient, glass, ink and accent parameters | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::BackdropScenes` | struct | — | — | Preset backdrop gradient: dusk | [4](04-appearance-and-tokens.md#view-metrics) |

## Design token constants

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `ctdesign::SPACE_1`, `SPACE_2`, `SPACE_3`, `SPACE_4` | constant | — | — | Spacing steps: 4, 8, 12, 16 | [4](04-appearance-and-tokens.md#spacing) |
| `ctdesign::HAIRLINE` | constant | — | — | Separator width, 1 px | [4](04-appearance-and-tokens.md#spacing) |
| `ctdesign::LARGE_TITLE`, `TITLE`, `TITLE_2`, `HEADLINE`, `BODY`, `CALLOUT`, `CAPTION` | constant | `Font.TextStyle` | `NSFont.TextStyle` | Type styles named after SwiftUI's | [4](04-appearance-and-tokens.md#type) |
| `ctdesign::MONO` | constant | `Font.monospaced()` | `NSFont.monospacedSystemFont(ofSize:weight:)` | Monospaced body text, Geist Mono | [4](04-appearance-and-tokens.md#type) |
| `ctdesign::SANS_FAMILIES`, `TEXT_LINE_HEIGHT` | constant | — | — | Sans-serif fallback list; line-height factor for measuring text | [4](04-appearance-and-tokens.md#type) |
| `ctdesign::DURATION_FAST_MS`, `DURATION_BASE`, `DURATION_SLOW_MS`, `THEME_FADE_WAIT_MS` | constant | — | — | Durations: 120, 220, 420 ms; theme fade wait | [4](04-appearance-and-tokens.md#motion) |
| `ctdesign::SPRING_DAMPING` | constant | ≈ `Spring.damping` | — | Damping of the kit's spring | [4](04-appearance-and-tokens.md#motion) |
| `ctdesign::LIGHT`, `DARK`, `DARK_TINT` | constant | — | — | Base palette of each theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::BG_WINDOW_LIGHT`, `BG_WINDOW_DARK`, `BG_WINDOW_DARK_TINT`, `BG_SURFACE_LIGHT`, `BG_SURFACE_DARK`, `BG_SURFACE_DARK_TINT`, `BG_SUNKEN_LIGHT`, `BG_SUNKEN_DARK`, `BG_SUNKEN_DARK_TINT`, `BG_RAISED_LIGHT`, `BG_RAISED_DARK`, `BG_RAISED_DARK_TINT`, `BG_CONTROL_LIGHT`, `BG_CONTROL_DARK`, `BG_CONTROL_DARK_TINT`, `BG_HOVER_LIGHT`, `BG_HOVER_DARK`, `BG_HOVER_DARK_TINT` | constant | — | ≈ `NSColor.windowBackgroundColor`… | Background layers per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::LAYER_FACTOR_BG_SUNKEN`, `LAYER_FACTOR_SURFACE`, `LAYER_FACTOR_RAISED`, `LAYER_FACTOR_CONTROL`, `LAYER_FACTOR_LINE` | constant | — | — | Share of the window tint each layer takes | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::WINDOW_TINT_ACCENT`, `TINT_STRENGTH` | constant | — | — | Accent tint amounts in light and dark; strength factors | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::TEXT_SIZES`, `POINTER_SIZES`, `DISPLAY_CONTRAST`, `COLOR_FILTER_INTENSITY` | constant | — | — | The steps of the Accessibility › Display sliders | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctdesign::WALLPAPER_TINTING`, `WALLPAPER_TINTING_GRID` | constant | — | — | Amount of the wallpaper behind a window; cells per side of its grid | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::ACCENT_MIN_CONTRAST_ON_WINDOW`, `ON_ACCENT_MIN_CONTRAST` | constant | — | — | Contrast minimums: accent on window 3, ink on accent 4.5 | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::DOMINANT_LIGHTNESS`, `DOMINANT_MIN_CHROMA` | constant | — | — | Lightness range and minimum chroma of a wallpaper's dominant color | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::ACCENT_SWATCHES` | constant | — | — | The eight named accent choices | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::MULTICOLOR` | constant | — | — | The rainbow stops of `ColorWell::multicolor` | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::CHART_SERIES` | constant | — | — | Seven chart series colors | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::LEVEL_INDICATOR_COLORS` | constant | — | — | Normal, warning and critical level colors | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::SHELL_LIGHT`, `SHELL_LIGHT_OPAQUE`, `SHELL_DARK`, `SHELL_DARK_OPAQUE`, `SHELL_DARK_TINT`, `SHELL_DARK_TINT_OPAQUE` | constant | — | — | Menu bar and Dock colors per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::THEME_SHADOW_OPACITY` | constant | — | — | Shadow opacity of the Freya shell theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::MENU_LIGHT`, `MENU_LIGHT_OPAQUE`, `MENU_DARK`, `MENU_DARK_OPAQUE`, `MENU_DARK_TINT`, `MENU_DARK_TINT_OPAQUE` | constant | — | — | Menu palettes per theme, translucent and opaque | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::SIDEBAR_LIGHT`, `SIDEBAR_LIGHT_OPAQUE`, `SIDEBAR_DARK`, `SIDEBAR_DARK_OPAQUE`, `SIDEBAR_DARK_TINT`, `SIDEBAR_DARK_TINT_OPAQUE` | constant | — | — | Flat sidebar background per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::SIDEBAR_ISLAND_LIGHT`, `SIDEBAR_ISLAND_DARK`, `SIDEBAR_ISLAND_DARK_TINT`, `SIDEBAR_ISLAND_RIM_LIGHT`, `SIDEBAR_ISLAND_RIM_DARK`, `SIDEBAR_ISLAND_RIM_DARK_TINT`, `SIDEBAR_ISLAND_INNER_LIGHT`, `SIDEBAR_ISLAND_INNER_DARK`, `SIDEBAR_ISLAND_INNER_DARK_TINT` | constant | — | — | Island sidebar background, rim and inner line per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::SIDEBAR_PALETTE_LIGHT`, `SIDEBAR_PALETTE_DARK`, `SIDEBAR_PALETTE_DARK_TINT` | constant | — | — | Sidebar inks per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::TOOLGROUP_BG_LIGHT`, `TOOLGROUP_BG_LIGHT_OPAQUE`, `TOOLGROUP_BG_DARK`, `TOOLGROUP_BG_DARK_OPAQUE`, `TOOLGROUP_BG_DARK_TINT`, `TOOLGROUP_BG_DARK_TINT_OPAQUE`, `TOOLGROUP_PALETTE_LIGHT`, `TOOLGROUP_PALETTE_DARK`, `TOOLGROUP_PALETTE_DARK_TINT` | constant | — | — | Toolbar group background and lines per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::WINDOW_MIN_SIZE`, `WINDOW_DEFAULT_SIZE` | constant | ≈ `.defaultSize(_:)` | ≈ `NSWindow.minSize` | Minimum and default window size: 480 × 360, 640 × 420 | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::WINDOW_CORNER`, `WINDOW_RESIZE` | constant | — | — | Window corner shape; resize band widths | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::DIALOG`, `DIALOG_PADDING`, `DIALOG_ALERT_WIDTH`, `POPUP`, `PANEL_GLOW_DEPTH` | constant | — | — | Dialog and pop-up corners, dialog padding, alert width, panel glow depth | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::SHADOW`, `SHADOW_DARK` | constant | — | ≈ `NSShadow` | Window shadows in light and dark | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::NATIVE_APP_ID_PREFIX`, `ALERT_LAYER_NAMESPACE`, `NOTIFICATIONS_LAYER_NAMESPACE`, `MENUBAR_LAYER_NAMESPACE`, `WALLPAPER_LAYER_NAMESPACE` | constant | — | — | App id prefix of kit apps; layer namespaces of shell alerts, notifications, the menu bar and the wallpaper | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::TITLEBAR`, `TITLEBAR_CLIENT_HEIGHT`, `TITLEBAR_PADDING_X`, `TITLEBAR_TITLE_GAP`, `TITLEBAR_DOUBLE_CLICK`, `TITLEBAR_RAIL` | constant | — | — | Titlebar sizes, lights, double click and rail | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::FULLSCREEN_BAR`, `FULLSCREEN_GLASS`, `FULLSCREEN_BAND`, `GLASS_MASK`, `HUD_BACKGROUND` | constant | — | — | Full-screen glass bars, their mask, HUD window background | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::TRAFFIC_GLYPHS`, `TRAFFIC_GLYPH_STANDARD`, `TRAFFIC_GLYPH_ON_DARK`, `TRAFFIC_GLYPH_ON_LIGHT`, `TRAFFIC_RIM_WIDTH`, `TRAFFIC_PRESSED_DARKEN`, `TRAFFIC_CLOSE_MINIMIZE_DARKEN`, `TRAFFIC_ZOOM_LIGHTEN` | constant | — | — | Traffic-light glyphs, inks, rim and color shifts | [4](04-appearance-and-tokens.md#traffic-lights) |
| `ctdesign::BUTTON`, `ICON_BUTTON`, `SWITCH`, `CHECKBOX`, `SLIDER`, `SLIDER_FAT`, `SLIDER_KEY_STEP`, `KNOB_SHADOW`, `SEGMENTED`, `STEPPER`, `TOOLBAR_SEARCH_WIDTH` | constant | — | — | Metrics of buttons, toggles, sliders and toolbar controls | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TEXT_FIELD`, `TOKEN_FIELD`, `SPELLING`, `COMBO_BOX`, `PATH_CONTROL`, `DATE_PICKER`, `IMAGE_WELL` | constant | — | — | Metrics of fields and pickers | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::LEVEL_INDICATOR`, `GAUGE`, `PROGRESS`, `BADGE`, `FOCUS_RING`, `SYMBOL`, `TOOLTIP`, `POPOVER`, `DRAG` | constant | — | — | Metrics of indicators, focus ring, symbols, tooltips, popovers, drag | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::MENU_POPOVER`, `MENU_MARK_TEXT_GAP`, `MENU_SUBMENU_INTENT_MS`, `MENU_SUBMENU_REVEAL_GAP`, `MENU_SCROLL_TICK_MS`, `MENU_SEARCH_ICON`, `MENU_SEARCH_ICON_GAP`, `MENU_SEARCH_CLEAR`, `MENU_SEARCH_CLEAR_GLYPH`, `MENU_SEARCH_CLEAR_MARGIN`, `MENU_SEARCH_CLEAR_MIX`, `MENU_HELP_ICON`, `MENU_SEPARATOR_INK_MIX` | constant | — | — | Menu and pop-up menu metrics and timing | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SIDEBAR`, `TOOLGROUP`, `SPLIT_VIEW`, `SPLIT_PANES`, `FLOATING_PANE`, `TABLE`, `COLLECTION`, `LIST_GROUP`, `GROUP_BOX`, `DISCLOSURE`, `TOOLBAR_TABS`, `SCROLL_BAR`, `BREAKPOINT` | constant | — | — | Metrics of sidebars, split views, lists, tables and groups | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::SETTINGS`, `FONT_PANEL`, `PRINT`, `CHART`, `MEDIA`, `BACKDROP`, `BACKDROP_SCENES` | constant | — | — | Metrics of settings, font panel, printing, charts, media, backdrops | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::MATERIAL` | constant | `Material` | `NSVisualEffectView.Material` | Opacity and blur of the five material thicknesses | [4](04-appearance-and-tokens.md#materials) |
| `ctdesign::MENUBAR_HEIGHT`, `MENUBAR_HEIGHT_WIDE`, `MENUBAR_OVER_LIGHT_INK`, `MENUBAR_OVER_LIGHT_GLASS`, `MENUBAR_OVER_DARK_INK`, `MENUBAR_OVER_DARK_GLASS`, `MENUBAR_FONT_SIZE`, `MENUBAR_FONT_SIZE_LARGE`, `MENUBAR_APP_NAME_WEIGHT`, `MENUBAR_MARGIN`, `MENUBAR_MARGIN_RIGHT`, `MENUBAR_FADED_ALPHA`, `MENUBAR_TITLE_PADDING_X`, `MENUBAR_TEXT_LIFT`, `MENUBAR_TITLE_MAX_WIDTH`, `MENUBAR_SYSTEM_MENU_MIN_WIDTH`, `MENUBAR_TITLE_BLEED`, `FOCUS_BAR_BACKGROUND`, `FOCUS_BAR_TEXT` | constant | — | — | Shell menu bar sizes; focus-mode bar colors | [4](04-appearance-and-tokens.md#shell) |
| `ctdesign::DOCK_TILESIZE`, `DOCK_GAP`, `DOCK_PADDING`, `DOCK_FONT_SIZE`, `DOCK_PROGRESS_HEIGHT`, `DOCK_BAR_HEIGHT` | constant | — | — | Shell Dock sizes | [4](04-appearance-and-tokens.md#shell) |
| `ctdesign::NOTIFICATION_WIDTH`, `NOTIFICATION_RADIUS`, `NOTIFICATION_PADDING`, `NOTIFICATION_GAP`, `NOTIFICATION_MARGIN`, `NOTIFICATION_ICON_SIZE`, `NOTIFICATION_TEXT_GAP`, `NOTIFICATION_CLOSE_SIZE`, `NOTIFICATION_STACK_PEEK`, `NOTIFICATION_STACK_INSET`, `NOTIFICATION_BODY_LINES`, `NOTIFICATION_MAX_BANNERS`, `NOTIFICATION_BANNER_MS`, `NOTIFICATION_SLIDE_IN_MS`, `NOTIFICATION_SLIDE_OUT_MS`, `NOTIFICATION_SLIDE_BOUNCE`, `NOTIFICATION_EDGE`, `NOTIFICATION_SWIPE`, `NOTIFICATION_WIDGET_SIZE`, `NOTIFICATION_WIDGET_RADIUS`, `NOTIFICATION_REPLY_HEIGHT`, `LIVE_ACTIVITY_WIDTH`, `LIVE_ACTIVITY_HEIGHT`, `LIVE_ACTIVITY_RADIUS`, `LIVE_ACTIVITY_GAP`, `LIVE_ACTIVITY_ENDED_MS` | constant | — | — | Notification banner, Notification Center, widget and Live Activity sizes, timing and slides | [4](04-appearance-and-tokens.md#shell) |
| `ctdesign::LAUNCHER_WIDTH`, `LAUNCHER_HEIGHT`, `LAUNCHER_PADDING`, `LAUNCHER_SEARCH_HEIGHT`, `LAUNCHER_RESULT_HEIGHT`, `LAUNCHER_RESULT_GAP`, `LAUNCHER_RESULT_RADIUS`, `LAUNCHER_ICON_SIZE`, `LAUNCHER_MAX_RESULTS`, `LAUNCHER_CORNER_RADIUS`, `LAUNCHER_FONT_SIZE`, `LAUNCHER_SUBTITLE_FONT_SIZE`, `LAUNCHER_SCALE_FROM`, `LAUNCHER_OPEN_MS`, `LAUNCHER_CLOSE_MS` | constant | — | — | Shell launcher sizes and animation | [4](04-appearance-and-tokens.md#shell) |
| `ctdesign::LOCK_CLOCK_FONT_SIZE`, `LOCK_DATE_FONT_SIZE`, `LOCK_TOP_PERCENT`, `LOCK_BOTTOM_MARGIN`, `LOCK_SPACING`, `LOCK_AVATAR_SIZE`, `LOCK_KEYBOARD_WIDTH`, `LOCK_FIELD_HEIGHT`, `LOCK_FIELD_MIN_WIDTH`, `LOCK_FIELD_MAX_WIDTH`, `LOCK_MASK_FONT_SIZE`, `LOCK_INK`, `LOCK_FIELD_FILL`, `LOCK_AVATAR_FILL`, `LOCK_FIELD_ERROR`, `LOCK_FIELD_INK` | constant | — | — | Lock screen sizes and colors over the wallpaper | [4](04-appearance-and-tokens.md#shell) |
| `ctdesign::LOCK_FADE_MS`, `LOCK_SETTLE_MS`, `LOCK_RISE_MS`, `LOCK_RISE_OFFSET`, `LOCK_LOGIN_MS` | constant | — | — | Timing of the transition from the desktop to the lock screen | [4](04-appearance-and-tokens.md#shell) |
| `ctdesign::ABOUT_WIDTH`, `ABOUT_HEIGHT`, `ABOUT_DRAWING_TOP`, `ABOUT_DRAWING_WIDTH`, `ABOUT_DRAWING_HEIGHT`, `ABOUT_TITLE_GAP`, `ABOUT_TITLE_SIZE`, `ABOUT_SPECS_GAP`, `ABOUT_SPEC_ROW_GAP`, `ABOUT_COLUMN_GAP`, `ABOUT_VALUE_WIDTH`, `ABOUT_BUTTON_GAP`, `ABOUT_FOOTER_BOTTOM` | constant | — | — | Sizes of the shell's «About this computer» window | [4](04-appearance-and-tokens.md#shell) |
| `ctdesign::ABOUT_APP_WIDTH`, `ABOUT_APP_HEIGHT`, `ABOUT_APP_ICON` | constant | — | — | Size of the shell's «About ‹app›» window and of its icon | [4](04-appearance-and-tokens.md#shell) |
| `ctdesign::CURSOR_SHAKE_SCALE`, `CURSOR_SHAKE_GROW_MS`, `CURSOR_SHAKE_SHRINK_MS`, `CURSOR_SHAKE_TICK_MS` | constant | — | — | Compositor: cursor growth when the pointer is shaken | [4](04-appearance-and-tokens.md#compositor) |
| `ctdesign::BACKGROUND_BLUR_PASSES`, `BACKGROUND_BLUR_OFFSET` | constant | — | ≈ `NSVisualEffectView` blur | Compositor: background blur strength (malaysia ADR 0071) | [4](04-appearance-and-tokens.md#compositor) |
| `ctdesign::SCREENSHOT_DIM`, `SCREENSHOT_IDLE_DIM`, `SCREENSHOT_BORDER`, `SCREENSHOT_BORDER_WIDTH` | constant | — | — | Compositor: screenshot selection overlay | [4](04-appearance-and-tokens.md#compositor) |
| `ctdesign::SPACE_GAP`, `SPACE_GAP_COLOR`, `SPACE_SLIDE_MS`, `SPACE_SLIDE_EASE` | constant | — | — | Compositor: gap and slide between Spaces | [4](04-appearance-and-tokens.md#compositor) |
| `ctdesign::TILE_MARGIN`, `TILE_GAP`, `DESKTOP_CLEAR` | constant | — | — | Compositor: tiled window margins; color behind everything | [4](04-appearance-and-tokens.md#compositor) |
| `ctdesign::WALLPAPER_FPS`, `WALLPAPER_THUMBNAIL`, `WALLPAPER_FRAME` | constant | — | — | Wallpaper: animation frame rate, thumbnail size, dynamic still size | [4](04-appearance-and-tokens.md#compositor) |

## Buttons

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `controls::Button` | struct | `Button` | `NSButton` | Push button with a text label | [5](05-controls.md#buttons) |
| `controls::Button::system_image` | fn | `Button(_:systemImage:)` | `NSButton(title:image:target:action:)` | Push button with a symbol before its label | [5](05-controls.md#buttons) |
| `controls::ControlSize` | enum | `ControlSize` | `NSControl.ControlSize` | `Small`, `Regular`, `Large` control heights | [5](05-controls.md#buttons) |
| `compact::IconButton` | struct | `≈ Button` | `≈ NSButton` | Button showing only a symbol; 36 capsule inside a tool group | [5](05-controls.md#compact-buttons) |
| `compact::Stepper` | struct | `Stepper` | `NSStepper` | Integer value with up and down arrows | [5](05-controls.md#compact-buttons) |
| `compact::ToggleGroup` | struct | — | `≈ NSSegmentedControl` | Row of independent formatting toggles (select-any tracking) | [5](05-controls.md#compact-buttons) |
| `compact::Face` | enum | — | — | `Bold`, `Italic`, `Underline`: how a toggle's text is drawn | [5](05-controls.md#compact-buttons) |
| `buttons::MenuButton` | struct | `Menu` | `≈ NSPopUpButton` | Button with a chevron that opens an action menu (pull-down) | [5](05-controls.md#menu-buttons) |
| `buttons::MenuButton::system_image` | function | `Menu(_:systemImage:)` | — | A menu button that shows only a symbol | [5](05-controls.md#menu-buttons) |
| `buttons::MenuButton::label_style` | method | `.labelStyle(_:)` | — | Shows the symbol alone or before the label | [5](05-controls.md#menu-buttons) |
| `buttons::LabelStyle` | enum | `LabelStyle` (`.iconOnly`, `.titleAndIcon`) | — | How a menu button with a symbol shows it | [5](05-controls.md#menu-buttons) |
| `buttons::ComboButton` | struct | `≈ Menu` | `NSComboButton` | Main action plus a chevron that opens a menu | [5](05-controls.md#menu-buttons) |
| `buttons::HelpLink` | struct | `HelpLink` | `≈ NSButton.BezelStyle.helpButton` | Round "?" help button | [5](05-controls.md#menu-buttons) |
| `buttons::ControlGroup` | struct | `ControlGroup` | — | Controls joined on one background with separators | [5](05-controls.md#menu-buttons) |
| `buttons::ShareLink` | struct | `ShareLink` | `≈ NSSharingServicePicker` | Menu to copy, email or open shared content; no share portal | [5](05-controls.md#menu-buttons) |
| `buttons::mail_uri` | fn | — | — | Builds a percent-encoded `mailto:?body=` URI | [5](05-controls.md#menu-buttons) |
| `controls::ToolGroup` | struct | `ToolbarItemGroup` | `NSToolbarItemGroup` | Toolbar items in 36 slots; island capsule with separators | [5](05-controls.md#toolbar-controls) |
| `compact::ToolbarSearch` | struct | `≈ .searchable(text:placement:prompt:)` | `NSSearchToolbarItem` | Toolbar search field, optionally collapsed to an icon | [5](05-controls.md#toolbar-controls) |
| `compact::ToolbarOverflow` | struct | — | — | Chevron button that lists extra toolbar items in a menu | [5](05-controls.md#toolbar-controls) |

## Toggles and choices

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `controls::Switch` | struct | `ToggleStyle.switch` | `NSSwitch` | On/off switch | [5](05-controls.md#switch-and-checkbox) |
| `controls::Checkbox` | struct | `ToggleStyle.checkbox` | `≈ NSButton` | Checkbox with its label (NSButton checkbox type) | [5](05-controls.md#switch-and-checkbox) |
| `choices::RadioGroup` | struct | `PickerStyle.radioGroup` | `≈ NSButton` | Column of radio options; arrows move the choice | [5](05-controls.md#choices) |
| `controls::SegmentedControl` | struct | `PickerStyle.segmented` | `NSSegmentedControl` | Segments filling the width; one is chosen | [5](05-controls.md#choices) |
| `popup_button::PopUpButton` | struct | `PickerStyle.menu` | `NSPopUpButton` | Shows the chosen item; opens the list as a menu | [5](05-controls.md#choices) |
| `popup_button::PopUpButton::section_header` | method | `Section(_:)` in a `Picker` | `NSMenuItem.sectionHeader(title:)` | Titles the items that follow, after a separator | [5](05-controls.md#choices) |
| `choices::ColorWell` | struct | `≈ PickerStyle.palette` | — | Row of preset color swatches; reports the chosen index | [5](05-controls.md#choices) |
| `choices::ColorWell::multicolor` | method | — | ≈ the «Multicolor» accent of System Settings | A rainbow swatch first, at index 0 | [5](05-controls.md#choices) |
| `choices::ChoiceCards` | struct | — | — | Large preview cards with captions; one is chosen | [5](05-controls.md#choices) |

## Sliders and indicators

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `controls::Slider` | struct | `Slider` | `NSSlider` | Stepped slider with ticks and optional step captions | [5](05-controls.md#sliders) |
| `level::LevelSlider` | struct | `Slider` | `NSSlider` | Continuous 0–1 slider, regular or fat with a symbol | [5](05-controls.md#sliders) |
| `progress::ProgressView` | struct | `ProgressView` | `NSProgressIndicator` | Determinate or indeterminate progress bar | [5](05-controls.md#progress) |
| `progress::ProgressRing` | struct | `ProgressViewStyle.circular` | `≈ NSProgressIndicator` | Determinate circular progress ring | [5](05-controls.md#progress) |
| `progress::Spinner` | struct | `≈ ProgressView` | `≈ NSProgressIndicator` | Indeterminate spinning indicator, spinning style | [5](05-controls.md#progress) |
| `indicator::CapacityIndicator` | struct | `≈ GaugeStyle.linearCapacity` | `≈ NSLevelIndicator` | Capacity bar, green-yellow-red by thresholds, continuous or discrete | [5](05-controls.md#level-indicators) |
| `indicator::RelevancyIndicator` | struct | — | `≈ NSLevelIndicator` | Relevancy style: thin lines proportional to the value | [5](05-controls.md#level-indicators) |
| `indicator::Gauge` | struct | `GaugeStyle.accessoryCircular` | — | Open ring with a marker; capacity mode is a filled ring | [5](05-controls.md#level-indicators) |
| `indicator::RatingIndicator` | struct | — | `≈ NSLevelIndicator` | Five-star rating, editable when it has `on_change` | [5](05-controls.md#level-indicators) |
| `indicator::level_color` | fn | — | — | Index 0–2 of the capacity color for a value | [5](05-controls.md#level-indicators) |
| `indicator::gauge_marker` | fn | — | — | Center of the gauge marker for a `GaugeStyle` and a value | [5](05-controls.md#level-indicators) |

## Text fields and pickers

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `controls::TextField` | struct | `TextField` | `NSTextField` | One-line field bound to a `State<String>`, with spell checking | [5](05-controls.md#text-fields) |
| `controls::TextField::on_submit` | method | `.onSubmit(_:)` | `NSTextField.action` | Runs when the user presses Enter, with the text | [5](05-controls.md#text-fields) |
| `controls::SearchField` | struct | `≈ .searchable(text:tokens:placement:prompt:token:)` | `NSSearchField` | Search field with clear button and optional tokens | [5](05-controls.md#text-fields) |
| `controls::SecureField` | struct | `SecureField` | `NSSecureTextField` | Masked password field; Enter submits the secret | [5](05-controls.md#text-fields) |
| `controls::SecureField::revealed` | method | — | — | Shows the typed text instead of dots | [5](05-controls.md#text-fields) |
| `combo_box::ComboBox` | struct | — | `NSComboBox` | Text field with an arrow that opens suggested values | [5](05-controls.md#text-fields) |
| `password::PasswordField` | struct | — | — | Zeroizing password buffer with idle, verifying and error states | [5](05-controls.md#passwords) |
| `password::State` | enum | — | — | `Idle`, `Verifying`, `Error(String)` of a password field | [5](05-controls.md#passwords) |
| `color_picker::ColorPicker` | struct | `ColorPicker` | `≈ NSColorWell` | Color well opening an HSB popover, not a color panel | [5](05-controls.md#color-picker) |
| `color_picker::hsb_to_rgb` | fn | — | `≈ NSColor.init(hue:saturation:brightness:alpha:)` | Opaque color from hue, saturation and brightness | [5](05-controls.md#color-picker) |
| `color_picker::rgb_to_hsb` | fn | — | `≈ NSColor.getHue(_:saturation:brightness:alpha:)` | Hue, saturation and brightness of a color | [5](05-controls.md#color-picker) |
| `color_picker::hex` | fn | — | — | `#RRGGBB` string of a color | [5](05-controls.md#color-picker) |
| `date_picker::DatePicker` | struct | `DatePicker` | `NSDatePicker` | Segmented date/time field with stepper and pop-up calendar | [5](05-controls.md#date-picker) |
| `date_picker::DateElements` | enum | `DatePickerComponents` | `NSDatePicker.ElementFlags` | `Date`, `Time` or `DateAndTime` parts shown | [5](05-controls.md#date-picker) |
| `date::CivilDate` | struct | — | — | Calendar date with day and month arithmetic | [5](05-controls.md#date-picker) |
| `date::CivilDateTime` | struct | — | — | Calendar date plus hour and minute | [5](05-controls.md#date-picker) |
| `date::DatePart` | enum | — | — | `Day`, `Month`, `Year`, `Hour`, `Minute` | [5](05-controls.md#date-picker) |
| `date::days_in_month` | fn | — | — | Days in a month, with leap years | [5](05-controls.md#date-picker) |
| `date::typing_is_done` | fn | — | — | Whether typed digits complete a date part | [5](05-controls.md#date-picker) |
| `path_control::PathControl` | struct | — | `NSPathControl` | Path parts with icons separated by chevrons | [5](05-controls.md#path-control) |
| `image_well::ImageWell` | struct | — | `≈ NSImageView` | Editable image well: drop, choose or clear an image | [5](05-controls.md#image-well) |
| `image_well::is_image` | fn | — | — | Whether a path has an image extension | [5](05-controls.md#image-well) |

## Adornments, scrolling and drawing

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `adornments::Badge` | struct | `≈ .badge(_:)` | — | Red capsule counter | [5](05-controls.md#adornments) |
| `adornments::Divider` | struct | `Divider` | `≈ NSBox` | Hairline separator filling the width | [5](05-controls.md#adornments) |
| `adornments::Avatar` | struct | — | — | Circle with initials | [5](05-controls.md#adornments) |
| `adornments::Link` | struct | `Link` | — | Pressable text in the link color | [5](05-controls.md#adornments) |
| `controls::ScrollView` | struct | `ScrollView` | `NSScrollView` | Scrolling viewport with overlay bar and edge effect | [5](05-controls.md#scrolling) |
| `controls::ScrollController` | struct | `≈ ScrollViewProxy` | `≈ NSClipView` | Scrolls a `ScrollView` from code | [5](05-controls.md#scrolling) |
| `controls::use_scroll_controller` | fn | `≈ ScrollViewReader` | — | Creates a `ScrollController` | [5](05-controls.md#scrolling) |
| `drag::DragSource` | struct | `≈ .draggable(_:)` | `NSDraggingSource` | Starts a drag to another app past a threshold | [5](05-controls.md#drag-source) |
| `drag::use_drag_source` | fn | `≈ .draggable(_:)` | — | Creates a `DragSource` | [5](05-controls.md#drag-source) |
| `drag::DragPayload` | enum | — | `≈ NSDraggingItem` | `Files(Vec<PathBuf>)` or `Text(String)` to drag | [5](05-controls.md#drag-source) |
| `drag::URI_LIST` | constant | — | `≈ NSPasteboard.PasteboardType.fileURL` | MIME type `text/uri-list` | [5](05-controls.md#drag-source) |
| `canvas::Canvas` | struct | `Canvas` | `≈ NSView` | Element that paints with Skia, repainting on a new revision | [5](05-controls.md#canvas) |
| `canvas::CanvasContext` | struct | `GraphicsContext` | `NSGraphicsContext` | Skia canvas, logical size, scale and fonts for a painter | [5](05-controls.md#canvas) |
| `interaction::Interaction` | struct | — | `≈ NSTrackingArea` | Hover, press and focus state of a custom control | [5](05-controls.md#building-a-new-control) |
| `interaction::use_interaction` | fn | `≈ .onHover(perform:)` | — | Creates an `Interaction` | [5](05-controls.md#building-a-new-control) |
| `modifiers::ViewModifiers::focus_ring` | method | — | `≈ NSView.drawFocusRingMask()` | Ring of a given color around an element with children | [7](07-menus-and-popovers.md#focus_ring) |

## Text editing

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `text_editor::TextEditor` | struct | `TextEditor` | `NSTextView` | Multi-line editor with find bar, rich text and spelling | [6](06-text-and-editing.md#text-editor) |
| `text_editor::find_matches` | fn | — | `≈ NSTextFinder` | Case-insensitive, non-overlapping matches in character indices | [6](06-text-and-editing.md#find-bar) |
| `rich_text::RunStyle` | struct | — | — | Bold, italic, link, family and size of a run | [6](06-text-and-editing.md#rich-text) |
| `rich_text::StyleRun` | struct | — | — | A `RunStyle` over a character range | [6](06-text-and-editing.md#rich-text) |
| `rich_text::FontChoice` | struct | `≈ Font` | `≈ NSFontDescriptor` | Family, size, bold and italic | [6](06-text-and-editing.md#rich-text) |
| `rich_text::Emphasis` | enum | — | `≈ NSFontDescriptor.SymbolicTraits` | `Bold` or `Italic` | [6](06-text-and-editing.md#rich-text) |
| `rich_text::char_styles` | fn | — | — | One style per character from runs | [6](06-text-and-editing.md#rich-text) |
| `rich_text::runs_from` | fn | — | — | Runs from per-character styles, merging neighbors | [6](06-text-and-editing.md#rich-text) |
| `rich_text::restyle_after_edit` | fn | — | — | Moves runs after an edit; typed text inherits, links do not | [6](06-text-and-editing.md#rich-text) |
| `rich_text::toggle` | fn | — | — | Toggles bold or italic on a range | [6](06-text-and-editing.md#rich-text) |
| `rich_text::font_at` | fn | — | — | Font at a character, `BODY` when unset | [6](06-text-and-editing.md#rich-text) |
| `rich_text::apply_font` | fn | — | `≈ NSFontManager.convert(_:)` | Applies a font choice to a range | [6](06-text-and-editing.md#rich-text) |
| `rich_text::link_at` | fn | — | — | Link at a character, if any | [6](06-text-and-editing.md#rich-text) |
| `font_panel::FontPicker` | struct | — | `≈ NSFontPanel` | Button with family and size that opens the font panel | [6](06-text-and-editing.md#font-panel) |
| `font_panel::faces` | fn | — | — | Typefaces a family offers; Regular always | [6](06-text-and-editing.md#font-panel) |
| `font_panel::parse_size` | fn | — | — | Typed font size if finite and positive | [6](06-text-and-editing.md#font-panel) |
| `font_panel::remember` | fn | — | — | Moves a family to the front of the recent list | [6](06-text-and-editing.md#font-panel) |
| `spelling::words` | fn | — | — | Character ranges of words; apostrophes join letters | [6](06-text-and-editing.md#spelling) |
| `spelling::misspelled` | fn | — | `≈ NSSpellChecker.checkSpelling(of:startingAt:)` | Ranges of words the checker does not know | [6](06-text-and-editing.md#spelling) |
| `spelling::word_at` | fn | — | — | Word range containing or touching a character | [6](06-text-and-editing.md#spelling) |
| `spelling::replace_chars` | fn | — | — | Text with a character range replaced | [6](06-text-and-editing.md#spelling) |
| `spelling::dictionary_name` | fn | — | `≈ NSSpellChecker.setLanguage(_:)` | Hunspell dictionary name for a language tag | [6](06-text-and-editing.md#spelling) |
| `spelling::personal_words` | fn | — | — | Words of an Enchant personal list | [6](06-text-and-editing.md#spelling) |
| `token_field::TokenField` | struct | — | `NSTokenField` | Tags as capsules; Enter or comma commits | [6](06-text-and-editing.md#token-field) |
| `token_field::TokenKey` | enum | — | — | `Backspace`, `Delete`, `Left`, `Right` on an empty input | [6](06-text-and-editing.md#token-field) |
| `token_field::commit_token` | fn | — | — | Pushes trimmed draft as a token; false if empty | [6](06-text-and-editing.md#token-field) |
| `token_field::token_selection` | fn | — | — | Next selected token and removed index for a key | [6](06-text-and-editing.md#token-field) |

## Printing

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `print::PageSetup` | struct | — | `≈ NSPrintInfo` | Page size and margins in millimetres from the portal | [6](06-text-and-editing.md#printing) |
| `print::PrintPreview` | struct | — | — | Column of rendered pages with page numbers | [6](06-text-and-editing.md#printing) |
| `print::text_pdf` | fn | — | `≈ NSView.dataWithPDF(inside:)` | Plain text as a searchable, font-subset PDF 1.7 | [6](06-text-and-editing.md#printing) |
| `print::text_pages` | fn | — | — | Plain text paginated as page images | [6](06-text-and-editing.md#printing) |
| `print::page_starts` | fn | — | — | Page tops that never split a line | [6](06-text-and-editing.md#printing) |
| `print::page_of` | fn | — | — | Page index holding a given line top | [6](06-text-and-editing.md#printing) |

## Rule editor

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `rule_editor::RuleEditor` | struct | — | `≈ NSPredicateEditor` | Edits a predicate as field, operator and value rows | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::RuleField` | struct | — | `≈ NSPredicateEditorRowTemplate` | A filterable field: label and kind | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::RuleKind` | enum | — | — | `Text`, `Number` or `Choice(&[..])` | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Operator` | enum | — | — | `Contains`, `BeginsWith`, `EndsWith`, `Is`, `IsNot`, `Greater`, `Less` | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::operators` | fn | — | — | Operators offered for a field kind | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Rule` | struct | — | — | One condition: field, operator and value | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Criterion` | enum | — | — | `Rule(Rule)` or a nested `Group` | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Predicate` | struct | — | — | All or any of its criteria; evaluates an item | [6](06-text-and-editing.md#rule-editor) |

## Menus

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `menu::MenuView` | struct | — | `≈ NSMenu` | Draws a list of menu items with keyboard, search and submenus | [7](07-menus-and-popovers.md#menu-view) |
| `menu::MenuItem` | enum | — | `NSMenuItem` | A menu row: `Action`, `Submenu`, `Header`, `Separator`, `Palette`, `SearchField`, `SearchResult` | [7](07-menus-and-popovers.md#menu-items) |
| `menu::CheckState` | enum | — | `≈ NSControl.StateValue` | Mark of an action: `None`, `Checkbox(bool)`, `Radio(bool)`; no mixed state | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuIcon` | enum | — | `≈ NSMenuItem.image` | Row icon: `Svg`, `Image` (encoded bytes, 64 KiB max) or `Swatch` color circle | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuNavigation` | enum | — | — | `Prev` or `Next`: ← or → asks the menu bar for a neighbouring menu | [7](07-menus-and-popovers.md#menu-view) |
| `menu::ItemsSource` | type | — | — | Closure that returns a menu's items on every render | [7](07-menus-and-popovers.md#helpers) |
| `menu::SubmenuSource` | type | — | `≈ NSMenuDelegate.menuNeedsUpdate(_:)` | Closure that returns a lazy submenu's items by id, once loaded | [7](07-menus-and-popovers.md#submenus) |
| `menu::submenu_contents` | fn | — | — | A "Loading…" row while a submenu is loading and empty | [7](07-menus-and-popovers.md#submenus) |
| `menu::normalize_separators` | fn | — | — | Removes leading, trailing and repeated separators, also in submenus | [7](07-menus-and-popovers.md#helpers) |
| `menu::MNEMONIC_MARK` | constant | — | `≈ NSMenuItem.mnemonic` | U+2063 placed before the letter that Alt underlines | [7](07-menus-and-popovers.md#mnemonics) |
| `menu::strip_mnemonic` | fn | — | — | Returns a label without the mnemonic mark | [7](07-menus-and-popovers.md#mnemonics) |
| `menu::format_shortcut` | fn | — | — | Turns shortcut text such as `"Ctrl+Shift+G"` into glyphs `"⌃⇧G"` | [7](07-menus-and-popovers.md#shortcuts) |
| `menu::shortcut_matches` | fn | — | — | Whether a key event with modifiers is the given shortcut text | [7](07-menus-and-popovers.md#shortcuts) |
| `menu::measure_text` | fn | — | — | Width in pixels of text in the system sans font | [7](07-menus-and-popovers.md#helpers) |
| `menu::measure_text_weight` | fn | — | — | Width in pixels of text at a given font weight | [7](07-menus-and-popovers.md#helpers) |

## Menu search

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `menu::SearchableEntry` | struct | — | `≈ NSUserInterfaceItemSearching` | One findable item: path, item, location and source | [7](07-menus-and-popovers.md#search) |
| `menu::SearchSource` | enum | — | — | Where a search entry comes from: `Menu` or `Help` | [7](07-menus-and-popovers.md#search) |
| `menu::SearchLocation` | struct | — | — | Top menu index and row indices that lead to a found item | [7](07-menus-and-popovers.md#search) |
| `menu::SearchQuery` | struct | — | — | Search text and whether it is selected, shared between menu rebuilds | [7](07-menus-and-popovers.md#search) |
| `menu::SearchableSource` | type | — | — | Closure that returns the searchable entries | [7](07-menus-and-popovers.md#search) |
| `menu::RevealRoots` | type | — | — | Closure that returns every top menu's items, to reveal found items | [7](07-menus-and-popovers.md#search) |
| `menu::normalize_query` | fn | — | — | Prepares text for search: no mnemonic, trimmed, lower case, no accents | [7](07-menus-and-popovers.md#search) |
| `menu::search_icon_svg` | fn | — | — | Fills `{{text}}` and `{{accent}}` colors in a search icon SVG | [7](07-menus-and-popovers.md#search) |

## Context menus, popovers and tooltips

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `modifiers::ViewModifiers` | trait | `View` modifiers | — | SwiftUI-style modifiers for every element with events | [7](07-menus-and-popovers.md#view-modifiers) |
| `modifiers::ViewModifiers::context_menu` | method | `.contextMenu(menuItems:)` | `NSMenu.popUpContextMenu(_:with:for:)` | Opens a menu at the pointer on secondary click | [7](07-menus-and-popovers.md#context_menu) |
| `popover::Popover` | struct | `.popover(isPresented:attachmentAnchor:arrowEdge:content:)` | `NSPopover` | Content in a popover with an arrow; `.show(anchor)` opens it below an area | [7](07-menus-and-popovers.md#popover) |
| `modifiers::ViewModifiers::help` | method | `.help(_:)` | `NSView.toolTip` | Tooltip with a fixed text after a 1 s hover | [7](07-menus-and-popovers.md#help-tooltips) |

## Windows

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `platform::run_window` | fn | `≈ App` | `≈ NSApplicationMain(_:_:)` | Opens a window with a root view and runs the event loop | [8](08-windows.md#running-a-window) |
| `platform::WindowOptions::theme` | method | ≈ `.environment(_:_:)` on the window | ≈ `NSAppearanceCustomization.appearance` | Gives one window its `Theme` | [8](08-windows.md#window-options) |
| `platform::WindowOptions` | struct | `≈ Window` | — | Title, name, app id, size, limits, menu and full-screen options of a window | [8](08-windows.md#window-options) |
| `platform::WindowControls` | struct | — | `≈ NSWindow` | Window state and commands: close, minimize, zoom, full screen, move, resize | [8](08-windows.md#window-controls) |
| `platform::WindowStatus` | struct | — | — | Configure state: activated, maximized, fullscreen, decoration, fixed | [8](08-windows.md#window-state) |
| `platform::DecorationResponse` | enum | — | — | Who draws the title bar: `Client` or `Server` | [8](08-windows.md#window-state) |
| `platform::WindowEdge` | enum | — | — | Edge or corner for a compositor resize | [8](08-windows.md#resizing-from-the-edges) |
| `app::UtilityWindow` | struct | `UtilityWindow` | `≈ NSPanel` | A floating panel child of the current window; `.open()` shows it | [8](08-windows.md#utility-windows) |
| `platform::PanelBehavior` | struct | — | `≈ NSPanel` | Panel flags: hides on deactivate, floating, key only if needed | [8](08-windows.md#utility-windows) |
| `platform::FullscreenBar` | enum | — | — | Overlaid full-screen title bar: `Solid`, `Glass`, `ProgressiveGlass` | [9](09-titlebars.md#the-bar-in-full-screen-fullscreenbar) |
| `platform::Bar` | enum | — | `≈ NSWindow.ToolbarStyle` | Bar the window draws, for its corner: `Titlebar`, `UnifiedCompact`, `Unified`, `Utility` | [9](09-titlebars.md#corner-radius) |
| `platform::MainMenuAddress` | struct | — | — | D-Bus service and path of a window's menu bar menu | [8](08-windows.md#menu-address) |
| `platform::ObjectId` | struct | — | — | Wayland object id, re-exported from `wayland-backend` | [8](08-windows.md#menu-address) |
| `platform::RunError` | enum | — | — | Why a window cannot run: display, protocol, Vulkan, event loop, I/O | [8](08-windows.md#errors) |

## Popups

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `platform::Popups` | struct | — | — | Opens and closes `xdg_popup`s from a window or popup | [8](08-windows.md#popups) |
| `platform::PopupSpec` | struct | — | — | Popup position, size, constraint adjustment, grab and dismiss callback | [8](08-windows.md#popups) |
| `platform::PopupAnchor` | enum | — | — | Point of the anchor rectangle a popup attaches to; eight values | [8](08-windows.md#popups) |
| `platform::PopupGravity` | enum | — | — | Direction a popup grows from its anchor point; eight values | [8](08-windows.md#popups) |
| `platform::PopupConstraintAdjustment` | struct | — | — | Flip, slide or resize flags for a popup that does not fit | [8](08-windows.md#popups) |
| `platform::PopupId` | struct | — | — | Identifies an open popup | [8](08-windows.md#popups) |
| `platform::PopupHandle` | struct | — | — | A popup's own handle: close, reposition, open a child | [8](08-windows.md#popups) |

## Timers and input

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `platform::Timers` | struct | — | — | Runs callbacks on the UI thread after a delay or on wake | [8](08-windows.md#timers) |
| `platform::WakeHandle` | struct | — | — | `Send` handle another thread uses to run wake callbacks | [8](08-windows.md#timers) |
| `platform::inserts_text` | fn | — | — | Whether a key press types text: no Control, Alt or Super | [8](08-windows.md#input-helpers) |

## Titlebars

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `titlebar::Titlebar` | struct | `≈ .toolbar(content:)` | `≈ NSToolbar` | Client-side title bar: traffic lights, left and right items, title | [9](09-titlebars.md#titlebar) |
| `traffic::TrafficLights` | struct | — | `≈ NSWindow.standardWindowButton(_:)` | Close, minimize and zoom buttons | [9](09-titlebars.md#traffic-lights) |
| `traffic::TrafficLights::for_window` | fn | — | `≈ NSWindow.standardWindowButton(_:)` | Traffic lights wired to the window and user settings | [9](09-titlebars.md#traffic-lights) |
| `titlebar::window_drag_handler` | fn | — | `≈ NSWindow.performDrag(with:)` | Press handler that moves the window or runs the double-click action | [9](09-titlebars.md#traffic-lights) |
| `titlebar::sidebar_bar_metrics` | fn | — | — | Height and top padding of the title bar row beside a sidebar | [9](09-titlebars.md#the-sidebar-with-the-rail) |
| `titlebar::app_id` | fn | — | — | Default Wayland app id for an app name | [9](09-titlebars.md#helpers) |
| `chrome::WindowChrome` | struct | — | — | Wraps the window and places lights, title and rail items | [9](09-titlebars.md#window-chrome) |
| `chrome::TitlebarLayout` | enum | — | — | `Top` horizontal title bar or `Rail` vertical bar on the left | [9](09-titlebars.md#windowchromestyle-layout-fields) |
| `chrome::RailStyle` | enum | — | — | `Normal` own strip or `Unified` column inside the sidebar | [9](09-titlebars.md#windowchromestyle-layout-fields) |
| `chrome::Reveal` | enum | — | — | When the bar shows: `Always`, `Drawer`, `Push`, `Corner`, `Popup` | [9](09-titlebars.md#reveal) |
| `chrome::RestingEdge` | enum | — | — | What a hidden bar leaves: `None`, `Line`, `Accent` | [9](09-titlebars.md#windowchromestyle-layout-fields) |

## Sidebar and split view

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `sidebar::Sidebar` | struct | `SidebarListStyle` | `NSTableView.Style.sourceList` | The navigation column; island or flat by the user's `sidebar_style` | [10](10-sidebar.md#sidebar) |
| `sidebar::SidebarItem` | struct | ≈ `Label` | — | A sidebar row with symbol tile and label; selection set by the app | [10](10-sidebar.md#sidebaritem) |
| `sidebar::SidebarSection` | struct | `Section` | — | A titled, collapsible group of sidebar rows | [10](10-sidebar.md#sidebarsection) |
| `sidebar::SidebarAccount` | struct | — | — | The account row: avatar with initials, name and a detail line | [10](10-sidebar.md#sidebaraccount) |
| `sidebar::SidebarBackdrop` | enum | — | — | Island look over a tinted backdrop: `Clear` or `Tinted` | [10](10-sidebar.md#sidebarbackdrop) |
| `sidebar::SidebarControl` | struct | ≈ `NavigationSplitViewVisibility` | ≈ `NSSplitViewController.toggleSidebar(_:)` | Handle to toggle, dismiss and read the enclosing `SplitView`'s sidebar | [10](10-sidebar.md#sidebarcontrol) |
| `sidebar::sidebar_control` | fn | — | — | The enclosing `SplitView`'s `SidebarControl`, or `None` outside one | [10](10-sidebar.md#sidebarcontrol) |
| `sidebar::SymbolTile` | struct | — | — | The colored square tile with a white symbol; `.rim(color)` adds a border | [10](10-sidebar.md#symboltile) |
| `views::split::SplitView` | struct | `NavigationSplitView` | `NSSplitViewController` | Sidebar, detail and optional inspector; resizes, hides and collapses the sidebar | [10](10-sidebar.md#splitview) |
| `views::split::SidebarLayout` | struct | — | — | Sidebar width and visibility reported by `on_layout`, serializable | [10](10-sidebar.md#sidebarlayout) |
| `views::split::NarrowSidebar` | enum | — | — | Where the sidebar opens in narrow windows: `Overlay` or `Push` | [10](10-sidebar.md#narrow-windows) |
| `views::split::SidebarToggle` | struct | — | `NSToolbarItem.Identifier.toggleSidebar` | The titlebar button that shows or hides the sidebar | [10](10-sidebar.md#hiding-sidebartoggle-and-toggle_sidebar) |
| `views::split::TOGGLE_SIDEBAR` | constant | — | `NSSplitViewController.toggleSidebar(_:)` | The show-or-hide-sidebar action, Ctrl+Alt+S | [10](10-sidebar.md#hiding-sidebartoggle-and-toggle_sidebar) |

## Forms and groups

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `views::form::Form` | struct | `Form` | — | Settings layout: sections of rows in rounded cards | [11](11-views.md#form) |
| `views::form::Section` | struct | `Section` | — | One form card with optional title and footer | [11](11-views.md#section) |
| `views::form::Section::header` | method | `Section(content:header:)` | — | A view in place of a form card's title | [11](11-views.md#section) |
| `views::form::Section::illustration` | method | — | — | A full-width view at the top of a form card | [11](11-views.md#section) |
| `views::form::Row` | struct | `LabeledContent` | — | A form row: control, value, navigation or destructive | [11](11-views.md#row) |
| `views::form::Row::page` | function | ≈ `NavigationLink(value:)` | — | A row that opens a page of a `SettingsWindow` | [11](11-views.md#row) |
| `views::form::Row::symbol` | method | ≈ `Label(_:systemImage:)` | — | A `SymbolTile` before the row title | [11](11-views.md#row) |
| `views::form::Row::icon` | method | `Label(title:icon:)` | — | Any view before the row title | [11](11-views.md#row) |
| `views::form::Row::hover_control` | method | ≈ `.onHover` | — | A control shown only while the pointer is over the row | [11](11-views.md#row) |
| `views::form::Row::stacked` | method | ≈ `.pickerStyle(.radioGroup)` under its label | — | The control goes under the title at any width | [11](11-views.md#row) |
| `views::group::GroupBox` | struct | `GroupBox` | `NSBox` | A rounded box with an optional label that groups controls | [11](11-views.md#groupbox) |
| `views::group::DisclosureGroup` | struct | `DisclosureGroup` | — | A chevron header that shows or hides its indented children | [11](11-views.md#disclosuregroup) |

## Data views

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `views::table::Table` | struct | `Table` | `NSTableView` | Text rows in sortable, resizable, hideable columns with multiple selection | [11](11-views.md#table) |
| `views::table::SortOrder` | struct | — | ≈ `NSTableView.sortDescriptors` | The sorted column index and direction; one column, not a list | [11](11-views.md#sortorder) |
| `views::table::next_sort` | fn | — | — | The order a header click asks for: flip or start ascending | [11](11-views.md#table-helpers) |
| `views::table::shown_columns` | fn | — | — | Column indices not hidden | [11](11-views.md#table-helpers) |
| `views::table::toggle_column` | fn | — | — | Hides or shows a column, keeping at least one visible | [11](11-views.md#table-helpers) |
| `views::collection::CollectionView` | struct | ≈ `LazyVGrid` | `NSCollectionView` | A wrapping grid of icon tiles with names and multiple selection | [11](11-views.md#collectionview) |
| `views::collection::select` | fn | — | — | Click selection rule: plain, Ctrl/Super toggle, Shift range | [11](11-views.md#select) |
| `views::tree::TreeNode` | struct | — | ≈ `NSTreeNode` | A node with title, optional icon and children | [11](11-views.md#treenode) |
| `views::tree::OutlineView` | struct | `OutlineGroup` | `NSOutlineView` | A tree as indented rows with disclosure chevrons | [11](11-views.md#outlineview) |
| `views::tree::Browser` | struct | — | `NSBrowser` | A tree as Finder-style columns | [11](11-views.md#browser) |
| `views::tree::visible_rows` | fn | — | — | Path and depth of each row shown for a set of open paths | [11](11-views.md#tree-helpers) |
| `views::tree::node_at` | fn | — | — | The node at a path, or `None` | [11](11-views.md#tree-helpers) |

## Containers

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `views::tabs::TabView` | struct | `TabView` | `NSTabView` | Pages chosen from a strip of icon-over-title tabs | [11](11-views.md#tabview) |
| `views::window_tabs::WindowTabs` | struct | — | ≈ `NSWindowTabGroup` | The tab bar of a tabbed window; the app keeps each tab's content | [11](11-views.md#windowtabs) |
| `views::split_panes::SplitPanes` | struct | `HSplitView`, `VSplitView` | `NSSplitView` | Panes in a row or column with draggable dividers; controlled ratios | [11](11-views.md#splitpanes) |
| `views::floating_pane::FloatingPane` | struct | — | ≈ `NSPanel` | A movable, resizable pane drawn inside the window, not a separate window | [11](11-views.md#floatingpane) |
| `views::floating_pane::PaneFrame` | struct | — | — | Position and size of a `FloatingPane` | [11](11-views.md#paneframe) |
| `views::text_view::TextView` | struct | ≈ `TextEditor` | `NSTextView` | Frameless multi-line editable text without scrolling | [11](11-views.md#textview) |
| `views::text_view::KeyPress` | struct | `KeyPress` | ≈ `NSEvent` | Key, modifiers and caret position passed to `TextView::on_key` | [11](11-views.md#keypress) |

## Feedback

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `views::feedback::ContentUnavailableView` | struct | `ContentUnavailableView` | — | A faint symbol over a line of text for an empty view | [11](11-views.md#contentunavailableview) |
| `views::feedback::Sheet` | struct | `.sheet(isPresented:onDismiss:content:)` | `NSWindow.beginSheet(_:completionHandler:)` | A modal child window centered on its window (`xdg-dialog-v1`); the parent takes no clicks and cannot close while it shows | [11](11-views.md#sheet) |
| `views::feedback::Alert` | struct | `.alert(_:isPresented:actions:message:)` | `NSAlert` | A sheet with title, message, two buttons and optional suppression checkbox | [11](11-views.md#alert) |

## Navigation and search

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `views::navigation::History` | struct | — | — | A back-and-forward list of pages | [11](11-views.md#history) |
| `views::navigation::NavigationButtons` | struct | — | — | Back and forward buttons in a tool group, with Alt+arrow shortcuts | [11](11-views.md#navigationbuttons) |
| `views::navigation::GO_BACK`, `views::navigation::GO_FORWARD` | constant | — | — | The Back (Alt+←) and Forward (Alt+→) actions | [11](11-views.md#go_back-and-go_forward) |
| `views::search::SearchIndex` | struct | — | — | Entries searched by title and keywords, ignoring case and accents | [11](11-views.md#searchindex) |
| `views::search::Entry` | struct | — | — | One search entry: key, title and keywords | [11](11-views.md#entry) |
| `views::search::fold` | fn | — | — | Lowercases text and removes accents for matching | [11](11-views.md#search-helpers) |
| `views::search::word_start` | fn | — | — | Whether the query starts a word other than the first | [11](11-views.md#search-helpers) |

## Adaptive layout

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `adaptive::use_size_class` | fn | ≈ `EnvironmentValues.horizontalSizeClass` | — | Creates a width reader for an element | [11](11-views.md#use_size_class) |
| `adaptive::SizeClassReader` | struct | — | — | The width reader: `attach` measures, `current` gives the breakpoint | [11](11-views.md#use_size_class) |
| `adaptive::SizeClass` | enum | ≈ `UserInterfaceSizeClass` | — | Width class: `Compact`, `Medium`, `Wide`; three classes, not two | [11](11-views.md#sizeclass) |

## Patterns

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `patterns::settings::SettingsWindow` | struct | ≈ `Settings` | — | Settings window with pane sidebar, search, history and saved state | [12](12-patterns.md#settingswindow) |
| `patterns::settings::Pane` | struct | — | — | One preferences pane: id, title, symbol, tint and form | [12](12-patterns.md#pane) |
| `patterns::settings::Pane::page` | method | ≈ `navigationDestination` | — | A page without a sidebar item, opened by `Row::page` | [12](12-patterns.md#pane) |
| `patterns::settings::PaneGroup` | struct | — | — | Panes under an optional sidebar section title | [12](12-patterns.md#panegroup) |
| `patterns::settings::index` | fn | — | — | Builds the search index over panes and their rows | [12](12-patterns.md#index) |
| `patterns::about::AboutView` | struct | — | ≈ `NSApplication.orderFrontStandardAboutPanel(_:)` | Content of an About window: icon, name, copyright, optional version | [12](12-patterns.md#aboutview) |
| `patterns::language::LanguagePicker` | struct | — | — | Pop-up that writes the app or system language to a domain file | [12](12-patterns.md#languagepicker) |
| `patterns::language::app_language_row` | fn | — | — | A "Language" form row with the app's picker | [12](12-patterns.md#app_language_row-and-system_language_row) |
| `patterns::language::system_language_row` | fn | — | — | A "Language" form row with the system-wide picker | [12](12-patterns.md#app_language_row-and-system_language_row) |
| `patterns::language::write_language` | fn | — | — | Sets or removes the `language` key in a TOML file | [12](12-patterns.md#write_language) |

## App and scenes

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `app::Application` | struct | `App` | `NSApplication` | The app; its name gives the `app_id` and saved window size | [2](02-first-app.md#what-application-does-for-you) |
| `app::Scene` | struct | `Scene`, `WindowGroup` | — | One window with a titlebar and centered content; `.options`, `.run` | [2](02-first-app.md#what-application-does-for-you) |

## Actions and shortcuts

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `app::Action` | struct | — | ≈ `NSMenuItem` | A command: id, translated title and shortcut, shared by menus and keys | [13](13-app.md#actions-and-shortcuts) |
| `app::Shortcut` | struct | `KeyboardShortcut` | ≈ `NSMenuItem.keyEquivalent` | Modifiers plus one key; gives the menu text and matches presses | [13](13-app.md#actions-and-shortcuts) |
| `app::ShortcutKey` | enum | `KeyEquivalent` | — | The key of a shortcut: `Char`, `Left`, `Right`, `Named` | [13](13-app.md#actions-and-shortcuts) |
| `app::use_menu_action` | fn | — | ≈ `NSApplication.sendAction(_:to:from:)` | Hook that receives every action id from menus, shortcuts and activation | [13](13-app.md#actions-and-shortcuts) |
| `app::UNDO` | constant | `CommandGroupPlacement.undoRedo` | — | The kit's Undo action, Ctrl+Z | [13](13-app.md#undo) |
| `app::REDO` | constant | `CommandGroupPlacement.undoRedo` | — | The kit's Redo action, Ctrl+Shift+Z | [13](13-app.md#undo) |

## Menu bar menu

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `main_menu::MainMenu` | struct | `Commands` | `NSApplication.mainMenu` | The window's menu bar menu, published over dbusmenu | [13](13-app.md#menu-bar-menu) |
| `main_menu::Menu` | struct | `CommandMenu` | `NSMenu` | One top-level menu: a title and its items | [13](13-app.md#menu-bar-menu) |
| `main_menu::MenuHandle` | struct | — | ≈ `NSMenu` | Live handle to the window's menu: `update`, `set_menu_items` | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_menu_enabled` | fn | `.disabled(_:)` | `NSMenuItemValidation` | Hook that enables or disables a menu item when the value changes | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_menu_label` | fn | — | `NSMenuItem.title` | Hook that changes a menu item's label | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_menu_checked` | fn | — | `NSMenuItem.state` | Hook that sets a menu item's checkbox | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_submenu_items` | fn | — | `NSMenuItem.submenu` | Hook that replaces a submenu's children | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_menu_update` | fn | — | — | Hook that applies any change to a menu item when a value changes | [13](13-app.md#changing-the-menu-live) |
| `main_menu::find_menu_shortcut` | fn | — | `NSMenu.performKeyEquivalent(with:)` | Finds the enabled action whose shortcut matches a key press | [13](13-app.md#changing-the-menu-live) |
| `main_menu::layout::DbusMenuLayout` | type | — | — | dbusmenu `GetLayout` reply: revision and root item | [13](13-app.md#menu-bar-menu) |
| `main_menu::layout::LayoutItem` | type | — | — | dbusmenu layout node: id, properties, children | [13](13-app.md#menu-bar-menu) |

## Undo

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `app::UndoManager` | struct | `EnvironmentValues.undoManager` | `UndoManager` | Undo and redo stacks for one `State`; `set`, `undo`, `redo` | [13](13-app.md#undo) |
| `app::use_undo` | fn | `EnvironmentValues.undoManager` | `UndoManager` | Hook that wraps a `State` and handles `UNDO` and `REDO` | [13](13-app.md#undo) |

## Settings and restoration

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `app::load_defaults` | fn | — | `UserDefaults` | Reads the app's settings from `<app_id>.domain` | [13](13-app.md#settings-and-restoration) |
| `app::save_defaults` | fn | — | `UserDefaults` | Writes the app's settings to `<app_id>.domain` atomically | [13](13-app.md#settings-and-restoration) |
| `app::use_defaults` | fn | `AppStorage` | — | Hook that reads the settings and reloads them when the file changes | [13](13-app.md#settings-and-restoration) |
| `app::use_watched` | fn | — | — | Hook that reruns a loader when any settings file changes | [13](13-app.md#settings-and-restoration) |
| `app::load_restoration` | fn | `SceneStorage` | ≈ `NSWindowRestoration` | Reads the app's saved state from `<app_id>.state` | [13](13-app.md#settings-and-restoration) |
| `app::save_restoration` | fn | `SceneStorage` | ≈ `NSWindowRestoration` | Writes the app's state to `<app_id>.state` | [13](13-app.md#settings-and-restoration) |
| `app::read_toml` | fn | — | — | Reads a TOML file into `T`; default on missing or invalid | [13](13-app.md#settings-and-restoration) |
| `app::write_toml` | fn | — | — | Writes `T` as TOML through a temporary file and rename | [13](13-app.md#settings-and-restoration) |

## Single instance

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `app::launch` | fn | — | — | Owns the app's bus name or forwards activation to the running instance | [13](13-app.md#single-instance) |
| `app::Launch` | enum | — | — | Result of `launch`: `Primary(Instance)` or `Forwarded` | [13](13-app.md#single-instance) |
| `app::Instance` | struct | — | ≈ `NSApplicationDelegate.applicationShouldHandleReopen(_:hasVisibleWindows:)` | The primary instance; `listen` presents the window and runs forwarded actions | [13](13-app.md#single-instance) |

## Desktop services

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `status_item::StatusItem` | struct | `MenuBarExtra` | `NSStatusItem` | Status area icon with a menu, over StatusNotifierItem | [13](13-app.md#status-item) |
| `status_item::STATUS_ITEM_ACTIVATE` | constant | — | — | Action id sent when the status icon is clicked | [13](13-app.md#status-item) |
| `status_item::Pixmaps` | type | — | — | StatusNotifierItem `IconPixmap` type: width, height, ARGB bytes | [13](13-app.md#status-item) |
| `notifications::Notification` | struct | — | ≈ `UNMutableNotificationContent` | A notification's app name, desktop entry, summary, body and action | [13](13-app.md#notifications) |
| `notifications::post` | fn | — | ≈ `UNUserNotificationCenter` | Posts a notification; the callback gets the activation token on click | [13](13-app.md#notifications) |
| `notifications::play_event_sound` | fn | — | ≈ `NSSound` | Plays a sound from the freedesktop sound theme | [13](13-app.md#notifications) |
| `secrets::Secrets` | struct | — | ≈ `SecItemCopyMatching(_:_:)` | Session with the Secret Service keyring: search, read, create, delete | [13](13-app.md#secrets) |
| `secrets::Item` | struct | — | — | A keyring item: object path, label and attributes | [13](13-app.md#secrets) |
| `secrets::Error` | enum | — | — | Keyring error: `Locked` (prompt showing) or `Bus` | [13](13-app.md#secrets) |
| `portal::open_files` | fn | `.fileImporter(isPresented:allowedContentTypes:allowsMultipleSelection:onCompletion:)` | `NSOpenPanel` | Async file or folder chooser through the portal | [13](13-app.md#portals) |
| `portal::save_file` | fn | `.fileExporter(isPresented:document:contentType:defaultFilename:onCompletion:)` | `NSSavePanel` | Async save chooser through the portal | [13](13-app.md#portals) |
| `portal::open_uri` | fn | `OpenURLAction` | `NSWorkspace.open(_:)` | Opens a URI in its default app through the portal | [13](13-app.md#portals) |
| `portal::notify` | fn | — | ≈ `UNUserNotificationCenter` | Sends a notification through the notification portal | [13](13-app.md#portals) |
| `portal::print` | fn | — | ≈ `NSPrintOperation` | Asks for page setup, builds the PDF and prints through the portal | [13](13-app.md#portals) |
| `portal::file_uri` | fn | — | ≈ `URL.init(fileURLWithPath:)` | `ctappearance::file_uri`: a path as a percent-encoded `file://` URI | [13](13-app.md#portals) |
| `portal::file_path` | fn | — | — | `ctappearance::file_path`: a `file://` URI back into a path | [13](13-app.md#portals) |

## Animation

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `animation::Spring` | struct | `Spring` | ≈ `CASpringAnimation` | The kit's spring curve with duration and bounce; a Freya animated value | [14](14-animations.md#spring) |
| `animation::use_reduced_motion` | fn | `EnvironmentValues.accessibilityReduceMotion` | `NSWorkspace.accessibilityDisplayShouldReduceMotion` | Hook returning the live reduced motion preference | [14](14-animations.md#reduced-motion) |
| `animation::motion_ms` | fn | — | — | A duration, or 0 when reduced motion is on | [14](14-animations.md#reduced-motion) |
| `animation::use_switch_progress` | fn | — | — | Hook easing 0–1 after a boolean state with an `Animation`; jumps with reduced motion | [14](14-animations.md#helpers) |
| `animation::use_alive` | fn | — | — | Hook flag that turns false when the component unmounts | [14](14-animations.md#helpers) |

## Backdrops and media

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `backdrop::Backdrop` | struct | — | — | Color backdrop from an image that gives its content a matching palette | [15](15-backdrops-and-media.md#backdrop) |
| `backdrop::BackdropSource` | enum | — | — | Where the backdrop's colors come from: `None`, `Colors`, `Image` | [15](15-backdrops-and-media.md#backdrop) |
| `backdrop::Accent` | enum | `.tint(_:)` | ≈ `NSColor.controlAccentColor` | Accent inside a backdrop: `System`, `Own`, `FromBackdrop` | [15](15-backdrops-and-media.md#backdrop) |
| `backdrop::Tint` | enum | `.tint(_:)` | — | A control's fill: `Accent`, `Color`, `Backdrop` gradient | [15](15-backdrops-and-media.md#tint) |
| `media::MediaButton` | struct | — | — | Large light pill button with an optional symbol, for media views | [15](15-backdrops-and-media.md#media) |
| `media::RoundButton` | struct | — | — | Round glass button with a symbol; filled light when selected | [15](15-backdrops-and-media.md#media) |
| `media::ScrimEdge` | enum | — | — | The solid edge of a scrim: `Leading` or `Bottom` | [15](15-backdrops-and-media.md#media) |
| `media::Scrim` | struct | — | — | Gradient that blends an image into the ground color | [15](15-backdrops-and-media.md#media) |
| `media::ground` | fn | — | — | The color under the content: backdrop average or window background | [15](15-backdrops-and-media.md#media) |
| `media::Material` | struct | `Material` | `NSVisualEffectView` | Blurred, tinted layer with a rim that fills its parent | [15](15-backdrops-and-media.md#media) |
| `media::TopGlass` | struct | ≈ `.scrollEdgeEffectStyle(_:for:)` | — | Progressive glass band along the top edge | [15](15-backdrops-and-media.md#media) |
| `media::use_artwork` | fn | ≈ `AsyncImage` | — | Hook that loads and decodes an image file in the background | [15](15-backdrops-and-media.md#media) |
| `media::use_artwork_from` | fn | ≈ `AsyncImage` | — | Hook that decodes image bytes from any loader in the background | [15](15-backdrops-and-media.md#media) |
| `glass_container::GlassContainer` | struct | `GlassEffectContainer` | `NSGlassEffectContainerView` | Row of glass shapes that merge when close and morph on change | [15](15-backdrops-and-media.md#glass-that-merges) |
| `glass_container::Frame` | struct | — | — | A glass shape's rectangle and corner radius; `lerp` mixes two | [15](15-backdrops-and-media.md#glass-that-merges) |
| `glass_container::Morph` | struct | — | — | One shape's move: id, from frame, to frame | [15](15-backdrops-and-media.md#glass-that-merges) |
| `glass_container::plan` | fn | ≈ `.glassEffectID(_:in:)` | — | Pairs shown and new frames by id; new and leaving shapes use neighbours | [15](15-backdrops-and-media.md#glass-that-merges) |
| `glass_container::MAX_SHAPES` | constant | — | — | Most shapes a container draws: 8 | [15](15-backdrops-and-media.md#glass-that-merges) |

## Charts

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `charts::Chart` | struct | `Chart` (Swift Charts) | — | Series on an axis with line, area, bar or point marks | [16](16-charts.md#chart) |
| `charts::Series` | struct | — | — | A named list of values | [16](16-charts.md#series) |
| `charts::MarkKind` | enum | `LineMark`, `AreaMark`, `BarMark`, `PointMark` | — | How a `Chart` draws: `Line`, `Area`, `Bar`, `Point` | [16](16-charts.md#chart) |
| `charts::Interpolation` | enum | `InterpolationMethod` | — | How points join: `Linear`, `Monotone`, `Step` | [16](16-charts.md#chart) |
| `charts::SectorChart` | struct | `SectorMark` | — | Pie or donut chart of shares of a total | [16](16-charts.md#sectorchart) |
| `charts::HeatMap` | struct | ≈ `RectangleMark` | — | Grid of cells colored by value | [16](16-charts.md#heatmap) |
| `charts::StreamChart` | struct | — | — | Streamgraph with wiggle baseline, or flow clock when polar | [16](16-charts.md#streamchart) |
| `charts::Chart3D` | struct | `Chart3D`, `SurfacePlot`, `PointPlot` | — | 3D surface, mesh or point cloud with an orbit camera | [16](16-charts.md#chart3d) |
| `charts::format_value` | fn | — | — | Formats a value with 0–2 decimals and a unit, as charts show it | [16](16-charts.md#helpers) |
| `charts::lerp_series` | fn | — | — | Mixes two lists of series; missing values start at 0 | [16](16-charts.md#helpers) |

## Languages

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `localize!` | macro | — | — | Creates an app's embedded translations, its `LOADER` and its `tr!` | [17](17-languages.md#localize) |
| `tr!` | macro | ≈ `LocalizedStringKey` | — | Returns the translated text for a Fluent id; not a key type | [17](17-languages.md#tr) |
| `i18n::load` | fn | — | — | Selects the requested languages into a Fluent loader | [17](17-languages.md#i18nload) |
| `i18n::use_app_language` | fn | — | — | Reads the app's chosen language from its `.domain` file, once | [17](17-languages.md#i18nuse_app_language) |

## Shell

| BarrierKit | Kind | SwiftUI | AppKit | What it is | Chapter |
|---|---|---|---|---|---|
| `shell::run_layer` | fn | — | — | Runs an app in a wlr layer surface (menu bar, Dock, background) | [18](18-platform.md#run_layer-and-layerspec) |
| `shell::LayerSpec` | struct | — | — | Namespace, layer, anchors, exclusive zone, keyboard and size of a layer surface | [18](18-platform.md#run_layer-and-layerspec) |
| `shell::LayerGeometry` | struct | — | ≈ `NSWindow.setFrame(_:display:)` | Anchors, exclusive zone, size and keyboard of a running layer surface | [18](18-platform.md#run_layer-and-layerspec) |
| `shell::CardFrames` | struct | — | — | Corner radius and rectangles of the cards a layer surface draws | [18](18-platform.md#run_layer-and-layerspec) |
| `shell::set_card_frames` | fn | — | — | Asks the compositor for the window frame (shadow, inner and outer border) around each card of the calling layer surface | [18](18-platform.md#run_layer-and-layerspec) |
| `shell::set_layer_geometry` | fn | — | ≈ `NSWindow.setFrame(_:display:)` | Moves, resizes or refocuses the layer surface of the calling component | [18](18-platform.md#run_layer-and-layerspec) |
| `shell::Layer` | enum | — | ≈ `NSWindow.Level` | Stacking layer of a layer surface: `Background`, `Bottom`, `Top`, `Overlay` | [18](18-platform.md#run_layer-and-layerspec) |
| `shell::Anchors` | struct | — | — | Edges a layer surface sticks to; `TOP` and `BOTTOM` constants | [18](18-platform.md#run_layer-and-layerspec) |
| `shell::KeyboardInteractivity` | enum | — | — | Keyboard focus of a layer surface: `None`, `Exclusive`, `OnDemand` | [18](18-platform.md#run_layer-and-layerspec) |
| `shell::run_lock` | fn | — | — | Locks the session and draws an app on every output | [18](18-platform.md#run_lock-and-sessionunlock) |
| `shell::SessionUnlock` | struct | — | — | Handle that ends the session lock | [18](18-platform.md#run_lock-and-sessionunlock) |
| `shell::SessionUnlock::keyboard_layout` | method | — | ≈ `TISCopyCurrentKeyboardInputSource` | Name of the active keyboard layout, from the compositor's keymap | [18](18-platform.md#run_lock-and-sessionunlock) |
| `shell::ScreenLock` | enum | — | — | Whether the session is locking, and for how long the fade lasts | [18](18-platform.md#screen-lock-and-blur-region) |
| `shell::use_screen_lock` | fn | — | — | Hook that follows the session lock (`infinity_session_v1`) | [18](18-platform.md#screen-lock-and-blur-region) |
| `shell::BlurRegion` | struct | `.background(.ultraThinMaterial)` | `NSVisualEffectView` | Rectangles the compositor blurs behind a surface | [18](18-platform.md#screen-lock-and-blur-region) |
| `shell::set_blur_region` | fn | — | — | Asks the compositor to blur behind the calling surface (`ext_background_effect_v1`) | [18](18-platform.md#screen-lock-and-blur-region) |
| `shell::Toplevels` | struct | — | — | Every app's windows, the shown space and desktop, and actions on windows | [18](18-platform.md#toplevels) |
| `shell::ToplevelInfo` | struct | — | — | One window of any app: id, title, app id, state, menu addresses | [18](18-platform.md#toplevels) |
| `shell::FocusBar` | enum | — | — | The bar over the shown space: `None`, `Hidden`, `Revealed` | [18](18-platform.md#toplevels) |
| `shell::GtkMenusAddress` | struct | — | — | D-Bus address of a GTK window's global menu | [18](18-platform.md#toplevels) |
| `shell::InfinityLayout` | enum | — | — | Placement for `Toplevels::arrange`: fill, center, halves, quarters | [18](18-platform.md#toplevels) |
| `shell::InfinityPairLayout` | enum | — | — | Pair tiling: `SideBySide`, `SideBySideReversed`, `Stacked`, `StackedReversed` | [18](18-platform.md#toplevels) |
| `shell::InfinityWindowManagerV1` | struct | — | — | Proxy of Malaysia's private window-management protocol, used by `Toplevels` | [18](18-platform.md#toplevels) |
| `shell::Activation` | struct | — | — | Requests xdg-activation tokens for the apps the shell launches | [18](18-platform.md#activation) |
| `shell::use_status_items` | fn | — | — | Hook that hosts other apps' StatusNotifierItem icons and menus | [18](18-platform.md#status-items) |
| `shell::TrayItem` | struct | — | ≈ `NSStatusItem` | Another app's status item, as the host reads it | [18](18-platform.md#status-items) |
| `shell::StatusHost` | struct | — | — | Sends activate, secondary activate and menu clicks to status items | [18](18-platform.md#status-items) |
| `shell::Pixmap` | struct | — | ≈ `NSBitmapImageRep` | RGBA bitmap of a status item's icon | [18](18-platform.md#status-items) |

## Freya names you will meet

| Freya | SwiftUI | What it is |
|---|---|---|
| `App` | `App` | Trait of a root type with `fn render`; why the kit's app is `Application` |
| `Component` | `View` | A reusable view; it must be `PartialEq` |
| `IntoElement` | `some View` | Anything a view can return |
| `use_state` | `@State` | State that survives between renders |
| `rect()` | — | The basic box; `VStack` and `HStack` are built on it |
| `label()` | — | A single run of text; `Text` is built on it |
| `paragraph()` | — | Text with several styled spans |
