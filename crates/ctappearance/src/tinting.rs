use ctdesign::{MENUBAR_TINT_SPREAD, Rgba, WALLPAPER_TINTING_GRID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct WallpaperGrid {
    pub aspect: f32,
    pub rows: Vec<Vec<[f32; 3]>>,
}

impl WallpaperGrid {
    pub fn from_rgba8(rgba8: &[u8], width: usize, height: usize) -> Option<Self> {
        if width == 0 || height == 0 || rgba8.len() < width * height * 4 {
            return None;
        }
        let cells = WALLPAPER_TINTING_GRID as usize;
        let span = |index: usize, length: usize| {
            let start = index * length / cells;
            start..((index + 1) * length / cells).max(start + 1).min(length)
        };
        let rows = (0..cells)
            .map(|row| {
                (0..cells)
                    .map(|column| {
                        let mut sum = [0.0f32; 3];
                        let mut count = 0.0f32;
                        for y in span(row, height) {
                            for x in span(column, width) {
                                let pixel = &rgba8[(y * width + x) * 4..][..3];
                                for (total, channel) in sum.iter_mut().zip(pixel) {
                                    *total += f32::from(*channel) / 255.0;
                                }
                                count += 1.0;
                            }
                        }
                        sum.map(|total| total / count)
                    })
                    .collect()
            })
            .collect();
        Some(Self {
            aspect: width as f32 / height as f32,
            rows,
        })
    }

    pub fn behind(&self, output: (f64, f64), window: (f64, f64, f64, f64)) -> Option<Rgba> {
        let (output_width, output_height) = output;
        let (x, y, width, height) = window;
        let (left, top) = (x.max(0.0), y.max(0.0));
        let (right, bottom) = (
            (x + width).min(output_width),
            (y + height).min(output_height),
        );
        let columns = self.rows.first()?.len();
        if right <= left || bottom <= top || columns == 0 {
            return None;
        }
        let (shown_width, shown_height) = self.cover(output_width, output_height);
        let to_cells = |position: f64, offset: f64, shown: f64, cells: usize| {
            (position - offset) / shown * cells as f64
        };
        let (offset_x, offset_y) = (
            (output_width - shown_width) / 2.0,
            (output_height - shown_height) / 2.0,
        );
        let horizontal = (
            to_cells(left, offset_x, shown_width, columns),
            to_cells(right, offset_x, shown_width, columns),
        );
        let vertical = (
            to_cells(top, offset_y, shown_height, self.rows.len()),
            to_cells(bottom, offset_y, shown_height, self.rows.len()),
        );
        let mut sum = [0.0f64; 3];
        let mut weight = 0.0;
        for (row, colors) in self.rows.iter().enumerate() {
            let row_weight = overlap(vertical, row);
            for (column, color) in colors.iter().enumerate() {
                let cell_weight = row_weight * overlap(horizontal, column);
                for (total, channel) in sum.iter_mut().zip(color) {
                    *total += f64::from(*channel) * cell_weight;
                }
                weight += cell_weight;
            }
        }
        (weight > 0.0).then(|| Rgba {
            r: (sum[0] / weight) as f32,
            g: (sum[1] / weight) as f32,
            b: (sum[2] / weight) as f32,
            a: 1.0,
        })
    }

    pub fn across(&self, output: (f64, f64), window: (f64, f64, f64, f64)) -> Option<Vec<Rgba>> {
        let (x, y, width, height) = window;
        let columns = self.rows.first()?.len();
        let slice = width / columns as f64;
        let colors = (0..columns)
            .map(|column| self.behind(output, (x + column as f64 * slice, y, slice, height)))
            .collect::<Option<Vec<_>>>()?;
        let spread = f64::from(MENUBAR_TINT_SPREAD) * columns as f64;
        let reach = (spread * 3.0).ceil() as isize;
        Some(
            (0..columns as isize)
                .map(|column| {
                    let mut sum = [0.0f64; 3];
                    let mut weight = 0.0;
                    for offset in -reach..=reach {
                        let kernel = (-0.5 * (offset as f64 / spread).powi(2)).exp();
                        let neighbour =
                            &colors[(column + offset).clamp(0, columns as isize - 1) as usize];
                        for (total, channel) in
                            sum.iter_mut().zip([neighbour.r, neighbour.g, neighbour.b])
                        {
                            *total += f64::from(channel) * kernel;
                        }
                        weight += kernel;
                    }
                    Rgba {
                        r: (sum[0] / weight) as f32,
                        g: (sum[1] / weight) as f32,
                        b: (sum[2] / weight) as f32,
                        a: 1.0,
                    }
                })
                .collect(),
        )
    }

    fn cover(&self, output_width: f64, output_height: f64) -> (f64, f64) {
        let aspect = f64::from(self.aspect);
        if output_width / output_height > aspect {
            (output_width, output_width / aspect)
        } else {
            (output_height * aspect, output_height)
        }
    }
}

fn overlap((start, end): (f64, f64), cell: usize) -> f64 {
    let cell = cell as f64;
    (end.min(cell + 1.0) - start.max(cell)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn halves(width: usize, height: usize) -> Vec<u8> {
        (0..width * height)
            .flat_map(|index| {
                if index % width < width / 2 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 255]
                }
            })
            .collect()
    }

    #[test]
    fn a_window_takes_the_colour_of_the_wallpaper_half_behind_it() {
        let grid = WallpaperGrid::from_rgba8(&halves(64, 32), 64, 32).unwrap();
        let left = grid
            .behind((1280.0, 640.0), (100.0, 100.0, 300.0, 300.0))
            .unwrap();
        let right = grid
            .behind((1280.0, 640.0), (800.0, 100.0, 300.0, 300.0))
            .unwrap();
        assert!((left.r - 1.0).abs() < 1e-6 && left.b.abs() < 1e-6);
        assert!((right.b - 1.0).abs() < 1e-6 && right.r.abs() < 1e-6);
    }

    #[test]
    fn a_window_across_the_middle_averages_both_halves_by_area() {
        let grid = WallpaperGrid::from_rgba8(&halves(64, 32), 64, 32).unwrap();
        let mixed = grid
            .behind((1280.0, 640.0), (440.0, 0.0, 400.0, 640.0))
            .unwrap();
        assert!((mixed.r - 0.5).abs() < 1e-3 && (mixed.b - 0.5).abs() < 1e-3);
    }

    #[test]
    fn a_wider_output_crops_the_top_and_bottom_of_the_wallpaper() {
        let rgba8: Vec<u8> = (0..64 * 64)
            .flat_map(|index| {
                if index < 64 * 16 {
                    [0, 255, 0, 255]
                } else {
                    [9, 9, 9, 255]
                }
            })
            .collect();
        let grid = WallpaperGrid::from_rgba8(&rgba8, 64, 64).unwrap();
        let top = grid
            .behind((1600.0, 800.0), (0.0, 0.0, 1600.0, 40.0))
            .unwrap();
        assert!(top.g < 0.1);
    }

    #[test]
    fn a_window_outside_the_output_has_no_colour() {
        let grid = WallpaperGrid::from_rgba8(&halves(64, 32), 64, 32).unwrap();
        assert_eq!(
            grid.behind((1280.0, 640.0), (1300.0, 0.0, 200.0, 200.0)),
            None
        );
    }

    #[test]
    fn the_bar_blends_smoothly_across_a_wallpaper_edge() {
        let grid = WallpaperGrid::from_rgba8(&halves(64, 32), 64, 32).unwrap();
        let bar = grid
            .across((1280.0, 640.0), (0.0, 0.0, 1280.0, 30.0))
            .unwrap();
        let (first, middle, last) = (bar[0].r, bar[bar.len() / 2].r, bar[bar.len() - 1].r);
        assert!(first > middle && middle > last);
        assert!(middle > 0.2 && middle < 0.8);
    }
}
