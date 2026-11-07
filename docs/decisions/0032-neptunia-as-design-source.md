# 0032 — A JSON design source

Status: **accepted**, inherited from Malaysia. The full record is
[Malaysia ADR 0032](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0032-neptunia-como-fuente-de-diseno.en.md).

## What still holds

1. The look comes from two JSON files, `crates/ctdesign/tengen/tokens.json`
   and `components.json`: colors, radii, spacing, typography, shadows and
   window metrics.
2. The themes are `light` (the default), `dark` and `dark-tint`.
3. The kit implements that look itself, on its own runner.

## What changed

- The design language is called Tengen, not Neptunia
  ([0046](0046-tengen-visual-language.md)).
- The window metrics of the original record (12 px traffic lights, a 44 px
  bar) are superseded: the current values are in `components.json`.
