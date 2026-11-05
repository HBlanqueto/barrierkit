use std::sync::LazyLock;

use vow_design::Rgba;

use crate::WHITE;
use crate::localize::tr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Genre {
    Comedy,
    Drama,
    Thriller,
    Horror,
    Fantasy,
    Animation,
    Documentary,
}

pub struct Logo {
    pub family: &'static str,
    pub weight: u16,
    pub italic: bool,
    pub size: f32,
    pub ink: Rgba,
}

pub struct Title {
    pub id: &'static str,
    pub name: &'static str,
    pub logo: Logo,
    pub art: (f32, Rgba, Rgba),
    pub genre: Genre,
    pub kind: &'static str,
    pub meta: [String; 4],
    pub lede: &'static str,
    pub series: bool,
}

const fn logo(family: &'static str, weight: u16, size: f32, ink: Rgba) -> Logo {
    Logo {
        family,
        weight,
        italic: false,
        size,
        ink,
    }
}

pub static CATALOG: LazyLock<[Title; 21]> = LazyLock::new(|| {
    [
        Title {
            id: "orilla",
            name: "La Última Orilla",
            logo: logo("Abril Fatface", 400, 54.0, WHITE),
            art: (
                160.0,
                Rgba::rgb8(0x5a, 0xa6, 0xd6),
                Rgba::rgb8(0x0e, 0x2c, 0x4a),
            ),
            genre: Genre::Drama,
            kind: tr!("kind-original-series"),
            meta: [
                tr!("genre-drama").into(),
                "2026".into(),
                "+13".into(),
                tr!("episodes", count = 8),
            ],
            lede: tr!("lede-orilla"),
            series: true,
        },
        Title {
            id: "neon",
            name: "Neón Sur",
            logo: logo("Monoton", 400, 58.0, Rgba::rgb8(0xff, 0x5a, 0xc8)),
            art: (
                140.0,
                Rgba::rgb8(0x2b, 0x0a, 0x3d),
                Rgba::rgb8(0x09, 0x02, 0x0f),
            ),
            genre: Genre::Thriller,
            kind: tr!("kind-film-premiere"),
            meta: [
                tr!("genre-thriller").into(),
                "2026".into(),
                "+16".into(),
                "1 h 52 min".into(),
            ],
            lede: tr!("lede-neon"),
            series: false,
        },
        Title {
            id: "pan",
            name: "Rompiendo Pan",
            logo: logo("Black Ops One", 400, 52.0, Rgba::rgb8(0xd9, 0xf9, 0x9d)),
            art: (
                170.0,
                Rgba::rgb8(0xe0, 0xb3, 0x54),
                Rgba::rgb8(0x4b, 0x3a, 0x12),
            ),
            genre: Genre::Drama,
            kind: tr!("kind-parody-series"),
            meta: [
                tr!("genre-drama").into(),
                "2026".into(),
                "+16".into(),
                tr!("seasons", count = 5),
            ],
            lede: tr!("lede-pan"),
            series: true,
        },
        Title {
            id: "raras",
            name: "Cosas Extrañísimas",
            logo: logo("Monoton", 400, 40.0, Rgba::rgb8(0xff, 0x2a, 0x2a)),
            art: (
                170.0,
                Rgba::rgb8(0x2a, 0x05, 0x05),
                Rgba::rgb8(0x05, 0x01, 0x01),
            ),
            genre: Genre::Horror,
            kind: tr!("kind-parody-series"),
            meta: [
                tr!("genre-scifi").into(),
                "2025".into(),
                "+13".into(),
                tr!("seasons", count = 4),
            ],
            lede: tr!("lede-raras"),
            series: true,
        },
        Title {
            id: "carton",
            name: "La Casa de Cartón",
            logo: logo("Bebas Neue", 400, 60.0, WHITE),
            art: (
                150.0,
                Rgba::rgb8(0xb3, 0x12, 0x1b),
                Rgba::rgb8(0x3a, 0x03, 0x06),
            ),
            genre: Genre::Thriller,
            kind: tr!("kind-parody-series"),
            meta: [
                tr!("genre-thriller").into(),
                "2024".into(),
                "+16".into(),
                tr!("parts", count = 3),
            ],
            lede: tr!("lede-carton"),
            series: true,
        },
        Title {
            id: "tronos",
            name: "Juego de Tronquitos",
            logo: logo("Cinzel", 900, 40.0, Rgba::rgb8(0xe8, 0xd9, 0xa8)),
            art: (
                170.0,
                Rgba::rgb8(0x3a, 0x4a, 0x5e),
                Rgba::rgb8(0x0b, 0x0f, 0x16),
            ),
            genre: Genre::Fantasy,
            kind: tr!("kind-parody-series"),
            meta: [
                tr!("genre-fantasy").into(),
                "2026".into(),
                "+16".into(),
                tr!("seasons", count = 8),
            ],
            lede: tr!("lede-tronos"),
            series: true,
        },
        Title {
            id: "oso",
            name: "El Oso Panadero",
            logo: logo("Rubik Mono One", 400, 34.0, Rgba::rgb8(0xff, 0xd2, 0x7a)),
            art: (
                160.0,
                Rgba::rgb8(0x3a, 0x26, 0x15),
                Rgba::rgb8(0x0f, 0x0a, 0x06),
            ),
            genre: Genre::Comedy,
            kind: tr!("kind-parody-series"),
            meta: [
                tr!("genre-comedy").into(),
                "2025".into(),
                "+13".into(),
                tr!("seasons", count = 3),
            ],
            lede: tr!("lede-oso"),
            series: true,
        },
        Title {
            id: "separ",
            name: "Separancia",
            logo: logo("Geist", 300, 46.0, Rgba::rgb8(0x0d, 0x4f, 0x4a)),
            art: (
                180.0,
                Rgba::rgb8(0xe9, 0xf3, 0xf1),
                Rgba::rgb8(0x9c, 0xcf, 0xc8),
            ),
            genre: Genre::Thriller,
            kind: tr!("kind-parody-series"),
            meta: [
                tr!("genre-mystery").into(),
                "2026".into(),
                "+13".into(),
                tr!("seasons", count = 2),
            ],
            lede: tr!("lede-separ"),
            series: true,
        },
        Title {
            id: "oficina",
            name: "La Oficinita",
            logo: logo("Special Elite", 400, 42.0, Rgba::rgb8(0x1f, 0x2a, 0x36)),
            art: (
                170.0,
                Rgba::rgb8(0xf2, 0xef, 0xe6),
                Rgba::rgb8(0xc8, 0xc2, 0xb2),
            ),
            genre: Genre::Comedy,
            kind: tr!("kind-parody-series"),
            meta: [
                tr!("genre-comedy").into(),
                "2021".into(),
                tr!("rating-all").into(),
                tr!("seasons", count = 9),
            ],
            lede: tr!("lede-oficina"),
            series: true,
        },
        Title {
            id: "lazo",
            name: "Ted Lazo",
            logo: logo("Righteous", 400, 52.0, Rgba::rgb8(0xff, 0xf7, 0xc2)),
            art: (
                170.0,
                Rgba::rgb8(0x1f, 0x6b, 0x2b),
                Rgba::rgb8(0x0a, 0x24, 0x10),
            ),
            genre: Genre::Comedy,
            kind: tr!("kind-parody-series"),
            meta: [
                tr!("genre-comedy").into(),
                "2024".into(),
                tr!("rating-all").into(),
                tr!("seasons", count = 3),
            ],
            lede: tr!("lede-lazo"),
            series: true,
        },
        Title {
            id: "espejo",
            name: "Espejo Negro Mate",
            logo: logo("Geist", 300, 40.0, Rgba::rgb8(0xe8, 0xe8, 0xe8)),
            art: (
                170.0,
                Rgba::rgb8(0x1b, 0x1b, 0x1b),
                Rgba::rgb8(0x00, 0x00, 0x00),
            ),
            genre: Genre::Horror,
            kind: tr!("kind-parody-anthology"),
            meta: [
                tr!("genre-scifi").into(),
                "2026".into(),
                "+16".into(),
                tr!("seasons", count = 7),
            ],
            lede: tr!("lede-espejo"),
            series: true,
        },
        Title {
            id: "hipotecas",
            name: "Sucesión de Hipotecas",
            logo: Logo {
                family: "Playfair Display",
                weight: 900,
                italic: true,
                size: 38.0,
                ink: Rgba::rgb8(0xf3, 0xe6, 0xc8),
            },
            art: (
                170.0,
                Rgba::rgb8(0x2a, 0x24, 0x18),
                Rgba::rgb8(0x0d, 0x0b, 0x07),
            ),
            genre: Genre::Drama,
            kind: tr!("kind-parody-series"),
            meta: [
                tr!("genre-drama").into(),
                "2023".into(),
                "+16".into(),
                tr!("seasons", count = 4),
            ],
            lede: tr!("lede-hipotecas"),
            series: true,
        },
        Title {
            id: "pixel",
            name: "Píxel Quest",
            logo: logo("Press Start 2P", 400, 26.0, WHITE),
            art: (
                180.0,
                Rgba::rgb8(0x5c, 0x94, 0xfc),
                Rgba::rgb8(0x28, 0x50, 0xb8),
            ),
            genre: Genre::Animation,
            kind: tr!("kind-original-animation"),
            meta: [
                tr!("genre-animation").into(),
                "2026".into(),
                tr!("rating-all").into(),
                tr!("episodes", count = 12),
            ],
            lede: tr!("lede-pixel"),
            series: true,
        },
        Title {
            id: "lobos",
            name: "Noche de Lobos",
            logo: logo("Creepster", 400, 50.0, Rgba::rgb8(0xc9, 0xff, 0x6b)),
            art: (
                170.0,
                Rgba::rgb8(0x1d, 0x14, 0x33),
                Rgba::rgb8(0x05, 0x03, 0x09),
            ),
            genre: Genre::Horror,
            kind: tr!("kind-film"),
            meta: [
                tr!("genre-horror").into(),
                "2025".into(),
                "+18".into(),
                "1 h 38 min".into(),
            ],
            lede: tr!("lede-lobos"),
            series: false,
        },
        Title {
            id: "cometas",
            name: "Huerto de Cometas",
            logo: logo("Pacifico", 400, 44.0, WHITE),
            art: (
                160.0,
                Rgba::rgb8(0x7a, 0xd3, 0xa0),
                Rgba::rgb8(0x1d, 0x6b, 0x56),
            ),
            genre: Genre::Comedy,
            kind: tr!("kind-original-comedy"),
            meta: [
                tr!("genre-comedy").into(),
                "2025".into(),
                tr!("rating-all").into(),
                tr!("episodes", count = 10),
            ],
            lede: tr!("lede-cometas"),
            series: true,
        },
        Title {
            id: "altamar",
            name: "ALTAMAR",
            logo: logo("Bungee", 400, 50.0, Rgba::rgb8(0xe6, 0xfb, 0xff)),
            art: (
                170.0,
                Rgba::rgb8(0x0f, 0x6f, 0x8c),
                Rgba::rgb8(0x06, 0x22, 0x2e),
            ),
            genre: Genre::Documentary,
            kind: tr!("genre-documentary"),
            meta: [
                tr!("genre-nature").into(),
                "2026".into(),
                tr!("rating-all").into(),
                tr!("episodes", count = 6),
            ],
            lede: tr!("lede-altamar"),
            series: true,
        },
        Title {
            id: "jujutsu",
            name: "Jujutsu Kansado",
            logo: logo("Bangers", 400, 60.0, Rgba::rgb8(0xc9, 0xb6, 0xff)),
            art: (
                160.0,
                Rgba::rgb8(0x3a, 0x1d, 0x6e),
                Rgba::rgb8(0x07, 0x03, 0x12),
            ),
            genre: Genre::Animation,
            kind: tr!("kind-parody-anime"),
            meta: [
                tr!("genre-animation").into(),
                "2026".into(),
                "+16".into(),
                tr!("seasons", count = 3),
            ],
            lede: tr!("lede-jujutsu"),
            series: true,
        },
        Title {
            id: "komi",
            name: "Komi-san no puede comentar",
            logo: logo("Pacifico", 400, 36.0, Rgba::rgb8(0xff, 0xff, 0xff)),
            art: (
                170.0,
                Rgba::rgb8(0xf6, 0xc6, 0xd6),
                Rgba::rgb8(0xd9, 0x7a, 0x95),
            ),
            genre: Genre::Animation,
            kind: tr!("kind-parody-anime"),
            meta: [
                tr!("genre-animation").into(),
                "2025".into(),
                tr!("rating-all").into(),
                tr!("seasons", count = 2),
            ],
            lede: tr!("lede-komi"),
            series: true,
        },
        Title {
            id: "naruton",
            name: "Narutón",
            logo: logo("Bangers", 400, 68.0, Rgba::rgb8(0xff, 0xb0, 0x3b)),
            art: (
                170.0,
                Rgba::rgb8(0xf2, 0x7a, 0x1a),
                Rgba::rgb8(0x3b, 0x24, 0x14),
            ),
            genre: Genre::Animation,
            kind: tr!("kind-parody-anime"),
            meta: [
                tr!("genre-animation").into(),
                "2024".into(),
                "+13".into(),
                tr!("episodes", count = 500),
            ],
            lede: tr!("lede-naruton"),
            series: true,
        },
        Title {
            id: "titos",
            name: "Ataque a los Titos",
            logo: logo("Cinzel", 900, 40.0, Rgba::rgb8(0xf1, 0xe4, 0xcc)),
            art: (
                170.0,
                Rgba::rgb8(0x9a, 0x8a, 0x7a),
                Rgba::rgb8(0x3a, 0x33, 0x2c),
            ),
            genre: Genre::Animation,
            kind: tr!("kind-parody-anime"),
            meta: [
                tr!("genre-animation").into(),
                "2023".into(),
                "+16".into(),
                tr!("seasons", count = 4),
            ],
            lede: tr!("lede-titos"),
            series: true,
        },
        Title {
            id: "sprite-fright",
            name: "Sprite Fright",
            logo: logo("Creepster", 400, 52.0, Rgba::rgb8(0xff, 0x4d, 0x5a)),
            art: (
                160.0,
                Rgba::rgb8(0x2e, 0x8b, 0x57),
                Rgba::rgb8(0x0b, 0x14, 0x24),
            ),
            genre: Genre::Animation,
            kind: tr!("kind-short-blender"),
            meta: [
                tr!("genre-animation").into(),
                "2021".into(),
                "+12".into(),
                tr!("short-film").into(),
            ],
            lede: tr!("lede-sprite-fright"),
            series: false,
        },
    ]
});

pub fn title(id: &str) -> &'static Title {
    CATALOG
        .iter()
        .find(|title| title.id == id)
        .unwrap_or(&CATALOG[0])
}

pub const FEATURED: [&str; 6] = ["sprite-fright", "orilla", "pan", "neon", "raras", "tronos"];

#[derive(PartialEq)]
pub struct Gallery {
    pub id: &'static str,
    pub franchise: &'static str,
    pub logo: &'static str,
    pub poster: &'static str,
    pub stills: [&'static str; 3],
}

const GALLERIES: [Gallery; 1] = [Gallery {
    id: "sprite-fright",
    franchise: "Blender Open Movie",
    logo: "sprite-fright-logo.png",
    poster: "sprite-fright-poster.jpg",
    stills: [
        "sprite-fright-1.jpg",
        "sprite-fright-2.jpg",
        "sprite-fright-3.jpg",
    ],
}];

pub fn gallery(id: &str) -> Option<&'static Gallery> {
    GALLERIES.iter().find(|gallery| gallery.id == id)
}

type Label = fn() -> &'static str;

pub const GENRES: [(Option<Genre>, Label); 8] = [
    (None, || tr!("genre-all")),
    (Some(Genre::Comedy), || tr!("genre-comedy")),
    (Some(Genre::Drama), || tr!("genre-drama")),
    (Some(Genre::Thriller), || tr!("genre-thriller")),
    (Some(Genre::Horror), || tr!("genre-horror")),
    (Some(Genre::Fantasy), || tr!("genre-fantasy")),
    (Some(Genre::Animation), || tr!("genre-animation")),
    (Some(Genre::Documentary), || tr!("genre-documentary")),
];

#[derive(PartialEq)]
pub struct Entry {
    pub id: &'static str,
    pub caption: Option<String>,
    pub watched: Option<f32>,
    pub new: bool,
}

fn entry(id: &'static str, caption: Option<String>, watched: Option<f32>) -> Entry {
    Entry {
        id,
        caption,
        watched,
        new: false,
    }
}

fn new_entry(id: &'static str, caption: String) -> Entry {
    Entry {
        new: true,
        ..entry(id, Some(caption), None)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShelfStyle {
    Wide,
    Ranked,
}

pub struct Shelf {
    pub name: &'static str,
    pub style: ShelfStyle,
    pub entries: Vec<Entry>,
}

pub static SHELVES: LazyLock<[Shelf; 6]> = LazyLock::new(|| {
    [
        Shelf {
            name: tr!("continue-watching"),
            style: ShelfStyle::Wide,
            entries: vec![
                entry("neon", Some(tr!("minutes-left", minutes = 34)), Some(0.62)),
                entry(
                    "pan",
                    Some(tr!("season-episode-short", season = 3, episode = 7)),
                    Some(0.4),
                ),
                entry(
                    "altamar",
                    Some(tr!("season-episode-short", season = 1, episode = 3)),
                    Some(0.35),
                ),
                entry(
                    "oso",
                    Some(tr!("season-episode-short", season = 2, episode = 1)),
                    Some(0.1),
                ),
                entry("separ", Some(tr!("minutes-left", minutes = 12)), Some(0.85)),
            ],
        },
        Shelf {
            name: tr!("shelf-top-ten"),
            style: ShelfStyle::Ranked,
            entries: vec![
                entry("raras", None, None),
                entry("pan", None, None),
                entry("carton", None, None),
                entry("tronos", None, None),
                entry("lazo", None, None),
                entry("oficina", None, None),
                entry("espejo", None, None),
                entry("hipotecas", None, None),
                entry("lobos", None, None),
                entry("cometas", None, None),
            ],
        },
        Shelf {
            name: tr!("shelf-parodies"),
            style: ShelfStyle::Wide,
            entries: vec![
                entry("separ", Some(tr!("genre-mystery").into()), None),
                entry("oficina", Some(tr!("genre-comedy").into()), None),
                entry("hipotecas", Some(tr!("genre-drama").into()), None),
                entry("carton", Some(tr!("genre-thriller").into()), None),
                entry("oso", Some(tr!("genre-comedy").into()), None),
                entry("espejo", Some(tr!("anthology").into()), None),
            ],
        },
        Shelf {
            name: tr!("shelf-new-this-week"),
            style: ShelfStyle::Wide,
            entries: vec![
                new_entry("lobos", tr!("labelled-new", label = tr!("genre-horror"))),
                new_entry("pixel", tr!("labelled-new", label = tr!("genre-animation"))),
                new_entry("orilla", tr!("season-new", season = 2)),
                entry("lazo", Some(tr!("genre-comedy").into()), None),
                entry("altamar", Some(tr!("genre-documentary").into()), None),
                entry("tronos", Some(tr!("genre-fantasy").into()), None),
            ],
        },
        Shelf {
            name: tr!("shelf-anime"),
            style: ShelfStyle::Wide,
            entries: vec![
                new_entry("jujutsu", tr!("season-new", season = 3)),
                entry("komi", Some(tr!("caption-school-comedy").into()), None),
                entry("naruton", Some(tr!("caption-filler").into()), None),
                entry("titos", Some(tr!("caption-final-season").into()), None),
                entry("pixel", Some(tr!("genre-animation").into()), None),
            ],
        },
        Shelf {
            name: tr!("shelf-open-films"),
            style: ShelfStyle::Wide,
            entries: vec![entry(
                "sprite-fright",
                Some(tr!("caption-short-youtube").into()),
                None,
            )],
        },
    ]
});
