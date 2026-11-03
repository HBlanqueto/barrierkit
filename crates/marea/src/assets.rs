use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use barrier_kit::video::{self, VideoStream};

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

const YOUTUBE: [(&str, &str); 1] = [("jjk-netflix", "Jgt6gGNtleE")];

// The decoders only take H.264 in MP4, and the pipeline has no audio branch: a video-only stream is enough.
const YOUTUBE_FORMAT: &str = "bv*[vcodec^=avc1][height<=1080][protocol=https]/b[vcodec^=avc1]";

static STREAMS: OnceLock<Vec<(&str, Option<String>)>> = OnceLock::new();

fn youtube(id: &str) -> Option<&'static str> {
    YOUTUBE
        .iter()
        .find(|(title, _)| *title == id)
        .map(|(_, video)| *video)
}

pub fn remote(id: &str) -> bool {
    youtube(id).is_some()
}

fn stream_url(video: &str) -> Option<String> {
    let output = Command::new("yt-dlp")
        .args(["--no-warnings", "-g", "-f", YOUTUBE_FORMAT])
        .arg(format!("https://www.youtube.com/watch?v={video}"))
        .output()
        .inspect_err(|error| tracing::warn!(%error, "marea: yt-dlp no se pudo lanzar"))
        .ok()?;
    let url = String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .next()?
        .to_owned();
    if url.is_empty() {
        tracing::warn!(video, "marea: yt-dlp no devolvió ningún flujo");
        return None;
    }
    Some(url)
}

pub fn resolve_remote() {
    std::thread::spawn(|| {
        STREAMS.get_or_init(|| {
            YOUTUBE
                .iter()
                .map(|&(id, video)| (id, stream_url(video)))
                .collect()
        });
    });
}

pub fn open_video(id: &str, looping: bool) -> Option<VideoStream> {
    let opened = match youtube(id) {
        Some(_) => {
            let url = STREAMS
                .wait()
                .iter()
                .find(|(title, _)| *title == id)
                .and_then(|(_, url)| url.as_deref())?;
            VideoStream::open_uri(url, looping)
        }
        None => VideoStream::open(media().join("videos").join(format!("{id}.mp4")), looping),
    };
    opened.ok()
}

pub fn still(id: &str) -> impl FnOnce() -> Option<Vec<u8>> + Send + 'static {
    let youtube = youtube(id);
    let path = media().join("stills").join(format!("{id}.jpg"));
    move || match youtube {
        Some(video) => ["maxresdefault", "hqdefault"].iter().find_map(|size| {
            video::fetch(&format!("https://i.ytimg.com/vi/{video}/{size}.jpg")).ok()
        }),
        None => std::fs::read(path).ok(),
    }
}
