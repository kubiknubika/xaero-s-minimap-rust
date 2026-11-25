//! Applies the ported Xaero packet effects to Horizon HUD state.
//!
//! The mixin bundle receives channel payloads, decodes them with
//! [`xaerominimap_network::PacketKind`] and calls [`PacketBridge::apply`]. The
//! bridge keeps the existing `ClientWorldDataHelper` and
//! `ClientSyncedTrackedPlayerManager` state, then mirrors synced players into
//! the radar dots drawn by the HUD layer.

use std::sync::{Arc, Mutex};

use xaerominimap_core::{DimensionKey, RadarEntity};
use xaerominimap_network::{
    ClientSyncedTrackedPlayerManager, ClientWorldData, ClientWorldDataHelper, PacketEffect,
    PacketKind, Side, SyncedTrackedPlayer,
};

use crate::MinimapState;

/// Client-side packet state shared with the HUD layer.
pub struct PacketBridge {
    state: Arc<MinimapState>,
    worlds: Mutex<ClientWorldDataHelper>,
    tracked: Mutex<ClientSyncedTrackedPlayerManager>,
    current_dimension: Mutex<Option<String>>,
}

impl PacketBridge {
    pub fn new(state: Arc<MinimapState>) -> Self {
        PacketBridge {
            state,
            worlds: Mutex::new(ClientWorldDataHelper::new()),
            tracked: Mutex::new(ClientSyncedTrackedPlayerManager::new()),
            current_dimension: Mutex::new(None),
        }
    }

    /// Registers the current world, mirroring
    /// `MinimapClientWorldDataHelper.putWorld` + `setCurrent`.
    pub fn set_current_world(&self, dimension: &DimensionKey) {
        if self
            .worlds
            .lock()
            .unwrap()
            .get_world_data(dimension)
            .is_none()
        {
            self.worlds.lock().unwrap().put_world(
                dimension,
                ClientWorldData::new(dimension, false, false),
            );
        }
        self.worlds.lock().unwrap().set_current(dimension);
        *self.current_dimension.lock().unwrap() = Some(dimension.path.clone());
    }

    /// Decodes nothing itself: applies the same effect the Java packet handler
    /// would. Returns `false` when the packet has no handler on `side`.
    pub fn apply(&self, packet: &PacketKind, side: Side) -> bool {
        let Some(effect) = packet.effect(side) else {
            return false;
        };
        self.apply_effect(&effect)
    }

    /// Applies a resolved effect directly (useful for mixin-side state sync).
    pub fn apply_effect(&self, effect: &PacketEffect) -> bool {
        match effect {
            PacketEffect::SetClientNetworkVersion(version)
            | PacketEffect::SetServerNetworkVersion(version) => {
                let mut worlds = self.worlds.lock().unwrap();
                let Some(world) = worlds.get_current_world_data_mut() else {
                    return false;
                };
                world.set_server_mod_network_version(*version);
                true
            }
            PacketEffect::SetServerLevelId(id) => {
                let mut worlds = self.worlds.lock().unwrap();
                let Some(world) = worlds.get_current_world_data_mut() else {
                    return false;
                };
                world.set_server_level_id(*id);
                true
            }
            PacketEffect::SetSyncedRules(rules) => {
                let mut worlds = self.worlds.lock().unwrap();
                let Some(world) = worlds.get_current_world_data_mut() else {
                    return false;
                };
                world.set_synced_rules(*rules);
                // Xaero hides radar dots when the server disallows radar.
                self.state.set_radar_enabled(rules.allow_radar_on_server);
                true
            }
            PacketEffect::UpdateTrackedPlayer(_) | PacketEffect::ResetTrackedPlayers => {
                let applied = self.tracked.lock().unwrap().apply(effect);
                self.sync_radar();
                applied
            }
        }
    }

    pub fn server_network_version(&self) -> Option<i32> {
        self.worlds
            .lock()
            .unwrap()
            .get_current_world_data()
            .map(ClientWorldData::get_server_mod_network_version)
    }

    pub fn server_level_id(&self) -> Option<i32> {
        self.worlds
            .lock()
            .unwrap()
            .get_current_world_data()
            .and_then(ClientWorldData::get_server_level_id)
    }

    pub fn tracked_player_count(&self) -> usize {
        self.tracked.lock().unwrap().count()
    }

    /// Copies the synced players of the current dimension into the radar list
    /// consumed by [`crate::MinimapLayer`].
    pub fn sync_radar(&self) {
        let dimension = self.current_dimension.lock().unwrap().clone();
        let tracked: Vec<SyncedTrackedPlayer> = self
            .tracked
            .lock()
            .unwrap()
            .get_players()
            .cloned()
            .collect();
        let radar = tracked
            .into_iter()
            .filter(|player| {
                dimension
                    .as_ref()
                    .is_none_or(|current| current == player.dimension())
            })
            .map(|player| to_radar_entity(&player))
            .collect();
        self.state.set_radar(radar);
    }
}

fn to_radar_entity(player: &SyncedTrackedPlayer) -> RadarEntity {
    let mut entity = RadarEntity::new(
        "player",
        player.x(),
        player.y(),
        player.z(),
        "minecraft:player",
    );
    entity.uuid = player.id().to_string();
    entity
}

#[cfg(test)]
mod tests {
    use super::*;
    use xaerominimap_network::{HandshakePacket, LevelMapProperties, RulesPacket, TrackedPlayerPacket, Uuid};

    #[test]
    fn handshake_and_level_id_reach_world_data() {
        let bridge = PacketBridge::new(Arc::new(MinimapState::new()));
        bridge.set_current_world(&DimensionKey::overworld());
        assert!(bridge.apply(
            &PacketKind::Handshake(HandshakePacket::compatible()),
            Side::Client
        ));
        assert_eq!(bridge.server_network_version(), Some(3));
        assert!(bridge.apply(
            &PacketKind::LevelMapProperties(LevelMapProperties::new(42)),
            Side::Client
        ));
        assert_eq!(bridge.server_level_id(), Some(42));
    }

    #[test]
    fn rules_packet_toggles_radar() {
        let state = Arc::new(MinimapState::new());
        let bridge = PacketBridge::new(state.clone());
        bridge.set_current_world(&DimensionKey::overworld());
        assert!(bridge.apply(
            &PacketKind::Rules(RulesPacket {
                allow_cave_mode_on_server: true,
                allow_nether_cave_mode_on_server: true,
                allow_radar_on_server: false,
            }),
            Side::Client
        ));
        assert!(!state.is_radar_enabled());
    }

    #[test]
    fn tracked_players_become_radar_dots_in_the_current_dimension() {
        let state = Arc::new(MinimapState::new());
        let bridge = PacketBridge::new(state.clone());
        bridge.set_current_world(&DimensionKey::overworld());
        let id = Uuid::parse("00000000-0000-0000-0000-000000000001").unwrap();
        let packet = PacketKind::TrackedPlayer(TrackedPlayerPacket::update(
            id,
            1.0,
            64.0,
            2.0,
            "minecraft:overworld",
            3,
        ));
        assert!(bridge.apply(&packet, Side::Client));
        assert_eq!(bridge.tracked_player_count(), 1);
        let radar = state.radar_entities();
        assert_eq!(radar.len(), 1);
        assert_eq!(radar[0].x, 1.0);
        assert_eq!(radar[0].uuid, id.to_string());

        // A player in another dimension is tracked but not shown here.
        let nether = PacketKind::TrackedPlayer(TrackedPlayerPacket::update(
            Uuid::parse("00000000-0000-0000-0000-000000000002").unwrap(),
            2.0,
            64.0,
            3.0,
            "minecraft:the_nether",
            3,
        ));
        bridge.apply(&nether, Side::Client);
        assert_eq!(bridge.tracked_player_count(), 2);
        assert_eq!(state.radar_entities().len(), 1);
    }

    #[test]
    fn reset_packet_clears_radar() {
        let state = Arc::new(MinimapState::new());
        let bridge = PacketBridge::new(state.clone());
        bridge.set_current_world(&DimensionKey::overworld());
        let id = Uuid::parse("00000000-0000-0000-0000-000000000001").unwrap();
        bridge.apply(
            &PacketKind::TrackedPlayer(TrackedPlayerPacket::update(
                id,
                0.0,
                0.0,
                0.0,
                "minecraft:overworld",
                3,
            )),
            Side::Client,
        );
        assert_eq!(state.radar_entities().len(), 1);
        bridge.apply(
            &PacketKind::TrackerReset(xaerominimap_network::TrackerResetPacket),
            Side::Client,
        );
        assert!(state.radar_entities().is_empty());
        assert_eq!(bridge.tracked_player_count(), 0);
    }

    #[test]
    fn packets_without_a_client_handler_are_ignored() {
        let bridge = PacketBridge::new(Arc::new(MinimapState::new()));
        bridge.set_current_world(&DimensionKey::overworld());
        assert!(!bridge.apply(
            &PacketKind::Rules(RulesPacket::default()),
            Side::Server
        ));
    }
}
