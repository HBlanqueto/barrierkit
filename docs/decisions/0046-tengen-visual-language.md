# 0046 — Tengen: the name of the visual language

Status: **accepted** (2026-10-28), inherited from Malaysia. The original record, in Spanish, is
[Malaysia ADR 0046](https://github.com/HBlanqueto/malaysia/blob/cae57594bdd8405e4a10fd61427ff6d07beefdf7/docs/decisiones/0046-tengen-lenguaje-visual.md) in its history.

## What still holds

1. The visual language is called **Tengen**. Its source is
   `crates/ctdesign/tengen/`, and `build.rs` writes `$OUT_DIR/tengen.rs`.
2. The shell theme is `tengen-shell`.
3. No alias keeps an old name or reads an old path.

The language itself is described in
[`docs/architecture/tengen.md`](../architecture/tengen.md).
