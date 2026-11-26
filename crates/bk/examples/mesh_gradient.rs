use bk::prelude::*;
use ctdesign::Rgba;

const POINTS: [(f32, f32); 9] = [
    (0.0, 0.0),
    (0.5, 0.0),
    (1.0, 0.0),
    (0.0, 0.5),
    (0.6, 0.4),
    (1.0, 0.5),
    (0.0, 1.0),
    (0.5, 1.0),
    (1.0, 1.0),
];

fn tone(red: u8, green: u8, blue: u8) -> ShapeStyle {
    ShapeStyle::Color(Rgba::rgba8(red, green, blue, 1.0))
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.mesh-gradient")
        .window("Mesh gradient", || {
            let (sun, night, sky) = (tone(255, 149, 0), tone(20, 20, 40), tone(90, 200, 250));
            MeshGradient::new(
                3,
                3,
                POINTS.to_vec(),
                vec![
                    sun.clone(),
                    sky.clone(),
                    night.clone(),
                    night.clone(),
                    sun.clone(),
                    sky.clone(),
                    sky,
                    night,
                    sun,
                ],
            )
        })
        .options(|window| window.size((420, 300)))
        .run()
}
