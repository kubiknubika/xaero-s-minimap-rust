//! Built-in info-display definitions from `xaero/hud/minimap/info/BuiltInInfoDisplays`.
//!
//! The compilers and widget factories are render/GUI glue and stay in the
//! game client; this table holds the data-side descriptor of each display:
//! id, translation key, default state, state codec, UI control kind and
//! whether a legacy settings getter exists.

/// State codec used by an info display (`BuiltInConfigValueIOCodecs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoDisplayCodec {
    Boolean,
    Integer,
}

/// UI control kind (`InfoDisplayCommonWidgetFactories`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoDisplayWidgetKind {
    OffOn,
    Cycle,
    AlwaysOn,
}

/// Data-side descriptor of a built-in info display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltInInfoDisplay {
    pub id: &'static str,
    pub translation_key: &'static str,
    pub default_bool: bool,
    pub default_int: i32,
    pub codec: InfoDisplayCodec,
    pub widget: InfoDisplayWidgetKind,
    pub has_legacy_getter: bool,
}

impl BuiltInInfoDisplay {
    /// Java `defaultState` as an integer-like value (booleans become 0/1).
    pub fn default_state(&self) -> i32 {
        if self.codec == InfoDisplayCodec::Boolean {
            i32::from(self.default_bool)
        } else {
            self.default_int
        }
    }
}

/// `BuiltInInfoDisplays.ALL`, in registration order.
pub const BUILTIN_INFO_DISPLAYS: &[BuiltInInfoDisplay] = &[
    BuiltInInfoDisplay { id: "coords", translation_key: "gui.xaero_infodisplay_coords", default_bool: true, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::OffOn, has_legacy_getter: true },
    BuiltInInfoDisplay { id: "overworld_coords", translation_key: "gui.xaero_infodisplay_overworld_coords", default_bool: false, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::OffOn, has_legacy_getter: false },
    BuiltInInfoDisplay { id: "chunk_coords", translation_key: "gui.xaero_infodisplay_chunk_coords", default_bool: false, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::OffOn, has_legacy_getter: false },
    BuiltInInfoDisplay { id: "angles", translation_key: "gui.xaero_infodisplay_angles", default_bool: false, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::OffOn, has_legacy_getter: true },
    BuiltInInfoDisplay { id: "dimension", translation_key: "gui.xaero_infodisplay_dimension", default_bool: false, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::OffOn, has_legacy_getter: true },
    BuiltInInfoDisplay { id: "biome", translation_key: "gui.xaero_infodisplay_biome", default_bool: false, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::OffOn, has_legacy_getter: true },
    BuiltInInfoDisplay { id: "weather", translation_key: "gui.xaero_infodisplay_weather", default_bool: false, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::OffOn, has_legacy_getter: true },
    BuiltInInfoDisplay { id: "light_level", translation_key: "gui.xaero_infodisplay_light_level", default_bool: false, default_int: 0, codec: InfoDisplayCodec::Integer, widget: InfoDisplayWidgetKind::Cycle, has_legacy_getter: true },
    BuiltInInfoDisplay { id: "time", translation_key: "gui.xaero_infodisplay_time", default_bool: false, default_int: 0, codec: InfoDisplayCodec::Integer, widget: InfoDisplayWidgetKind::Cycle, has_legacy_getter: true },
    BuiltInInfoDisplay { id: "real_time", translation_key: "gui.xaero_infodisplay_real_time", default_bool: false, default_int: 0, codec: InfoDisplayCodec::Integer, widget: InfoDisplayWidgetKind::Cycle, has_legacy_getter: false },
    BuiltInInfoDisplay { id: "highlights", translation_key: "gui.xaero_infodisplay_highlights", default_bool: true, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::AlwaysOn, has_legacy_getter: false },
    BuiltInInfoDisplay { id: "light_overlay_indicator", translation_key: "gui.xaero_infodisplay_light_overlay_indicator", default_bool: true, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::AlwaysOn, has_legacy_getter: false },
    BuiltInInfoDisplay { id: "manual_cave_mode_indicator", translation_key: "gui.xaero_infodisplay_manual_cave_mode_indicator", default_bool: true, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::AlwaysOn, has_legacy_getter: false },
    BuiltInInfoDisplay { id: "custom_sub_world", translation_key: "gui.xaero_infodisplay_custom_sub_world", default_bool: true, default_int: 0, codec: InfoDisplayCodec::Boolean, widget: InfoDisplayWidgetKind::AlwaysOn, has_legacy_getter: false },
];

/// `BuiltInInfoDisplays.forEach`.
pub fn for_each(mut visit: impl FnMut(&BuiltInInfoDisplay)) {
    for display in BUILTIN_INFO_DISPLAYS {
        visit(display);
    }
}

/// `InfoDisplayManager.adaptOrder(Stream<String>)`: keep known ids from
/// the incoming order and insert any missing default ids at their default
/// positions.
pub fn adapt_display_order(order: &[String]) -> Vec<String> {
    let mut result: Vec<String> = order
        .iter()
        .filter(|id| BUILTIN_INFO_DISPLAYS.iter().any(|d| d.id == id.as_str()))
        .cloned()
        .collect();
    let mut last_index: i32 = -1;
    for display in BUILTIN_INFO_DISPLAYS {
        let index = result.iter().position(|id| id == display.id);
        let insert_at = match index {
            Some(index) => index,
            None if last_index == -1 => 0,
            None => (last_index + 1) as usize,
        };
        if index.is_none() {
            result.insert(insert_at.min(result.len()), display.id.to_string());
        }
        last_index = insert_at as i32;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_table_matches_java_registration() {
        assert_eq!(BUILTIN_INFO_DISPLAYS.len(), 14);
        assert_eq!(BUILTIN_INFO_DISPLAYS[0].id, "coords");
        assert_eq!(BUILTIN_INFO_DISPLAYS[0].translation_key, "gui.xaero_infodisplay_coords");
        assert!(BUILTIN_INFO_DISPLAYS[0].default_bool);
        assert!(!BUILTIN_INFO_DISPLAYS[1].default_bool);
        assert_eq!(BUILTIN_INFO_DISPLAYS[7].codec, InfoDisplayCodec::Integer);
        assert_eq!(BUILTIN_INFO_DISPLAYS[7].widget, InfoDisplayWidgetKind::Cycle);
        assert_eq!(BUILTIN_INFO_DISPLAYS[10].widget, InfoDisplayWidgetKind::AlwaysOn);
    }

    #[test]
    fn for_each_visits_all_displays() {
        let mut count = 0;
        for_each(|_| count += 1);
        assert_eq!(count, 14);
    }

    #[test]
    fn adapt_order_filters_unknown_and_inserts_defaults() {
        let order = vec!["coords".to_string(), "unknown".to_string(), "biome".to_string()];
        let adapted = adapt_display_order(&order);
        assert_eq!(adapted[0], "coords");
        assert!(!adapted.contains(&"unknown".to_string()));
        assert_eq!(adapted[1], "overworld_coords");
        assert_eq!(adapted[2], "chunk_coords");
        assert!(adapted.contains(&"biome".to_string()));
        assert_eq!(adapted.len(), BUILTIN_INFO_DISPLAYS.len());
    }
}
