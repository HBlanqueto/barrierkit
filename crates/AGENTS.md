# crates/AGENTS.md — rules for code

These rules add to the root [`AGENTS.md`](../AGENTS.md), whose «Minimal code»
rules still hold; rule numbers in «Minimal code» refer to that file.

## macOS and Linux

1. **`cfg(target_os)` only in `crates/bk/src/platform/mod.rs`,
   `platform/linux/`, `platform/macos/`, `crates/ctappearance/` and
   `Cargo.toml`.** `scripts/check-platform-cfg.sh` checks it, with no
   exceptions.
2. **One name, one signature, two bodies.** What a system provides
   (clipboard, keychain, notifications, menus) is a function with the same
   signature in `platform/macos/` and in `platform/linux/`, re-exported once
   from `platform/mod.rs` (`use linux|macos as sys`). No traits while there is
   one body per system (rule 3 of «Minimal code»).
3. **The decision lives in pure code, without `cfg`:** recognizers, state
   machines, `ctdesign` numbers. The backend only translates system events into
   that logic and applies its effects, like `gesture::trackpad`.
4. **You touch a file with a stray `cfg`:** move it to its backend in that
   commit.

## Rust (new code or code you touch)

Do not rewrite existing code only to meet these (rule 7 of «Minimal code»).

1. **No `unwrap()` in the library or the recipes:** handle the error. An
   invariant that cannot break uses `expect("why it cannot fail")`. Tests may
   use `unwrap()`.
2. **Import with `crate::`, not `super::`.** Exception: `use super::*` in a
   `mod tests`.
3. **A component is a data type with `#[derive(PartialEq)]` that implements
   `Component`.** A public view or control that an app can put in a list also
   implements `KeyExt` (`.key(..)`), so Freya reconciles by key.
4. **Components import with `use freya_core::prelude::*;`.**
5. **Doc comments:** one `///` line on a new public API whose name does not
   state its contract (what it returns, what it assumes, when it fails). Rule 6
   still holds: none that repeats the name, and no other comments inside the
   code.
6. **An element that changes is not kept in a variable to mutate it:** it is
   declared as the component's last value and changed with `maybe(..)`,
   `maybe_child(..)` or `map(..)`.
7. **A `&str` or a `String` already is a Freya `label`** where a child is
   expected: do not write `label().text("Hello")` only for that. Apps use
   `Text` (rule 3).
8. **Short control flow:** `?` instead of a `match` that only propagates the
   error; `if let` instead of a one-arm `match`; `while let` instead of `loop`
   with `match`.
9. **Iterators before loops with `push`:** `filter`, `map`, `filter_map`,
   `flatten`, `fold`, with no intermediate collections.
10. **Errors:** `?` when `From` converts on its own; `map_err` only if it adds
   context that `From` cannot give.
11. **Imports grouped by crate:** `use crate::{a, b::c};`. Import the names;
   do not use qualified paths in signatures.

## How the rules apply in barrierkit

```rust
// ✗ literal and recomputed geometry
let x = 8.0 + i as f64 * (12.0 + 8.0);
// ✓ token and function that already exist (ctdesign, ctappearance::traffic)
let (x, y, d) = ctappearance::traffic::button_rect(i);
```

```rust
// ✗ doc that repeats the name
/// Whether the window currently has focus.
pub fn focused(mut self, focused: bool) -> Self { … }
// ✓ no comment: the name is enough
pub fn focused(mut self, focused: bool) -> Self { … }
```

```rust
// ✓ a comment that stays (platform/linux/vulkan.rs)
// SAFETY: instance, device and queue live in `Self` and outlive the Skia context.
```

- An app starts with `use bk::prelude::*;` and
  `Application::new(..).window(..).run()`; if it draws its own title bar,
  with `.options(|w| w.window_style(WindowStyle::HiddenTitleBar))`
  (ADR 0115).
- New colors, radii, durations and metrics go to `ctdesign`, never to `bk`.
- The system decides light or dark (ADR 0114): an app does not set its scheme;
  only a part of a window can force it with `use_appearance`.
- A new control composes kit primitives; it does not reimplement the sidebar,
  the menus or the traffic lights (ADR 0047).
- Visible text goes in `i18n/{en,es}/<crate>.ftl` and is read with `tr!`;
  never a literal in the code (`docs/barrierkit/17-languages.md`).
- Symbols are SVG, never PNG: `Image::system`, or an SVG with
  `currentColor`. A one-color icon (SVG or PNG, like freedesktop's
  `-symbolic` ones) is drawn with
  `.rendering_mode(TemplateRenderingMode::Template)`: black in light and white
  in dark. App icons, photos and illustrations stay `Original`
  (`docs/barrierkit/04-appearance-and-tokens.md`).
