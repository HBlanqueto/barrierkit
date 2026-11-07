# 15. Glossary

**Goal:** find the BarrierKit name for a concept you know from SwiftUI or
AppKit, and name new items the same way.
**Reader:** anyone who writes an app with the kit, adds an item to the kit, or
is an AI that generates code for it ([`docs/ai/prompt.md`](../ai/prompt.md)).
**Rule:** a public item is added to this glossary in the same commit that adds
or renames it (`AGENTS.md`, «Nombres y glosario»).

## How names are chosen

1. **SwiftUI first.** When SwiftUI has the concept, use its name: `Text`,
   `VStack`, `Button`, `TabView`.
2. **Then AppKit.** When only AppKit has it, use the AppKit name without the
   `NS` prefix: `ColorWell`, `PathControl`, `TokenField`.
3. **Then plain English.** Otherwise, one or two common words that say what
   the item is, not how it is built: `Palette`, `Sidebar`.
4. **Rust case.** Types `UpperCamelCase`; functions and methods
   `snake_case` (`foregroundStyle` → `foreground_style`); constants
   `SCREAMING_SNAKE_CASE` (`.title` → `TITLE`).
5. **Fixed shapes.** A constructor is `Type::new(required)`. Options are
   chained methods named after what they set (`.font`, `.size`). Events are
   `on_<verb>` (`on_press`, `on_change`). Booleans read as adjectives
   (`disabled`, `focused`). Hooks are `use_<thing>`.
6. **Taken names.** When Freya already uses a name (`App`, `label`, `rect`),
   take the next clear one (`Application`) and record it here.

## Start an app

| BarrierKit | SwiftUI | AppKit | What it does |
|---|---|---|---|
| `use barrier_kit::prelude::*` | `import SwiftUI` | `import AppKit` | Brings everything below, Freya's elements and hooks, and the type and spacing tokens |
| `Application::new(name)` | `struct …: App` | `NSApplication` | The app. `name` gives the `app_id` and the saved window size |
| `.window(title, \|\| content)` | `WindowGroup { }` | `NSWindow` | Adds the window; returns a `Scene` |
| `Scene::options(\|window\| …)` | `.defaultSize`, `.windowResizability` | `NSWindow` properties | Changes any `WindowOptions` setting |
| `Scene::run()` | `@main` | `NSApplication.run()` | Opens the window and runs until it closes |
| `run_window(content, options)` | — | — | The same without the titlebar and background: you draw the whole window |
| `WindowOptions` | Scene modifiers | `NSWindow` style | Title, size, limits, menu, full-screen bar (chapter [6](06-ventanas.md)) |
| `RunError` | — | — | Why the window could not open (no Wayland, no Vulkan…) |

`Scene::run()` draws the window background, a `Titlebar` with the title, and
centers the content. Chapter [2](02-primera-app.md) shows it step by step.

## Text and layout

| BarrierKit | SwiftUI | AppKit | Defaults |
|---|---|---|---|
| `Text::new("…")` | `Text("…")` | `NSTextField(labelWithString:)` | Font `BODY`, color `Palette::current().label`, wraps |
| `.font(TITLE)` | `.font(.title)` | `NSFont.preferredFont` | — |
| `VStack::new()` | `VStack { }` | `NSStackView` (vertical) | Centered, spacing `SPACE_3` |
| `HStack::new()` | `HStack { }` | `NSStackView` (horizontal) | Centered, spacing `SPACE_3` |
| `.child(view)` | the stack's body | `addArrangedSubview` | — |
| `.padding(40.)` | `.padding(40)` | layout margins | — |
| `.spacing(SPACE_1)` | `VStack(spacing:)` | `NSStackView.spacing` | — |
| `.expanded()` | `.frame(maxWidth: .infinity, maxHeight: .infinity)` | Auto Layout to the edges | Also on `ScrollView` |
| `.center()` | `.frame(alignment: .center)` | centering constraints | — |
| `rect()` | — | `NSView` | Freya's basic box, with no defaults |
| `text(content, font, color)` | — | — | A single-line `Text` with every value explicit; the kit uses it inside controls |

`Text`, `VStack` and `HStack` accept Freya's layout, text-style and event
methods (`width`, `padding`, `color`, `on_press`…). `VStack` and `HStack` also
accept children, alignment, background and corner methods (`child`,
`main_align`, `background`, `corner_radius`…), like `rect()`.

## Typography

| BarrierKit | SwiftUI | Size / weight |
|---|---|---|
| `LARGE_TITLE` | `.largeTitle` | 26 / 700 |
| `TITLE` | `.title` | 22 / 650 |
| `TITLE_2` | `.title2` | 15 / 600 |
| `HEADLINE` | `.headline` | 13 / 700 |
| `BODY` | `.body` | 13 / 400 |
| `CALLOUT` | `.callout` | 12 / 400 |
| `CAPTION` | `.caption` | 11 / 500 |
| `MONO` | `.monospaced()` | 13 / 400, Geist Mono |

The values come from `crates/vow-design/tengen/tokens.json`
(`font.styles`). Until 2026-11-07, `BODY` was `TEXT`, `LARGE_TITLE` was
`DISPLAY` and `TITLE` was `TITLE_1`.

## State and data

| BarrierKit | SwiftUI | AppKit |
|---|---|---|
| `use_state(\|\| value)` | `@State` | a property and `needsDisplay` |
| `.read()` / `.write()` | reading / assigning the property | — |
| `use_defaults(app_id)` | `@AppStorage` | `UserDefaults` |
| `UndoManager`, `use_undo` | `UndoManager` | `NSUndoManager` |

## Controls

| BarrierKit | SwiftUI | AppKit |
|---|---|---|
| `Button` | `Button` | `NSButton` |
| `Switch` | `Toggle` (`.switch`) | `NSSwitch` |
| `Checkbox` | `Toggle` (`.checkbox`) | `NSButton` (checkbox) |
| `Slider` | `Slider` | `NSSlider` |
| `SegmentedControl` | `Picker` (`.segmented`) | `NSSegmentedControl` |
| `PopUpButton` | `Picker` (`.menu`) | `NSPopUpButton` |
| `ComboBox` | — | `NSComboBox` |
| `MenuButton` | `Menu` | `NSPopUpButton` (pull-down) |
| `TextField` | `TextField` | `NSTextField` |
| `SecureField` | `SecureField` | `NSSecureTextField` |
| `SearchField` | `.searchable` | `NSSearchField` |
| `Stepper` | `Stepper` | `NSStepper` |
| `DatePicker` | `DatePicker` | `NSDatePicker` |
| `ColorWell` | `ColorPicker` | `NSColorWell` |
| `ProgressBar`, `ProgressRing`, `Spinner` | `ProgressView` | `NSProgressIndicator` |
| `LevelSlider` | `Gauge` | `NSLevelIndicator` |
| `TokenField` | — | `NSTokenField` |
| `PathControl` | — | `NSPathControl` |
| `ImageWell` | — | `NSImageView` (editable) |
| `Tooltip` | `.help("…")` | `toolTip` |

Chapter [5](05-controles.md) shows each control.

## Views and windows

| BarrierKit | SwiftUI | AppKit |
|---|---|---|
| `Titlebar` | (drawn by the system) | `NSTitlebarAccessoryViewController` |
| `Sidebar` | `List` (`.sidebar`) | `NSOutlineView` (source list) |
| `SplitView` | `NavigationSplitView` | `NSSplitViewController` |
| `Form` | `Form` | — |
| `GroupBox` | `GroupBox` | `NSBox` |
| `DisclosureGroup` | `DisclosureGroup` | disclosure button |
| `Table` | `Table` | `NSTableView` |
| `OutlineView` | `OutlineGroup` | `NSOutlineView` |
| `CollectionView` | `LazyVGrid` | `NSCollectionView` |
| `TabView` | `TabView` | `NSTabView` |
| `ScrollView` | `ScrollView` | `NSScrollView` |
| `Alert` | `.alert` | `NSAlert` |
| `show_popover` | `.popover` | `NSPopover` |
| `context_menu` | `.contextMenu` | `NSMenu` (contextual) |
| `PreferencesWindow` | `Settings` | preferences window |
| `about_window` | standard About panel | `orderFrontStandardAboutPanel` |
| `Chart` | `Chart` (Swift Charts) | — |

## App services

| BarrierKit | SwiftUI | AppKit |
|---|---|---|
| `AppMenu` | `.commands`, `CommandMenu` | `NSMenu` (main menu) |
| `use_menu_action` | `Button` in `.commands` | `@IBAction` |
| `StatusItem` | `MenuBarExtra` | `NSStatusItem` |
| `launch(app_id)` | (single instance by default) | `NSRunningApplication` |
| `Palette::current()` | semantic colors (`.primary`, `.secondary`) | `NSColor.labelColor`… |

## Freya names you will meet

| Freya | Meaning |
|---|---|
| `App` | Trait of a root type with `fn render`. Why the kit's app is `Application` |
| `Component` | A reusable view; it must be `PartialEq` |
| `IntoElement` | Anything a view can return |
| `rect()`, `label()`, `paragraph()` | The basic elements. `Text` is built on `label()` |
