# 16. Charts

**Goal:** draw data with the kit's charts, from a line on an axis to a 3D
surface.
**Scope:** everything in `bk::charts`, behind the `charts` feature
([1](01-what-is-barrierkit.md#the-crates)). The design is in
[ADR 0060](../decisions/0060-charts-in-barrierkit.md); the series colors and
the `CHART` tokens come from `ctdesign`
([4. Appearance and tokens](04-appearance-and-tokens.md)).
**Reader:** anyone who shows series, shares or measurements, like a system
monitor.
**Input:** numbers in `Vec<f64>`. **Output:** an accessible chart with axes,
a legend and pointer readout.

## Steps

1. Pick the chart for the data:

   | Component | Marks | For |
   |---|---|---|
   | [`Chart`](#chart) | `MarkKind::Line`, `Area`, `Bar`, `Point` | Series on an axis: histories, comparisons |
   | [`SectorChart`](#sectorchart) | Pie, or donut with `.donut(true)` | Shares of a total |
   | [`HeatMap`](#heatmap) | A grid of cells | Intensity by row and column |
   | [`StreamChart`](#streamchart) | Stream, or flow clock with `.polar(true)` | How shares change over time |
   | [`Chart3D`](#chart3d) | Surface, mesh or points | z = f(x, y), 3D models, point clouds |

2. Create it with its accessible label and add the data:

   ```rust
   Chart::new("CPU usage", MarkKind::Area)
       .stacked(true)
       .y_domain(0.0, 100.0)
       .unit("%")
       .series(Series::new("User", user))
       .series(Series::new("System", system))
   ```

3. Add the axis details the data needs: `.time_axis(seconds)` for samples over
   time, or `.x_labels(..)` for categories, and `.rule(value, label)` for a
   limit.
4. Put it in the layout. Charts fill the width of their parent, except
   `SectorChart` and the flow clock, which are squares. The height is
   `CHART.height` unless you set one.
5. Rebuild it with new values when the data changes. The chart animates from
   the old values to the new ones.

Every chart is one Skia element; axes, legend and readout are kit text. The
label is the chart's accessible name (role `Figure`).

## Series

`Series::new(name, values)` is one named list of values. `name` and `values`
are public fields. The n-th series of a chart takes the n-th color of
`CHART_SERIES` (blue, green, orange, purple, red, yellow, pink), and the list
starts again after the last.

## Chart

`Chart::new(label, kind)` draws series on an axis, like Swift Charts' `Chart`.

| Option | Default | What it does |
|---|---|---|
| `.series(series)` | none | Adds a series |
| `.stacked(bool)` | `false` | Each series starts on top of the previous one; unstacked bars stand side by side |
| `.interpolation(i)` | `Linear` | How points join |
| `.y_domain(min, max)` | from the data | Fixes the value axis |
| `.unit(unit)` | `""` | Suffix of every value: `"%"`, `" MB"` |
| `.x_labels(labels)` | none | Category labels; about six are shown |
| `.rule(value, label)` | none | A reference line with its label above it |
| `.height(px)` | `CHART.height` | Height of the data area |
| `.time_axis(seconds)` | off | Each sample is `seconds` after the previous one and the last is "now" |
| `.scrollable(bool)` | `false` | Zoom and pan the visible samples |
| `.horizontal(bool)` | `false` | With `MarkKind::Bar`: categories on the left, values below |

| `MarkKind` | Swift Charts | Draws |
|---|---|---|
| `Line` | `LineMark` | A line per series |
| `Area` | `AreaMark` | A line with a fill at `CHART.area_opacity` |
| `Bar` | `BarMark` | Bars, grouped or stacked, with no corner radius |
| `Point` | `PointMark` | A dot per value |

| `Interpolation` | Points join with… |
|---|---|
| `Linear` (default) | Straight segments |
| `Monotone` | A curve that never overshoots the data |
| `Step` | Steps |

### Axes

- The value axis is on the trailing side and the legend below, as in Swift
  Charts.
- Without `.y_domain`, the axis starts at 0 (or the lowest negative value) and
  ends at the next round tick. Ticks are steps of 1, 2 or 5 × 10ⁿ, about
  `CHART.ticks` of them.
- With `.time_axis`, the category axis marks the age of the samples in round
  steps: `now`, `−30 s`, `−2 min`, `−1 h`.

### Pointer and selection

1. With the pointer over the chart, a rule marks the nearest sample and the
   legend shows each series' value there. Without a pointer it shows the last
   value.
2. A click pins the selection, like `chartXSelection`: the rule stays and an
   annotation shows the category and the values. It uses the tooltip look and
   moves to the left of the rule in the right half.
3. A second click on the same sample clears it. A drag that scrolls does not
   select.

With `.scrollable(true)`, the vertical wheel zooms around the pointer, and
horizontal scrolling or a drag moves the visible window. The window never
leaves the data or drops below `CHART.scroll_min_span` samples. A horizontal
bar chart does not scroll.

### Animation

When the series change, the chart animates with the `animation` of its
`ChartStyle`, `DURATION_BASE` in Tengen. With
`.time_axis` and the same number of samples, the curve slides one step.
Otherwise each value moves from the old to the new one, and new points grow
from 0. With reduced motion the change is immediate
([14. Animations](14-animations.md#reduced-motion)).

## SectorChart

`SectorChart::new(label)` shows shares of a total, like `SectorMark`.

| Option | Default | What it does |
|---|---|---|
| `.slice(name, value)` | none | Adds a sector; negative values count as 0 |
| `.donut(bool)` | `false` | Leaves a hole of `CHART.sector_inner` of the radius and shows the total in it |
| `.unit(unit)` | `""` | Suffix of the values |
| `.size(px)` | `CHART.height` | Side of the square the chart fills |

- Sectors start at twelve and run clockwise, separated by
  `CHART.sector_inset` and rounded by `CHART.sector_radius`.
- The legend, below, shows each value and its percentage.
- With the pointer over a sector, the others drop to
  `CHART.unselected_opacity`, and the donut shows that sector's value and
  name, like `chartAngleSelection`.

## HeatMap

`HeatMap::new(label, rows)` draws `rows: Vec<Vec<f64>>` as a grid of rounded
cells, like a chart of `RectangleMark`s. Each cell mixes from the control
background (lowest value) to the accent (highest). `.height(px)` sets its
height; default `CHART.height`. It has no axis labels or legend.

## StreamChart

`StreamChart::new(label)` stacks series around a moving center line, so the
shape shows how each share changes.

| Option | Default | What it does |
|---|---|---|
| `.series(series)` | none | Adds a layer |
| `.polar(bool)` | `false` | Wraps time around a clock: the flow clock |
| `.unit(unit)` | `""` | Suffix of the legend values |
| `.size(px)` | `CHART.height` | Height; with `.polar(true)`, the side of the square |

- The baseline is the "wiggle" one (Byron and Wattenberg), which minimizes the
  slope of the layers. The series that peak first go to the middle.
- With `.polar(true)`, time starts at twelve and runs clockwise. The last
  sample stops one step short of twelve, so the start and the end do not
  touch. The center leaves a hole of `CHART.stream_hole`.
- With the pointer over it, the legend shows each series' value at that time.

## Chart3D

`Chart3D` draws in a box from −1 to 1 on each axis, like Swift Charts'
`Chart3D`.

| Constructor | Draws |
|---|---|
| `Chart3D::surface(label, grid)` | A surface: `grid[row][column]` is the height; columns run along x, rows along z, and the heights fill the box |
| `Chart3D::mesh(label, vertices, triangles)` | A closed model of `[x, y, z]` vertices and index triangles, fitted to the box with its proportions |
| `Chart3D::points(label, points)` | A point cloud of `[x, y, z]` in box coordinates |

`.height(px)` sets the height; default 1.5 × `CHART.height`. Colors go from the
first series color (low) to the third (high) along y. A mesh is drawn on the
GPU in a single call (ADR 0060).

A segmented control above the chart picks the camera:

1. **Free**, like Blender: dragging with the left or middle button orbits;
   Shift + drag or the right button pans; the wheel zooms between
   `CHART.zoom_min` and `CHART.zoom_max`. The drag continues when the pointer
   leaves the area.
2. **Custom:** sliders for azimuth, inclination and zoom. **Keep this camera**
   copies the camera on screen there.
3. **Fixed planes:** front (XY), top (XZ), side (YZ) and isometric. A drag or
   the wheel switches to Free from the same pose.

The first pose is `CHART.azimuth` and `CHART.inclination`. Below the chart, an
accessible readout shows azimuth, inclination and zoom.

## Look

Every value is in `components.chart` of `ctdesign` (`CHART`):

| Token | Value | Use |
|---|---|---|
| `height` | 180 | Default height of the data area |
| `line-width` | 2 | Line thickness |
| `point-size` | 7 | Point diameter |
| `bar-gap` / `bar-radius` | 0.25 / 0 | Gap between bar groups; no radius, like `BarMark` |
| `area-opacity` | 0.35 | Opacity of area fills |
| `grid-width` / `rule-width` | 0.5 / 1 | Grid and rules |
| `ticks` | 4 | Desired value axis ticks |
| `y-axis-width` / `axis-gap` | 52 / 6 | Value column width; gap to the labels |
| `legend-swatch` / `legend-gap` | 8 / 14 | Legend dot; gap between entries |
| `sector-inner` / `sector-inset` / `sector-radius` | 0.618 / 1.5 / 5 | Donut hole, gap and sector corner, like the WWDC23 sample |
| `unselected-opacity` | 0.5 | Unselected sectors ([AppCoda](https://www.appcoda.com/swiftui-chart-ios17/)) |
| `heat-gap` / `heat-radius` | 2 / 3 | Heat map cell gap and corner |
| `stream-hole` | 0.32 | Center hole of the flow clock |
| `hover-opacity` | 0.12 | Highlight of the hovered row of horizontal bars |
| `azimuth` / `inclination` | −30° / 25° | First `Chart3D` pose |
| `drag-degrees` | 0.5 | Degrees per point when orbiting |
| `zoom-min` / `zoom-max` / `wheel-zoom` | 0.4 / 4 / 0.01 | Zoom limits and rate per wheel point |
| `scroll-min-span` | 4 | Fewest visible samples when zooming in |
| `series` | blue … pink | Series colors, the system accent colors (`CHART_SERIES`) |

### Style

The five charts draw from one `ChartStyle`, built from these tokens: series
colors, grid, labels, legend, readout, line widths, marks and animation.
`.style(|s| ..)` changes one chart and `Theme::tengen().style(|s: ChartStyle| ..)`
every chart of the app. The fields and which chart draws each are in
[Styles and themes](../../agents/styles.md#chartstyle).

## Helpers

| Function | What it does |
|---|---|
| `format_value(value, unit)` | Formats a value as the charts do: no decimals for whole numbers and from 100 up, one from 10, two below; then `unit`. `4.256, "%"` → `4.26%` |
| `lerp_series(from, to, t)` | Mixes two lists of series at `t` 0–1, keeping `to`'s names; values missing in `from` start at 0 |

The scale and 3D math (`charts::scale`, `charts::space`) is private to the
module.

Next: [17. Languages](17-languages.md).
