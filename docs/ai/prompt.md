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
   [glossary](../barrierkit/20-glossary.md). Do not guess a name.
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
- **Layers:** BarrierKit (`bk`) is built on Freya 0.4.3
  (`freya-core`), its layout engine Torin, and the design tokens of
  `ctdesign`. An app depends only on `bk`; the prelude re-exports
  what it needs from the others.
- **Model:** declarative. A view is a function of the state; when the state
  changes, the kit calls it again and applies the difference.
- **Shape of every app:** `use bk::prelude::*;`, then
  `Application::new(name).window(title, || view).run()` in `main`. The program
  and what each part does are in
  [chapter 2](../barrierkit/02-first-app.md#hello-world).

## Hard rules

1. **One import:** `use bk::prelude::*;`. Import from a module only
   what the prelude lacks (for example `bk::views::form::Form`).
2. **No literal colors, sizes or fonts.** Colors from `Palette::current()`,
   sizes from the `SPACE_*` and component tokens, fonts from `LARGE_TITLE`,
   `TITLE`, `TITLE_2`, `HEADLINE`, `BODY`, `CALLOUT`, `CAPTION`, `MONO`. `Text`
   already picks the right color.
3. **No literal visible text in a real app.** Text goes in
   `i18n/{en,es}/<crate>.ftl` and is read with `tr!("id")`. Literals are only
   for throwaway samples.
4. **Controls are controlled.** Pass the value (`.value`, `.on`, `.selected`)
   and handle the change (`.on_change`, `.on_toggle`, `.on_press`).
5. **Use the kit's control** before building one from `rect()`.
6. **Names come from the glossary.** Freya owns `App`, `label` and `rect`; the
   kit's app type is `Application`.
7. **No comments** except `// SAFETY:` on `unsafe` and one line explaining a
   non-obvious invariant. Names explain the code.
8. **Build only inside `nix develop`.** Outside it, `build.rs` stops on
   purpose.

## Read more only when the task needs it

| When you need to… | Read |
|---|---|
| Find the name of anything (control, view, token, hook) | [`20-glossary.md`](../barrierkit/20-glossary.md) |
| Copy a working program | [`recipes.md`](recipes.md) |
| Check your work before you answer | [`checklist.md`](checklist.md) |
| Understand the first app line by line | [`02-first-app.md`](../barrierkit/02-first-app.md) |
| Colors, fonts, symbols and tokens | [`04-appearance-and-tokens.md`](../barrierkit/04-appearance-and-tokens.md) |
| Use a control | [`05-controls.md`](../barrierkit/05-controls.md) |
| Edit, style, spell-check or print text | [`06-text-and-editing.md`](../barrierkit/06-text-and-editing.md) |
| Menus, context menus, popovers, tooltips | [`07-menus-and-popovers.md`](../barrierkit/07-menus-and-popovers.md) |
| Window size, limits, popups, timers | [`08-windows.md`](../barrierkit/08-windows.md) |
| Title bars and traffic lights | [`09-titlebars.md`](../barrierkit/09-titlebars.md) |
| A sidebar app | [`10-sidebar.md`](../barrierkit/10-sidebar.md) |
| Forms, tables, tabs, alerts, search | [`11-views.md`](../barrierkit/11-views.md) |
| Preferences or About window | [`12-patterns.md`](../barrierkit/12-patterns.md) |
| Menu bar menu, shortcuts, single instance, saved settings, undo | [`13-app.md`](../barrierkit/13-app.md) |
| Animations | [`14-animations.md`](../barrierkit/14-animations.md) |
| Tinted backdrops, glass, video | [`15-backdrops-and-media.md`](../barrierkit/15-backdrops-and-media.md) |
| Charts | [`16-charts.md`](../barrierkit/16-charts.md) |
| Translate the app | [`17-languages.md`](../barrierkit/17-languages.md) |
| Change the kit itself | [`AGENTS.md`](../../AGENTS.md) and [`03-layers.md`](../barrierkit/03-layers.md) |
