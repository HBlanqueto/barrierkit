# 0034 — BarrierKit on Freya

Status: **accepted** (2026-10-22), inherited from Malaysia. Supersedes
[0033](0033-skia-in-clients-and-celestialkit-from-otto-kit.md). The original record, in Spanish, is
[Malaysia ADR 0034](https://github.com/HBlanqueto/malaysia/blob/c5f3060f66273c141168159017c90f98de4d87f1/docs/decisiones/0034-celestialkit-sobre-freya.md) in its history.

## What still holds

1. The kit is built on Freya's core crates, pinned exactly: `freya-core`,
   `freya-components` (without default features), `freya-edit`,
   `freya-animation`, `freya-engine`, `freya-clipboard`, `torin` and
   `ragnarok`. It never uses `freya` or `freya-winit`;
   `scripts/check-client-only-deps.sh` enforces it.
2. The kit has its own Wayland runner on smithay-client-toolkit and
   `calloop` ([18](../barrierkit/18-platform.md#the-runner)).
3. Skia runs only in client processes. The compositor never links `freya-*`.

## What changed

- The runner draws with Vulkan, not EGL ([0056](0056-barrierkit-on-vulkan.md)).
- The crates are `bk`, `ctdesign` and `ctappearance`, in their own
  repository ([0048](0048-barrierkit-own-repository.md),
  [0062](0062-short-crate-names.md)).
