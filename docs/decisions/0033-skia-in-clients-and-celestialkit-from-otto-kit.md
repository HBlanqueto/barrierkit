# 0033 — Skia in clients and CelestialKit from otto-kit

Status: **superseded by [0034](0034-celestialkit-on-freya.md)**. It modified [ADR 0024](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0024-stack-de-render-del-toolkit.md).

Yuki is now Malaysia and CelestialKit (`celestial-kit`) is BarrierKit (`barrier-kit`) since [ADR 0043](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0043-rebranding-a-malaysia.md), whose crate is `bk` since [ADR 0062](0062-short-crate-names.md).

## Context

ADR 0024 chose GLES + `cosmic-text` + own shaders for the whole stack and
left `celestial-kit` as a toolkit still to be written. Otto has already solved that
same problem with `otto-kit` (MIT, Riccardo Canalicchio) and Skia. Rewriting by
hand what already exists is worse than reusing it with a clear boundary: the
compositor does not need Skia, but the shell and own apps do benefit
from a full 2D canvas.

The important boundary is still the one in ADR 0024: **the compositor draws
windows and effects; the shell and own apps draw their own UI**. They are
separate processes and can use different render stacks without duplicating the
visual language, because the tokens live in `yuki-design`.

## Decision

1. **Skia is allowed in client processes** (`yuki-shell` and own apps)
   and **never in the compositor**. The compositor keeps GLES, `grade-graphics`
   and `cosmic-text`.
2. **CelestialKit (`celestial-kit`, ≈ AppKit) starts from `otto-kit`** (MIT,
   Riccardo Canalicchio). It is brought in **selectively**, renamed to its own
   names (no `otto_*`) and uses the `yuki-design` tokens.
3. **Grade Animation (≈ Core Animation) becomes the `grade-animation` crate**.
   **It does not depend on `laye-rs` while that crate has no licence**: it is
   reimplemented taking only ideas (curves, springs and transactions), as
   `docs/investigacion/otto-y-forks.md` states. If upstream publishes a compatible
   licence, a new ADR can revisit this decision.
4. **Infinity (≈ SkyLight) is the private app↔compositor protocol**, the equivalent
   of `otto-surface-style`: corners, shadows, blur and animations. Until
   the compositor implements it, the kit treats it as **optional**.
5. **Grade Graphics stays as the compositor's shaders.** The kit matches the
   look with the same `yuki-design` tokens; it does not need the compositor's
   shaders to draw its own content.
6. **`otto-foundations`, `otto-dbus` and `otto-surface-style` are not used.**
   They are replaced by `yuki-design`, `yuki-appearance` and Infinity,
   respectively.

## Consequences

- The compositor gains no Skia dependency and does not change its render path
  (ADR 0009, 0024, 0027).
- `celestial-kit` can reuse widgets, layout and drawing from `otto-kit`, but
  with its own names, no `otto_*` in the public API and with the
  `yuki-design` tokens as the only source of colours and metrics.
- Animations stop being an internal module and move to `grade-animation`,
  an own implementation of curves, springs and transactions. It contains no
  code from `laye-rs`.
- Infinity can appear in the kit from the start as an optional capability:
  if the compositor does not advertise it, the kit keeps working with standard Wayland.
- `grade-graphics` remains the compositor's visual meeting point; the
  kit does not use it to render, it only shares the tokens.
- The MIT notice of `otto-kit` lives in `NOTICE`. `laye-rs` has no licence and is
  neither used nor copied.

## References

- [`0024`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0024-stack-de-render-del-toolkit.md) — toolkit render stack,
  modified by this ADR.
- [`0020`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0020-politica-de-diseno-y-unificacion.md) — chrome and effects.
- [`0011`](0011-design-values-as-rust-constants.md) — tokens in
  `yuki-design`.
- [`0023`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0023-los-componentes-del-shell-corren-como-procesos-separados.md) —
  one process per component.
- [`0025`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0025-la-shell-es-nativa.md) — native shell `yuki-shell`.
- [`0032`](0032-neptunia-as-design-source.md) — Neptunia as the source.
- `otto-kit` — MIT, <https://github.com/nongio/otto>.


## Correction (2026-10-22)

`laye-rs` (`nongio/layers` and the fork `HBlanqueto/layers`) publishes no licence:
there is no `LICENSE` and no `license` field in its `Cargo.toml`. Therefore:

- The MIT notice of `laye-rs` is removed from `NOTICE`; there is none of its code in the
  repository.
- `grade-animation` does not depend on `laye-rs`. It is reimplemented with its own ideas
  (curves, springs and transactions), as
  `docs/investigacion/otto-y-forks.md` already stated.
- If upstream publishes a compatible licence, a new ADR can revisit the
  decision.


## Correction

`laye-rs` has no licence and **was never a dependency** of the repository.
ADR 0034 supersedes this decision: CelestialKit is built on Freya's core
crates and the animation engine is an own module on top of
`freya-animation`.
