//! The library: albums with covers painted at start-up, so the backdrop has
//! real images to read colours from.

use freya_core::prelude::Bytes;
use freya_engine::prelude::{
    Color4f, Colors, EncodedImageFormat, Gradient, Interpolation, Paint, Rect, TileMode,
    raster_n32_premul, shaders,
};
use vow_design::Rgba;

pub struct Album {
    pub title: &'static str,
    pub artist: &'static str,
    pub year: u16,
    pub tracks: &'static [&'static str],
    pub cover: Bytes,
}

impl Album {
    pub fn seconds(&self, track: usize) -> u32 {
        184 + (self.title.len() as u32 * 37 + track as u32 * 53) % 110
    }
}

#[derive(Clone, Copy)]
enum Art {
    Bloom,
    Stripes,
    Grid,
    Moon,
}

type Entry = (
    &'static str,
    &'static str,
    u16,
    Art,
    [u32; 4],
    &'static [&'static str],
);

const LIBRARY: [Entry; 8] = [
    (
        "Marea baja",
        "Nadia Ferro",
        2024,
        Art::Stripes,
        [0x1f6f6a, 0xe86f51, 0xf3d8b5, 0x0f2f3a],
        &[
            "Bajamar",
            "Sal en los ojos",
            "Faro apagado",
            "Corriente de resaca",
            "Las cinco",
        ],
    ),
    (
        "Luces de garaje",
        "Los Tranvías",
        2023,
        Art::Bloom,
        [0x2a1a14, 0xf2a33a, 0x6c3b8f, 0xc45a2c],
        &[
            "Energía",
            "Cable pelado",
            "Persiana",
            "Domingo a las tres",
            "Cortocircuito",
        ],
    ),
    (
        "Halcón de papel",
        "Irene Solís",
        2022,
        Art::Grid,
        [0xe9a6c1, 0x2f7d5b, 0xf6efe4, 0xd4485e],
        &["Origami", "Viento sur", "Cometa", "Tinta verde"],
    ),
    (
        "Cuarto creciente",
        "Beto Arce",
        2021,
        Art::Moon,
        [0x14213d, 0xfca311, 0x3a4f7a, 0xe5e5e5],
        &["Hoz", "Marea alta", "Insomnio", "Penumbra", "Amanece"],
    ),
    (
        "Sal y cobre",
        "Dúo Bermejo",
        2024,
        Art::Stripes,
        [0xb8632f, 0x2a9db0, 0xf0d9c2, 0x47241a],
        &["Cobre", "Pátina", "Azul óxido", "Fragua"],
    ),
    (
        "Invernadero",
        "Mila Okafor",
        2020,
        Art::Bloom,
        [0x1d4d2b, 0xc13f8a, 0x8fc96a, 0x0d1f12],
        &["Esqueje", "Humedad", "Raíz aérea", "Clorofila", "Vidrio"],
    ),
    (
        "Ruido blanco",
        "Kaplan",
        2019,
        Art::Grid,
        [0xd9d6d0, 0xc8352e, 0x2b2b2b, 0x8a8680],
        &["Estática", "Señal", "Interferencia"],
    ),
    (
        "Verano 98",
        "Las Hortensias",
        2025,
        Art::Bloom,
        [0xf5c84c, 0x2d58b8, 0xf08aa4, 0x163063],
        &["Piscina", "Chicle", "Siesta", "Bicis", "Último día"],
    ),
];

const SIDE: f32 = 256.0;

pub fn library() -> Vec<Album> {
    LIBRARY
        .iter()
        .enumerate()
        .map(|(seed, (title, artist, year, art, colors, tracks))| Album {
            title,
            artist,
            year: *year,
            tracks,
            cover: paint(*art, colors.map(rgb), seed as u32),
        })
        .collect()
}

fn rgb(hex: u32) -> Color4f {
    let [_, r, g, b] = hex.to_be_bytes();
    let c = Rgba::rgb8(r, g, b);
    Color4f::new(c.r, c.g, c.b, 1.0)
}

struct Dice(u32);

impl Dice {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
    }
}

fn paint(art: Art, [ground, a, b, c]: [Color4f; 4], seed: u32) -> Bytes {
    let mut surface = raster_n32_premul((SIDE as i32, SIDE as i32)).expect("raster surface");
    let canvas = surface.canvas();
    let mut dice = Dice(seed * 977 + 13);
    let mut brush = Paint::new(ground, None);
    brush.set_anti_alias(true);
    canvas.draw_rect(Rect::from_wh(SIDE, SIDE), &brush);
    match art {
        Art::Bloom => {
            for colour in [a, b, c, a, b, c, a] {
                let (x, y) = (dice.next() * SIDE, dice.next() * SIDE);
                let radius = SIDE * (0.18 + dice.next() * 0.3);
                let stops = [colour, Color4f::new(colour.r, colour.g, colour.b, 0.0)];
                let glow = Gradient::new(
                    Colors::new_evenly_spaced(&stops, TileMode::Clamp, None),
                    Interpolation::default(),
                );
                brush.set_shader(shaders::radial_gradient(((x, y), radius), &glow, None));
                canvas.draw_rect(Rect::from_wh(SIDE, SIDE), &brush);
                brush.set_shader(None);
            }
        }
        Art::Stripes => {
            canvas.save();
            canvas.rotate(-28.0, Some((SIDE / 2.0, SIDE / 2.0).into()));
            for i in 0..14 {
                brush.set_color4f([a, b, c, a, ground][i % 5], None);
                canvas.draw_rect(
                    Rect::from_xywh(
                        -SIDE,
                        -SIDE / 2.0 + i as f32 * SIDE * 0.15,
                        SIDE * 3.0,
                        SIDE * 0.09,
                    ),
                    &brush,
                );
            }
            canvas.restore();
        }
        Art::Grid => {
            let cell = SIDE / 6.0;
            for column in 0..6 {
                for row in 0..6 {
                    brush.set_color4f([a, b, c, ground][(dice.next() * 4.0) as usize % 4], None);
                    let (x, y) = (column as f32 * cell, row as f32 * cell);
                    if dice.next() < 0.5 {
                        canvas.draw_rect(Rect::from_xywh(x, y, cell, cell), &brush);
                    } else {
                        canvas.draw_circle((x + cell / 2.0, y + cell / 2.0), cell * 0.42, &brush);
                    }
                }
            }
        }
        Art::Moon => {
            brush.set_color4f(b, None);
            for _ in 0..60 {
                canvas.draw_circle((dice.next() * SIDE, dice.next() * SIDE), 1.2, &brush);
            }
            brush.set_color4f(a, None);
            canvas.draw_circle((SIDE * 0.55, SIDE * 0.45), SIDE * 0.26, &brush);
            brush.set_color4f(ground, None);
            canvas.draw_circle((SIDE * 0.66, SIDE * 0.4), SIDE * 0.24, &brush);
            brush.set_color4f(c, None);
            canvas.draw_rect(Rect::from_xywh(0.0, SIDE * 0.82, SIDE, SIDE * 0.18), &brush);
        }
    }
    let png = surface
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("cover encodes as png");
    Bytes::copy_from_slice(png.as_bytes())
}
