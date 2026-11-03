use std::path::PathBuf;

use barrier_kit::video::VideoStream;

const IDS: [&str; 16] = [
    "orilla",
    "neon",
    "pan",
    "raras",
    "carton",
    "tronos",
    "oso",
    "separ",
    "oficina",
    "lazo",
    "espejo",
    "hipotecas",
    "pixel",
    "lobos",
    "cometas",
    "altamar",
];

fn media(kind: &str, id: &str, extension: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("media")
        .join(kind)
        .join(format!("{id}.{extension}"))
}

#[test]
fn every_title_has_a_video_that_decodes_and_a_still() {
    for id in IDS {
        assert!(
            media("stills", id, "jpg").exists(),
            "{id}: falta la imagen fija"
        );
        let stream = VideoStream::open(media("videos", id, "mp4"), true)
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        assert!(
            (stream.duration() - 10.0).abs() < 0.1,
            "{id}: dura {} s",
            stream.duration()
        );
    }
}
