# Documentación de BarrierKit

BarrierKit es el kit para crear las apps y la shell de Malaysia. Esta
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
| 6 | [Ventanas](06-ventanas.md) | ¿Cómo abro una ventana, la redimensiono y leo su estado? |
| 7 | [Titlebars](07-titlebars.md) | ¿Qué titlebars y semáforos hay y cómo se ven en cada estado? |
| 8 | [Barra lateral y SplitView](08-barra-lateral.md) | ¿Cómo se configura, se redimensiona y se pliega? |
| 9 | [Vistas](09-vistas.md) | Formularios, tablas, rejillas, pestañas, alertas, navegación y búsqueda |
| 10 | [Patrones](10-patrones.md) | Ventana de preferencias y «Acerca de» |
| 11 | [Infraestructura de app](11-app.md) | Acciones, atajos, menú global, instancia única, preferencias, deshacer |
| 12 | [Animaciones](12-animaciones.md) | ¿Qué se anima, con qué curva y cómo se reduce el movimiento? |
| 13 | [Fondos tintados y medios](13-fondos-y-medios.md) | Fondos desde una imagen, acento propio, vídeo |
| 14 | [Plataforma](14-plataforma.md) | Runner, Vulkan, protocolos Wayland, la parte de la shell |
| 16 | [Preguntas frecuentes](16-preguntas.md) | ¿Qué tenemos y qué falta? |
| 17 | [Perfilar con Tracy](17-perfilado.md) | ¿Dónde se va el tiempo de cada frame? |
| 18 | [Idiomas](18-idiomas.md) | ¿Cómo traduzco una app y cómo se elige el idioma? |
| 19 | [Gráficas](19-graficas.md) | ¿Cómo dibujo series, repartos y gráficas 3D? |

## Fuentes únicas

Esta documentación enlaza y no copia. Cuando un dato cambia, cambia allí:

- Decisiones: [`docs/decisiones/`](../decisiones/), sobre todo el
  [ADR 0047](../decisiones/0047-barrierkit-en-capas.md) (capas y estado).
- Tokens: `crates/vow-design/tengen/tokens.json` y `components.json`.
- Opciones de apariencia: `crates/vow-appearance/src/config.rs`
  (`AppearanceConfig`).
