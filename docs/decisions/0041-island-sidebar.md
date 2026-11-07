# 0041 — Island sidebar

Status: **accepted** (2026-10-26), inherited from Malaysia. The full record is
[Malaysia ADR 0041](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0041-barra-lateral-en-isla.en.md).

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
