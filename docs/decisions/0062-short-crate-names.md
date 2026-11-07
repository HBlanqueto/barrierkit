# 0062 — Short crate names: `bk`, `ctdesign` and `ctappearance`

Status: **accepted** (2026-11-07).

**Goal:** record why the crates are named `bk`, `ctdesign` and `ctappearance`.
**Scope:** package, folder, crate and translation-domain names; no item
inside the crates changes name.
**Reader:** anyone who depends on these crates.
**Input:** the crates `barrier-kit`, `vow-design` and `vow-appearance`.
**Output:** the same crates under short names.

## Context

Paths such as `barrier_kit::views::form::Form` were long, and the token and
appearance crates still carried the "Vow" name of
[ADR 0043](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisions/0043-rebranding-to-malaysia.md).
The project now groups those two under **CT**, for *Curse Techniques*.

Rust already names an item through its crate (`bk::Button`), so a short crate
name does the job a prefix like AppKit's `NS` does in Objective-C, without
repeating it in every type.

## Decision

1. `barrier-kit` becomes **`bk`**: package `bk`, folder `crates/bk`, Rust crate
   `bk`, Fluent domain `bk` (`i18n/{en,es}/bk.ftl`). In prose it is still
   **BarrierKit**.
2. `vow-design` becomes **`ctdesign`** (CTDesign): package and crate `ctdesign`,
   folder `crates/ctdesign`.
3. `vow-appearance` becomes **`ctappearance`** (CTAppearance): package and crate
   `ctappearance`, folder `crates/ctappearance`.
4. The names are one lowercase word, as Cargo and the Rust naming conventions
   expect ([Rust API Guidelines, C-CASE](https://rust-lang.github.io/api-guidelines/naming.html)).
5. Commit `a6315d1` used `ct-design` and `ct-appearance` for a few hours; the
   old names are listed in the [glossary](../barrierkit/20-glossary.md#renamed-items).

## Consequences

- An app depends on `bk` and writes `use bk::prelude::*;`. Tokens are
  `ctdesign::…`; the appearance API is `ctappearance::…`.
- Features are `bk/shell`, `bk/tracy` and `bk/video`.
- malaysia and curseterm rename their dependencies, paths and feature flags
  when they move their pinned `rev` past this change:

  ```sh
  git grep -lE 'barrier[-_]kit|vow[-_](design|appearance)|ct[-_](design|appearance)' | xargs sed -i -E 's/barrier[-_]kit/bk/g; s/(vow|ct)[-_]design/ctdesign/g; s/(vow|ct)[-_]appearance/ctappearance/g'
  ```
