# BarrierKit: prompt for AI agents

**Goal:** let an AI agent write or change a BarrierKit app correctly.
**Scope:** from a task to code that builds and follows the kit's rules. This is
the only entry point; it says which other file to read and when.
**Reader:** an AI agent. Copy this file whole into its context, or point it here.
**Input:** a task. **Output:** a program or a change, checked with the
[checklist](checklist.md).

---

You are writing a desktop app, or a change to one, with **BarrierKit**: a Rust
toolkit for Wayland whose API follows SwiftUI. You write Rust; the reader of
your code knows SwiftUI or AppKit. Keep every program as short as its SwiftUI
version.

## Steps

1. Write down the SwiftUI code a Mac developer would write for the task.
2. Translate it name by name with the
   [glossary](../docs/barrierkit/20-glossary.md). Do not guess a name. Translate
   what is not a name (`@Binding`, `@Environment`, `@Observable`, `@FocusState`,
   `ForEach`, `if/else`, actions, effects) with
   [SwiftUI to the kit](../docs/barrierkit/21-swiftui-to-the-kit.md).
3. Start from the closest [recipe](recipes.md); change only what the task
   needs.
4. Open the chapter of every item you use (the glossary links it) and check
   its arguments and defaults.
5. Build and lint inside `nix develop`: `cargo build`, then
   `cargo clippy -- -D warnings`, then `cargo fmt`.
6. Go through the [checklist](checklist.md). Report what you ran and what you
   could not run (a window needs a Wayland session to open).

## What you are working with

- **Language:** Rust 2024, stable toolchain.
- **Platform:** Linux on Wayland, drawn with Vulkan and Skia. No macOS, no
  Windows, no X11.
- **Layers:** BarrierKit (`bk`) is built on Freya 0.5.3
  (`freya-core`), its layout engine Torin, and the design tokens of
  `ctdesign`. An app depends on `bk`; the prelude re-exports
  what most apps need from the others. A chapter names the crate to add when
  an app needs more (`ctdesign` tokens, `freya-animation`, `freya-clipboard`).
- **Model:** declarative. A view is a function of the state; when the state
  changes, the kit calls it again and applies the difference.
- **Features:** an app enables none: `shell` is for `malaysia-shell` and
  `tracy` is for profiling
  ([chapter 1](../docs/barrierkit/01-what-is-barrierkit.md#the-crates)).
- **Shape of every app:** `use bk::prelude::*;`, then
  `Application::new(name).window(title, || view).run()` in `main`. The program
  and what each part does are in
  [chapter 2](../docs/barrierkit/02-first-app.md#hello-world).

## Hard rules

1. **One import:** `use bk::prelude::*;`. Import from a module only
   what the prelude lacks (for example `bk::views::form::Form`).
2. **No literal colors, sizes or fonts in a real app.** Colors from
   `Palette::current()`, or from a `ShapeStyle::Role` or `Accent` in a style
   ([`styles.md`](styles.md)); sizes from the `SPACE_*` and component tokens,
   fonts from `LARGE_TITLE`, `TITLE`, `TITLE_2`, `TITLE_3`, `HEADLINE`,
   `SUBHEADLINE`, `BODY`, `CALLOUT`, `FOOTNOTE`, `CAPTION`, `CAPTION_2`,
   `MONO`. `Text` already picks the right color. A throwaway sample, like the
   [recipes](recipes.md), may use literals.
3. **No literal visible text in a real app.** Text goes in
   `i18n/{en,es}/<crate>.ftl` and is read with `tr!("id")`. Literals are only
   for throwaway samples.
4. **Controls are controlled.** Pass the value (`.value`, `.on`, `.selected`)
   and handle the change (`.on_change`, `.on_toggle`, `.on_press`). A text
   field takes the `State<String>` it edits instead
   ([chapter 5](../docs/barrierkit/05-controls.md#text-fields)).
5. **Use the kit's control** before building one from `rect()`.
6. **Names come from the glossary.** Freya owns `App`, `label` and `rect`; the
   kit's app type is `Application`.
7. **Views are types, modifiers are methods**, as in SwiftUI: write
   `Divider::new()`, `Image::system(Symbol::Plus)` or
   `ContentUnavailableView::new(..)`, and `.padding(..)`, `.margin(..)`,
   `.background(..)`, `.help("…")` or `.context_menu(items, …)` on a view or a
   control, in any order. A modifier sets a property on that one node instead
   of wrapping it, so the padding stays inside the background and outer space
   is `.margin` ([chapter 5](../docs/barrierkit/05-controls.md#modifiers-on-controls)).
   `StyleScope`, `Sheet`, `Alert`, `SidebarToggle` and `Titlebar` take none:
   put them in a `rect()`. Call `.help` unconditionally: it uses a hook.
8. **No comments** except `// SAFETY:` on `unsafe` and one line explaining a
   non-obvious invariant. Names explain the code.
9. **Build only inside `nix develop`.** Outside it, `build.rs` stops on
   purpose.
10. **Symbols are SVG, never PNG.** Use `Image::system(..)`; for another
    one-color icon, `Image::data(svg).rendering_mode(TemplateRenderingMode::Template)`,
    which paints it black in light and white in dark
    ([Images and template rendering](../docs/barrierkit/04-appearance-and-tokens.md#images-and-template-rendering)).
    The 124 `Symbol`s are named after SF Symbols; look an Apple name up in
    the [glossary](../docs/barrierkit/20-glossary.md#symbol-variants) before
    drawing a new one.

## Read more only when the task needs it

| When you need to… | Read |
|---|---|
| Find the name of anything (control, view, token, hook) | [`20-glossary.md`](../docs/barrierkit/20-glossary.md) |
| Write `@Binding`, `@Environment`, `@Observable`, `@FocusState`, `ForEach`, `if/else`, an action or an effect | [`21-swiftui-to-the-kit.md`](../docs/barrierkit/21-swiftui-to-the-kit.md) |
| Copy a working program | [`recipes.md`](recipes.md) |
| Check your work before you answer | [`checklist.md`](checklist.md) |
| Understand the first app line by line | [`02-first-app.md`](../docs/barrierkit/02-first-app.md) |
| Colors, fonts, symbols and tokens | [`04-appearance-and-tokens.md`](../docs/barrierkit/04-appearance-and-tokens.md) |
| Use a control | [`05-controls.md`](../docs/barrierkit/05-controls.md) |
| Change how a control looks (color, shape, size, motion) or build a theme | [`styles.md`](styles.md) |
| Edit, style or spell-check text | [`06-text-and-editing.md`](../docs/barrierkit/06-text-and-editing.md) |
| Menus, context menus, popovers, tooltips | [`07-menus-and-popovers.md`](../docs/barrierkit/07-menus-and-popovers.md) |
| Window size, limits, popups, timers | [`08-windows.md`](../docs/barrierkit/08-windows.md) |
| Title bars and traffic lights | [`09-titlebars.md`](../docs/barrierkit/09-titlebars.md) |
| A sidebar app | [`10-sidebar.md`](../docs/barrierkit/10-sidebar.md) |
| Forms, tables, tabs, alerts, search | [`11-views.md`](../docs/barrierkit/11-views.md) |
| Preferences or About window | [`12-patterns.md`](../docs/barrierkit/12-patterns.md) |
| Menu bar menu, shortcuts, single instance, saved settings, undo | [`13-app.md`](../docs/barrierkit/13-app.md) |
| Animations | [`14-animations.md`](../docs/barrierkit/14-animations.md) |
| Tinted backdrops, glass, artwork | [`15-backdrops-and-media.md`](../docs/barrierkit/15-backdrops-and-media.md) |
| Translate the app | [`17-languages.md`](../docs/barrierkit/17-languages.md) |
| Draw the app as an HTML artifact | [`artifacts.md`](artifacts.md) |
| Measure an artifact against what the kit draws | [`compare.md`](compare.md) |
| Change the kit itself | [`AGENTS.md`](../AGENTS.md) and [`03-layers.md`](../docs/barrierkit/03-layers.md) |
