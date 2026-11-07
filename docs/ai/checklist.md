# Checklist before answering

**Goal:** catch the usual mistakes before a program or a change is handed in.
**Scope:** the code, the build, and changes to the kit itself.
**Reader:** an AI agent sent here by [`prompt.md`](prompt.md), or a reviewer.
**Input:** finished code. **Output:** every question answered "yes", or the
reason it cannot be.

## Steps

1. Answer each question below in order.
2. Fix the code for every "no", then start again from question 1.
3. Report the questions you could not check and why.

## Code

1. Does the file start with `use barrier_kit::prelude::*;` and import only what
   the prelude lacks?
2. Is every color from `Palette::current()` (or left to `Text`), every size a
   `SPACE_*` or component token, and every font one of the typography tokens?
3. Is every visible string in `i18n/{en,es}/<crate>.ftl` and read with `tr!`
   (except in throwaway samples)?
4. Does every control receive its value and handle its change?
5. Is every name the glossary name? Nothing named `App` of your own, nothing
   that copies a kit control.
6. Are hooks (`use_state`, `use_defaults`…) at the top of the view, never
   inside a condition or a loop?
7. Are there no comments besides `// SAFETY:` and one-line invariants?
8. Is the `Application` name the same one passed to `use_app_language` and
   `load_defaults`?

## Build

9. Did you run, inside `nix develop`, `cargo build` and
   `cargo clippy -- -D warnings` with no warnings?
10. Did you run `cargo fmt`?
11. If you could not open the window (no Wayland session), did you say so?

## Changes to the kit itself

12. Is every new or renamed public item in
    [`docs/barrierkit/20-glossary.md`](../barrierkit/20-glossary.md), named by
    its rules, in the same commit?
13. Does the chapter that describes the changed behavior say what the code
    does now?
14. If the change affects how an agent writes apps, are `prompt.md`,
    `recipes.md` and this checklist updated?
