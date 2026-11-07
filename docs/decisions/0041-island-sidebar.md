# 0041 — Island sidebar

Status: **accepted** (2026-10-26), inherited from Malaysia. The original record, in Spanish, is
[Malaysia ADR 0041](https://github.com/HBlanqueto/malaysia/blob/3a1ed3f50321dbd76a4efab8d481b2e4e427dc21/docs/decisiones/0041-barra-lateral-en-isla.md) in its history.

## What still holds

1. The sidebar has two styles, island and flat. Island is the default.
2. Their values (inset, radius, rim, colors, shadow and blur) live in
   `components.window.sidebar.island` and `.flat` of
   `crates/ctdesign/tengen/components.json`, never in the apps.

## What changed

- The choice is the user's `sidebar_style` in
  `ctappearance::AppearanceConfig`, not a `default-style` token.
- An app can change any of the values through `SidebarStyle`
  ([0064](0064-styles-and-themes.md)).
