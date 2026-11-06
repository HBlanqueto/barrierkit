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
}
