# 0060 — Charts in barrierkit

Status: **accepted** (2026-11-06). Implemented in `barrier_kit::charts` and in
the Trazo test app.

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
4. **3D without its own GPU path:** the surface is projected on the CPU and
   each face is painted in depth order, shaded by orientation and colored by
   height. The user changes azimuth and inclination by dragging.
5. **Values in `vow-design`** (ADR 0011): `components.chart` (`CHART`). Series
   colors point to the system accent colors (`CHART_SERIES`) without copying
   them.
6. **Tested in a separate app, Trazo**, not in Tengen App Show: 2D charts, a
   monitor with real data from `/proc` and 3D.

## Consequences

- An app draws a chart with one line per series; the future system monitor
  reuses `Chart`, `SectorChart`, `HeatMap` and `Table`.
- 3D works up to a few thousand faces per frame; more would need GPU
  painting.
- The pointer readout only exists in `Chart`; sectors and the heat map show
  their values in the legend.

## Rejected alternatives

- **An external charting crate (plotters):** it draws on its own backend and
  does not follow the kit's tokens or typography.
- **One freya node per bar or point:** thousands of nodes per history and
  expensive diffs on every sample.
- **3D with its own Vulkan surface:** too much for a chart; CPU projection is
  enough for a monitor's data size.
