use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use vow_design::{Rgba, TINT_STRENGTH, TITLEBAR};

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
    Wallpaper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowTintScope {
    #[default]
    Windows,
    Desktop,
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
}

impl Default for TintParts {
    fn default() -> Self {
        Self {
            sidebars: true,
            window: true,
            groups: true,
            controls: true,
            menus: true,
        }
    }
}

fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
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
    };
    for name in list {
        match name.as_str() {
            "sidebars" => parts.sidebars = true,
            "window" => parts.window = true,
            "groups" => parts.groups = true,
            "controls" => parts.controls = true,
            "menus" => parts.menus = true,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TrafficLightShape {
    #[default]
    Square,
    Round,
}

impl TrafficLightShape {
    pub fn corner_radius(self, diameter: f32) -> f32 {
        match self {
            Self::Square => {
                diameter * TITLEBAR.buttons.corner_radius as f32 / TITLEBAR.buttons.diameter as f32
            }
            Self::Round => diameter / 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrafficLightStyle {
    #[default]
    Auto,
    #[serde(alias = "macos")]
    Standard,
    Accent,
    Wallpaper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollBars {
    #[default]
    WhenScrolling,
    Always,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub reduce_transparency: bool,
    #[serde(deserialize_with = "lenient")]
    pub window_color: WindowColorMode,
    #[serde(deserialize_with = "lenient")]
    pub window_tint_scope: WindowTintScope,
    #[serde(deserialize_with = "lenient")]
    pub traffic_lights: TrafficLightStyle,
    #[serde(deserialize_with = "lenient")]
    pub traffic_light_shape: TrafficLightShape,
    #[serde(deserialize_with = "lenient")]
    pub sidebar_style: SidebarStyle,
    #[serde(deserialize_with = "lenient")]
    pub tint_strength_accent: TintStrength,
    #[serde(deserialize_with = "lenient")]
    pub tint_strength_wallpaper: TintStrength,
    #[serde(
        deserialize_with = "deserialize_tint_parts",
        serialize_with = "serialize_tint_parts"
    )]
    pub tint_parts: TintParts,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default(),
            accent_color: None,
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::default(),
            reduce_transparency: true,
            window_color: WindowColorMode::default(),
            window_tint_scope: WindowTintScope::default(),
            traffic_lights: TrafficLightStyle::default(),
            traffic_light_shape: TrafficLightShape::default(),
            sidebar_style: SidebarStyle::default(),
            tint_strength_accent: TintStrength::default(),
            tint_strength_wallpaper: TintStrength::default(),
            tint_parts: TintParts::default(),
        }
    }
}

impl AppearanceConfig {
    pub fn traffic_light_style(&self) -> TrafficLightStyle {
        match self.traffic_lights {
            TrafficLightStyle::Auto => match self.window_color {
                WindowColorMode::Neutral => TrafficLightStyle::Standard,
                WindowColorMode::Accent => TrafficLightStyle::Accent,
                WindowColorMode::Wallpaper => TrafficLightStyle::Wallpaper,
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

pub fn write_appearance(path: &Path, config: &AppearanceConfig) -> std::io::Result<()> {
    let parsed = match std::fs::read_to_string(path) {
        Ok(text) => text.parse::<toml_edit::DocumentMut>().ok(),
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
    if let Some(accent) = config.accent_color {
        doc["accent_color"] = toml_edit::value(accent_array(accent));
    } else {
        doc.remove("accent_color");
    }

    replace_file(path, &doc.to_string())
}

fn replace_file(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
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
    read_appearance().ok().flatten().unwrap_or_default()
}

#[derive(Debug, Deserialize)]
struct WallpaperState {
    dominant: Option<[f32; 3]>,
}

pub fn load_dominant() -> Option<Rgba> {
    read_dominant_from(&wallpaper_state_path())
}

pub fn write_dominant(path: &Path, dominant: Option<Rgba>) -> std::io::Result<()> {
    match dominant {
        Some(color) => replace_file(
            path,
            &format!(
                "dominant = [{:.6}, {:.6}, {:.6}]\n",
                color.r, color.g, color.b
            ),
        ),
        None => match std::fs::remove_file(path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        },
    }
}

fn read_dominant_from(path: &Path) -> Option<Rgba> {
    let text = std::fs::read_to_string(path).ok()?;
    let state: WallpaperState = toml::from_str(&text).ok()?;
    let [r, g, b] = state.dominant?;
    Some(Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: 1.0,
    })
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
    fn reduce_transparency_defaults_to_true() {
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
        assert!(without_key.reduce_transparency);

        let with_false: AppearanceConfig = toml::from_str("reduce_transparency = false\n").unwrap();
        assert!(!with_false.reduce_transparency);

        assert!(AppearanceConfig::default().reduce_transparency);
    }

    fn assert_configs_equal(a: &AppearanceConfig, b: &AppearanceConfig) {
        assert_eq!(a.color_scheme, b.color_scheme);
        assert_eq!(a.accent_color, b.accent_color);
        assert_eq!(a.contrast, b.contrast);
        assert_eq!(a.reduced_motion, b.reduced_motion);
        assert_eq!(a.reduce_transparency, b.reduce_transparency);
        assert_eq!(a.window_color, b.window_color);
        assert_eq!(a.window_tint_scope, b.window_tint_scope);
        assert_eq!(a.traffic_lights, b.traffic_lights);
        assert_eq!(a.traffic_light_shape, b.traffic_light_shape);
        assert_eq!(a.sidebar_style, b.sidebar_style);
        assert_eq!(a.tint_strength_accent, b.tint_strength_accent);
        assert_eq!(a.tint_strength_wallpaper, b.tint_strength_wallpaper);
        assert_eq!(a.tint_parts, b.tint_parts);
    }

    #[test]
    fn write_appearance_roundtrips() {
        let dir = std::env::temp_dir().join(format!("vow-appearance-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        let config = AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            accent_color: Some([0.184, 0.435, 0.894]),
            contrast: false,
            reduced_motion: false,
            scroll_bars: ScrollBars::WhenScrolling,
            reduce_transparency: true,
            window_color: WindowColorMode::Accent,
            window_tint_scope: WindowTintScope::Desktop,
            traffic_lights: TrafficLightStyle::Accent,
            traffic_light_shape: TrafficLightShape::Round,
            sidebar_style: SidebarStyle::Flat,
            tint_strength_accent: TintStrength::Strong,
            tint_strength_wallpaper: TintStrength::Subtle,
            tint_parts: TintParts {
                sidebars: true,
                window: false,
                groups: true,
                controls: false,
                menus: true,
            },
        };
        write_appearance(&path, &config).unwrap();
        let read = read_appearance_from(&path).unwrap().unwrap();
        assert_configs_equal(&config, &read);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_preserves_unknown_keys_and_comments() {
        let dir =
            std::env::temp_dir().join(format!("vow-appearance-preserve-{}", std::process::id()));
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
        let dir =
            std::env::temp_dir().join(format!("vow-appearance-backup-{}", std::process::id()));
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
    fn write_appearance_does_not_back_up_a_readable_domain() {
        let dir =
            std::env::temp_dir().join(format!("vow-appearance-nobackup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        std::fs::write(&path, "color_scheme = \"light\"\n").unwrap();

        write_appearance(&path, &AppearanceConfig::default()).unwrap();

        assert!(!dir.join("appearance.domain.bak").exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_appearance_creates_missing_file_and_leaves_no_temp() {
        let dir =
            std::env::temp_dir().join(format!("vow-appearance-create-{}", std::process::id()));
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
        assert_eq!(config.tint_strength_accent, TintStrength::Normal);
        assert_eq!(config.tint_strength_wallpaper, TintStrength::Normal);
        assert_eq!(config.tint_parts, TintParts::default());
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
            }
        );
    }

    #[test]
    fn write_dominant_records_and_clears_the_wallpaper_state() {
        let dir = std::env::temp_dir().join(format!("vow-appearance-state-{}", std::process::id()));
        let path = dir.join("wallpaper.state");
        let color = Rgba::rgb8(51, 102, 153);
        write_dominant(&path, Some(color)).unwrap();
        let read = read_dominant_from(&path).unwrap();
        assert!((read.r - color.r).abs() < 1e-6 && (read.b - color.b).abs() < 1e-6);
        write_dominant(&path, None).unwrap();
        assert!(!path.exists());
        write_dominant(&path, None).unwrap();
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn tint_parts_roundtrip_through_write_appearance() {
        let dir = std::env::temp_dir().join(format!(
            "vow-appearance-parts-roundtrip-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("appearance.domain");
        let config = AppearanceConfig {
            tint_strength_accent: TintStrength::Intense,
            tint_strength_wallpaper: TintStrength::Subtle,
            tint_parts: TintParts {
                sidebars: true,
                window: false,
                groups: true,
                controls: false,
                menus: true,
            },
            ..AppearanceConfig::default()
        };
        write_appearance(&path, &config).unwrap();
        let read = read_appearance_from(&path).unwrap().unwrap();
        assert_configs_equal(&config, &read);
        std::fs::remove_dir_all(&dir).ok();
    }
}
