use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};

use ctappearance::{config_dir, data_dir, file_path, file_uri, write_toml};
use serde::{Deserialize, Serialize};

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
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub paused: BTreeSet<String>,
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

    pub fn paused_for(&self, desktop: Option<u32>) -> bool {
        self.paused.contains(&self.owner(desktop))
    }

    pub fn set_source(&mut self, desktop: Option<u32>, source: Option<WallpaperSource>) {
        if desktop.is_none() {
            self.desktops.retain(|key, _| key == ALL_DESKTOPS);
            self.scales.clear();
            self.crop.retain(|key, _| owner_of(key) == ALL_DESKTOPS);
            self.appearance.retain(|key, _| key == ALL_DESKTOPS);
            self.paused.retain(|key| key == ALL_DESKTOPS);
        }
        self.set_entry(desktop, source);
    }

    pub fn replace_source(&mut self, old: &WallpaperSource, new: Option<&WallpaperSource>) {
        let old = old.value();
        let using: Vec<String> = self
            .desktops
            .iter()
            .filter(|(_, source)| **source == old)
            .map(|(key, _)| key.clone())
            .collect();
        for key in using {
            match new {
                Some(new) => {
                    self.desktops.insert(key, new.value());
                }
                None => self.set_entry(key.parse().ok(), None),
            }
        }
    }

    fn set_entry(&mut self, desktop: Option<u32>, source: Option<WallpaperSource>) {
        let key = desktop_key(desktop);
        match source {
            Some(source) => {
                self.desktops.insert(key.clone(), source.value());
            }
            None => {
                self.desktops.remove(&key);
                self.scales.remove(&key);
            }
        }
        self.crop.retain(|crop, _| owner_of(crop) != key);
        self.appearance.remove(&key);
        self.paused.remove(&key);
        if self.owner(desktop) == key && self.scale_for(desktop) == WallpaperScale::Custom {
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
        if self.paused.contains(ALL_DESKTOPS) {
            self.paused.insert(key.clone());
        }
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
        let paused = self.paused.contains(&key);
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
        if paused {
            self.paused.insert(ALL_DESKTOPS.into());
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

    pub fn set_paused(&mut self, desktop: Option<u32>, paused: bool) {
        let owner = self.owner(desktop);
        if paused {
            self.paused.insert(owner);
        } else {
            self.paused.remove(&owner);
        }
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
    write_toml(path, selection)
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
    ctappearance::xdg::data_dirs()
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
        ctappearance::xdg::cache_home()?
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
        ctappearance::xdg::cache_home()?
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
    let (Some(bytes), Some(modified)) = (png_header(thumbnail), modified_seconds(file)) else {
        return false;
    };
    png_text(&bytes, "Thumb::MTime").is_some_and(|value| value == modified.to_string())
}

pub fn thumbnail_image_size(thumbnail: &Path) -> Option<(u32, u32)> {
    let bytes = png_header(thumbnail)?;
    let side = |key| png_text(&bytes, key)?.parse().ok();
    Some((side("Thumb::Image::Width")?, side("Thumb::Image::Height")?))
}

fn png_header(png: &Path) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(png)
        .ok()?
        .take(64 * 1024)
        .read_to_end(&mut bytes)
        .ok()?;
    Some(bytes)
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
    write_toml(path, config)
}
