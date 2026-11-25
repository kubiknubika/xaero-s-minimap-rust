//! `xaero/hud/minimap/module/MinimapSession`-style session state: the current
//! world and the settings of the minimap.

use std::collections::BTreeMap;

use xaerominimap_controls::{ControlEvent, ControlsRegister, KeyStateSource};
use xaerominimap_core::radar::RadarEntity;
use xaerominimap_core::world::{DimensionKey, MinimapWorld};
use xaerominimap_render::{
    IconCache, IconDefinitionManager, IconForm, IconVariant, ThirdPartyWaypointManager,
};
use xaerominimap_settings::{RadarCategory, RadarCategoryManager, RadarEntityInfo, SettingValue, Settings};

/// Session state of the minimap: the world it is shown in, its settings and
/// the entity icon definitions used by the radar.
#[derive(Debug, Clone)]
pub struct MinimapSession {
    world: Option<MinimapWorld>,
    settings: Settings,
    icon_definitions: IconDefinitionManager,
    icon_cache: IconCache,
    radar_categories: RadarCategoryManager,
    controls: ControlsRegister,
    third_party_waypoints: ThirdPartyWaypointManager,
}

impl Default for MinimapSession {
    fn default() -> Self {
        MinimapSession {
            world: None,
            settings: Settings::new(),
            // The Java mod loads the definition resources when the radar
            // icon manager is created, so do the same for a new session.
            icon_definitions: IconDefinitionManager::with_builtins(),
            icon_cache: IconCache::new(),
            radar_categories: RadarCategoryManager::with_defaults(),
            controls: ControlsRegister::new(),
            third_party_waypoints: ThirdPartyWaypointManager::new(),
        }
    }
}

impl MinimapSession {
    pub fn new() -> Self {
        MinimapSession::default()
    }

    /// Session for a world that already exists.
    pub fn with_world(world: MinimapWorld) -> Self {
        MinimapSession {
            world: Some(world),
            ..Default::default()
        }
    }

    /// The current world, if any.
    pub fn world(&self) -> Option<&MinimapWorld> {
        self.world.as_ref()
    }

    /// Mutable access to the current world.
    pub fn world_mut(&mut self) -> Option<&mut MinimapWorld> {
        self.world.as_mut()
    }

    /// Sets the session's world.
    pub fn set_world(&mut self, world: MinimapWorld) {
        self.world = Some(world);
    }

    /// The session settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Mutable access to the session settings.
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Entity radar icon definitions (`RadarIconDefinitionManager`).
    pub fn icon_definitions(&self) -> &IconDefinitionManager {
        &self.icon_definitions
    }

    /// Mutable access to the icon definitions, e.g. to add resources from a
    /// mod (the Java mod reloads them through `RadarIconDefinitionReloader`).
    pub fn icon_definitions_mut(&mut self) -> &mut IconDefinitionManager {
        &mut self.icon_definitions
    }

    /// Cached radar icons (`RadarIconCache`).
    pub fn icon_cache(&self) -> &IconCache {
        &self.icon_cache
    }

    /// Mutable access to the icon cache.
    pub fn icon_cache_mut(&mut self) -> &mut IconCache {
        &mut self.icon_cache
    }

    /// The radar icon form for an entity type and its variant id.
    pub fn icon_form(&self, entity_id: &str, variant: &str) -> IconForm {
        self.icon_definitions.form_for(entity_id, variant)
    }

    /// The radar icon form for a typed variant id.
    pub fn icon_form_for(&self, entity_id: &str, variant: &IconVariant) -> IconForm {
        self.icon_definitions.form_for_variant(entity_id, variant)
    }

    /// Entity radar category tree (`EntityRadarCategoryManager`).
    pub fn radar_categories(&self) -> &RadarCategoryManager {
        &self.radar_categories
    }

    pub fn radar_categories_mut(&mut self) -> &mut RadarCategoryManager {
        &mut self.radar_categories
    }

    /// The deepest category matching the full entity info supplied by the
    /// game-side glue.
    pub fn radar_category_for_info(&self, info: &RadarEntityInfo) -> &RadarCategory {
        self.radar_categories.category_for(info)
    }

    /// Effective radar settings for the full entity info.
    pub fn radar_settings_for_info(&self, info: &RadarEntityInfo) -> BTreeMap<String, SettingValue> {
        self.radar_categories.settings_for(info)
    }

    /// Best-effort category for a [`RadarEntity`] from the radar list. The
    /// game glue should prefer [`Self::radar_category_for_info`] when it has
    /// the full entity state (team, tamed flags, light level, etc.).
    pub fn radar_category_for(&self, entity: &RadarEntity) -> &RadarCategory {
        let mut info = RadarEntityInfo::new(entity.entity_type.clone());
        info.custom_name = entity.name.clone();
        info.is_item = entity.entity_type.ends_with(":item");
        info.is_living = !info.is_item;
        if entity.entity_type.ends_with(":player") {
            info.is_player = true;
            info.player_name = entity.name.clone();
        }
        self.radar_category_for_info(&info)
    }

    /// Registered key mappings and their attached functions
    /// (`KeyMappingControllerManager`).
    pub fn controls(&self) -> &ControlsRegister {
        &self.controls
    }

    /// Mutable access to the key mapping controllers, e.g. to register
    /// additional mod-owned keys.
    pub fn controls_mut(&mut self) -> &mut ControlsRegister {
        &mut self.controls
    }

    /// Runs one client tick of the minimap key handler and returns the
    /// actions the game glue should apply.
    pub fn tick_controls(&mut self, state: &mut impl KeyStateSource) -> Vec<ControlEvent> {
        self.controls.tick(state)
    }

    /// Waypoints contributed by other mods, keyed by their origin id
    /// (`ThirdPartyWaypointManager`).
    pub fn third_party_waypoints(&self) -> &ThirdPartyWaypointManager {
        &self.third_party_waypoints
    }

    /// Mutable access used by addon glue to register waypoint origins.
    pub fn third_party_waypoints_mut(&mut self) -> &mut ThirdPartyWaypointManager {
        &mut self.third_party_waypoints
    }

    /// Returns the world, creating one for `dim_id` if there is none yet
    /// (`getOrInsert`-style, like the Java session helpers that make a
    /// world on first use).
    pub fn ensure_world(&mut self, dim_id: DimensionKey) -> &mut MinimapWorld {
        self.world
            .get_or_insert_with(|| MinimapWorld::new("", dim_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_ensure_world_and_settings() {
        let mut session = MinimapSession::new();
        assert!(session.world().is_none());
        let world = session.ensure_world(DimensionKey::overworld());
        world.add_waypoint_set("Default");
        assert_eq!(session.world().unwrap().get_set_count(), 1);
        session.settings_mut().set("minimapScale", 0.5f32);
        assert_eq!(session.settings().float("minimapScale", 0.0), 0.5);
    }

    #[test]
    fn session_loads_builtin_icon_definitions() {
        let mut session = MinimapSession::new();
        assert!(session.icon_definitions().get("minecraft:pig").is_some());
        assert_eq!(
            session.icon_form("minecraft:pig", "any variant").form_type,
            crate::IconFormType::Model
        );

        let saddled = crate::IconVariant::Saddle {
            texture: "minecraft:textures/entity/pig/pig.png".into(),
            saddled: true,
        };
        assert_eq!(
            session.icon_form_for("minecraft:pig", &saddled).form_type,
            crate::IconFormType::Model
        );
        assert!(session.icon_cache().is_empty());
        session.icon_cache_mut().get_entity_cache("minecraft:pig").add(
            crate::IconKey::new(saddled, None),
            crate::CachedIcon::new("model:default"),
        );
        assert!(!session.icon_cache().is_empty());
    }

    #[test]
    fn session_registers_minimap_controls() {
        use xaerominimap_controls::KeyStateSource;
        use std::collections::BTreeMap;

        #[derive(Default)]
        struct State {
            down: BTreeMap<i32, bool>,
            clicks: BTreeMap<i32, usize>,
        }
        impl KeyStateSource for State {
            fn is_down(&self, code: i32) -> bool {
                self.down.get(&code).copied().unwrap_or(false)
            }
            fn consume_click(&mut self, code: i32) -> bool {
                let n = self.clicks.entry(code).or_insert(0);
                if *n > 0 {
                    *n -= 1;
                    true
                } else {
                    false
                }
            }
        }

        let mut session = MinimapSession::new();
        assert_eq!(
            session
                .controls()
                .key_mapping_controllers()
                .controllers()
                .len(),
            21
        );
        let mut state = State::default();
        state.down.insert(5, true);
        state.clicks.insert(5, 1);
        let events = session.tick_controls(&mut state);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, crate::MinimapAction::AddWaypoint);
        assert_eq!(events[0].kind, crate::ControlEventKind::Press);
    }

    #[test]
    fn session_manages_third_party_waypoint_origins() {
        let mut session = MinimapSession::new();
        let waypoint = crate::Waypoint::new(1, 2, 3, "Waystone", "");
        session
            .third_party_waypoints_mut()
            .get_or_create("waystones:main")
            .add("stone-1".to_string(), waypoint);
        assert_eq!(session.third_party_waypoints().origin_count(), 1);
        assert_eq!(session.third_party_waypoints().count(), 1);
        assert_eq!(
            session
                .third_party_waypoints()
                .get("waystones:main")
                .unwrap()
                .get("stone-1")
                .unwrap()
                .third_party_origin(),
            Some("waystones:main")
        );
    }

    #[test]
    fn session_selects_radar_category_and_settings() {
        let session = MinimapSession::new();
        let mut info = crate::RadarEntityInfo::new("minecraft:zombie");
        info.is_living = true;
        info.is_hostile = true;
        assert_eq!(
            session.radar_category_for_info(&info).name,
            "gui.xaero_entity_category_hostile"
        );
        assert_eq!(
            session.radar_settings_for_info(&info).get("displayed"),
            Some(&crate::SettingValue::Bool(true))
        );

        let entity = crate::RadarEntity::new("Zed", 1.0, 2.0, 3.0, "minecraft:zombie");
        assert_eq!(
            session.radar_category_for(&entity).name,
            "gui.xaero_entity_category_friendly"
        );
    }
}
