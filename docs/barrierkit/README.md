# Documentación de BarrierKit

BarrierKit es el kit para crear las apps y la shell de MalaysiaDreams. Esta
documentación lo explica desde la primera ventana hasta el runner Wayland.

- **Lector:** quien escribe una app con el kit o trabaja dentro del kit.
- **Requisitos:** Rust básico y saber qué es un compositor Wayland.
- **Fuente:** el código de `main` a 2026-11-03. Las imágenes son capturas reales
  del código que se muestra (ver [`assets/README.md`](assets/README.md)).

English: [README.en.md](README.en.md).

## Recorrido

Cada capítulo se apoya en los anteriores.

| # | Capítulo | Responde a |
|---|---|---|
| 1 | [Qué es BarrierKit](01-que-es.md) | ¿Qué es, qué crates tiene, usa GPU o CPU? |
| 2 | [Tu primera app](02-primera-app.md) | ¿Cómo abro una ventana con un botón? |
| 3 | [Capas](03-capas.md) | ¿Cómo está organizado el kit y qué puedo usar? |
| 4 | [Apariencia y tokens](04-apariencia-y-tokens.md) | ¿De dónde salen colores y medidas? ¿Cómo pongo los míos? |
| 5 | [Controles](05-controles.md) | ¿Qué elementos interactivos hay y cómo se ven? |
| 6 | [Ventanas y titlebars](06-ventanas-y-titlebars.md) | ¿Qué tipos de titlebar y semáforos hay? |
| 7 | [Barra lateral y SplitView](07-barra-lateral.md) | ¿Cómo se configura, se redimensiona y se pliega? |
| 8 | [Vistas](08-vistas.md) | Formularios, tablas, rejillas, pestañas, alertas, navegación y búsqueda |
| 9 | [Patrones](09-patrones.md) | Ventana de preferencias y «Acerca de» |
| 10 | [Infraestructura de app](10-app.md) | Acciones, atajos, menú global, instancia única, preferencias, deshacer |
| 11 | [Animaciones](11-animaciones.md) | ¿Qué se anima, con qué curva y cómo se reduce el movimiento? |
| 12 | [Fondos tintados y medios](12-fondos-y-medios.md) | Fondos desde una imagen, acento propio, vídeo |
| 13 | [Plataforma](13-plataforma.md) | Runner, Vulkan, protocolos Wayland, la parte de la shell |
| 14 | [Apps de prueba](14-apps-de-prueba.md) | Tengen App Show, Motion Show, Ondina, Rótulo y Marea |
| 15 | [Preguntas frecuentes](15-preguntas.md) | ¿Qué tenemos y qué falta? |

## Fuentes únicas

Esta documentación enlaza y no copia. Cuando un dato cambia, cambia allí:

- Decisiones: [`docs/decisiones/`](../decisiones/), sobre todo el
  [ADR 0047](../decisiones/0047-barrierkit-en-capas.md) (capas y estado).
- Tokens: `crates/vow-design/tengen/tokens.json` y `components.json`.
- Opciones de apariencia: `crates/vow-appearance/src/lib.rs`
  (`AppearanceConfig`).
