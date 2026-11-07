# 0060 — Charts in barrierkit

Status: **accepted** (2026-11-06). Implemented in `barrier_kit::charts`. The test apps (Master Tengen App Show, Marea, Ondina and Trazo) and the examples were removed from the repository on 2026-11-07.

## Context

A system monitor and Settings need histories, shares and intensity maps. The
kit had no charts, and freya 0.4.3 ships none. macOS 26 offers them with Swift
Charts: marks (`LineMark`, `AreaMark`, `BarMark`, `PointMark`, `RuleMark`,
`SectorMark`, `RectangleMark`) and, since macOS 26, `Chart3D` with
`SurfacePlot`.

## Decision

1. **One module, `charts`, shaped like Swift Charts:** `Chart` (line, area,
   bar and point, with stacking, linear, monotone or step interpolation and
   rules), `SectorChart` (pie and donut), `HeatMap`, `StreamChart` (stream with a
   "wiggle" baseline, also polar) and `Chart3D` (surface and point cloud).
2. **One Skia element per chart** (`ElementExt`, like `glass.rs`): marks are
   painted with strokes and fills in a single node. Axes, legend and readout
   are kit text, so they follow the typography and accessibility.
3. **Pure logic apart and tested:** `scale.rs` (linear scale, Heckbert "nice"
   ticks, stacking, sector angles and Fritsch–Carlson monotone tangents) and
   `space.rs` (pose, orthographic projection and back-to-front paint order).
4. **3D as one GPU mesh:** surfaces, models and point clouds are projected
   on the CPU, closed meshes drop their back faces and triangles are sorted
   back to front. The whole mesh goes in one Skia `Vertices` with per-vertex
   color (smooth shading from averaged normals and color by height) and is
   rasterized on the GPU with `draw_vertices`, in a single call on Vulkan
   (ADR 0056). The camera is free in the Blender style (orbit, pan and
   zoom), custom by coordinates or fixed on a plane: front, top, side or
   isometric.
5. **Values in `ct-design`** (ADR 0011): `components.chart` (`CHART`). Series
   colors point to the system accent colors (`CHART_SERIES`) without copying
   them.
6. **Tested in a separate app, Trazo**, not in Tengen App Show: 2D charts, a
   monitor with real data from `/proc` and 3D.

## Consequences

- An app draws a chart with one line per series; the future system monitor
  reuses `Chart`, `SectorChart`, `HeatMap` and `Table`.
- The GPU rasterizes the mesh, but projection and depth sorting stay on the
  CPU: `freya-skia-safe` 0.98 does not expose `SkMesh` (its `mesh.rs` is
  empty), which would allow projecting in a vertex shader. Suzanne (3936
  triangles) runs well within that.
- The pointer readout only exists in `Chart`; sectors and the heat map show
  their values in the legend.

## Rejected alternatives

- **An external charting crate (plotters):** it draws on its own backend and
  does not follow the kit's tokens or typography.
- **One freya node per bar or point:** thousands of nodes per history and
  expensive diffs on every sample.
- **3D with its own Vulkan surface:** too much for a chart; the Skia mesh is
  already rasterized on the GPU.
- **One path per face:** one call per triangle, with no color interpolation
  between vertices.
