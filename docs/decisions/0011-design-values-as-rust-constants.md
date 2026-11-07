# 0011 — Design values are hand-written Rust constants in `yuki-design`

Status: accepted (block 0, supersedes ADR 0008)

`yuki-design` was renamed to `vow-design` in [ADR 0043](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0043-rebranding-a-malaysia.md).

## Context

ADR 0008 defined a contract in which `design/v1.nix` was the only hand-authored
source and a generator would emit a versioned Rust module inside
`yuki-design`. That contract was written before any value existed.

Block 0 then had to populate the crate. There is no rendering and no
consumer yet, there is exactly one look in two modes, and the project
explicitly does not want runtime theming, a configuration file or
per-application rules yet. Building a generator now would be
machinery for a problem that has not appeared: it would add a Nix-to-Rust step,
a staleness check and a regeneration command to keep a
handful of constants in sync with a second declaration of themselves.

The values themselves are the system defaults: titlebar metrics,
window shape, typography choices, palettes and
shadows. They are design choices, not algorithmic constants, so
they belong in `yuki-design` instead of next to their future consumers.

## Options considered

1. **Keep ADR 0008: `design/v1.nix` plus a generator.** A single
   hand-authored place with a reviewable generated diff. But it means two representations
   of the same constants and a build step to maintain, for values
   with a single consumer that does not exist yet, and with no runtime theming to
   justify the layer.
2. **Write the constants by hand in `yuki-design`.** A single source, in
   the language that uses it, reviewable and directly checkable. No generation,
   no `design/v1.nix`, no build dependency. The cost is that if a
   non-Rust consumer (for example a future documentation or
   asset pipeline) ever needs the values, a generator will have to be introduced
   later.
3. **Constants at the point of use.** What ADR 0008 was written to
   avoid: the same colour copied into several modules and drifting out of sync.
   Rejected.

## Decision

Option 2. `yuki-design` owns the design values as public
compile-time constants: `Rgba`, `Palette`, `LIGHT`, `DARK`,
`TITLEBAR`, `WINDOW`, `TEXT` and `SHADOW`. There is no `design/v1.nix`, no generator,
and no runtime override. Each value is documented with its unit, and
`yuki-design` is the only crate that may define a design value.

The two states of a window are modelled explicitly: a colour that differs
between focus states is a `FocusPair`, and the inner line is an
`Option<Rgba>` so that a renderer can skip the pass in light mode
instead of drawing a fully transparent line.

This inverts the contract of ADR 0008. If a second non-Rust consumer
of these values appears, the generator decision can be revisited then,
with a concrete consumer to justify it.

## Consequences

- ADR 0008 is superseded; `design/v1.nix` is removed.
- A design change is an ordinary Rust diff, and the compiler checks it
  like any other constant.
- There is no ready path for a non-Rust consumer. Introducing one
  later means writing the generator that ADR 0008 described, now against
  a real need.
- `yuki-design` stays without dependencies and without runtime
  behaviour; it is only data and the constructors that make the literals readable.
- Per-application rules and runtime theming remain
  explicitly out of scope and undecided, as
  recorded in the Foundations document.