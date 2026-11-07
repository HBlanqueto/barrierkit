# BarrierKit: prompt for AI agents

Copy this file whole into the agent's context, or point the agent at it. It is
the only entry point: it says which other files to read and when.

---

You are writing a desktop app, or a change to one, with **BarrierKit**: a Rust
toolkit for Wayland whose API follows SwiftUI. You write Rust; the reader of
your code knows SwiftUI or AppKit. Keep every program as short as the SwiftUI
version of it.

## 1. What you are working with

- **Language:** Rust 2024, stable toolchain.
- **Platform:** Linux on Wayland, drawn with Vulkan and Skia. No macOS, no
  Windows, no X11.
- **Layers:** BarrierKit (`barrier-kit`) is built on Freya 0.4.3
  (`freya-core`), its layout engine Torin, and the design tokens of
  `vow-design`. You only depend on `barrier-kit`; the prelude re-exports what
  you need from the others.
- **Model:** declarative. A view is a function of the state; when the state
  changes, the kit calls it again and applies the difference.

## 2. The shape of every app

```rust
use barrier_kit::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("hello")
        .window("Hello", || Text::new("Hello, world").font(TITLE).padding(40.))
        .run()
}
```

- `Application::new(name)`: `name` is short, lowercase and stable (it becomes
  the `app_id`, the config domain and the saved window size).
- `.window(title, || view)`: the closure is the view. Hooks (`use_state`…) go
  at its top.
- `.options(|window| …)`: size, limits and menu (`WindowOptions`).
- `.run()`: opens the window with titlebar and background and centers the view.

## 3. Hard rules

1. **One import:** `use barrier_kit::prelude::*;`. Import from a module only
   what the prelude lacks (for example `barrier_kit::views::form::Form`).
2. **No literal colors, sizes or fonts.** Colors from `Palette::current()`,
   sizes from `SPACE_*`, fonts from `LARGE_TITLE`, `TITLE`, `TITLE_2`,
   `HEADLINE`, `BODY`, `CALLOUT`, `CAPTION`, `MONO`. `Text` already picks the
   right color.
3. **No literal visible text in a real app.** Text goes in
   `i18n/{en,es}/<crate>.ftl` and is read with `tr!("id")`. Literals are only
   for throwaway samples.
4. **Controls are controlled.** Pass the value (`.value`, `.on`, `.selected`)
   and handle the change (`.on_change`, `.on_toggle`, `.on_press`).
5. **Use the kit's control** before building one from `rect()`. The glossary
   lists them.
6. **SwiftUI names.** If you know the SwiftUI name, look it up in the
   glossary; do not guess. Freya owns `App`, `label` and `rect`; the kit's app
   type is `Application`.
7. **No comments** except `// SAFETY:` on `unsafe` and one line explaining a
   non-obvious invariant. Names explain the code.
8. **Build only inside `nix develop`.** Outside it, `build.rs` stops on
   purpose.

## 4. Read more only when the task needs it

| When you need to… | Read |
|---|---|
| Find the name of anything (control, view, token, hook) | [`docs/barrierkit/15-glossary.md`](../barrierkit/15-glossary.md) |
| Copy a working pattern (state, form, component, custom window) | [`recipes.md`](recipes.md) |
| Check your work before you answer | [`checklist.md`](checklist.md) |
| Understand the first app line by line | [`docs/barrierkit/02-first-app.md`](../barrierkit/02-first-app.md) |
| Use a specific control | [`docs/barrierkit/05-controls.md`](../barrierkit/05-controls.md) |
| Change window size, limits, menu or full screen | [`docs/barrierkit/06-windows.md`](../barrierkit/06-windows.md) |
| Build a sidebar app | [`docs/barrierkit/08-sidebar.md`](../barrierkit/08-sidebar.md) |
| Forms, tables, tabs, alerts, search | [`docs/barrierkit/09-views.md`](../barrierkit/09-views.md) |
| Preferences or About window | [`docs/barrierkit/10-patterns.md`](../barrierkit/10-patterns.md) |
| Menus, shortcuts, single instance, saved settings, undo | [`docs/barrierkit/11-app.md`](../barrierkit/11-app.md) |
| Translate the app | [`docs/barrierkit/18-languages.md`](../barrierkit/18-languages.md) |
| Charts | [`docs/barrierkit/19-charts.md`](../barrierkit/19-charts.md) |
| Change the kit itself | [`AGENTS.md`](../../AGENTS.md) and [`docs/barrierkit/03-layers.md`](../barrierkit/03-layers.md) |

## 5. How to work

1. Restate the app as the SwiftUI code a Mac developer would write.
2. Translate it with the glossary, name by name.
3. Start from the closest recipe; change only what the task needs.
4. Build and lint inside `nix develop` (`cargo clippy -- -D warnings`).
5. Go through the checklist. Report what you ran and what you could not run
   (the window needs a Wayland session to open).
