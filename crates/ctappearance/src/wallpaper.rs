use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{config_dir, data_dir, replace_file};

pub const ALL_DESKTOPS: &str = "default";

pub fn desktop_key(desktop: Option<u32>) -> String {
    desktop.map_or_else(|| ALL_DESKTOPS.to_owned(), |number| number.to_string())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperScale {
    #[default]
    Fill,
    Fit,
    Stretch,
    Center,
    Custom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperShuffle {
    Login,
    #[default]
    Hourly,
    Daily,
}

impl WallpaperShuffle {
    pub fn seconds(self) -> Option<u64> {
        match self {
            Self::Login => None,
            Self::Hourly => Some(60 * 60),
            Self::Daily => Some(24 * 60 * 60),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperAppearance {
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WallpaperSource {
    BuiltIn,
    File(PathBuf),
    ShuffleColors,
    ShuffleCollection(String),
}

const BUILT_IN: &str = "builtin";
const SHUFFLE_COLORS: &str = "shuffle:colors";
const SHUFFLE_COLLECTION: &str = "shuffle:collection:";

impl WallpaperSource {
    pub fn parse(value: &str) -> Option<Self> {
        if value == BUILT_IN {
            return Some(Self::BuiltIn);
        }
        if value == SHUFFLE_COLORS {
            return Some(Self::ShuffleColors);
        }
        if let Some(name) = value.strip_prefix(SHUFFLE_COLLECTION) {
            return Some(Self::ShuffleCollection(name.to_owned()));
        }
        file_path(value).map(Self::File)
    }

    pub fn value(&self) -> String {
        match self {
            Self::BuiltIn => BUILT_IN.to_owned(),
            Self::File(path) => file_uri(path),
            Self::ShuffleColors => SHUFFLE_COLORS.to_owned(),
            Self::ShuffleCollection(name) => format!("{SHUFFLE_COLLECTION}{name}"),
        }
    }
}

pub fn file_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    for &byte in path.as_os_str().as_encoded_bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~!$&'()*+,;=:@/".contains(&byte) {
            uri.push(byte as char);
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri
}

pub fn file_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?.as_bytes();
    let mut bytes = Vec::with_capacity(rest.len());
    let mut index = 0;
    while index < rest.len() {
        let escaped = (rest[index] == b'%')
            .then(|| rest.get(index + 1..index + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok());
        match escaped {
            Some(byte) => {
                bytes.push(byte);
                index += 3;
            }
            None => {
                bytes.push(rest[index]);
                index += 1;
            }
        }
    }
    Some(PathBuf::from(String::from_utf8_lossy(&bytes).into_owned()))
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct WallpaperSelection {
    #[serde(default)]
    pub scale: WallpaperScale,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub scales: BTreeMap<String, WallpaperScale>,
    #[serde(default)]
    pub desktops: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub crop: BTreeMap<String, [f32; 4]>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub appearance: BTreeMap<String, WallpaperAppearance>,
    #[serde(default)]
    pub shuffle: WallpaperShuffle,
}

impl WallpaperSelection {
    fn owner(&self, desktop: Option<u32>) -> String {
        let own = desktop_key(desktop);
        if self.desktops.contains_key(&own) {
            own
        } else {
            ALL_DESKTOPS.to_owned()
        }
    }

    pub fn has_own(&self, desktop: u32) -> bool {
        self.desktops.contains_key(&desktop.to_string())
    }

    pub fn own_desktops(&self) -> impl Iterator<Item = u32> + '_ {
        self.desktops.keys().filter_map(|key| key.parse().ok())
    }

    pub fn source(&self, desktop: Option<u32>) -> Option<WallpaperSource> {
        self.desktops
            .get(&self.owner(desktop))
            .and_then(|value| WallpaperSource::parse(value))
    }

    pub fn scale_for(&self, desktop: Option<u32>) -> WallpaperScale {
        let owner = self.owner(desktop);
        if owner == ALL_DESKTOPS {
            self.scale
        } else {
            self.scales.get(&owner).copied().unwrap_or_default()
        }
    }

    pub fn crop_for(&self, desktop: Option<u32>, output: &str) -> Option<[f32; 4]> {
        let owner = self.owner(desktop);
        self.crop
            .get(&format!("{owner}/{output}"))
            .or_else(|| self.crop.get(&owner))
            .copied()
    }

    pub fn appearance_for(&self, desktop: Option<u32>) -> Option<WallpaperAppearance> {
        self.appearance.get(&self.owner(desktop)).copied()
    }

    pub fn set_source(&mut self, desktop: Option<u32>, source: Option<WallpaperSource>) {
        let key = desktop_key(desktop);
        if desktop.is_none() {
            self.desktops.retain(|key, _| key == ALL_DESKTOPS);
            self.scales.clear();
            self.crop.retain(|key, _| owner_of(key) == ALL_DESKTOPS);
            self.appearance.retain(|key, _| key == ALL_DESKTOPS);
        }
        match source {
            Some(source) => self.desktops.insert(key.clone(), source.value()),
            None => self.desktops.remove(&key),
        };
        self.crop.retain(|crop, _| owner_of(crop) != key);
        self.appearance.remove(&key);
        if self.scale_for(desktop) == WallpaperScale::Custom {
            self.set_scale(desktop, WallpaperScale::Fill);
        }
    }

    pub fn make_own(&mut self, desktop: u32) {
        if self.has_own(desktop) {
            return;
        }
        let key = desktop.to_string();
        let source = self.source(None).unwrap_or(WallpaperSource::BuiltIn);
        self.desktops.insert(key.clone(), source.value());
        self.scales.insert(key.clone(), self.scale);
        let shared: Vec<(String, [f32; 4])> = self
            .crop
            .iter()
            .filter(|(crop, _)| owner_of(crop) == ALL_DESKTOPS)
            .map(|(crop, value)| (crop.replacen(ALL_DESKTOPS, &key, 1), *value))
            .collect();
        self.crop.extend(shared);
        if let Some(appearance) = self.appearance.get(ALL_DESKTOPS).copied() {
            self.appearance.insert(key, appearance);
        }
    }

    pub fn show_on_all_desktops(&mut self, desktop: u32) {
        if !self.has_own(desktop) {
            return;
        }
        let key = desktop.to_string();
        let source = self.desktops.get(&key).cloned();
        let scale = self.scale_for(Some(desktop));
        let appearance = self.appearance.get(&key).copied();
        let crops: Vec<(String, [f32; 4])> = self
            .crop
            .iter()
            .filter(|(crop, _)| owner_of(crop) == key)
            .map(|(crop, value)| (crop.replacen(&key, ALL_DESKTOPS, 1), *value))
            .collect();
        *self = Self {
            scale,
            crop: crops.into_iter().collect(),
            shuffle: self.shuffle,
            ..Self::default()
        };
        if let Some(source) = source {
            self.desktops.insert(ALL_DESKTOPS.into(), source);
        }
        if let Some(appearance) = appearance {
            self.appearance.insert(ALL_DESKTOPS.into(), appearance);
        }
    }

    pub fn set_scale(&mut self, desktop: Option<u32>, scale: WallpaperScale) {
        let owner = self.owner(desktop);
        if owner == ALL_DESKTOPS {
            self.scale = scale;
        } else {
            self.scales.insert(owner, scale);
        }
    }

    pub fn set_crop(&mut self, desktop: Option<u32>, output: Option<&str>, crop: [f32; 4]) {
        let owner = self.owner(desktop);
        match output {
            Some(output) => {
                self.crop.insert(format!("{owner}/{output}"), crop);
            }
            None => {
                self.crop.retain(|key, _| owner_of(key) != owner);
                self.crop.insert(owner.clone(), crop);
            }
        }
        self.set_scale(desktop, WallpaperScale::Custom);
    }

    pub fn set_appearance(
        &mut self,
        desktop: Option<u32>,
        appearance: Option<WallpaperAppearance>,
    ) {
        let owner = self.owner(desktop);
        match appearance {
            Some(appearance) => self.appearance.insert(owner, appearance),
            None => self.appearance.remove(&owner),
        };
    }
}

fn owner_of(crop_key: &str) -> &str {
    crop_key.split('/').next().unwrap_or(crop_key)
}

pub fn wallpaper_selection_path() -> PathBuf {
    data_dir().join("wallpaper-selection.state")
}

pub fn read_wallpaper_selection(path: &Path) -> Result<WallpaperSelection, toml::de::Error> {
    std::fs::read_to_string(path).map_or_else(
        |_| Ok(WallpaperSelection::default()),
        |text| toml::from_str(&text),
    )
}

pub fn write_wallpaper_selection(
    path: &Path,
    selection: &WallpaperSelection,
) -> std::io::Result<()> {
    replace_file(
        path,
        &toml::to_string(selection).map_err(std::io::Error::other)?,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallpaperKind {
    Image,
    Video,
    Shader,
}

impl WallpaperKind {
    pub fn of(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "png" | "jpg" | "jpeg" | "heic" | "heif" => Some(Self::Image),
            "mp4" | "mov" | "webm" | "mkv" => Some(Self::Video),
            "frag" | "glsl" => Some(Self::Shader),
            _ => None,
        }
    }
}

pub fn wallpapers_in(folder: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && WallpaperKind::of(path).is_some())
        .collect();
    found.sort();
    found
}

pub fn system_wallpapers_dir() -> Option<PathBuf> {
    crate::xdg::data_dirs()
        .into_iter()
        .map(|dir| dir.join("wallpapers"))
        .find(|dir| dir.is_dir())
}

fn uri_hash(file: &Path) -> String {
    use md5::Digest;
    let digest = md5::Md5::digest(file_uri(file).as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn thumbnail_path(file: &Path) -> Option<PathBuf> {
    let name = format!("{}.png", uri_hash(file));
    Some(
        crate::xdg::cache_home()?
            .join("thumbnails")
            .join("x-large")
            .join(name),
    )
}

pub fn frame_path(file: &Path, appearance: WallpaperAppearance) -> Option<PathBuf> {
    let suffix = match appearance {
        WallpaperAppearance::Light => "light",
        WallpaperAppearance::Dark => "dark",
    };
    let name = format!("{}-{suffix}.png", uri_hash(file));
    Some(
        crate::xdg::cache_home()?
            .join("malaysia")
            .join("wallpaper-frames")
            .join(name),
    )
}

pub fn modified_seconds(file: &Path) -> Option<u64> {
    let modified = std::fs::metadata(file).ok()?.modified().ok()?;
    Some(
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs(),
    )
}

// Thumbnail spec: a thumbnail is valid while its Thumb::MTime equals the file's modification time.
pub fn thumbnail_is_fresh(thumbnail: &Path, file: &Path) -> bool {
    let (Ok(bytes), Some(modified)) = (std::fs::read(thumbnail), modified_seconds(file)) else {
        return false;
    };
    png_text(&bytes, "Thumb::MTime").is_some_and(|value| value == modified.to_string())
}

fn png_text(png: &[u8], key: &str) -> Option<String> {
    let mut at = 8;
    while at + 8 <= png.len() {
        let length = u32::from_be_bytes(png[at..at + 4].try_into().ok()?) as usize;
        let kind = &png[at + 4..at + 8];
        let data = png.get(at + 8..at + 8 + length)?;
        if kind == b"IDAT" {
            return None;
        }
        if kind == b"tEXt"
            && let Some(value) = data
                .strip_prefix(key.as_bytes())
                .and_then(|rest| rest.strip_prefix(b"\0"))
        {
            return Some(String::from_utf8_lossy(value).into_owned());
        }
        at += 12 + length;
    }
    None
}

pub fn custom_colors_dir() -> PathBuf {
    data_dir().join("wallpapers").join("colors")
}

pub fn wallpaper_colors() -> Vec<PathBuf> {
    let system = system_wallpapers_dir().map(|dir| dir.join("solid-colors"));
    system
        .into_iter()
        .chain([custom_colors_dir()])
        .flat_map(|folder| wallpapers_in(&folder))
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct WallpaperCollection {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub folders: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<PathBuf>,
}

impl WallpaperCollection {
    pub fn wallpapers(&self) -> Vec<PathBuf> {
        let mut all: Vec<PathBuf> = self
            .folders
            .iter()
            .flat_map(|folder| wallpapers_in(folder))
            .chain(self.images.iter().filter(|path| path.is_file()).cloned())
            .collect();
        let mut seen = std::collections::HashSet::new();
        all.retain(|path| seen.insert(path.clone()));
        all
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct WallpaperConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<WallpaperCollection>,
}

impl WallpaperConfig {
    pub fn collection(&self, name: &str) -> Option<&WallpaperCollection> {
        self.collections
            .iter()
            .find(|collection| collection.name == name)
    }
}

pub fn wallpaper_config_path() -> PathBuf {
    config_dir().join("wallpaper.domain")
}

pub fn read_wallpaper_config(path: &Path) -> Result<WallpaperConfig, toml::de::Error> {
    std::fs::read_to_string(path).map_or_else(
        |_| Ok(WallpaperConfig::default()),
        |text| toml::from_str(&text),
    )
}

pub fn write_wallpaper_config(path: &Path, config: &WallpaperConfig) -> std::io::Result<()> {
    replace_file(
        path,
        &toml::to_string(config).map_err(std::io::Error::other)?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bandura() -> WallpaperSource {
        WallpaperSource::File("/w/Bandura.jpg".into())
    }

    #[test]
    fn a_missing_selection_fills_the_screen_with_the_built_in_wallpaper() {
        let path = std::env::temp_dir().join("ctappearance-no-such-selection.state");
        let selection = read_wallpaper_selection(&path).unwrap();
        assert_eq!(selection, WallpaperSelection::default());
        assert_eq!(selection.source(Some(1)), None);
    }

    #[test]
    fn a_selection_keeps_every_field_through_a_write() {
        let dir =
            std::env::temp_dir().join(format!("ctappearance-selection-{}", std::process::id()));
        let path = dir.join("wallpaper-selection.state");
        let mut selection = WallpaperSelection::default();
        selection.set_source(None, Some(bandura()));
        selection.set_source(Some(2), Some(WallpaperSource::ShuffleColors));
        selection.set_crop(None, Some("eDP-1"), [0.1, 0.2, 0.5, 0.4]);
        selection.set_appearance(Some(2), Some(WallpaperAppearance::Dark));
        selection.shuffle = WallpaperShuffle::Daily;
        write_wallpaper_selection(&path, &selection).unwrap();
        assert_eq!(read_wallpaper_selection(&path).unwrap(), selection);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn showing_a_desktop_on_all_desktops_makes_its_choices_the_shared_ones() {
        let mut selection = WallpaperSelection::default();
        selection.make_own(2);
        selection.make_own(3);
        selection.set_source(Some(2), Some(bandura()));
        selection.set_crop(Some(2), Some("eDP-1"), [0.1, 0.2, 0.5, 0.4]);
        selection.set_appearance(Some(2), Some(WallpaperAppearance::Dark));
        selection.show_on_all_desktops(2);
        assert_eq!(selection.own_desktops().count(), 0);
        assert_eq!(selection.source(Some(3)), Some(bandura()));
        assert_eq!(selection.scale_for(Some(3)), WallpaperScale::Custom);
        assert_eq!(
            selection.crop_for(None, "eDP-1"),
            Some([0.1, 0.2, 0.5, 0.4])
        );
        assert_eq!(
            selection.appearance_for(Some(1)),
            Some(WallpaperAppearance::Dark)
        );
    }

    #[test]
    fn a_desktop_without_its_own_wallpaper_shows_the_shared_one_with_its_framing() {
        let mut selection = WallpaperSelection::default();
        selection.set_source(None, Some(bandura()));
        selection.set_crop(None, None, [0.0, 0.0, 0.5, 0.5]);
        assert_eq!(selection.source(Some(3)), Some(bandura()));
        assert_eq!(selection.scale_for(Some(3)), WallpaperScale::Custom);
        assert_eq!(
            selection.crop_for(Some(3), "eDP-1"),
            Some([0.0, 0.0, 0.5, 0.5])
        );
    }

    #[test]
    fn a_desktop_with_its_own_wallpaper_ignores_the_shared_framing() {
        let mut selection = WallpaperSelection::default();
        selection.set_crop(None, None, [0.0, 0.0, 0.5, 0.5]);
        selection.set_source(Some(2), Some(bandura()));
        assert_eq!(selection.scale_for(Some(2)), WallpaperScale::Fill);
        assert_eq!(selection.crop_for(Some(2), "eDP-1"), None);
    }

    #[test]
    fn an_output_crop_wins_over_the_crop_for_every_output() {
        let mut selection = WallpaperSelection::default();
        selection.set_crop(None, None, [0.0, 0.0, 1.0, 1.0]);
        selection.set_crop(None, Some("HDMI-A-1"), [0.2, 0.2, 0.5, 0.5]);
        assert_eq!(
            selection.crop_for(None, "HDMI-A-1"),
            Some([0.2, 0.2, 0.5, 0.5])
        );
        assert_eq!(
            selection.crop_for(None, "eDP-1"),
            Some([0.0, 0.0, 1.0, 1.0])
        );
        selection.set_crop(None, None, [0.1, 0.1, 0.8, 0.8]);
        assert_eq!(
            selection.crop_for(None, "HDMI-A-1"),
            Some([0.1, 0.1, 0.8, 0.8])
        );
    }

    #[test]
    fn choosing_for_every_desktop_drops_their_own_choices() {
        let mut selection = WallpaperSelection::default();
        selection.set_source(Some(2), Some(WallpaperSource::ShuffleColors));
        selection.set_crop(Some(2), None, [0.0, 0.0, 0.5, 0.5]);
        selection.set_source(None, Some(bandura()));
        assert!(!selection.has_own(2));
        assert!(selection.crop.is_empty());
        assert_eq!(selection.source(Some(2)), Some(bandura()));
    }

    #[test]
    fn sources_round_trip_through_their_text() {
        let spaced = WallpaperSource::File("/home/h/Mis fotos/Año 1%.jpg".into());
        for source in [
            WallpaperSource::BuiltIn,
            spaced,
            WallpaperSource::ShuffleColors,
            WallpaperSource::ShuffleCollection("Viajes".into()),
        ] {
            assert_eq!(WallpaperSource::parse(&source.value()), Some(source));
        }
    }

    #[test]
    fn a_collection_lists_its_folders_and_images_once() {
        let dir =
            std::env::temp_dir().join(format!("ctappearance-collection-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["a.jpg", "b.heic", "notes.txt", "c.mp4"] {
            std::fs::write(dir.join(name), b"").unwrap();
        }
        let collection = WallpaperCollection {
            name: "Viajes".into(),
            folders: vec![dir.clone()],
            images: vec![dir.join("a.jpg")],
        };
        let names: Vec<_> = collection
            .wallpapers()
            .iter()
            .map(|path| path.file_name().unwrap().to_owned())
            .collect();
        assert_eq!(names, ["a.jpg", "b.heic", "c.mp4"]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_desktop_made_its_own_keeps_what_it_showed_and_then_changes_alone() {
        let mut selection = WallpaperSelection::default();
        selection.set_source(None, Some(bandura()));
        selection.set_crop(None, Some("eDP-1"), [0.1, 0.1, 0.5, 0.5]);
        selection.make_own(2);
        assert_eq!(selection.source(Some(2)), Some(bandura()));
        assert_eq!(
            selection.crop_for(Some(2), "eDP-1"),
            Some([0.1, 0.1, 0.5, 0.5])
        );
        selection.set_scale(Some(2), WallpaperScale::Fit);
        assert_eq!(selection.scale_for(Some(2)), WallpaperScale::Fit);
        assert_eq!(selection.scale_for(Some(1)), WallpaperScale::Custom);
    }

    #[test]
    fn file_uris_escape_like_glib_so_thumbnails_are_shared() {
        assert_eq!(
            file_uri(Path::new("/home/h/Mis fotos/a&b (1).jpg")),
            "file:///home/h/Mis%20fotos/a&b%20(1).jpg"
        );
    }

    #[test]
    fn a_thumbnail_text_is_read_from_the_chunks_before_the_image_data() {
        let chunk = |kind: &[u8], data: &[u8]| {
            let mut bytes = (data.len() as u32).to_be_bytes().to_vec();
            bytes.extend_from_slice(kind);
            bytes.extend_from_slice(data);
            bytes.extend_from_slice(&[0; 4]);
            bytes
        };
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        png.extend(chunk(b"IHDR", &[0; 13]));
        png.extend(chunk(b"tEXt", b"Thumb::MTime\x001700000000"));
        png.extend(chunk(b"IDAT", &[]));
        png.extend(chunk(b"tEXt", b"Thumb::URI\x00file:///late"));
        assert_eq!(
            png_text(&png, "Thumb::MTime").as_deref(),
            Some("1700000000")
        );
        assert_eq!(png_text(&png, "Thumb::URI"), None);
    }
}
