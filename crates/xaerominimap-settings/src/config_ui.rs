//! GUI control types assigned to config options, generated from the Java
//! `MinimapConfigOptionUIRegister.registerAll` table. This is the data side
//! of the option screens; the actual widgets stay in the game client.
//!
//! | Java class | Rust type |
//! | --- | --- |
//! | `.../option/ui/type/ConfigOptionUIType` | [`ConfigUiType`] |
//! | `.../option/ui/ConfigOptionUITypeManager` | [`ConfigUiTypeManager`] |

use std::collections::BTreeMap;

/// UI control used to edit a config option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ConfigUiType {
    /// Java `BuiltInConfigOptionUITypes.DOUBLE_INDEXED_SLIDER` / `MinimapConfigOptionUITypes.DOUBLE_INDEXED_SLIDER`.
    DoubleIndexedSlider,
    /// Java `BuiltInConfigOptionUITypes.INFO_DISPLAY_CONFIG_EDITOR` / `MinimapConfigOptionUITypes.INFO_DISPLAY_CONFIG_EDITOR`.
    InfoDisplayConfigEditor,
    /// Java `BuiltInConfigOptionUITypes.INT_INDEXED_BUTTON` / `MinimapConfigOptionUITypes.INT_INDEXED_BUTTON`.
    IntIndexedButton,
    /// Java `BuiltInConfigOptionUITypes.INT_INDEXED_SLIDER` / `MinimapConfigOptionUITypes.INT_INDEXED_SLIDER`.
    IntIndexedSlider,
    /// Java `BuiltInConfigOptionUITypes.RADAR_CATEGORIES_EDITOR` / `MinimapConfigOptionUITypes.RADAR_CATEGORIES_EDITOR`.
    RadarCategoriesEditor,
    /// Java `BuiltInConfigOptionUITypes.STRING_STRING_EDIT` / `MinimapConfigOptionUITypes.STRING_STRING_EDIT`.
    StringStringEdit,
    /// Java `BuiltInConfigOptionUITypes.TOGGLE` / `MinimapConfigOptionUITypes.TOGGLE`.
    Toggle,
}

impl ConfigUiType {
    pub const ALL: [ConfigUiType; 7] = [ConfigUiType::DoubleIndexedSlider, ConfigUiType::InfoDisplayConfigEditor, ConfigUiType::IntIndexedButton, ConfigUiType::IntIndexedSlider, ConfigUiType::RadarCategoriesEditor, ConfigUiType::StringStringEdit, ConfigUiType::Toggle];

    /// Java field name of the UI type.
    pub fn name(self) -> &'static str {
        match self {
            ConfigUiType::DoubleIndexedSlider => "DOUBLE_INDEXED_SLIDER",
            ConfigUiType::InfoDisplayConfigEditor => "INFO_DISPLAY_CONFIG_EDITOR",
            ConfigUiType::IntIndexedButton => "INT_INDEXED_BUTTON",
            ConfigUiType::IntIndexedSlider => "INT_INDEXED_SLIDER",
            ConfigUiType::RadarCategoriesEditor => "RADAR_CATEGORIES_EDITOR",
            ConfigUiType::StringStringEdit => "STRING_STRING_EDIT",
            ConfigUiType::Toggle => "TOGGLE",
        }
    }

    pub fn from_name(name: &str) -> Option<ConfigUiType> {
        ConfigUiType::ALL.into_iter().find(|t| t.name() == name)
    }
}

/// One `registerUIType(option, uiType)` call, in Java registration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigUiBinding {
    /// Java owner class of the option field.
    pub owner: &'static str,
    /// Java static field name of the option.
    pub option: &'static str,
    pub ui_type: ConfigUiType,
}

/// All 115 UI-type registrations of `MinimapConfigOptionUIRegister`.
pub const UI_BINDINGS: &[ConfigUiBinding] = &[
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "DISPLAY_MINIMAP", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "MINIMAP_ITEM", ui_type: ConfigUiType::StringStringEdit },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "SAFE_MODE", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "UI_SCALE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapPrimaryClientConfigOptions", option: "UPDATE_NOTIFICATIONS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "RADAR_CATEGORIES", ui_type: ConfigUiType::RadarCategoriesEditor },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "DISPLAY_RADAR", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "RADAR_DOTS_STYLE", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "RADAR_SMOOTH_DOTS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "RADAR_NAME_SCALE", ui_type: ConfigUiType::DoubleIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "RADAR_MAIN_ENTITY", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "RADAR_MAIN_DOT_SIZE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "ARROW_SCALE", ui_type: ConfigUiType::DoubleIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "ARROW_COLOR", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "ARROW_OPACITY", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "TRACKED_PLAYERS_ON_MINIMAP", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "TRACKED_PLAYERS_IN_WORLD", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "TRACKED_PLAYER_MINIMAP_ICON_SCALE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "TRACKED_PLAYER_WORLD_ICON_SCALE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "TRACKED_PLAYER_WORLD_NAME_SCALE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINTS_ON_MINIMAP", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINTS_IN_WORLD", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINTS_ALL_SETS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "DEATHPOINTS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "OLD_DEATHPOINTS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "DELETE_REACHED_DEATHPOINTS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "AUTO_WAYPOINTS_ON_DEATH", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_ICON_SCALE_IN_WORLD", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_NAME_SCALE_IN_WORLD", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_CLOSE_SCALE_IN_WORLD", ui_type: ConfigUiType::DoubleIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_DISTANCE_SCALE_IN_WORLD", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "MULTIPLE_WAYPOINTS_INFO", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_DISTANCE_IN_WORLD", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_NAME_IN_WORLD", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_SHORT_DISTANCE_IN_WORLD", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_VERTICAL_POINTING_ANGLE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_HORIZONTAL_POINTING_ANGLE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_MAX_DISTANCE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_MAX_DISTANCE_DIMENSION_SCALE", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_MIN_DISTANCE_IN_WORLD", ui_type: ConfigUiType::DoubleIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_CONVERT_DISTANCE_TO_KM_AT", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_DISTANCE_PRECISION", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_OPACITY_IN_WORLD", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_OPACITY_ON_MINIMAP", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_ICON_SCALE_ON_MINIMAP", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "TEMPORARY_WAYPOINTS_GLOBAL", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_DEFAULT_TELEPORT_FORMAT", ui_type: ConfigUiType::StringStringEdit },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_DEFAULT_TELEPORT_ROTATION_FORMAT", ui_type: ConfigUiType::StringStringEdit },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_TELEPORT_CROSS_DIMENSION", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_PARTIAL_Y_TELEPORT", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "NEW_WAYPOINTS_TO_BOTTOM", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "HIDE_WORLD_NAMES", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "COMPASS_OVER_EVERYTHING", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "HIDE_WAYPOINT_COORDINATES", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "COMPASS_LOCATION", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "COMPASS_SHADOW_COLOR", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "COMPASS_NORTH_SHADOW_COLOR", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "COMPASS_SCALE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "BLOCK_COLORS", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "BIOMES_IN_VANILLA_COLORS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "CAVE_MODE_ALLOWED", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "AUTO_CAVE_MODE", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "MANUAL_CAVE_MODE_START", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "CAVE_MODE_DEPTH", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "LEGIBLE_CAVE_MAPS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "CAVE_MODE_TOGGLE_TIMER", ui_type: ConfigUiType::DoubleIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "BIOME_BLENDING", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "DISPLAY_WORLD_MAP_CHUNKS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "TERRAIN_DEPTH", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "TERRAIN_SLOPES", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "DISPLAY_REDSTONE", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "DISPLAY_FLOWERS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "BLOCK_TRANSPARENCY", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "DISPLAY_STAINED_GLASS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "ADJUST_HEIGHT_FOR_SHORT_BLOCKS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "ANTI_ALIASING", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "SIZE", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "SHAPE", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "NORTH_LOCKED", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "ZOOM", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "CAVE_ZOOM", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "ZOOM_ENLARGED", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "CENTERED_ENLARGED", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "KEEP_ENLARGED_UNLOCKED", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "TOGGLED_ENLARGED", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "LIGHTING", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "OPACITY", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "FRAME", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "FRAME_COLOR", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "HIDE_UNDER_SCREEN", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "HIDE_UNDER_F3", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "BOSS_HEALTH_PUSH_BOX", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "POTION_EFFECT_PUSH_BOX", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "CHUNK_GRID", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "CHUNK_GRID_LINE_WIDTH", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "SLIME_CHUNKS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "OPEN_SLIME_CHUNKS_SCREEN", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "LIGHT_OVERLAY_TYPE", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "LIGHT_OVERLAY_MAX_LIGHT", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "LIGHT_OVERLAY_MIN_LIGHT", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "LIGHT_OVERLAY_COLOR", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "OPAC_CLAIMS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "OPAC_CLAIMS_FILL_OPACITY", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "OPAC_CLAIMS_BORDER_OPACITY", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "OPAC_CURRENT_CLAIM", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "INFO_DISPLAY_BG_OPACITY", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "INFO_DISPLAY_ALIGNMENT", ui_type: ConfigUiType::IntIndexedButton },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "INFO_DISPLAY_CONFIG", ui_type: ConfigUiType::InfoDisplayConfigEditor },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "RENDER_LAYER", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "RADAR_HIDE_INVISIBLE", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "FURTHER_ZOOMOUT", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "UNDISCOVERED_OPACITY", ui_type: ConfigUiType::IntIndexedSlider },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "DISCOVERED_WAYSTONE_WAYPOINTS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "OTHER_WAYSTONE_WAYPOINTS", ui_type: ConfigUiType::Toggle },
    ConfigUiBinding { owner: "MinimapProfiledConfigOptions", option: "WAYPOINT_DISTANCE_IN_MENU", ui_type: ConfigUiType::Toggle },
];

/// `ConfigOptionUITypeManager`: option field name → UI type.
#[derive(Debug, Clone, Default)]
pub struct ConfigUiTypeManager {
    types: BTreeMap<String, ConfigUiType>,
}

impl ConfigUiTypeManager {
    pub fn new() -> Self { Self::default() }

    /// Registers one option type; duplicates overwrite (matching a map put).
    pub fn register(&mut self, option: &str, ui_type: ConfigUiType) {
        self.types.insert(option.to_string(), ui_type);
    }

    /// `MinimapConfigOptionUIRegister.registerAll`.
    pub fn register_all(&mut self) {
        for binding in UI_BINDINGS {
            self.register(binding.option, binding.ui_type);
        }
    }

    pub fn get(&self, option: &str) -> Option<ConfigUiType> {
        self.types.get(option).copied()
    }

    pub fn len(&self) -> usize { self.types.len() }

    pub fn is_empty(&self) -> bool { self.types.is_empty() }

    pub fn bindings(&self) -> &BTreeMap<String, ConfigUiType> { &self.types }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_table_matches_the_java_register_all() {
        assert_eq!(UI_BINDINGS.len(), 115);
        assert_eq!(UI_BINDINGS[0].option, "DISPLAY_MINIMAP");
        assert_eq!(UI_BINDINGS[0].ui_type, ConfigUiType::Toggle);
        assert_eq!(UI_BINDINGS[5].ui_type, ConfigUiType::RadarCategoriesEditor);
    }

    #[test]
    fn ui_type_names_round_trip() {
        for ui_type in ConfigUiType::ALL {
            assert_eq!(ConfigUiType::from_name(ui_type.name()), Some(ui_type));
        }
    }

    #[test]
    fn manager_registers_every_binding() {
        let mut manager = ConfigUiTypeManager::new();
        manager.register_all();
        assert_eq!(manager.len(), 115);
        assert_eq!(manager.get("DISPLAY_MINIMAP"), Some(ConfigUiType::Toggle));
        assert_eq!(manager.get("RADAR_CATEGORIES"), Some(ConfigUiType::RadarCategoriesEditor));
        assert_eq!(manager.get("MINIMAP_ITEM"), Some(ConfigUiType::StringStringEdit));
    }
}
