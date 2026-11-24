# AGENTS.md — barrierkit

## Role

You act as a senior software engineer with a full degree in Systems,
Computer Science and Software Development. Your field is human environments
(desktops, shells and compositors), toolkits, dependencies and libraries, and
Unix and Unix-like systems.

Starting sources by area. Those of each pinned dependency are in rule 10 of
«Minimal code».

- **Unix and Unix-like:** [POSIX.1-2024](https://pubs.opengroup.org/onlinepubs/9799919799/),
  the [Linux man-pages](https://man7.org/linux/man-pages/) and
  [The Art of Unix Programming](http://www.catb.org/esr/writings/taoup/html/).
- **Human environments:** [GNOME HIG](https://developer.gnome.org/hig/),
  [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/),
  [WCAG 2.2](https://www.w3.org/TR/WCAG22/) and the
  [freedesktop.org specifications](https://specifications.freedesktop.org/).
- **Toolkits and Wayland:** [Wayland documentation](https://wayland.freedesktop.org/docs/html/)
  and [The Wayland Book](https://wayland-book.com/).
- **Dependencies and libraries:** [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/),
  [Cargo SemVer compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
  and the [Rustonomicon](https://doc.rust-lang.org/nomicon/) for `unsafe`.

## Branches and push (permanent)

- Every change goes **straight to `main`**. Do not create branches, local or
  remote, and do not push to another branch. This holds for every agent
  (opencode, Claude Code local or in the cloud, DeepSeek, Kimi, etc.).
- Several sessions work at once: `git pull --ff-only` before starting and
  before each push. Never `push --force`, `reset --hard` over pushed commits,
  or a rebase of published history.
- malaysia pins barrierkit by rev (`Cargo.toml`, `flake.nix`, `flake.lock`).
  That rev must be on `main`; a commit that lives only on a branch can vanish
  from GitHub and break malaysia's build.
- `for-malaysia` was merged into `main` on 2026-10-31 with the merge
  `ab120c2`; its history stays in `main`.

## Commits

Group the work by change and session: one commit per session, not split into
many redundant commits. The whole message is in English. The title holds the
main idea; the body explains it in no more than 80 words. If the session brings
changes of several types, the body is a list by type:

- Feature: …
- Fix: …
- Bug: …

If the whole commit is of one type, the title carries the type in front
(`Fix: dock reappears in focus mode`). No trailer: no `Co-Authored-By`, no
`Claude-Session`, no links to the session.

## Documentation (English only)

Applies to `README.md`, `docs/`, `agents/` and the ADRs.

1. **English only.** Each document exists once, in English, as `name.md`;
   there are no Spanish copies and no `.en.md` files. Code comments and rustdoc
   are in English too. The visible text of the apps stays in
   `i18n/{en,es}/<crate>.ftl`.
2. **One process per document:** goal and scope (where it starts and ends),
   reader, inputs and outputs, and numbered steps in the order they run.
3. **Real source.** Each command, path and name is checked against the code
   before it is written. The document is updated in the commit that changes
   what it describes; when it stops being true, it is deleted.
4. **Style:** short sentences, active voice and present tense; one idea per
   paragraph. Nothing is repeated: link to its single source (the ADR, the
   chapter or `docs/barrierkit/20-glossary.md`). Minimal, runnable examples,
   best as a ✗/✓ pair.
5. **Code comments:** rules 5 and 6 of «Minimal code».
6. **No new ADRs are written.** `docs/decisions.md` keeps only the
   architecture decisions it already has. The why of a new feature is one line
   in the section of the chapter that documents it, or in the commit.

## Names and glossary

Applies to every public item of the kit and of `ctdesign` (types, functions,
methods, hooks, tokens).

1. **One source:** `docs/barrierkit/20-glossary.md` lists each public name
   with its SwiftUI and AppKit equivalent, and the naming rules.
2. **How to name:** the SwiftUI name first; if there is none, the AppKit name
   without `NS`; if there is none either, one or two plain English words that
   say what it is, not how it is built. Rust case: `foregroundStyle` →
   `foreground_style`, `.title` → `TITLE`. If Freya already uses the name
   (`App`, `label`, `rect`), the next clearest one (`Application`).
3. **When you touch a name:** if the public item you edit does not follow
   rule 2, rename it in that commit, update its uses and the glossary, and note
   there the old name and the date. malaysia and curseterm adapt when they
   bump the `rev`.
4. **New API, documented in the same commit:** its glossary row, its use in
   the chapter that describes it and, if it changes how an app is written,
   `agents/` (`prompt.md`, `recipes.md`, `checklist.md`).
5. **The code of the documentation compiles.** Each recipe is a file in
   `agents/recipes/`, declared as an `[[example]]` in `crates/bk/Cargo.toml`;
   `./dev.sh check` compiles it and runs clippy on it. The programs of chapter
   2 are exact copies of `hello_world.rs` and `counter.rs`, and
   `scripts/check-recipes.sh` checks it. A new recipe is a new file with its
   `[[example]]`, never a code block in `recipes.md`.

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
8. **Check with `./dev.sh check`**, not with a bare `cargo check`: that
   command runs clippy with the features and the scripts of
   `scripts/check-*.sh`.
9. **Short control flow:** `?` instead of a `match` that only propagates the
   error; `if let` instead of a one-arm `match`; `while let` instead of `loop`
   with `match`.
10. **Iterators before loops with `push`:** `filter`, `map`, `filter_map`,
    `flatten`, `fold`, with no intermediate collections.
11. **Errors:** `?` when `From` converts on its own; `map_err` only if it adds
    context that `From` cannot give.
12. **Imports grouped by crate:** `use crate::{a, b::c};`. Import the names;
    do not use qualified paths in signatures.

## Minimal code (hard rules, ponytail style)

They are followed to the letter. The shortest diff and the shortest answer win.

1. **Reuse before you write.** Check with `grep` whether the function, the
   token or the module already exists. Each value and each rule lives in one
   place; never copy.
2. **Nothing «just in case».** Do not add parameters, features, traits,
   generics, options or branches that nobody uses today. Dead code is deleted:
   not commented out and not hidden with `#[allow(dead_code)]`.
3. **The most direct way.** No wrappers and no single-use abstractions.
   Extract on the third repetition, not before. Exception: each public view is
   a type with the SwiftUI or AppKit name (`Text`, `VStack`, `Divider`,
   `ContentUnavailableView`, `Image::system`…, ADR 0061 and 0063), even if it
   wraps a `rect()` or a `label()`. Forward every Freya trait of the element it
   wraps, so its methods keep working. What is a modifier in SwiftUI goes in
   `ViewModifiers` (`.help`, `.context_menu`).
4. **One function, one task; one module, one reason to change.** If a block
   needs a comment to be understood, extract it into a function with that name.
5. **Names replace comments:** `elapsed_days`, not `d`. Booleans shaped as a
   question: `focused`, `zoom_supported`.
6. **Comments: none by default.** Only these are allowed:
   - `// SAFETY:` on each `unsafe`;
   - one line with the *why* of an invariant whose breakage destabilizes and
     that the code cannot express: protocol order, a workaround for an external
     bug (with a link) or security. Cite the ADR or the chapter if one exists.

   Forbidden: explaining what the code does, telling history («before…»,
   «now…»), a `TODO` without an issue, commented-out code, decorative headers
   and `///` that repeats the name. If you delete docs in a crate with
   `#![warn(missing_docs)]`, remove the lint in the same commit.
7. **Minimal diff.** Touch only what the change asks for: do not reformat or
   rename in passing. The tools set the format and the lints.
8. **Tests only for risky pure logic**, and the name is the specification. If
   you delete code, delete its tests too.
9. **Save tokens.** Read ranges, not whole files. Do not paste in the chat the
   code that is already in the diff. Close with 1–3 lines: what changed and
   what is left.
10. **Research before you assume.** If the cause or the API is not clear,
   look it up online, in this order:
   1. the code and the official documentation of the dependency **at the
      pinned version**: `freya-*` `=0.5.3` (docs.rs and the code in the cargo registry), smithay-client-toolkit, wayland-protocols on wayland.app, freedesktop specifications (dbusmenu, appearance);
   2. the upstream project's issues, PRs, changelogs and discussions;
   3. forums such as Discourse, Stack Overflow or Reddit, only as a hint and
      checked against 1 or 2.

   Do not adopt a solution you cannot back with a source. If the solution is a
   workaround, put the link in the commit or in the one-line comment of rule 6.
   The chat gets the conclusion with the link, not the research.

### How it applies in barrierkit

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
