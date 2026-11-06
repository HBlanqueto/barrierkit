# 0060 — Gráficas en barrierkit

Estado: **aceptado** (2026-11-06). Implementado en `barrier_kit::charts` y en la
app de pruebas Trazo.

## Contexto

Un monitor del sistema y Ajustes necesitan historiales, repartos y mapas de
intensidad. El kit no tenía gráficas, y freya 0.4.3 no trae ninguna. macOS 26
las ofrece con Swift Charts: marcas (`LineMark`, `AreaMark`, `BarMark`,
`PointMark`, `RuleMark`, `SectorMark`, `RectangleMark`) y, desde macOS 26,
`Chart3D` con `SurfacePlot`.

## Decisión

1. **Un módulo, `charts`, con la forma de Swift Charts:** `Chart` (línea,
   área, barra y punto, con apilado, interpolación lineal, monótona o en
   escalón y reglas), `SectorChart` (pastel y donut), `HeatMap`, `StreamChart` (flujo con
   línea base «wiggle», también polar) y `Chart3D` (superficie y nube de
   puntos).
2. **Un elemento Skia por gráfica** (`ElementExt`, como `glass.rs`): las
   marcas se pintan con trazos y rellenos en un solo nodo. Los ejes, la
   leyenda y la lectura son texto del kit, así que siguen la tipografía y la
   accesibilidad.
3. **Lógica pura aparte y con tests:** `scale.rs` (escala lineal, marcas
   «redondas» de Heckbert, apilado, ángulos de sectores y tangentes monótonas
   de Fritsch–Carlson) y `space.rs` (pose, proyección ortográfica y orden de
   pintado de atrás hacia delante).
4. **3D sin GPU propia:** la superficie se proyecta en la CPU y cada cara se
   pinta en orden de profundidad, con sombra por orientación y color por
   altura. El usuario cambia azimut e inclinación arrastrando.
5. **Valores en `vow-design`** (ADR 0011): `components.chart` (`CHART`). Los
   colores de las series apuntan a los colores de acento del sistema
   (`CHART_SERIES`), sin copiarlos.
6. **Prueba en una app aparte, Trazo**, no en Tengen App Show: gráficas 2D,
   un monitor con datos reales de `/proc` y 3D.

## Consecuencias

- Una app dibuja una gráfica con una línea por serie; el monitor del sistema
  futuro reutiliza `Chart`, `SectorChart`, `HeatMap` y `Table`.
- El 3D va bien hasta unos miles de caras por fotograma; más necesitaría
  pintar en la GPU.
- La lectura al pasar el puntero solo existe en `Chart`; los sectores y el
  mapa de calor muestran sus valores en la leyenda.

## Alternativas descartadas

- **Una crate de gráficas externa (plotters):** dibuja en su propio backend y
  no sigue los tokens ni la tipografía del kit.
- **Un nodo de freya por barra o punto:** miles de nodos por historial y
  diffs caros en cada muestra.
- **3D con una superficie Vulkan propia:** demasiado para una gráfica; la
  proyección en la CPU basta para el tamaño de datos de un monitor.
