# 0056 — BarrierKit dibuja con Vulkan

Estado: **aceptado** (2026-11-01).

## Contexto

El runner del kit (ADR 0034) dibujaba Skia sobre EGL/GLES. Freya 0.4.3 ya compila Skia con la feature `vulkan` en Linux (`freya-engine`), pero su driver Vulkan vive en `freya-winit`, que el kit no puede usar.

## Decisión

1. `platform/vulkan.rs` sustituye a `platform/egl.rs`, sin respaldo GL. Usa `ash` con `VK_KHR_wayland_surface` sobre el `wl_display` y el `wl_surface` de SCTK, y está portado del driver de `freya-winit`.
2. Hay una instancia, un dispositivo y un `DirectContext` compartidos (`GraphicsShared`), y una swapchain por superficie.
3. La GPU es la primera que puede presentar en el display. La capa device-select de Mesa pone primero la del compositor.
4. Se usa `MAILBOX` si existe, porque los frame callbacks ya marcan el ritmo (freya#2260). Si no existe, `FIFO`. El alfa compuesto es `PRE_MULTIPLIED`.

## Consecuencias

- Los clientes del kit necesitan `libvulkan` y un ICD. El dev shell apunta `VK_DRIVER_FILES` al RADV del flake, como `malaysia-wallpaper` (ADR 0049).
- Skia sigue enlazando GL por las features que fija `freya-engine`. Esas librerías se quedan en el flake aunque el kit ya no las use.
