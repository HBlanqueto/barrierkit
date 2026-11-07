# 1. Qué es BarrierKit

**Objetivo:** saber qué es el kit, qué trae y qué necesita para funcionar.
**Lector:** cualquiera que empiece con el kit.

## En una frase

BarrierKit es a Malaysia lo que AppKit es a macOS o GTK + libadwaita a
GNOME: el conjunto de piezas con el que se construyen las apps propias y la
shell. Una app no dibuja botones ni barras laterales: los pide al kit, y todas
se ven y se comportan igual.

## Equivalencias

| Concepto | AppKit / SwiftUI | libadwaita | BarrierKit |
|---|---|---|---|
| Estilo de la API | SwiftUI declarativo | GtkBuilder + señales | Declarativo, sobre Freya |
| Botón | `NSButton` | `GtkButton` | `controls::Button` |
| Barra lateral + detalle | `NSSplitView` | `AdwNavigationSplitView` | `views::split::SplitView` |
| Lista de origen | `NSOutlineView` (source list) | `GtkListBox` `.navigation-sidebar` | `sidebar::Sidebar` |
| Formulario | `Form` (SwiftUI) | `AdwPreferencesGroup` | `views::form::Form` |
| Ventana de preferencias | `Settings` | `AdwPreferencesWindow` | `patterns::preferences::PreferencesWindow` |
| Acciones y atajos | `NSMenuItem` + responder chain | `GAction` | `app::Action` |
| Preferencias de la app | `UserDefaults` | `GSettings` | `app::load_defaults` / `save_defaults` |
| Colores del sistema | `NSColor.labelColor`… | variables CSS | `style::Palette` |

## Los crates

El repositorio [`barrierkit`](https://github.com/HBlanqueto/barrierkit) tiene
cuatro crates de biblioteca:

| Crate | Qué es |
|---|---|
| `barrier-kit` | El kit: runner Wayland, estilo, controles, vistas, patrones e infraestructura de app. |
| `vow-design` | **Tengen**, el sistema de diseño. Convierte `tengen/tokens.json` y `tengen/components.json` en constantes Rust. |
| `vow-appearance` | Lee `appearance.domain` y el fondo de pantalla y los resuelve en una paleta concreta. Vigila los cambios. |
| `infinity-protocol` | Protocolo Wayland privado entre las apps y el compositor de Malaysia. |

El kit se apoya en [Freya](https://freyaui.dev) `=0.4.3`, pero solo en sus
crates de núcleo: `freya-core`, `freya-components` (sin features),
`freya-animation`, `freya-engine`, `torin` y `ragnarok`. Nunca en `freya` ni en
`freya-winit`: el kit tiene su propio runner Wayland ([ADR 0034](../decisiones/0034-celestialkit-sobre-freya.md)).

## ¿Usa GPU o CPU?

| Trabajo | Dónde se hace |
|---|---|
| Diseño (layout) con `torin` | CPU |
| Dibujo con Skia | **GPU, por Vulkan** ([ADR 0056](../decisiones/0056-barrierkit-sobre-vulkan.md)) |
| Presentación | Swapchain Vulkan sobre `VK_KHR_wayland_surface`, `MAILBOX` si existe, si no `FIFO` |
| Vídeo (feature `video`) | GPU con VA-API e importación dmabuf; si falta, descarga a la CPU o decodifica en CPU |

No hay camino por EGL ni por `wl_shm`. El kit elige el primer dispositivo
Vulkan que puede presentar en el display. Ese dispositivo puede ser una GPU
real o un driver por software como lavapipe (`llvmpipe`): las capturas de esta
documentación se hicieron así, sin GPU. Si no hay ningún dispositivo Vulkan, la
ventana no abre y `run_window` devuelve `RunError::Vulkan`.

Freya 0.4.3 repinta el árbol entero en cada frame; aún no hay damage parcial
([ADR 0047](../decisiones/0047-barrierkit-en-capas.md), «Pendiente»).

## Qué necesita

- Un compositor Wayland con `wl_compositor`, `xdg_wm_base` y `wl_seat`. El resto
  de protocolos es opcional (capítulo [14](14-plataforma.md)).
- `libvulkan` y un driver Vulkan (ICD).
- Compilar dentro de `nix develop` (ADR 0007 de Malaysia): el dev shell
  trae Skia, las fuentes (Geist) y el driver RADV.

Siguiente: [2. Tu primera app](02-primera-app.md).
