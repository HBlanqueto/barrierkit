pub struct Model {
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

fn corner(token: &str, count: usize) -> Option<u32> {
    let index: i64 = token.split('/').next()?.parse().ok()?;
    let resolved = if index < 0 {
        count as i64 + index
    } else {
        index - 1
    };
    u32::try_from(resolved)
        .ok()
        .filter(|i| (*i as usize) < count)
}

pub fn parse_obj(text: &str) -> Model {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        match fields.next() {
            Some("v") => {
                let xyz: Vec<f64> = fields.take(3).filter_map(|f| f.parse().ok()).collect();
                if let [x, y, z] = xyz[..] {
                    vertices.push([x, y, z]);
                }
            }
            Some("f") => {
                let corners: Vec<u32> = fields.filter_map(|f| corner(f, vertices.len())).collect();
                for i in 1..corners.len().saturating_sub(1) {
                    triangles.push([corners[0], corners[i], corners[i + 1]]);
                }
            }
            _ => {}
        }
    }
    Model {
        vertices,
        triangles,
    }
}

pub fn suzanne() -> Model {
    parse_obj(include_str!("../assets/suzanne.obj"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quads_become_two_triangles_and_slashes_keep_the_vertex() {
        let model = parse_obj("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1/1/1 2/2/2 3/3/3 4/4/4\n");
        assert_eq!(model.triangles, vec![[0, 1, 2], [0, 2, 3]]);
    }

    #[test]
    fn negative_indices_count_back_from_the_last_vertex() {
        let model = parse_obj("v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n");
        assert_eq!(model.triangles, vec![[0, 1, 2]]);
    }

    #[test]
    fn the_blender_monkey_loads_whole() {
        let model = suzanne();
        assert_eq!((model.vertices.len(), model.triangles.len()), (2012, 3936));
    }
}
