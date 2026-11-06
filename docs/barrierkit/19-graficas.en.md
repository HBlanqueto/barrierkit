# 19. Charts

**Goal:** draw data with the kit's charts, from 2D to 3D.
**Reader:** anyone who shows series, shares or measurements, like a system
monitor.

## What there is

The `barrier_kit::charts` module follows Swift Charts: a chart takes series
and draws them with a mark. Everything is painted with Skia in one element;
axes, legend and readout are kit text.

| Component | Marks | For |
|---|---|---|
| `Chart` | `MarkKind::Line`, `Area`, `Bar`, `Point` | Series on an axis: histories, comparisons |
| `SectorChart` | pie or `.donut(true)` | Shares of a total |
| `HeatMap` | grid of cells | Intensity by row and column |
| `StreamChart` | stream or `.polar(true)` (flow clock) | How a share changes over time |
| `Chart3D` | `surface`, `mesh` or `points` | Surfaces z = f(x, y), 3D models and point clouds |

## Steps

1. Create the chart with its accessibility label and its mark:

   ```rust
   Chart::new("CPU usage", MarkKind::Area)
       .stacked(true)
       .y_domain(0.0, 100.0)
       .unit("%")
       .series(Series::new("User", user))
       .series(Series::new("System", system))
   ```

2. Choose how points join with `.interpolation(..)`: `Linear` (default),
   `Monotone` (a curve that never overshoots the data) or `Step`.
3. Add X axis labels with `.x_labels(..)` and reference lines with
   `.rule(value)`. Without `.y_domain`, the Y axis runs from 0 to the rounded
   maximum.
4. For a share, use `SectorChart::new(..).slice(name, value)`; the legend
   shows each value and its percentage, and the donut the total.
5. To show how a share changes over time, use `StreamChart`. Its "wiggle"
   baseline (Byron and Wattenberg) minimizes the layers' slope, and the
   series that peak first go to the middle. With `.polar(true)` time goes
   around a clock that starts at twelve and leaves one step open where the
   start and the end meet.
6. For 3D, pass a grid of heights to `Chart3D::surface`, vertices and
   triangles to `Chart3D::mesh` (the model is fitted to the box) or
   `[x, y, z]` points to `Chart3D::points`. The user rotates it by dragging
   or picks a plane: free, front, top, side or isometric. The mesh is drawn
   on the GPU in a single call (ADR 0060).

As in Swift Charts, the Y axis is on the trailing side and the legend below.
When the pointer moves over a `Chart`, a rule marks the position and the
legend shows each series' value at that point. Without a pointer, it shows
the last value.

## Values

They all live in `components.chart` in `vow-design` (`CHART`):

| Token | Value | Use |
|---|---|---|
| `height` | 180 | Default height of the data area |
| `line-width` | 2 | Line thickness |
| `point-size` | 7 | Point diameter |
| `bar-gap` / `bar-radius` | 0.25 / 0 | Gap between bar groups; no radius, like `BarMark` |
| `area-opacity` | 0.35 | Opacity of area fills |
| `grid-width` / `rule-width` | 0.5 / 1 | Grid and rules |
| `ticks` | 4 | Desired Y axis ticks (steps of 1, 2 or 5 × 10ⁿ) |
| `sector-inner` / `sector-inset` / `sector-radius` | 0.618 / 1.5 / 5 | Donut hole, gap and sector corner, like the WWDC23 sample |
| `stream-hole` | 0.32 | Center hole of the flow clock |
| `azimuth` / `inclination` | −30° / 25° | Initial `Chart3D` pose |
| `drag-degrees` | 0.5 | Degrees per point when dragging |

Series colors follow `series` (blue, green, orange, purple, red, yellow,
pink), which points to the system accent colors (`CHART_SERIES`).

## Try it

`./dev.sh app trazo` opens Trazo: a tab with every 2D mark, one with a system
monitor that reads `/proc` every second and one with the 3D charts (chapter
[15](15-apps-de-prueba.en.md#trazo-trazo)).
