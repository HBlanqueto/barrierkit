# 0063 — Views are types, modifiers are methods, Apple names

Status: **accepted** (2026-11-07).

**Goal:** record the rename that makes the kit read like SwiftUI and AppKit.
**Scope:** public names of `bk`; no behavior changes (the 57 reference images
render the same before and after).
**Reader:** anyone who writes an app with the kit or adds to it.
**Input:** views built by functions (`divider()`, `empty_state(..)`), modifiers
built by wrappers (`context_menu(element, ..)`, `use_tooltip(..).attach(..)`)
and names that did not match Apple's. **Output:** the rules and the new names.

## Context

SwiftUI has three shapes: views are types (`Divider()`, `Image(systemName:)`),
modifiers are methods on any view (`.help(_:)`, `.contextMenu`), and names
follow the platform (`ProgressView`, `Gauge`, `ShareLink`). The kit mixed
functions and types for views, wrapped elements to add a tooltip or a context
menu, and used names Apple does not (`AppMenu` for the main menu,
`PreferencesWindow` for Settings).

## Decision

1. **Every public view is a type** built with `Type::new(..)` and optional
   chained methods. It forwards Freya's traits when it wraps a `rect()`, so
   layout and event methods keep working.
2. **SwiftUI modifiers are methods** of the `ViewModifiers` trait, implemented
   for every element with events: `.help(text)`, `.context_menu(items, …)`,
   `.focus_ring(radius, color)`.
3. **Names follow SwiftUI, then AppKit**, short and plain
   ([glossary rules](../barrierkit/20-glossary.md#how-names-are-chosen)).
4. **Low-level primitives stay functions**, like Freya's `rect()` and
   `label()`: `text(content, font, color)`.

| Before | Now |
|---|---|
| `adornments::badge`, `divider`, `avatar` | `Badge::new`, `Divider::new`, `Avatar::new` |
| `canvas::canvas` | `Canvas::new` |
| `views::feedback::empty_state(symbol, title)`, `sheet` | `ContentUnavailableView::new(title, symbol)`, `Sheet::new` |
| `media::scrim`, `material`, `top_glass` | `Scrim::new`, `Material::new`, `TopGlass::new` |
| `views::navigation::navigation_buttons` | `NavigationButtons::new` |
| `sidebar::symbol_tile(glyph, tint, rim)` | `SymbolTile::new(glyph, tint).rim(color)` |
| `symbol::symbol(symbol, size, color)` | `Image::system(symbol).size(..).color(..)` |
| `patterns::about::about_window` | `AboutView::new(..).version(..)` |
| `popover::show_popover(anchor, size, content)` | `Popover::new(size, content).show(anchor)` |
| `platform::open_utility_window` | `UtilityWindow::new(title, name, content).open()` |
| `titlebar::window_traffic_lights` | `TrafficLights::for_window` |
| `context_menu::context_menu(element, ..)` | `element.context_menu(..)` |
| `tooltip::use_tooltip(text).attach(element)` | `element.help(text)` |
| `interaction::ring(element, ..)` | `element.focus_ring(..)` |
| `PreferencesWindow` (`patterns::preferences`) | `SettingsWindow` (`patterns::settings`) |
| `AppMenu`, `AppMenuAddress` (`app_menu`) | `MainMenu`, `MainMenuAddress` (`main_menu`) |
| `ProgressBar`, `CircularGauge` | `ProgressView`, `Gauge` |
| `SplitButton`, `HelpButton`, `ShareButton` | `ComboButton`, `HelpLink`, `ShareLink` |
| `ColumnBrowser` | `Browser` |
| `Breakpoint`, `Adaptive`, `use_breakpoint` | `SizeClass`, `SizeClassReader`, `use_size_class` |
| `notifications::Notice` | `notifications::Notification` |

The Infinity protocol keeps its event name `app_menu`: it is a wire protocol
shared with the compositor.

## Consequences

- An app reads like its SwiftUI version: `Divider::new()`,
  `Image::system(Symbol::Plus)`, `.help("Add")`.
- `.help(..)` uses a hook, so it is called on every render, never inside a
  condition.
- malaysia and curseterm rename these items when they move their pinned `rev`
  past this change; the table above is the list.
