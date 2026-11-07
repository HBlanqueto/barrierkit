# 0035 — Identifying own apps and CSD

Status: **accepted** (2026-10-23), inherited from Malaysia. The full record is
[Malaysia ADR 0035](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0035-identificacion-de-apps-propias-y-csd.en.md).

## What still holds in the kit

1. A window's `app_id` is `ctdesign::NATIVE_APP_ID_PREFIX` followed by the
   app's name, unless `WindowOptions` gives another one.
2. Every window asks for client-side decoration
   (`WindowDecorations::RequestClient`). If the compositor answers with
   server-side decoration, the kit turns off its edge resizing
   ([18](../barrierkit/18-platform.md#wayland-protocols)).

How the compositor treats the prefix is Malaysia's part of the record.
