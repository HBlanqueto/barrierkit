use std::path::{Path, PathBuf};

use ctdesign::{
    COLOR_FILTER_INTENSITY, DISPLAY_CONTRAST, POINTER_SIZES, Rgba, TEXT_SIZES, TINT_STRENGTH,
    TITLEBAR,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::tinting::WallpaperGrid;
use crate::xdg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    #[default]
    Auto,
    Dark,
    #[serde(rename = "dark_tint")]
    DarkTint,
    Light,
}

impl ColorScheme {
    pub fn resolved(self) -> Self {
        match self {
            Self::Auto => Self::Light,
            scheme => scheme,
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark | Self::DarkTint)
    }

    pub(crate) fn pick<T>(self, [light, dark, dark_tint]: [T; 3]) -> T {
        match self {
            Self::Auto | Self::Light => light,
            Self::Dark => dark,
            Self::DarkTint => dark_tint,
        }
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
    T: serde::de::DeserializeOwned + Default,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).unwrap_or_else(|error| {
        tracing::warn!(%error, "invalid appearance value; using its default");
        T::default()
    }))
}

fn stepped<'de, D: Deserializer<'de>>(deserializer: D, steps: &[f32]) -> Result<f32, D::Error> {
    let value = f32::deserialize(deserializer)?;
    Ok(value.max(steps[0]).min(steps[steps.len() - 1]))
}

fn display_contrast<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    stepped(deserializer, &DISPLAY_CONTRAST)
}

fn text_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    stepped(deserializer, &TEXT_SIZES)
}

fn toolkit_text_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f32>, D::Error> {
    text_size(deserializer).map(Some)
}

fn pointer_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    stepped(deserializer, &POINTER_SIZES)
}

fn color_filter_intensity<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    stepped(deserializer, &COLOR_FILTER_INTENSITY)
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
#[serde(rename_all = "lowercase")]
pub enum SidebarStyle {
    #[default]
    Island,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct TrafficLightShape(u32);

impl TrafficLightShape {
    pub const ROUND: Self = Self(TITLEBAR.buttons.diameter / 2);

    pub fn new(radius: u32) -> Self {
        Self(radius.min(Self::ROUND.0))
    }

    pub fn radius(self) -> u32 {
        self.0
    }

    pub fn corner_radius(self, diameter: f32) -> f32 {
        diameter * self.0 as f32 / TITLEBAR.buttons.diameter as f32
    }
}

impl Default for TrafficLightShape {
    fn default() -> Self {
        Self(TITLEBAR.buttons.corner_radius)
    }
}

impl<'de> Deserialize<'de> for TrafficLightShape {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Stored {
            Radius(u32),
            Named(String),
        }
        Ok(match Stored::deserialize(deserializer)? {
            Stored::Radius(radius) => Self::new(radius),
            Stored::Named(name) if name == "round" => Self::ROUND,
            Stored::Named(_) => Self::default(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrafficLightStyle {
    #[default]
    Auto,
    #[serde(alias = "macos")]
    Standard,
    #[serde(alias = "wallpaper")]
    Accent,
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
pub enum BlurBehind {
    #[default]
    Mixed,
    Wallpaper,
    Windows,
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
    #[serde(skip_serializing)]
    pub accent_color: Option<[f32; 3]>,
    pub contrast: bool,
    pub reduced_motion: bool,
    #[serde(deserialize_with = "lenient")]
    pub scroll_bars: ScrollBars,
    #[serde(deserialize_with = "lenient")]
    pub scroll_bar_click: ScrollBarClick,
    pub reduce_transparency: bool,
    #[serde(deserialize_with = "lenient")]
    pub window_color: WindowColorMode,
    #[serde(deserialize_with = "lenient")]
    pub traffic_lights: TrafficLightStyle,
    #[serde(deserialize_with = "lenient")]
    pub traffic_light_shape: TrafficLightShape,
    #[serde(deserialize_with = "lenient")]
    pub sidebar_style: SidebarStyle,
    #[serde(deserialize_with = "lenient")]
    pub tint_strength: TintStrength,
    #[serde(
        deserialize_with = "deserialize_tint_parts",
        serialize_with = "serialize_tint_parts"
    )]
    pub tint_parts: TintParts,
    pub wallpaper_tinting: bool,
    pub wallpaper_tinting_in_light: bool,
    pub invert_colors: bool,
    pub show_borders: bool,
    pub differentiate_without_color: bool,
    #[serde(deserialize_with = "display_contrast")]
    pub display_contrast: f32,
    #[serde(deserialize_with = "text_size")]
    pub text_size: f32,
    pub text_sizes: TextSizes,
    #[serde(deserialize_with = "lenient")]
    pub menu_bar_size: MenuBarSize,
    pub shake_to_locate: bool,
    #[serde(deserialize_with = "pointer_size")]
    pub pointer_size: f32,
    pub color_filters: bool,
    #[serde(deserialize_with = "lenient")]
    pub color_filter: ColorFilter,
    #[serde(deserialize_with = "color_filter_intensity")]
    pub color_filter_intensity: f32,
    #[serde(deserialize_with = "lenient")]
    pub blur_behind: BlurBehind,
    pub blur_all_windows: bool,
    pub gtk_bridge: bool,
    pub qt_bridge: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default(),
            accent_color: None,
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::default(),
            scroll_bar_click: ScrollBarClick::default(),
            reduce_transparency: false,
            window_color: WindowColorMode::default(),
            traffic_lights: TrafficLightStyle::default(),
            traffic_light_shape: TrafficLightShape::default(),
            sidebar_style: SidebarStyle::default(),
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
            shake_to_locate: true,
            pointer_size: POINTER_SIZES[0],
            color_filters: false,
            color_filter: ColorFilter::default(),
            color_filter_intensity: COLOR_FILTER_INTENSITY[COLOR_FILTER_INTENSITY.len() - 1],
            blur_behind: BlurBehind::default(),
            blur_all_windows: false,
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

    pub fn transparency_reduced(&self) -> bool {
        self.reduce_transparency || self.contrast
    }

    pub fn traffic_light_style(&self) -> TrafficLightStyle {
        match self.traffic_lights {
            TrafficLightStyle::Auto => match self.window_color {
                WindowColorMode::Neutral => TrafficLightStyle::Standard,
                WindowColorMode::Accent => TrafficLightStyle::Accent,
            },
            style => style,
        }
    }
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

fn read_appearance_from(path: &Path) -> Result<Option<AppearanceConfig>, toml::de::Error> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            warn_legacy_traffic_light_name(&text);
            toml::from_str(&text).map(Some)
        }
        Err(_) => Ok(None),
    }
}

const RETIRED_KEYS: [&str; 3] = [
    "tint_strength_accent",
    "wallpaper_accent",
    "window_tint_scope",
];

pub fn write_appearance(path: &Path, config: &AppearanceConfig) -> std::io::Result<()> {
    let parsed = match std::fs::read_to_string(path) {
        Ok(text) => text
            .parse::<toml_edit::DocumentMut>()
            .ok()
            .filter(|_| toml::from_str::<AppearanceConfig>(&text).is_ok()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Some(toml_edit::DocumentMut::new())
        }
        Err(_) => None,
    };
    let mut doc = match parsed {
        Some(doc) => doc,
        None => {
            let backup = path.with_extension("domain.bak");
            std::fs::copy(path, &backup)?;
            tracing::warn!(
                path = %path.display(),
                backup = %backup.display(),
                "unreadable appearance domain backed up before overwrite"
            );
            toml_edit::DocumentMut::new()
        }
    };

    let fresh = toml_edit::ser::to_document(config).map_err(std::io::Error::other)?;
    for (key, item) in fresh.iter() {
        doc[key] = item.clone();
    }
    // A key that no field reads any more would stay in the file forever.
    for retired in RETIRED_KEYS {
        doc.remove(retired);
    }
    if let Some(accent) = config.accent_color {
        doc["accent_color"] = toml_edit::value(accent_array(accent));
    } else {
        doc.remove("accent_color");
    }

    replace_file(path, &doc.to_string())
}

// XDG Base Directory Specification: a missing directory is created with permission 0700.
pub(crate) fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    std::os::unix::fs::DirBuilderExt::mode(std::fs::DirBuilder::new().recursive(true), 0o700)
        .create(dir)
}

pub fn replace_file(path: &Path, text: &str) -> std::io::Result<()> {
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

fn accent_array(accent: [f32; 3]) -> toml_edit::Array {
    let mut arr = toml_edit::Array::new();
    for channel in accent {
        arr.push((f64::from(channel.clamp(0.0, 1.0)) * 1000.0).round() / 1000.0);
    }
    arr
}

fn warn_legacy_traffic_light_name(text: &str) {
    static WARNED: std::sync::Once = std::sync::Once::new();
    let legacy = text.lines().any(|line| {
        let line = line.split('#').next().unwrap_or(line).trim();
        line.starts_with("traffic_lights")
            && line
                .split_once('=')
                .map(|(_, value)| value.trim().trim_matches('"') == "macos")
                .unwrap_or(false)
    });
    if legacy {
        WARNED.call_once(|| {
            tracing::warn!(
                "traffic_lights = \"macos\" is deprecated; the style is named \"standard\""
            );
        });
    }
}

pub fn load_appearance() -> AppearanceConfig {
    match read_appearance() {
        Ok(config) => config.unwrap_or_default(),
        Err(error) => {
            tracing::warn!(%error, "invalid appearance domain; using defaults");
            AppearanceConfig::default()
        }
    }
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
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub grids: std::collections::BTreeMap<String, WallpaperGrid>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<WallpaperOutput>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub shown: std::collections::BTreeMap<String, String>,
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

pub fn write_wallpaper_state(path: &Path, state: &WallpaperState) -> std::io::Result<()> {
    if *state == WallpaperState::default() {
        return match std::fs::remove_file(path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    }
    replace_file(
        path,
        &toml::to_string(state).map_err(std::io::Error::other)?,
    )
}

fn read_wallpaper_state(path: &Path) -> Option<WallpaperState> {
    toml::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_legacy_style_name_deserializes_to_standard() {
        let config: AppearanceConfig = toml::from_str("traffic_lights = \"macos\"\n").unwrap();
        assert_eq!(config.traffic_lights, TrafficLightStyle::Standard);
        let config: AppearanceConfig = toml::from_str("traffic_lights = \"standard\"\n").unwrap();
        assert_eq!(config.traffic_lights, TrafficLightStyle::Standard);
    }

    #[test]
    fn reduce_transparency_defaults_to_false() {
        let without_key: AppearanceConfig = toml::from_str(
            r#"
color_scheme = "dark"
accent_color = [0.2, 0.5, 0.8]
contrast = false
reduced_motion = false
window_color = "neutral"
window_tint_scope = "windows"
traffic_lights = "auto"
"#,
        )
        .unwrap();
        assert!(!without_key.reduce_transparency);

        let with_true: AppearanceConfig = toml::from_str("reduce_transparency = true\n").unwrap();
        assert!(with_true.reduce_transparency);

        assert!(!AppearanceConfig::default().reduce_transparency);
    }

    #[test]
    fn a_toolkit_without_its_own_text_size_uses_the_preferred_one() {
        let config: AppearanceConfig =
            toml::from_str("text_size = 1.3\n[text_sizes]\nqt = 2.0\n").unwrap();
        assert_eq!(config.text_size_for(Toolkit::Gtk), 1.3);
        assert_eq!(config.text_size_for(Toolkit::BarrierKit), 1.3);
        assert_eq!(config.text_size_for(Toolkit::Qt), 2.0);
    }

    #[test]
    fn write_appearance_roundtrips() {
        let dir = std::env::temp_dir().join(format!("ctappearance-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            accent_color: Some([0.184, 0.435, 0.894]),
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::WhenScrolling,
            scroll_bar_click: ScrollBarClick::ClickedSpot,
            reduce_transparency: true,
            window_color: WindowColorMode::Accent,
            traffic_lights: TrafficLightStyle::Accent,
            traffic_light_shape: TrafficLightShape::new(2),
            sidebar_style: SidebarStyle::Flat,
            tint_strength: TintStrength::Strong,
            tint_parts: TintParts {
                sidebars: true,
                window: false,
                groups: true,
                controls: false,
                menus: true,
                bars: false,
            },
            wallpaper_tinting: false,
            wallpaper_tinting_in_light: true,
            invert_colors: true,
            show_borders: true,
            differentiate_without_color: true,
            display_contrast: DISPLAY_CONTRAST[2],
            text_size: TEXT_SIZES[3],
            text_sizes: TextSizes {
                barrierkit: None,
                gtk: Some(TEXT_SIZES[1]),
                qt: Some(TEXT_SIZES[5]),
            },
            menu_bar_size: MenuBarSize::Large,
            shake_to_locate: false,
            pointer_size: POINTER_SIZES[4],
            color_filters: true,
            color_filter: ColorFilter::BlueYellow,
            color_filter_intensity: COLOR_FILTER_INTENSITY[1],
            blur_behind: BlurBehind::Windows,
            blur_all_windows: true,
            gtk_bridge: false,
            qt_bridge: false,
        };
        write_appearance(&path, &config).unwrap();
        let read = read_appearance_from(&path).unwrap().unwrap();
        assert_eq!(config, read);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_preserves_unknown_keys_and_comments() {
        let dir =
            std::env::temp_dir().join(format!("ctappearance-preserve-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        std::fs::write(
            &path,
            "# legacy comment preserved\nlegacy_key = true\ncolor_scheme = \"light\"\n",
        )
        .unwrap();
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            accent_color: Some([0.184, 0.435, 0.894]),
            ..AppearanceConfig::default()
        };
        write_appearance(&path, &config).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("accent_color = [0.184, 0.435, 0.894]"));
        assert!(text.contains("legacy comment preserved"));
        assert!(text.contains("legacy_key = true"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_backs_up_an_unreadable_domain() {
        let dir = std::env::temp_dir().join(format!("ctappearance-backup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        let original = "color_scheme = ";
        std::fs::write(&path, original).unwrap();
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            ..AppearanceConfig::default()
        };

        write_appearance(&path, &config).unwrap();

        let backup = dir.join("appearance.domain.bak");
        assert_eq!(std::fs::read(&backup).unwrap(), original.as_bytes());
        let read = read_appearance_from(&path).unwrap().unwrap();
        assert_eq!(read.color_scheme, ColorScheme::Dark);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_backs_up_a_domain_with_a_wrongly_typed_value() {
        let dir =
            std::env::temp_dir().join(format!("ctappearance-mistyped-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        let original = "contrast = true\ntext_size = \"1.15\"\n";
        std::fs::write(&path, original).unwrap();

        write_appearance(&path, &AppearanceConfig::default()).unwrap();

        let backup = dir.join("appearance.domain.bak");
        assert_eq!(std::fs::read(&backup).unwrap(), original.as_bytes());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_size_outside_its_steps_stops_at_the_nearest_one_and_nan_at_the_first() {
        let config: AppearanceConfig = toml::from_str(
            "display_contrast = 0.0\ntext_size = nan\npointer_size = 9.0\n\
             color_filter_intensity = -inf\n[text_sizes]\ngtk = 40.0\n",
        )
        .unwrap();
        assert_eq!(config.display_contrast, DISPLAY_CONTRAST[0]);
        assert_eq!(config.text_size, TEXT_SIZES[0]);
        assert_eq!(config.pointer_size, POINTER_SIZES[6]);
        assert_eq!(config.color_filter_intensity, COLOR_FILTER_INTENSITY[0]);
        assert_eq!(config.text_sizes.gtk, Some(TEXT_SIZES[5]));
        assert_eq!(config.text_sizes.qt, None);
    }

    #[test]
    fn write_appearance_does_not_back_up_a_readable_domain() {
        let dir =
            std::env::temp_dir().join(format!("ctappearance-nobackup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        std::fs::write(&path, "color_scheme = \"light\"\n").unwrap();

        write_appearance(&path, &AppearanceConfig::default()).unwrap();

        assert!(!dir.join("appearance.domain.bak").exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_creates_missing_file_and_leaves_no_temp() {
        let dir = std::env::temp_dir().join(format!("ctappearance-create-{}", std::process::id()));
        let path = dir.join("appearance.domain");
        assert!(!path.exists());
        write_appearance(&path, &AppearanceConfig::default()).unwrap();
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        read_appearance_from(&path)
            .unwrap()
            .expect("created file parses");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn old_domain_without_new_fields_uses_defaults() {
        let config: AppearanceConfig = toml::from_str(
            r#"
color_scheme = "light"
accent_color = [0.2, 0.5, 0.8]
contrast = false
reduced_motion = false
window_color = "accent"
window_tint_scope = "windows"
traffic_lights = "auto"
sidebar_style = "island"
"#,
        )
        .unwrap();
        assert_eq!(config.tint_strength, TintStrength::Normal);
        assert_eq!(config.tint_parts, TintParts::default());
        assert!(config.wallpaper_tinting);
        assert!(!config.wallpaper_tinting_in_light);
    }

    #[test]
    fn the_old_wallpaper_window_colour_reads_as_neutral() {
        let config: AppearanceConfig =
            toml::from_str("window_color = \"wallpaper\"\ntraffic_lights = \"accent\"\n").unwrap();
        assert_eq!(config.window_color, WindowColorMode::Neutral);
        assert_eq!(config.traffic_lights, TrafficLightStyle::Accent);
        assert!(config.wallpaper_tinting);
    }

    #[test]
    fn an_unknown_value_falls_back_only_for_its_field() {
        let config: AppearanceConfig = toml::from_str(
            r#"
color_scheme = "dark"
sidebar_style = "bubbly"
tint_parts = ["window", "halo"]
"#,
        )
        .unwrap();
        assert_eq!(config.color_scheme, ColorScheme::Dark);
        assert_eq!(config.sidebar_style, SidebarStyle::Island);
        assert_eq!(
            config.tint_parts,
            TintParts {
                sidebars: false,
                window: true,
                groups: false,
                controls: false,
                menus: false,
                bars: false,
            }
        );
    }

    #[test]
    fn write_wallpaper_state_records_and_clears_the_state() {
        let dir = std::env::temp_dir().join(format!("ctappearance-state-{}", std::process::id()));
        let path = dir.join("wallpaper.state");
        let state = WallpaperState {
            dominant: Some([0.2, 0.4, 0.6]),
            grid: Some(WallpaperGrid {
                aspect: 1.6,
                rows: vec![vec![[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]]],
            }),
            ..WallpaperState::default()
        };
        write_wallpaper_state(&path, &state).unwrap();
        assert_eq!(read_wallpaper_state(&path), Some(state));
        write_wallpaper_state(&path, &WallpaperState::default()).unwrap();
        assert!(!path.exists());
        write_wallpaper_state(&path, &WallpaperState::default()).unwrap();
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_named_traffic_light_shapes_read_as_their_radius_and_a_radius_stops_at_round() {
        let read = |text: &str| toml::from_str::<AppearanceConfig>(text).unwrap();
        assert_eq!(
            read("traffic_light_shape = \"square\"\n").traffic_light_shape,
            TrafficLightShape::default()
        );
        assert_eq!(
            read("traffic_light_shape = \"round\"\n").traffic_light_shape,
            TrafficLightShape::ROUND
        );
        assert_eq!(
            read("traffic_light_shape = 0\n")
                .traffic_light_shape
                .radius(),
            0
        );
        assert_eq!(
            read("traffic_light_shape = 99\n").traffic_light_shape,
            TrafficLightShape::ROUND
        );
    }

    #[test]
    fn a_retired_key_next_to_its_replacement_still_parses_and_a_write_removes_it() {
        let text = "tint_strength_accent = \"intense\"\ntint_strength = \"strong\"\ntraffic_lights = \"wallpaper\"\n";
        let config: AppearanceConfig = toml::from_str(text).unwrap();
        assert_eq!(config.tint_strength, TintStrength::Strong);
        assert_eq!(config.traffic_lights, TrafficLightStyle::Accent);

        let dir = std::env::temp_dir().join(format!("ctappearance-retired-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        std::fs::write(&path, text).unwrap();
        write_appearance(&path, &config).unwrap();
        assert!(
            !std::fs::read_to_string(&path)
                .unwrap()
                .contains("tint_strength_accent")
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn tint_parts_roundtrip_through_write_appearance() {
        let dir = std::env::temp_dir().join(format!(
            "ctappearance-parts-roundtrip-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        let config = AppearanceConfig {
            tint_strength: TintStrength::Intense,
            tint_parts: TintParts {
                sidebars: true,
                window: false,
                groups: true,
                controls: false,
                menus: true,
                bars: false,
            },
            ..AppearanceConfig::default()
        };
        write_appearance(&path, &config).unwrap();
        let read = read_appearance_from(&path).unwrap().unwrap();
        assert_eq!(config, read);
        std::fs::remove_dir_all(&dir).ok();
    }
}
