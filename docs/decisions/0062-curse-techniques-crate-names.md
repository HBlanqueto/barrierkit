# 0062 — CTDesign and CTAppearance: the Curse Techniques crate names

Status: **accepted** (2026-11-07).

**Goal:** record why the token and appearance crates are named `ct-design`
and `ct-appearance`. **Scope:** crate, folder and path names; no API changes.
**Reader:** anyone who depends on these crates. **Input:** the crates
`vow-design` and `vow-appearance`. **Output:** the same crates under new names.

## Context

The two crates carried the "Vow" name since the rebranding of
[ADR 0043](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0043-rebranding-a-malaysia.md).
The project now groups them under **CT**, for *Curse Techniques*.

## Decision

1. `vow-design` becomes **CTDesign**: package `ct-design`, path
   `crates/ct-design`, Rust crate `ct_design`.
2. `vow-appearance` becomes **CTAppearance**: package `ct-appearance`, path
   `crates/ct-appearance`, Rust crate `ct_appearance`.
3. Package and crate names stay lowercase, as Cargo and the Rust naming
   conventions require
   ([Rust API Guidelines, C-CASE](https://rust-lang.github.io/api-guidelines/naming.html)).
   `CTDesign` and `CTAppearance` are the names used in prose.
4. Nothing inside the crates changes name; the old names are listed in the
   [glossary](../barrierkit/20-glossary.md#renamed-items).

## Consequences

- An app imports `ct_design::…` and `ct_appearance::…`.
- malaysia and curseterm change `vow-design` and `vow-appearance` in their
  `Cargo.toml` and `vow_design::`/`vow_appearance::` in their code when they
  move their pinned `rev` past this change:

  ```sh
  git grep -l 'vow[-_]\(design\|appearance\)' | xargs sed -i 's/vow\([-_]\)design/ct\1design/g; s/vow\([-_]\)appearance/ct\1appearance/g'
  ```
