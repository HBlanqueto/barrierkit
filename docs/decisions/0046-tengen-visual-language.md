# 0046 — Tengen: renaming the visual language

Status: **accepted** (2026-10-28).

## Context

The desktop's visual language was called **Neptunia**. From this date on it
is called **Tengen**. The names of the subsystems around it do not
change:

- **Tengen** = the visual language itself (the equivalent of Aqua in macOS).
- **VowDesign** = the compiled tokens (`crates/vow-design`).
- **VowAppearance** = the resolved active appearance (`crates/vow-appearance`).
- **BarrierKit** = the shell toolkit (`crates/barrier-kit`).

Only the name of the visual language and of its source folder change; colors,
radii, shadows, measurements and behaviors are exactly the same.

## Decision

1. **Full rename.** The following table summarises the mapping:

   | Before | After |
   |---|---|
   | `crates/vow-design/neptunia/` | `crates/vow-design/tengen/` |
   | `$OUT_DIR/neptunia.rs` | `$OUT_DIR/tengen.rs` |
   | BarrierKit themes `neptunia-shell`, `neptunia-shell-light`, `neptunia-shell-dark`, `neptunia-lock` | `tengen-shell`, `tengen-shell-light`, `tengen-shell-dark`, `tengen-lock` |

2. **No compatibility aliases.** The code keeps no old names
   and old paths are not read.

3. **Historical documentation untouched.** The ADRs before 0046, the
   block reports, the research and the audit keep the name
   **Neptunia** because they are a historical record. ADR 0032 receives only one line
   that refers to this decision; the rest of its text does not change.

## Consequences

- The desktop's design source is now called **Tengen** throughout the
  code, the scripts, the living documentation and the generated artifacts.
- There is no visual change: the tokens, colors and measurements are identical.
- Any external reference to old paths or types must be updated by
  hand.

## Revision of 2026-11-05

The Slint, CSS and QML generator in `tengen/` (`default.nix`, `generated/`,
`demo.slint`) and the JSON fields only it read are deleted: nobody consumed
them. Tengen produces only the `vow-design` constants. The living process is
in `docs/architecture/tengen.md`.

## References

- `crates/vow-design/tengen/` — renamed source folder.
- `crates/vow-design/build.rs` — generates `$OUT_DIR/tengen.rs`.
- `docs/decisions/0032-neptunia-as-design-source.md` — original ADR,
  now with a referral line.
- `docs/architecture/tengen.md` — living documentation of the visual language.
