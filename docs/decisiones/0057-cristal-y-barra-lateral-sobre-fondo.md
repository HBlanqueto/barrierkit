# 0057 — El cristal de pantalla completa cubre toda la ventana; la barra lateral sobre un fondo tintado

Estado: **aceptado** (2026-11-03).

## Contexto

Ondina pasó a usar `SplitView`, `Sidebar` y `Titlebar` del kit (`6689f78`). Con eso aparecieron dos fallos:

- En pantalla completa con la barra superpuesta (ADR 0054), el cristal lo dibujaba la `Titlebar` con el ancho de su columna. La barra lateral y el inspector quedaban fuera: la cola de Ondina se veía nítida bajo los botones mientras el centro se desenfocaba.
- La `Sidebar` pintaba siempre el fondo de barra lateral del sistema. Sobre un `Backdrop` aparecía una isla gris opaca encima del degradado, aunque la `Titlebar` ya se volvía transparente con `backdrop::tinted()`.

## Decisión

### 1. Regla del cristal de pantalla completa

**En pantalla completa con la titlebar superpuesta, el cristal progresivo cubre toda la franja superior de la ventana, de borde a borde, sin importar qué haya debajo:** barra lateral en isla o plana, tintada o no, detalle, inspector o cualquier otra composición de la app. Ninguna columna lo limita ni lo recorta, y se ve igual que en el mockup aceptado (Main e Isla tintada):

- desenfoque fuerte en el borde superior que se desvanece hacia abajo (`FULLSCREEN_GLASS`);
- por encima del cristal solo quedan la titlebar, los semáforos y las capas superpuestas (menús, popups, raíles verticales);
- todo el contenido que pasa por debajo, en cualquier columna, se desenfoca igual.

Implementación: la `Titlebar` coloca el cristal con `Position::new_global()` y `Size::window_percent(100.0)`, en `TITLEBAR_OVERLAY_LAYER` y antes de la barra. La barra conserva su ancho y sus márgenes (`rail_overlap`); el cristal no. Sigue dentro del árbol de la app para leer el tinte del `Backdrop` (`glass_tint`).

### 2. La barra lateral sobre un fondo tintado

Isla o plana sigue siendo preferencia del usuario (`sidebar_style`). La app solo elige una variante para cuando la barra está dentro de un `Backdrop`:

```rust
pub enum SidebarBackdrop { Clear, Tinted }   // Clear por defecto
Sidebar::new().backdrop(SidebarBackdrop::Tinted)
```

| | Sin `Backdrop` | `Clear` | `Tinted` |
|---|---|---|---|
| Plana | fondo del sistema | transparente, línea con `Palette::separator` | igual que `Clear` |
| Isla | fondo, borde y borde interior del sistema | sin relleno ni bordes | `Backdrop::average()` con alfa `BACKDROP.island` y borde `Palette::separator` |

Dentro de un `Backdrop`, las filas toman la tinta y el acento de `Palette::current()` (`sidebar_palette`), no la paleta de barra lateral del sistema.

## Consecuencias

- Una app nueva no tiene que hacer nada para que el cristal cubra toda la ventana; tampoco puede limitarlo a una columna.
- Si una app mete la `Titlebar` dentro de un contenedor con `Overflow::Clip`, ese contenedor recorta el cristal: la `Titlebar` no debe ir dentro de uno.
- Ondina ofrece *Transparente / Tintada* en Apariencia; Master Tengen App Show muestra las dos en «Fondo tintado».

## Referencias

- [`0041`](0041-barra-lateral-en-isla.md): barra lateral en isla.
- [`0047`](0047-barrierkit-en-capas.md): capas y variantes.
- ADR 0054 de malaysia: titlebar superpuesta en foco.
