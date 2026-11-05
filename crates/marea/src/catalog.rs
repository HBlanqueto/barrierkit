use vow_design::Rgba;

use crate::WHITE;

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
    pub meta: [&'static str; 4],
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

pub const CATALOG: [Title; 21] = [
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
        kind: "Serie original",
        meta: ["Drama", "2026", "+13", "8 episodios"],
        lede: "Un faro abandonado, una radio que vuelve a sonar y un pueblo que esconde lo que pasó la noche de la tormenta.",
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
        kind: "Película · Estreno",
        meta: ["Thriller", "2026", "+16", "1 h 52 min"],
        lede: "Una mensajera nocturna recibe un paquete que nadie quiere recoger en una ciudad que nunca apaga las luces.",
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
        kind: "Parodia · Serie",
        meta: ["Drama", "2026", "+16", "5 temporadas"],
        lede: "Un profesor de química se convierte en el panadero más temido del desierto. Su masa madre no perdona.",
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
        kind: "Parodia · Serie",
        meta: ["Ciencia ficción", "2025", "+13", "4 temporadas"],
        lede: "Un pueblo, unas luces de Navidad que parpadean solas y un sótano del que nadie vuelve con wifi.",
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
        kind: "Parodia · Serie",
        meta: ["Thriller", "2024", "+16", "3 partes"],
        lede: "Ocho atracadores con máscaras de cartón planean el golpe del siglo a la fábrica de embalajes.",
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
        kind: "Parodia · Serie",
        meta: ["Fantasía", "2026", "+16", "8 temporadas"],
        lede: "Siete reinos se disputan la única silla de madera que no cojea. El invierno trae astillas.",
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
        kind: "Parodia · Serie",
        meta: ["Comedia", "2025", "+13", "3 temporadas"],
        lede: "Un chef de alta cocina vuelve a casa para salvar la panadería familiar. Sí, chef.",
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
        kind: "Parodia · Serie",
        meta: ["Misterio", "2026", "+13", "2 temporadas"],
        lede: "Empleados que olvidan su vida al cruzar el ascensor. Lo peor: tampoco recuerdan dónde aparcaron.",
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
        kind: "Parodia · Serie",
        meta: ["Comedia", "2021", "TP", "9 temporadas"],
        lede: "Un documental sobre una distribuidora de papel donde nadie distribuye nada.",
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
        kind: "Parodia · Serie",
        meta: ["Comedia", "2024", "TP", "3 temporadas"],
        lede: "Un entrenador de rodeo acaba dirigiendo un equipo de fútbol. No sabe qué es un fuera de juego. Da igual.",
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
        kind: "Parodia · Antología",
        meta: ["Ciencia ficción", "2026", "+16", "7 temporadas"],
        lede: "Historias sobre la tecnología que usamos cada día y el botón de «aceptar» que nadie lee.",
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
        kind: "Parodia · Serie",
        meta: ["Drama", "2023", "+16", "4 temporadas"],
        lede: "Cuatro hermanos pelean por heredar el imperio familiar: un piso con hipoteca a 40 años.",
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
        kind: "Original · Animación",
        meta: ["Animación", "2026", "TP", "12 episodios"],
        lede: "Un fontanero de ocho bits descubre que su mundo se acaba en el borde derecho de la pantalla.",
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
        kind: "Película",
        meta: ["Terror", "2025", "+18", "1 h 38 min"],
        lede: "Una excursión escolar, un bosque sin cobertura y una luna demasiado grande.",
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
        kind: "Serie original · Comedia",
        meta: ["Comedia", "2025", "TP", "10 episodios"],
        lede: "Tres hermanas heredan un vivero que en realidad vende cometas, y ninguna sabe hacerlas volar.",
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
        kind: "Documental",
        meta: ["Naturaleza", "2026", "TP", "6 episodios"],
        lede: "Un año a bordo de un velero científico siguiendo a las ballenas jorobadas del Atlántico.",
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
        kind: "Parodia · Anime",
        meta: ["Animación", "2026", "+16", "3 temporadas"],
        lede: "Un estudiante se traga un dedo maldito y ahora tiene que aprobar exorcismo y matemáticas el mismo día.",
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
        kind: "Parodia · Anime",
        meta: ["Animación", "2025", "TP", "2 temporadas"],
        lede: "La chica más popular del instituto quiere hacer cien amigos sin decir una palabra. Lleva una libreta y mucha paciencia.",
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
        kind: "Parodia · Anime",
        meta: ["Animación", "2024", "+13", "500 episodios"],
        lede: "Un ninja rubio grita a todas horas que será el jefe de la aldea. Antes tiene que pagar el ramen que debe.",
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
        kind: "Parodia · Anime",
        meta: ["Animación", "2023", "+16", "4 temporadas"],
        lede: "Tres murallas protegen la ciudad de unos tíos gigantes que solo quieren saludar y quedarse a comer.",
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
        kind: "Cortometraje · Blender Studio",
        meta: ["Animación", "2021", "+12", "Cortometraje"],
        lede: "Unos adolescentes acampan en un bosque donde no todas las setas son amigas. Prueba de destacado: póster, galería y vista previa desde YouTube.",
        series: false,
    },
];

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

pub const GENRES: [(Option<Genre>, &str); 8] = [
    (None, "Todo"),
    (Some(Genre::Comedy), "Comedia"),
    (Some(Genre::Drama), "Drama"),
    (Some(Genre::Thriller), "Thriller"),
    (Some(Genre::Horror), "Terror"),
    (Some(Genre::Fantasy), "Fantasía"),
    (Some(Genre::Animation), "Animación"),
    (Some(Genre::Documentary), "Documental"),
];

#[derive(Clone, Copy, PartialEq)]
pub struct Entry {
    pub id: &'static str,
    pub caption: Option<&'static str>,
    pub watched: Option<f32>,
}

const fn entry(id: &'static str, caption: Option<&'static str>, watched: Option<f32>) -> Entry {
    Entry {
        id,
        caption,
        watched,
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
    pub entries: &'static [Entry],
}

pub const SHELVES: [Shelf; 6] = [
    Shelf {
        name: "Seguir viendo",
        style: ShelfStyle::Wide,
        entries: &[
            entry("neon", Some("Quedan 34 min"), Some(0.62)),
            entry("pan", Some("T3 · E7"), Some(0.4)),
            entry("altamar", Some("T1 · E3"), Some(0.35)),
            entry("oso", Some("T2 · E1"), Some(0.1)),
            entry("separ", Some("Quedan 12 min"), Some(0.85)),
        ],
    },
    Shelf {
        name: "Top 10 de hoy",
        style: ShelfStyle::Ranked,
        entries: &[
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
        name: "Parodias que no pediste",
        style: ShelfStyle::Wide,
        entries: &[
            entry("separ", Some("Misterio"), None),
            entry("oficina", Some("Comedia"), None),
            entry("hipotecas", Some("Drama"), None),
            entry("carton", Some("Thriller"), None),
            entry("oso", Some("Comedia"), None),
            entry("espejo", Some("Antología"), None),
        ],
    },
    Shelf {
        name: "Estrenos de la semana",
        style: ShelfStyle::Wide,
        entries: &[
            entry("lobos", Some("Terror · Nuevo"), None),
            entry("pixel", Some("Animación · Nuevo"), None),
            entry("orilla", Some("T2 · Nuevo"), None),
            entry("lazo", Some("Comedia"), None),
            entry("altamar", Some("Documental"), None),
            entry("tronos", Some("Fantasía"), None),
        ],
    },
    Shelf {
        name: "Anime que no pediste",
        style: ShelfStyle::Wide,
        entries: &[
            entry("jujutsu", Some("T3 · Nuevo"), None),
            entry("komi", Some("Comedia escolar"), None),
            entry("naruton", Some("Relleno incluido"), None),
            entry("titos", Some("Temporada final, otra vez"), None),
            entry("pixel", Some("Animación"), None),
        ],
    },
    Shelf {
        name: "Películas abiertas",
        style: ShelfStyle::Wide,
        entries: &[entry("sprite-fright", Some("Cortometraje · YouTube"), None)],
    },
];
