//! Settings of the Xaero's Minimap Rust rewrite
//! (mirrors `xaero/common/settings/ModSettings` and
//! `xaero/common/config/LegacyCommonConfigIO` of the original mod).
//!
//! Settings are a typed map of string keys to values, persisted by
//! [`LegacyCommonConfigIO`] in a `key=value` line format, like the legacy
//! config files the Java mod writes.
//!
//! The `categories` module ports the newer category/rule/settings
//! system of `xaero/hud/category`.

pub mod builtin_info_displays;
pub mod categories;
pub mod config_ui;
pub mod info_display;
pub mod info_display_codec;
pub mod info_display_io;
pub mod radar_categories;

pub use builtin_info_displays::{
    adapt_display_order, for_each as for_each_builtin_info_display, BuiltInInfoDisplay,
    BUILTIN_INFO_DISPLAYS, InfoDisplayCodec, InfoDisplayWidgetKind,
};
pub use categories::{
    EditorSettingType, ExcludeListMode, FilterObjectCategory, FilterObjectCategoryBuilder,
    FilterObjectCategoryData,
    HardRulePredicate, ObjectCategory, ObjectCategoryBuilder, ObjectCategoryData,
    ObjectCategoryDataBuilder, ObjectCategoryHardRule, HardRuleBuilder, ObjectCategoryListRule,
    ObjectCategoryListRuleType, ObjectCategoryRule, ObjectCategoryRuleResolver,
    ObjectCategorySetting, ObjectCategorySettingBuilder,
};
pub use config_ui::{
    ConfigUiBinding, ConfigUiType, ConfigUiTypeManager, UI_BINDINGS,
};
pub use info_display::{
    InfoDisplayConfigData, InfoDisplayManagerConfigData, InfoDisplayManagerConfigDataBuilder,
};
pub use info_display_codec::InfoDisplayState;
pub use info_display_io::{decode as decode_info_display, encode as encode_info_display};
pub use radar_categories::{
    all_settings, default_settings, setting_by_id, RadarCategory, RadarCategoryData,
    RadarCategoryManager, RadarEntityInfo, RadarHardRule, RadarListRuleType, RadarSetting,
};

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::PathBuf;

/// A setting value, as stored by the legacy config files.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingValue {
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Str(String),
}

/// The neutral default for a setting is an empty string: builders start from
/// it and replace it with the real default value when one is supplied.
impl Default for SettingValue {
    fn default() -> Self {
        SettingValue::Str(String::new())
    }
}

impl SettingValue {
    /// Serialized form used in the config files (`true`, `5`, `3.5`, `text`).
    pub fn to_text(&self) -> String {
        match self {
            SettingValue::Bool(b) => b.to_string(),
            SettingValue::Int(i) => i.to_string(),
            SettingValue::Long(l) => l.to_string(),
            SettingValue::Float(f) => format!("{f}"),
            SettingValue::Str(s) => s.clone(),
        }
    }

    /// Parses a serialized value: booleans, integers, floats and the rest as
    /// strings.
    pub fn parse(text: &str) -> SettingValue {
        match text {
            "true" => SettingValue::Bool(true),
            "false" => SettingValue::Bool(false),
            _ => match text.parse::<i32>() {
                Ok(i) => SettingValue::Int(i),
                Err(_) => match text.parse::<i64>() {
                    Ok(l) => SettingValue::Long(l),
                    Err(_) => match text.parse::<f32>() {
                        Ok(f) => SettingValue::Float(f),
                        Err(_) => SettingValue::Str(text.to_string()),
                    },
                },
            },
        }
    }

    /// The value as `bool` if it is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SettingValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// The value as `f32`, converting the numeric kinds.
    pub fn as_float(&self) -> Option<f32> {
        match self {
            SettingValue::Float(f) => Some(*f),
            SettingValue::Int(i) => Some(*i as f32),
            SettingValue::Long(l) => Some(*l as f32),
            SettingValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            SettingValue::Str(_) => None,
        }
    }
}

impl From<bool> for SettingValue {
    fn from(v: bool) -> Self {
        SettingValue::Bool(v)
    }
}

impl From<i32> for SettingValue {
    fn from(v: i32) -> Self {
        SettingValue::Int(v)
    }
}

impl From<i64> for SettingValue {
    fn from(v: i64) -> Self {
        SettingValue::Long(v)
    }
}

impl From<f32> for SettingValue {
    fn from(v: f32) -> Self {
        SettingValue::Float(v)
    }
}

impl From<&str> for SettingValue {
    fn from(v: &str) -> Self {
        SettingValue::Str(v.to_string())
    }
}

/// The settings store: a deterministic (sorted) map of setting keys.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Settings {
    values: BTreeMap<String, SettingValue>,
}

impl Settings {
    pub fn new() -> Self {
        Settings::default()
    }

    /// Sets `key` to `value`.
    pub fn set(&mut self, key: &str, value: impl Into<SettingValue>) {
        self.values.insert(key.to_string(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&SettingValue> {
        self.values.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(|k| k.as_str())
    }

    /// Typed getters with defaults, mirroring the accessor style of
    /// `ModSettings`.
    pub fn bool(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(SettingValue::as_bool).unwrap_or(default)
    }

    pub fn int(&self, key: &str, default: i32) -> i32 {
        match self.get(key).and_then(SettingValue::as_float) {
            Some(f) => f as i32,
            None => default,
        }
    }

    pub fn long(&self, key: &str, default: i64) -> i64 {
        match self.get(key).and_then(SettingValue::as_float) {
            Some(f) => f as i64,
            None => default,
        }
    }

    pub fn float(&self, key: &str, default: f32) -> f32 {
        self.get(key).and_then(SettingValue::as_float).unwrap_or(default)
    }

    pub fn string(&self, key: &str, default: &str) -> String {
        match self.get(key) {
            Some(SettingValue::Str(s)) => s.clone(),
            _ => default.to_string(),
        }
    }
}

/// Setting keys used by the minimap (names mirror the Java mod's
/// `ModSettings` keys).
pub mod keys {
    pub const MINIMAP_DISABLED: &str = "minimapDisabled";
    pub const CAVE_MAPS_DISABLED: &str = "caveMapsDisabled";
    pub const SHOW_WAYPOINTS_DISABLED: &str = "showWaypointsDisabled";
    pub const DEATHPOINTS_DISABLED: &str = "deathpointsDisabled";
    pub const SLIME_CHUNKS: &str = "slimeChunks";
    pub const MINIMAP_SCALE: &str = "minimapScale";
    pub const MAX_WAYPOINTS_DISTANCE: &str = "maxWaypointsDistance";
    pub const ASSUME_USING_FBO: &str = "assumeUsingFBO";
    pub const IGNORE_HEIGHTMAPS: &str = "isIgnoreHeightmaps";
}

/// `xaero/common/config/LegacyCommonConfigIO`: loads and writes the legacy
/// `key=value` config files.
#[derive(Debug, Clone)]
pub struct LegacyCommonConfigIO {
    /// `configFilePathLjava/nio/file/Path;`.
    pub config_file_path: PathBuf,
    /// Mirrors the other boolean flags of the Java IO.
    pub allow_cave_mode_on_server: bool,
    /// Mirrors the other boolean flags of the Java IO.
    pub allow_nether_cave_mode_on_server: bool,
    /// Mirrors the other boolean flags of the Java IO.
    pub should_enable_everyone_tracks_everyone: bool,
}

impl LegacyCommonConfigIO {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        LegacyCommonConfigIO {
            config_file_path: path.into(),
            allow_cave_mode_on_server: false,
            allow_nether_cave_mode_on_server: false,
            should_enable_everyone_tracks_everyone: false,
        }
    }

    /// `load()`: reads the file into a [`Settings`] map.
    pub fn load(&self) -> Settings {
        let mut settings = Settings::new();
        if let Ok(text) = std::fs::read_to_string(&self.config_file_path) {
            for line in text.lines() {
                self.read_line(&mut settings, line);
            }
        }
        settings
    }

    /// `readLine(String[])Z`-style line parsing, as `load` uses it.
    pub fn read_line(&self, settings: &mut Settings, line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return true;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            settings.set(key.trim(), SettingValue::parse(value.trim()));
            return true;
        }
        false
    }

    /// `writeSettings(PrintWriter)`: writes every setting as `key=value`.
    pub fn write_settings(&self, settings: &Settings, out: &mut dyn Write) -> io::Result<()> {
        for key in settings.keys() {
            if let Some(value) = settings.get(key) {
                writeln!(out, "{}={}", key, value.to_text())?;
            }
        }
        out.flush()
    }

    /// Writes the settings to [`Self::config_file_path`].
    pub fn save_settings(&self, settings: &Settings) -> io::Result<()> {
        let mut buf = Vec::new();
        self.write_settings(settings, &mut buf)?;
        std::fs::write(&self.config_file_path, buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_parse_round_trips() {
        assert_eq!(SettingValue::parse("true"), SettingValue::Bool(true));
        assert_eq!(SettingValue::parse("False"), SettingValue::Str("False".into()));
        assert_eq!(SettingValue::parse("3"), SettingValue::Int(3));
        assert_eq!(SettingValue::parse("3000000000"), SettingValue::Long(3_000_000_000));
        assert_eq!(SettingValue::parse("0.5"), SettingValue::Float(0.5));
        assert_eq!(SettingValue::parse("hello"), SettingValue::Str("hello".into()));
    }

    #[test]
    fn settings_typed_accessors() {
        let mut settings = Settings::new();
        settings.set(keys::MINIMAP_SCALE, 0.5f32);
        settings.set(keys::MINIMAP_DISABLED, false);
        settings.set(keys::MAX_WAYPOINTS_DISTANCE, 100_000i32);
        assert_eq!(settings.float(keys::MINIMAP_SCALE, 1.0), 0.5);
        assert!(!settings.bool(keys::MINIMAP_DISABLED, true));
        assert_eq!(settings.int(keys::MAX_WAYPOINTS_DISTANCE, 0), 100_000);
        assert!(settings.contains(keys::MINIMAP_SCALE));
        assert_eq!(settings.string("missing", "d"), "d");
    }

    #[test]
    fn io_round_trip() {
        let io = LegacyCommonConfigIO::new("/tmp/xaero-test-config.txt");
        let mut settings = Settings::new();
        settings.set("k1", true);
        settings.set("k2", "some text");
        settings.set("k3", 42i32);
        let mut buf: Vec<u8> = Vec::new();
        io.write_settings(&settings, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("k1=true"));
        assert!(text.contains("k2=some text"));
        let mut loaded = Settings::new();
        for line in text.lines() {
            io.read_line(&mut loaded, line);
        }
        assert!(loaded.bool("k1", false));
        assert_eq!(loaded.string("k2", ""), "some text");
        assert_eq!(loaded.int("k3", 0), 42);
    }
}
