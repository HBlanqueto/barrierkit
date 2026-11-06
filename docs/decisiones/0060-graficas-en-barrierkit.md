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
4. **3D como una malla en la GPU:** superficies, modelos y nubes de puntos
   se proyectan en la CPU, se descartan las caras traseras de las mallas
   cerradas y los triángulos se ordenan de atrás hacia delante. Toda la malla
   va en un `Vertices` de Skia con color por vértice (sombra suave por
   normales promediadas y color por altura) y se rasteriza en la GPU con
   `draw_vertices`, en una sola llamada sobre Vulkan (ADR 0056). La cámara
   es libre al estilo Blender (orbitar, desplazar y zoom), personalizada por
   coordenadas o fija en un plano: frente, arriba, lado o isométrica.
5. **Valores en `vow-design`** (ADR 0011): `components.chart` (`CHART`). Los
   colores de las series apuntan a los colores de acento del sistema
   (`CHART_SERIES`), sin copiarlos.
6. **Prueba en una app aparte, Trazo**, no en Tengen App Show: gráficas 2D,
   un monitor con datos reales de `/proc` y 3D.

## Consecuencias

- Una app dibuja una gráfica con una línea por serie; el monitor del sistema
  futuro reutiliza `Chart`, `SectorChart`, `HeatMap` y `Table`.
- La GPU rasteriza la malla, pero la proyección y el orden por profundidad
  siguen en la CPU: `freya-skia-safe` 0.98 no expone `SkMesh` (su
  `mesh.rs` está vacío), que permitiría proyectar en un shader de vértices.
  Con Suzanne (3936 triángulos) basta de sobra.
- La lectura al pasar el puntero solo existe en `Chart`; los sectores y el
  mapa de calor muestran sus valores en la leyenda.

## Alternativas descartadas

- **Una crate de gráficas externa (plotters):** dibuja en su propio backend y
  no sigue los tokens ni la tipografía del kit.
- **Un nodo de freya por barra o punto:** miles de nodos por historial y
  diffs caros en cada muestra.
- **3D con una superficie Vulkan propia:** demasiado para una gráfica; la
  malla de Skia ya se rasteriza en la GPU.
- **Un trazado por cara:** una llamada por triángulo, sin interpolar color
  entre vértices.
