use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::{
        Mutex, PoisonError,
        atomic::{AtomicU8, Ordering},
    },
    time::{Duration, SystemTime},
};

use ctdesign::{
    COLOR_FILTER_INTENSITY, DISPLAY_CONTRAST, POINTER_SIZES, Rgba, TEXT_SIZES, TINT_STRENGTH,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};
use toml_edit::{Array, DocumentMut};

use crate::{
    accent,
    icon_style::{IconStyle, IconTone},
    tinting::WallpaperGrid,
    xdg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    Dark,
    #[serde(rename = "dark_tint")]
    DarkTint,
    #[default]
    #[serde(alias = "auto")]
    Light,
}

impl ColorScheme {
    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark | Self::DarkTint)
    }

    pub(crate) fn pick<T>(self, [light, dark, dark_tint]: [T; 3]) -> T {
        match self {
            Self::Light => light,
            Self::Dark => dark,
            Self::DarkTint => dark_tint,
        }
    }

    fn following(self, dark: bool) -> Self {
        match (dark, self) {
            (false, _) => Self::Light,
            (true, Self::DarkTint) => Self::DarkTint,
            (true, _) => Self::Dark,
        }
    }
}

static SYSTEM_SCHEME: AtomicU8 = AtomicU8::new(0);

/// Records the light or dark choice of the desktop (macOS, the freedesktop portal); `None` when it states none.
pub fn set_system_scheme(scheme: Option<ColorScheme>) {
    let code = scheme.map_or(0, |scheme| if scheme.is_dark() { 2 } else { 1 });
    SYSTEM_SCHEME.store(code, Ordering::Relaxed);
}

fn system_scheme() -> Option<ColorScheme> {
    match SYSTEM_SCHEME.load(Ordering::Relaxed) {
        1 => Some(ColorScheme::Light),
        2 => Some(ColorScheme::Dark),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowColorMode {
    #[default]
    Neutral,
    Accent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TintStrength {
    Subtle,
    #[default]
    Normal,
    Strong,
    Intense,
}

impl TintStrength {
    pub fn factor(self) -> f32 {
        TINT_STRENGTH[self as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TintParts {
    pub sidebars: bool,
    pub window: bool,
    pub groups: bool,
    pub controls: bool,
    pub menus: bool,
    pub bars: bool,
}

impl Default for TintParts {
    fn default() -> Self {
        Self {
            sidebars: true,
            window: true,
            groups: true,
            controls: true,
            menus: true,
            bars: false,
        }
    }
}

pub(crate) fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).unwrap_or_else(|error| {
        tracing::warn!(%error, "invalid appearance value; using its default");
        T::default()
    }))
}

fn stepped<'de, D: Deserializer<'de>>(
    deserializer: D,
    steps: &[f32],
) -> Result<Option<f32>, D::Error> {
    Ok(lenient::<D, Option<f32>>(deserializer)?.and_then(|value| {
        steps
            .iter()
            .copied()
            .min_by(|a, b| (a - value).abs().total_cmp(&(b - value).abs()))
    }))
}

fn display_contrast<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    Ok(stepped(deserializer, &DISPLAY_CONTRAST)?.unwrap_or(DISPLAY_CONTRAST[0]))
}

fn hour<'de, D: Deserializer<'de>, const FALLBACK: u8>(deserializer: D) -> Result<u8, D::Error> {
    Ok(lenient::<D, Option<u8>>(deserializer)?.map_or(FALLBACK, |hour| hour.min(23)))
}

fn lenient_true<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(lenient::<D, Option<bool>>(deserializer)?.unwrap_or(true))
}

const DARK_FROM: u8 = 19;
const DARK_UNTIL: u8 = 7;

fn text_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    Ok(toolkit_text_size(deserializer)?.unwrap_or(TEXT_SIZES[0]))
}

fn toolkit_text_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f32>, D::Error> {
    stepped(deserializer, &TEXT_SIZES)
}

fn pointer_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    Ok(stepped(deserializer, &POINTER_SIZES)?.unwrap_or(POINTER_SIZES[0]))
}

fn color_filter_intensity<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let strongest = COLOR_FILTER_INTENSITY[COLOR_FILTER_INTENSITY.len() - 1];
    Ok(stepped(deserializer, &COLOR_FILTER_INTENSITY)?.unwrap_or(strongest))
}

fn liquid_glass_tint<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let amount = lenient::<D, Option<f32>>(deserializer)?.unwrap_or(0.0);
    Ok(if amount.is_nan() {
        0.0
    } else {
        amount.clamp(0.0, 1.0)
    })
}

fn deserialize_tint_parts<'de, D>(deserializer: D) -> Result<TintParts, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(list) = lenient::<D, Option<Vec<String>>>(deserializer)? else {
        return Ok(TintParts::default());
    };
    let mut parts = TintParts {
        sidebars: false,
        window: false,
        groups: false,
        controls: false,
        menus: false,
        bars: false,
    };
    for name in list {
        match name.as_str() {
            "sidebars" => parts.sidebars = true,
            "window" => parts.window = true,
            "groups" => parts.groups = true,
            "controls" => parts.controls = true,
            "menus" => parts.menus = true,
            "bars" => parts.bars = true,
            _ => tracing::warn!(name, "unknown tint part ignored"),
        }
    }
    Ok(parts)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrafficLightStyle {
    #[serde(alias = "macos")]
    Standard,
    Accent,
    #[default]
    Wallpaper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollBars {
    #[default]
    Automatic,
    WhenScrolling,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollBarClick {
    #[default]
    NextPage,
    ClickedSpot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MenuBarSize {
    #[default]
    Default,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorFilter {
    #[default]
    Grayscale,
    RedGreen,
    GreenRed,
    BlueYellow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toolkit {
    BarrierKit,
    Gtk,
    Qt,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct TextSizes {
    #[serde(
        deserialize_with = "toolkit_text_size",
        skip_serializing_if = "Option::is_none"
    )]
    pub barrierkit: Option<f32>,
    #[serde(
        deserialize_with = "toolkit_text_size",
        skip_serializing_if = "Option::is_none"
    )]
    pub gtk: Option<f32>,
    #[serde(
        deserialize_with = "toolkit_text_size",
        skip_serializing_if = "Option::is_none"
    )]
    pub qt: Option<f32>,
}

impl TextSizes {
    pub fn get_mut(&mut self, toolkit: Toolkit) -> &mut Option<f32> {
        match toolkit {
            Toolkit::BarrierKit => &mut self.barrierkit,
            Toolkit::Gtk => &mut self.gtk,
            Toolkit::Qt => &mut self.qt,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct AppearanceConfig {
    #[serde(deserialize_with = "lenient")]
    pub color_scheme: ColorScheme,
    #[serde(deserialize_with = "lenient")]
    pub automatic_scheme: bool,
    #[serde(deserialize_with = "hour::<_, DARK_FROM>")]
    pub automatic_dark_from: u8,
    #[serde(deserialize_with = "hour::<_, DARK_UNTIL>")]
    pub automatic_dark_until: u8,
    #[serde(skip_serializing, deserialize_with = "lenient")]
    pub accent: Option<String>,
    #[serde(skip_serializing, deserialize_with = "lenient")]
    pub accent_color: Option<[f32; 3]>,
    #[serde(deserialize_with = "lenient")]
    pub wallpaper_accent: bool,
    #[serde(deserialize_with = "lenient")]
    pub contrast: bool,
    #[serde(deserialize_with = "lenient")]
    pub reduced_motion: bool,
    #[serde(deserialize_with = "lenient")]
    pub scroll_bars: ScrollBars,
    #[serde(deserialize_with = "lenient")]
    pub scroll_bar_click: ScrollBarClick,
    #[serde(deserialize_with = "lenient")]
    pub reduce_transparency: bool,
    #[serde(deserialize_with = "lenient")]
    pub window_color: WindowColorMode,
    #[serde(deserialize_with = "lenient")]
    pub traffic_lights: TrafficLightStyle,
    #[serde(deserialize_with = "lenient")]
    pub tint_strength: TintStrength,
    #[serde(
        deserialize_with = "deserialize_tint_parts",
        serialize_with = "serialize_tint_parts"
    )]
    pub tint_parts: TintParts,
    #[serde(deserialize_with = "lenient_true")]
    pub wallpaper_tinting: bool,
    #[serde(deserialize_with = "lenient")]
    pub wallpaper_tinting_in_light: bool,
    #[serde(deserialize_with = "lenient")]
    pub invert_colors: bool,
    #[serde(deserialize_with = "lenient")]
    pub show_borders: bool,
    #[serde(deserialize_with = "lenient")]
    pub differentiate_without_color: bool,
    #[serde(deserialize_with = "display_contrast")]
    pub display_contrast: f32,
    #[serde(deserialize_with = "text_size")]
    pub text_size: f32,
    #[serde(deserialize_with = "lenient")]
    pub text_sizes: TextSizes,
    #[serde(deserialize_with = "lenient")]
    pub menu_bar_size: MenuBarSize,
    #[serde(deserialize_with = "lenient")]
    pub icon_style: IconStyle,
    #[serde(deserialize_with = "lenient")]
    pub icon_tone: IconTone,
    #[serde(skip_serializing, deserialize_with = "lenient")]
    pub icon_color: Option<[f32; 3]>,
    #[serde(deserialize_with = "lenient_true")]
    pub shake_to_locate: bool,
    #[serde(deserialize_with = "pointer_size")]
    pub pointer_size: f32,
    #[serde(deserialize_with = "lenient")]
    pub color_filters: bool,
    #[serde(deserialize_with = "lenient")]
    pub color_filter: ColorFilter,
    #[serde(deserialize_with = "color_filter_intensity")]
    pub color_filter_intensity: f32,
    #[serde(deserialize_with = "liquid_glass_tint")]
    pub liquid_glass_tint: f32,
    #[serde(deserialize_with = "lenient_true")]
    pub gtk_bridge: bool,
    #[serde(deserialize_with = "lenient_true")]
    pub qt_bridge: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default(),
            automatic_scheme: false,
            automatic_dark_from: DARK_FROM,
            automatic_dark_until: DARK_UNTIL,
            accent: None,
            accent_color: None,
            wallpaper_accent: false,
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::default(),
            scroll_bar_click: ScrollBarClick::default(),
            reduce_transparency: false,
            window_color: WindowColorMode::default(),
            traffic_lights: TrafficLightStyle::default(),
            tint_strength: TintStrength::default(),
            tint_parts: TintParts::default(),
            wallpaper_tinting: true,
            wallpaper_tinting_in_light: false,
            invert_colors: false,
            show_borders: false,
            differentiate_without_color: false,
            display_contrast: DISPLAY_CONTRAST[0],
            text_size: TEXT_SIZES[0],
            text_sizes: TextSizes::default(),
            menu_bar_size: MenuBarSize::default(),
            icon_style: IconStyle::default(),
            icon_tone: IconTone::default(),
            icon_color: None,
            shake_to_locate: true,
            pointer_size: POINTER_SIZES[0],
            color_filters: false,
            color_filter: ColorFilter::default(),
            color_filter_intensity: COLOR_FILTER_INTENSITY[COLOR_FILTER_INTENSITY.len() - 1],
            liquid_glass_tint: 0.0,
            gtk_bridge: true,
            qt_bridge: true,
        }
    }
}

impl AppearanceConfig {
    pub fn text_size_for(&self, toolkit: Toolkit) -> f32 {
        let mut sizes = self.text_sizes;
        sizes.get_mut(toolkit).unwrap_or(self.text_size)
    }

    pub fn scheme_at(&self, hour: u8) -> ColorScheme {
        if !self.automatic_scheme {
            return self.color_scheme;
        }
        let (from, until) = (self.automatic_dark_from, self.automatic_dark_until);
        let night = if from <= until {
            (from..until).contains(&hour)
        } else {
            hour >= from || hour < until
        };
        self.color_scheme.following(night)
    }

    // The system's light or dark wins over the domain's; Dark Tint stays the dark it asks for (ADR 0114).
    fn following_system(self, system: Option<ColorScheme>) -> Self {
        let color_scheme = match system {
            Some(system) => self.color_scheme.following(system.is_dark()),
            None => self.current_scheme(),
        };
        Self {
            color_scheme,
            ..self
        }
    }

    pub fn current_scheme(&self) -> ColorScheme {
        local_hour().map_or(self.color_scheme, |hour| self.scheme_at(hour))
    }

    pub fn transparency_reduced(&self) -> bool {
        self.reduce_transparency || self.contrast
    }
}

fn local_time() -> Option<libc::tm> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let now = libc::time_t::try_from(now).ok()?;
    let mut time = std::mem::MaybeUninit::<libc::tm>::uninit();
    // SAFETY: localtime_r writes only into `time`; on a null return `time` is not read.
    let time = unsafe {
        if libc::localtime_r(&now, time.as_mut_ptr()).is_null() {
            return None;
        }
        time.assume_init()
    };
    Some(time)
}

fn local_hour() -> Option<u8> {
    u8::try_from(local_time()?.tm_hour).ok()
}

pub fn local_date() -> Option<(i32, u32, u32)> {
    let time = local_time()?;
    Some((
        time.tm_year + 1900,
        u32::try_from(time.tm_mon + 1).ok()?,
        u32::try_from(time.tm_mday).ok()?,
    ))
}

pub fn config_dir() -> PathBuf {
    std::env::var_os("MALAYSIA_CONFIG_DIR").map_or_else(
        || {
            xdg::config_home()
                .unwrap_or_else(|| PathBuf::from(".config"))
                .join("malaysia")
        },
        PathBuf::from,
    )
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("MALAYSIA_DATA_DIR").map_or_else(
        || {
            xdg::data_home()
                .unwrap_or_else(|| PathBuf::from(".local/share"))
                .join("malaysia")
        },
        PathBuf::from,
    )
}

pub fn appearance_path() -> PathBuf {
    config_dir().join("appearance.domain")
}

pub fn wallpaper_state_path() -> PathBuf {
    data_dir().join("wallpaper.state")
}

pub fn read_appearance() -> Result<Option<AppearanceConfig>, toml::de::Error> {
    read_appearance_from(&appearance_path())
}

pub fn read_appearance_from(path: &Path) -> Result<Option<AppearanceConfig>, toml::de::Error> {
    Ok(read_unresolved(path)?.map(|config| config.following_system(None)))
}

fn read_unresolved(path: &Path) -> Result<Option<AppearanceConfig>, toml::de::Error> {
    static PARSED: Slot<AppearanceConfig> = Mutex::new(None);
    read_parsed(&PARSED, path, |text| {
        warn_legacy_traffic_light_name(text);
        let mut config: AppearanceConfig = toml::from_str(text)?;
        if monochrome_icons(text) {
            config.icon_color.get_or_insert(graphite());
        }
        Ok(config)
    })
}

type Stamp = (u64, SystemTime, u64);

struct Parsed<T> {
    path: PathBuf,
    stamp: Stamp,
    value: T,
}

type Slot<T> = Mutex<Option<Parsed<T>>>;

// A file changed this recently can change again within the same timestamp tick, so it is parsed on every read.
const SETTLED: Duration = Duration::from_secs(1);

// The stamp is taken from the open file, before its text: a change in between leaves a stamp the next read rejects.
fn read_parsed<T: Clone, E>(
    slot: &Slot<T>,
    path: &Path,
    parse: impl FnOnce(&str) -> Result<T, E>,
) -> Result<Option<T>, E> {
    let Ok(mut file) = File::open(path) else {
        return Ok(None);
    };
    let Ok(metadata) = file.metadata() else {
        return Ok(None);
    };
    let modified = metadata.modified().ok();
    let stamp = modified.map(|modified| (metadata.ino(), modified, metadata.len()));
    let mut slot = slot.lock().unwrap_or_else(PoisonError::into_inner);
    if let (Some(stamp), Some(cached)) = (stamp, slot.as_ref())
        && cached.path == path
        && cached.stamp == stamp
    {
        return Ok(Some(cached.value.clone()));
    }
    let mut text = String::new();
    if file.read_to_string(&mut text).is_err() {
        return Ok(None);
    }
    let value = parse(&text)?;
    *slot = stamp
        .filter(|(_, modified, _)| modified.elapsed().is_ok_and(|age| age >= SETTLED))
        .map(|stamp| Parsed {
            path: path.to_path_buf(),
            stamp,
            value: value.clone(),
        });
    Ok(Some(value))
}

const RETIRED_KEYS: [&str; 6] = [
    "blur_all_windows",
    "blur_behind",
    "glass_blur",
    "tint_strength_accent",
    "traffic_light_shape",
    "window_tint_scope",
];

pub fn write_appearance(path: &Path, config: &AppearanceConfig) -> io::Result<()> {
    keep_unparsed::<AppearanceConfig>(path)?;
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let file = toml::from_str::<AppearanceConfig>(&text).ok();
    let mut doc = file
        .as_ref()
        .and_then(|_| text.parse::<DocumentMut>().ok())
        .unwrap_or_default();

    // Reads resolve `color_scheme` to the hour's scheme; writing that echo back would lose Dark Tint.
    let kept_scheme = doc
        .get("color_scheme")
        .cloned()
        .filter(|_| file.is_some_and(|file| file.current_scheme() == config.color_scheme));
    let fresh = toml_edit::ser::to_document(config).map_err(std::io::Error::other)?;
    for (key, item) in fresh.iter() {
        doc[key] = item.clone();
    }
    if let Some(scheme) = kept_scheme {
        doc["color_scheme"] = scheme;
    }
    // A key that no field reads any more would stay in the file forever.
    for retired in RETIRED_KEYS {
        doc.remove(retired);
    }
    match &config.accent {
        Some(name) => doc["accent"] = toml_edit::value(name.as_str()),
        None => {
            doc.remove("accent");
        }
    }
    for (key, color) in [
        ("accent_color", config.accent_color),
        ("icon_color", config.icon_color),
    ] {
        match color {
            Some(color) => doc[key] = toml_edit::value(accent_array(color)),
            None => {
                doc.remove(key);
            }
        }
    }

    replace_file(path, &doc.to_string())
}

// XDG Base Directory Specification: a missing directory is created with permission 0700.
pub(crate) fn create_private_dir(dir: &Path) -> io::Result<()> {
    std::os::unix::fs::DirBuilderExt::mode(std::fs::DirBuilder::new().recursive(true), 0o700)
        .create(dir)
}

pub fn replace_file(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        create_private_dir(parent)?;
    }
    static TEMPORARY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(
        ".{}.{}.tmp",
        std::process::id(),
        TEMPORARY.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let written = std::fs::File::create(&tmp)
        .and_then(|mut file| {
            std::io::Write::write_all(&mut file, text.as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| std::fs::rename(&tmp, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    written
}

fn serialize_tint_parts<S: Serializer>(
    parts: &TintParts,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let named = [
        ("sidebars", parts.sidebars),
        ("window", parts.window),
        ("groups", parts.groups),
        ("controls", parts.controls),
        ("menus", parts.menus),
        ("bars", parts.bars),
    ];
    serializer.collect_seq(
        named
            .into_iter()
            .filter(|(_, on)| *on)
            .map(|(name, _)| name),
    )
}

fn accent_array(accent: [f32; 3]) -> Array {
    accent
        .into_iter()
        .map(|channel| (f64::from(channel.clamp(0.0, 1.0)) * 1000.0).round() / 1000.0)
        .collect()
}

fn value_of<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        let (name, value) = line.split('#').next()?.split_once('=')?;
        (name.trim() == key).then(|| value.trim().trim_matches('"'))
    })
}

// The retired monochrome style is the tinted one in graphite.
fn monochrome_icons(text: &str) -> bool {
    value_of(text, "icon_style") == Some("monochrome")
}

fn graphite() -> [f32; 3] {
    let color = accent::graphite().light.control;
    [color.r, color.g, color.b]
}

fn warn_legacy_traffic_light_name(text: &str) {
    static WARNED: std::sync::Once = std::sync::Once::new();
    if value_of(text, "traffic_lights") == Some("macos") {
        WARNED.call_once(|| {
            tracing::warn!(
                "traffic_lights = \"macos\" is deprecated; the style is named \"standard\""
            );
        });
    }
}

pub fn load_appearance() -> AppearanceConfig {
    let config = match read_unresolved(&appearance_path()) {
        Ok(config) => config.unwrap_or_default(),
        Err(error) => {
            tracing::warn!(%error, "invalid appearance domain; using defaults");
            AppearanceConfig::default()
        }
    };
    config.following_system(system_scheme())
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WallpaperOutput {
    pub name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct WallpaperState {
    pub dominant: Option<[f32; 3]>,
    pub grid: Option<WallpaperGrid>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub grids: BTreeMap<String, WallpaperGrid>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<WallpaperOutput>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub shown: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shuffled_at: Option<u64>,
}

impl WallpaperState {
    pub fn grid_for(&self, desktop: Option<u32>) -> Option<&WallpaperGrid> {
        desktop
            .and_then(|number| self.grids.get(&number.to_string()))
            .or(self.grid.as_ref())
    }
}

pub fn load_wallpaper_state() -> WallpaperState {
    read_wallpaper_state(&wallpaper_state_path()).unwrap_or_default()
}

pub fn load_dominant() -> Option<Rgba> {
    let [r, g, b] = read_wallpaper_state(&wallpaper_state_path())?.dominant?;
    Some(Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: 1.0,
    })
}

// A file that exists but cannot be read is never replaced; one that does not parse is copied to `<name>.bak` first.
fn keep_unparsed<T: DeserializeOwned>(path: &Path) -> io::Result<()> {
    match std::fs::read_to_string(path) {
        Ok(text) if toml::from_str::<T>(&text).is_err() => {
            let mut backup = path.as_os_str().to_owned();
            backup.push(".bak");
            std::fs::copy(path, &backup)?;
            tracing::warn!(
                path = %path.display(),
                backup = %Path::new(&backup).display(),
                "unreadable file backed up before overwrite"
            );
        }
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    Ok(())
}

pub(crate) fn write_toml<T: Serialize + DeserializeOwned>(
    path: &Path,
    value: &T,
) -> io::Result<()> {
    keep_unparsed::<T>(path)?;
    replace_file(
        path,
        &toml::to_string(value).map_err(std::io::Error::other)?,
    )
}

pub fn write_wallpaper_state(path: &Path, state: &WallpaperState) -> io::Result<()> {
    if *state == WallpaperState::default() {
        keep_unparsed::<WallpaperState>(path)?;
        return match std::fs::remove_file(path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    }
    write_toml(path, state)
}

fn read_wallpaper_state(path: &Path) -> Option<WallpaperState> {
    static PARSED: Slot<WallpaperState> = Mutex::new(None);
    read_parsed(&PARSED, path, |text| toml::from_str(text))
        .ok()
        .flatten()
}
