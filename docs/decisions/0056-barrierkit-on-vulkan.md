# 0056 — BarrierKit draws with Vulkan

Status: **accepted** (2026-11-01).

## Context

The kit's runner (ADR 0034) drew Skia on EGL/GLES. Freya 0.4.3 already builds Skia with the `vulkan` feature on Linux (`freya-engine`), but its Vulkan driver lives in `freya-winit`, which the kit cannot use.

## Decision

1. `platform/vulkan.rs` replaces `platform/egl.rs`, with no GL fallback. It uses `ash` with `VK_KHR_wayland_surface` on SCTK's `wl_display` and `wl_surface`, and it is ported from the `freya-winit` driver.
2. There is one shared instance, device and `DirectContext` (`GraphicsShared`), and one swapchain per surface.
3. The GPU is the first one that can present on the display. Mesa's device-select layer puts the compositor's GPU first.
4. `MAILBOX` is used if it exists, because the frame callbacks already set the pace (freya#2260). If it does not exist, `FIFO`. The composite alpha is `PRE_MULTIPLIED`.

## Consequences

- The kit's clients need `libvulkan` and an ICD. The dev shell adds the flake's Mesa ICDs with `VK_ADD_DRIVER_FILES`, so the system's drivers stay available.
- Since Freya 0.5, `freya-engine` takes Skia's backends as features. `bk` enables only `vulkan`, so Skia no longer builds or links GL, EGL or X11.
