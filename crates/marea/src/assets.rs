use std::path::PathBuf;

const FONTS: [&[u8]; 14] = [
    include_bytes!("../fonts/AbrilFatface.ttf"),
    include_bytes!("../fonts/Bangers.ttf"),
    include_bytes!("../fonts/BebasNeue.ttf"),
    include_bytes!("../fonts/BlackOpsOne.ttf"),
    include_bytes!("../fonts/Bungee.ttf"),
    include_bytes!("../fonts/Cinzel.ttf"),
    include_bytes!("../fonts/Creepster.ttf"),
    include_bytes!("../fonts/Monoton.ttf"),
    include_bytes!("../fonts/Pacifico.ttf"),
    include_bytes!("../fonts/PlayfairDisplay.ttf"),
    include_bytes!("../fonts/PressStart2P.ttf"),
    include_bytes!("../fonts/Righteous.ttf"),
    include_bytes!("../fonts/RubikMonoOne.ttf"),
    include_bytes!("../fonts/SpecialElite.ttf"),
];

pub fn register_fonts() {
    for font in FONTS {
        if !barrier_kit::register_font(font) {
            tracing::warn!("marea: una tipografía de carátula no se pudo cargar");
        }
    }
}

fn media() -> PathBuf {
    std::env::var_os("MAREA_MEDIA").map_or_else(
        || PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/media")),
        PathBuf::from,
    )
}

pub fn video(id: &str) -> PathBuf {
    media().join("videos").join(format!("{id}.mp4"))
}

pub fn still(id: &str) -> PathBuf {
    media().join("stills").join(format!("{id}.jpg"))
}
