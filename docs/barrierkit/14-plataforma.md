# 14. Plataforma

**Objetivo:** entender qué hace el runner del kit por debajo de los controles.
**Lector:** quien trabaja dentro del kit o depura una app en otro escritorio.

## El runner

`platform::run_window` abre una conexión Wayland con smithay-client-toolkit,
crea una superficie `xdg_toplevel` y corre un bucle `calloop`. En cada frame:

1. Atiende los eventos de Wayland (entrada, configure, frame callbacks).
2. Actualiza el árbol de Freya y calcula el diseño con `torin`.
3. Dibuja con Skia sobre una imagen de la swapchain Vulkan y la presenta.

El kit redibuja cuando cambia el estado o la apariencia, al ritmo de los frame
callbacks del compositor.

## Vulkan

| Paso | Detalle |
|---|---|
| Instancia | `VK_KHR_surface` y `VK_KHR_wayland_surface` |
| Dispositivo | El primero que puede presentar en el display; la capa device-select de Mesa pone primero la GPU del compositor |
| Contexto | Un `DirectContext` de Skia compartido por todas las ventanas, con 128 MB de caché; cada 10 s libera lo que lleva 10 s sin usarse |
| Swapchain | Una por superficie, `B8G8R8A8_UNORM`, alfa premultiplicado; `MAILBOX` si existe, si no `FIFO`; una `SkSurface` por imagen. Con `VK_EXT_swapchain_maintenance1`, cada present lleva un fence y un redimensionado retira el swapchain viejo sin esperar a la GPU; se destruye cuando señala su último present |

Con la feature `video` el dispositivo activa además las extensiones de
importación dmabuf (capítulo [13](13-fondos-y-medios.md)). La decisión está en
el [ADR 0056](../decisiones/0056-barrierkit-sobre-vulkan.md).

## Protocolos Wayland

| Protocolo | ¿Obligatorio? | Para qué |
|---|---|---|
| `wl_compositor`, `xdg_wm_base`, `wl_seat`, `wl_output` | Sí | Ventanas, entrada y pantallas |
| `wp_fractional_scale_manager_v1` + `wp_viewporter` | No | Escala fraccional; sin ellos, escalas enteras |
| `wp_cursor_shape_manager_v1` | No | Cursores por componente (`col-resize`, texto…); sin él, el cursor por defecto |
| `zxdg_decoration_manager_v1` | No | Decoración del servidor si la app la pide |
| `xdg_activation_v1` | No | Traer la ventana al frente al reactivar una instancia |
| `wl_data_device_manager` | No | Portapapeles |
| `org_kde_kwin_appmenu_manager` | No | Enseñar el menú global en la barra de menús |
| `infinity_focus_manager_v1` | No | Avisos del compositor de Malaysia sobre la barra de foco |

Si falta un protocolo opcional, el runner lo registra con `tracing::info!` y
sigue. Si falta uno obligatorio, `run_window` devuelve
`RunError::MissingGlobal`.

## Entrada, foco y accesibilidad

- El teclado va al nodo enfocado; Tab activa la navegación por teclado y el
  anillo de foco.
- El árbol de accesibilidad se publica por AT-SPI con `accesskit_unix`: un
  lector de pantalla puede enfocar y pulsar.
- `Cursor::set(CursorIcon::..)` cambia el cursor si hay `wp_cursor_shape_manager_v1`.

## La parte de la shell (feature `shell`)

Solo `malaysia-shell` activa `barrier-kit/shell`. Añade:

| API | Protocolo | Para |
|---|---|---|
| `shell::run_layer(app, LayerSpec)` | `zwlr_layer_shell_v1` | Barra de menús, Dock, fondo |
| `shell::run_lock(app)` | `ext_session_lock_v1` | Pantalla de bloqueo |
| `shell::Toplevels` | `ext_foreign_toplevel_list_v1`, `zwlr_foreign_toplevel_manager_v1` | Lista y control de ventanas para el Dock |
| `shell::InfinityLayout`, `Toplevels::arrange` | `infinity_window_manager_v1` | Colocar ventanas en mosaico |
| `shell::Activation` | `xdg_activation_v1` | Lanzar apps desde el Dock |

Los submódulos de `shell/` son privados y su API sale solo de `shell/mod.rs`.
`platform/` no sabe nada de la shell: la shell se engancha por una única
`Extension` del runner ([ADR 0047](../decisiones/0047-barrierkit-en-capas.md) §4).

## Errores

`RunError` tiene las variantes `NoDisplay`, `MissingGlobal`, `Vulkan`,
`EventLoop`, `Skia`, `Wayland`, `Registry` e `Io`.

## El primer frame

El kit dibuja el primer frame de una ventana o una capa al recibir su primer
configure, sin esperar un frame callback. Así lo piden `xdg_surface` y
`zwlr_layer_surface_v1`, y un compositor puede no enviar frame callbacks a una
superficie sin contenido. Los frames siguientes esperan su frame callback.
Comprobado el 2026-11-03 en Weston 16 (ventana) y el 2026-11-04 en sway 1.12
(capa), ambos sin pantalla.

Siguiente: [15. Apps de prueba](15-apps-de-prueba.md).
