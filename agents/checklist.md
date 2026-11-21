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

1. Does the file start with `use bk::prelude::*;` and import only what
   the prelude lacks?
2. Is every color from `Palette::current()` (or left to `Text`, or a
   `ShapeStyle::Role` in a style), every size a `SPACE_*` or component token,
   and every font one of the typography tokens (except in throwaway samples)?
3. Is every visible string in `i18n/{en,es}/<crate>.ftl` and read with `tr!`
   (except in throwaway samples)?
4. Does every control receive its value and handle its change (a text field
   takes the `State<String>` it edits)?
5. Is every name the glossary name? Nothing named `App` of your own, nothing
   that copies a kit control (a knob drawn by hand included: `Switch`, `Slider`
   and `LevelSlider` hand their held knob to the compositor on their own).
6. Are hooks (every `use_*`, and `.help(..)`, which hides one) called on every
   render, never inside a condition, a loop or an iterator closure?
7. Are there no comments besides `// SAFETY:` and one-line invariants?
8. Is the `Application` name the same one passed to `use_app_language` and
   `load_defaults`?
9. Does every change to a control's look go through its style
   ([`styles.md`](styles.md)), with overrides that name only what changes and
   end in `..s`?
10. Is every symbol an SVG (`Image::system`, or an SVG with `currentColor`),
    and does every one-color image use `TemplateRenderingMode::Template` so
    it is black in light and white in dark?
    Is a new `Symbol` fitted to its SF name (ink box within 0.5, mass within
    5%, `scripts/appkit-compare.py icons`) and listed in the glossary's
    [Symbol variants](../docs/barrierkit/20-glossary.md#symbol-variants)?

11. Do the modifiers sit on the view or control they change, with outer space
    as `.margin` and not a second `.padding`? A modifier sets one node; it does
    not wrap it ([chapter 5](../docs/barrierkit/05-controls.md#modifiers-on-controls)).

## Build

12. Did you run, inside `nix develop`, `cargo build` and
    `cargo clippy -- -D warnings` with no warnings?
13. Did you run `cargo fmt`?
14. If you could not open the window (no Wayland session), did you say so?

## Changes to the kit itself

15. Is every new or renamed public item in
    [`docs/barrierkit/20-glossary.md`](../docs/barrierkit/20-glossary.md), named by
    its rules, in the same commit?
16. Does the chapter that describes the changed behavior say what the code
    does now?
17. If the change affects how an agent writes apps, are `prompt.md`,
    `recipes.md` and this checklist updated?
18. Does a new control in `bk` hold `overrides: Overrides`, call
    `forward_overrides!` and end `render` with `self.overrides.apply(root)`, so
    it takes the modifiers?
