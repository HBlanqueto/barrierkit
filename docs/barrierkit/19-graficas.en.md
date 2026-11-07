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
   `.rule(value, label)`; the label sits above the line. Without `.y_domain`, the Y axis runs from 0 to the rounded
   maximum. `Chart` options:
   - `.time_axis(seconds)`: each sample is that many seconds from the next
     and the last one is "now". The X axis marks the age (−30 s, −2 min,
     −1 h) in round steps.
   - `.scrollable(true)`: the wheel zooms around the pointer, and dragging
     or horizontal scrolling moves the visible window. The window never
     leaves the data nor drops below `scroll-min-span` samples.
   - `.horizontal(true)` with `MarkKind::Bar`: categories on the left,
     values below; grouped or stacked like vertical bars.

   When the data changes, the chart animates for `DURATION_BASE`. With
   `.time_axis`, the curve slides one step; without it, each value moves
   from the old to the new one, and new points grow from 0. With reduced
   motion, the change is immediate.
4. For a share, use `SectorChart::new(..).slice(name, value)`; the legend
   shows each value and its percentage, and the donut the total. With the
   pointer over a sector, the others drop to `unselected-opacity` and the
   donut shows that sector's value and name (`chartAngleSelection`).
5. To show how a share changes over time, use `StreamChart`. Its "wiggle"
   baseline (Byron and Wattenberg) minimizes the layers' slope, and the
   series that peak first go to the middle. With `.polar(true)` time goes
   around a clock that starts at twelve and leaves one step open where the
   start and the end meet.
6. For 3D, pass a grid of heights to `Chart3D::surface`, vertices and
   triangles to `Chart3D::mesh` (the model is fitted to the box) or
   `[x, y, z]` points to `Chart3D::points`. The mesh is drawn on the GPU in
   a single call (ADR 0060).
7. The 3D camera has three modes, picked in its selector:
   - **Free**, like Blender: dragging with the left or middle button orbits,
     Shift + drag or the right button pans, and the wheel zooms. The drag
     keeps going when the pointer leaves the area.
   - **Custom:** azimuth, inclination and zoom sliders. "Keep this camera"
     copies the camera on screen there.
   - **Fixed planes:** front (XY), top (XZ), side (YZ) and isometric.

   Below it, an accessible readout shows azimuth, inclination and zoom.

As in Swift Charts, the Y axis is on the trailing side and the legend below.
When the pointer moves over a `Chart`, a rule marks the position and the
legend shows each series' value at that point. Without a pointer, it shows
the last value. A click pins the selection (`chartXSelection`): the rule stays
and an annotation next to it shows the category and the values. Another click
on the same point clears it. The annotation uses the tooltip look and flips
side in the right half so it stays inside the chart. A drag that scrolls the
window does not select.

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
| `unselected-opacity` | 0.5 | Unselected sectors ([AppCoda](https://www.appcoda.com/swiftui-chart-ios17/)) |
| `stream-hole` | 0.32 | Center hole of the flow clock |
| `azimuth` / `inclination` | −30° / 25° | Initial `Chart3D` pose |
| `y-axis-width` | 52 | Width of the axis value column |
| `scroll-min-span` | 4 | Minimum visible samples when zooming in |
| `drag-degrees` | 0.5 | Degrees per point when dragging |
| `zoom-min` / `zoom-max` / `wheel-zoom` | 0.4 / 4 / 0.01 | Zoom limits and their rate per wheel point |

Series colors follow `series` (blue, green, orange, purple, red, yellow,
pink), which points to the system accent colors (`CHART_SERIES`).
