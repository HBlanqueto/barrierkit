#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub azimuth: f64,
    pub inclination: f64,
}

// x right, y up, z toward the viewer: azimuth turns about y, then inclination tips about x.
pub fn project(point: [f64; 3], pose: Pose) -> [f64; 3] {
    let (sa, ca) = pose.azimuth.to_radians().sin_cos();
    let (si, ci) = pose.inclination.to_radians().sin_cos();
    let [x, y, z] = point;
    let (x1, z1) = (x * ca + z * sa, -x * sa + z * ca);
    [x1, y * ci - z1 * si, y * si + z1 * ci]
}

pub fn back_to_front(depths: &[f64]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..depths.len()).collect();
    order.sort_by(|a, b| depths[*a].total_cmp(&depths[*b]));
    order
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Free,
    Front,
    Top,
    Side,
    Isometric,
}

impl View {
    pub const ALL: [View; 5] = [
        View::Free,
        View::Front,
        View::Top,
        View::Side,
        View::Isometric,
    ];

    pub fn pose(self) -> Option<Pose> {
        let (azimuth, inclination) = match self {
            View::Free => return None,
            View::Front => (0.0, 0.0),
            View::Top => (0.0, 90.0),
            View::Side => (90.0, 0.0),
            View::Isometric => (-45.0, (1.0f64 / 3.0f64.sqrt()).asin().to_degrees()),
        };
        Some(Pose {
            azimuth,
            inclination,
        })
    }
}

pub fn fit(vertices: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let mut low = [f64::MAX; 3];
    let mut high = [f64::MIN; 3];
    for v in vertices {
        for axis in 0..3 {
            low[axis] = low[axis].min(v[axis]);
            high[axis] = high[axis].max(v[axis]);
        }
    }
    let half = (0..3)
        .map(|a| (high[a] - low[a]) / 2.0)
        .fold(f64::EPSILON, f64::max);
    vertices
        .iter()
        .map(|v| std::array::from_fn(|a| (v[a] - (low[a] + high[a]) / 2.0) / half))
        .collect()
}

pub fn surface_triangles(grid: &[Vec<f64>]) -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
    let rows = grid.len();
    let cols = grid.iter().map(Vec::len).min().unwrap_or(0);
    if rows < 2 || cols < 2 {
        return (Vec::new(), Vec::new());
    }
    let heights: Vec<f64> = grid
        .iter()
        .flat_map(|row| row[..cols].iter().copied())
        .collect();
    let (min, max) = heights
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), h| (lo.min(*h), hi.max(*h)));
    let span = (max - min).max(f64::EPSILON);
    let vertices = (0..rows)
        .flat_map(|r| {
            (0..cols).map(move |c| {
                [
                    c as f64 / (cols - 1) as f64 * 2.0 - 1.0,
                    (grid[r][c] - min) / span * 2.0 - 1.0,
                    r as f64 / (rows - 1) as f64 * 2.0 - 1.0,
                ]
            })
        })
        .collect();
    let at = |r: usize, c: usize| (r * cols + c) as u32;
    let triangles = (0..rows - 1)
        .flat_map(|r| {
            (0..cols - 1).flat_map(move |c| {
                [
                    [at(r, c), at(r, c + 1), at(r + 1, c + 1)],
                    [at(r, c), at(r + 1, c + 1), at(r + 1, c)],
                ]
            })
        })
        .collect();
    (vertices, triangles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f64; 3], b: [f64; 3]) -> bool {
        a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-9)
    }

    #[test]
    fn a_zero_pose_is_the_front_view() {
        let front = Pose {
            azimuth: 0.0,
            inclination: 0.0,
        };
        assert!(close(project([1.0, 2.0, 3.0], front), [1.0, 2.0, 3.0]));
    }

    #[test]
    fn a_quarter_turn_sends_x_away_from_the_viewer() {
        let side = Pose {
            azimuth: 90.0,
            inclination: 0.0,
        };
        assert!(close(project([1.0, 0.0, 0.0], side), [0.0, 0.0, -1.0]));
    }

    #[test]
    fn faces_are_painted_back_to_front() {
        assert_eq!(back_to_front(&[0.5, -2.0, 1.0]), vec![1, 0, 2]);
    }
    #[test]
    fn the_top_view_looks_down_the_y_axis() {
        let top = View::Top.pose().unwrap();
        assert!(close(project([0.0, 1.0, 0.0], top), [0.0, 0.0, 1.0]));
    }

    #[test]
    fn the_isometric_view_shows_the_three_axes_equally() {
        let iso = View::Isometric.pose().unwrap();
        let lengths: Vec<f64> = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            .iter()
            .map(|axis| {
                let [x, y, _] = project(*axis, iso);
                x.hypot(y)
            })
            .collect();
        assert!(lengths.iter().all(|l| (l - lengths[0]).abs() < 1e-9));
    }

    #[test]
    fn fit_centers_a_mesh_and_keeps_its_proportions() {
        let fitted = fit(&[[0.0, 0.0, 0.0], [4.0, 2.0, 0.0]]);
        assert_eq!(fitted, vec![[-1.0, -0.5, 0.0], [1.0, 0.5, 0.0]]);
    }

    #[test]
    fn a_surface_grid_splits_each_cell_in_two_triangles() {
        let (vertices, triangles) = surface_triangles(&[vec![0.0, 1.0, 2.0], vec![1.0, 2.0, 3.0]]);
        assert_eq!((vertices.len(), triangles.len()), (6, 4));
    }
}
