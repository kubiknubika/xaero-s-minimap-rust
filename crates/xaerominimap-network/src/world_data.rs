//! Data side of `xaero/common/minimap/mcworld` from the original mod.
//!
//! `MinimapClientWorldData` stores the server-mod handshake state, the
//! server level id, shadow tint and synced rules for one client world. The
//! engine-only `EnvironmentAttributeSystem` and `ClientLevel` lookups stay
//! in the game client.

use std::collections::BTreeMap;

use xaerominimap_core::DimensionKey;

use crate::RulesPacket;

/// `xaero/common/minimap/mcworld/MinimapClientWorldData`.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientWorldData {
    server_mod_network_version: i32,
    server_level_id: Option<i32>,
    shadow_r: f32,
    shadow_g: f32,
    shadow_b: f32,
    synced_rules: Option<RulesPacket>,
}

impl ClientWorldData {
    /// Java `MinimapClientWorldData(ClientLevel)` — dimension-based shadow
    /// colours. The two boolean flags correspond to the dimension type
    /// registration checks for the overworld/nether types.
    pub fn new(dimension: &DimensionKey, overworld_type: bool, nether_type: bool) -> Self {
        let (r, g, b) = if dimension == &DimensionKey::overworld() || overworld_type {
            (0.5180000066757202, 0.6779999732971191, 1.0)
        } else if dimension == &DimensionKey::the_nether() || nether_type {
            (1.0, 0.0, 0.0)
        } else {
            (1.0, 1.0, 1.0)
        };
        ClientWorldData {
            server_mod_network_version: 0,
            server_level_id: None,
            shadow_r: r as f32,
            shadow_g: g as f32,
            shadow_b: b as f32,
            synced_rules: None,
        }
    }

    pub fn set_server_mod_network_version(&mut self, version: i32) {
        self.server_mod_network_version = version;
    }

    pub fn get_server_mod_network_version(&self) -> i32 {
        self.server_mod_network_version
    }

    /// `MinimapClientWorldDataHelper.getCurrentWorldData` writes this field
    /// from the `SetServerLevelId` packet effect.
    pub fn set_server_level_id(&mut self, id: i32) {
        self.server_level_id = Some(id);
    }

    pub fn get_server_level_id(&self) -> Option<i32> {
        self.server_level_id
    }

    pub fn set_synced_rules(&mut self, rules: RulesPacket) {
        self.synced_rules = Some(rules);
    }

    /// Java lazily creates `new ClientboundRulesPacket(true, true, true)`
    /// when no rules have been received yet.
    pub fn get_synced_rules(&mut self) -> &RulesPacket {
        self.synced_rules.get_or_insert_with(RulesPacket::default)
    }

    pub fn shadow_rgb(&self) -> (f32, f32, f32) {
        (self.shadow_r, self.shadow_g, self.shadow_b)
    }
}

/// `xaero/common/minimap/mcworld/MinimapClientWorldDataHelper`: the Java
/// helper looks the data up by `ClientLevel`; the Rust port keys by dimension
/// id and tracks the current one explicitly.
#[derive(Debug, Clone, Default)]
pub struct ClientWorldDataHelper {
    worlds: BTreeMap<String, ClientWorldData>,
    current: Option<String>,
}

impl ClientWorldDataHelper {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the data for a client world.
    pub fn put_world(&mut self, dimension: &DimensionKey, data: ClientWorldData) {
        self.worlds.insert(dimension.path.clone(), data);
    }

    /// Marks a registered world as current.
    pub fn set_current(&mut self, dimension: &DimensionKey) {
        self.current = Some(dimension.path.clone());
    }

    /// `getWorldData(ClientLevel)`.
    pub fn get_world_data(&self, dimension: &DimensionKey) -> Option<&ClientWorldData> {
        self.worlds.get(&dimension.path)
    }

    pub fn get_world_data_mut(
        &mut self,
        dimension: &DimensionKey,
    ) -> Option<&mut ClientWorldData> {
        self.worlds.get_mut(&dimension.path)
    }

    /// `getCurrentWorldData()`.
    pub fn get_current_world_data(&self) -> Option<&ClientWorldData> {
        self.current
            .as_ref()
            .and_then(|dimension| self.worlds.get(dimension))
    }

    pub fn get_current_world_data_mut(&mut self) -> Option<&mut ClientWorldData> {
        let key = self.current.clone()?;
        self.worlds.get_mut(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_colours_follow_dimension() {
        let overworld = ClientWorldData::new(&DimensionKey::overworld(), false, false);
        assert!((overworld.shadow_rgb().0 - 0.518).abs() < 0.001);
        assert!((overworld.shadow_rgb().1 - 0.678).abs() < 0.001);
        assert_eq!(overworld.shadow_rgb().2, 1.0);

        let nether = ClientWorldData::new(&DimensionKey::the_nether(), false, false);
        assert_eq!(nether.shadow_rgb(), (1.0, 0.0, 0.0));

        let end = ClientWorldData::new(&DimensionKey::new("minecraft:the_end"), false, false);
        assert_eq!(end.shadow_rgb(), (1.0, 1.0, 1.0));

        // Dimension-type registrations override the key lookup.
        let typed = ClientWorldData::new(&DimensionKey::new("custom:type"), true, false);
        assert!((typed.shadow_rgb().0 - 0.518).abs() < 0.001);
    }

    #[test]
    fn rules_default_to_all_allowed_until_received() {
        let mut data = ClientWorldData::new(&DimensionKey::overworld(), false, false);
        assert_eq!(
            *data.get_synced_rules(),
            RulesPacket {
                allow_cave_mode_on_server: true,
                allow_nether_cave_mode_on_server: true,
                allow_radar_on_server: true,
            }
        );
        let rules = RulesPacket {
            allow_cave_mode_on_server: false,
            allow_nether_cave_mode_on_server: true,
            allow_radar_on_server: false,
        };
        data.set_synced_rules(rules);
        assert_eq!(*data.get_synced_rules(), rules);
    }

    #[test]
    fn helper_tracks_current_world_data() {
        let mut helper = ClientWorldDataHelper::new();
        let dim = DimensionKey::overworld();
        let mut data = ClientWorldData::new(&dim, false, false);
        data.set_server_mod_network_version(3);
        data.set_server_level_id(7);
        helper.put_world(&dim, data);
        assert!(helper.get_current_world_data().is_none());
        helper.set_current(&dim);
        let current = helper.get_current_world_data_mut().unwrap();
        assert_eq!(current.get_server_mod_network_version(), 3);
        assert_eq!(current.get_server_level_id(), Some(7));
        assert!(helper.get_world_data(&dim).is_some());
    }
}
