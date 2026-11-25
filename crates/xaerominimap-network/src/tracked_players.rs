//! Synced tracked-player state, ported from
//! `xaero/common/server/radar/tracker` and
//! `xaero/hud/minimap/player/tracker/synced`.
//!
//! The server sends [`crate::TrackedPlayerPacket`]s when a tracked player
//! moves and a [`crate::TrackerResetPacket`] to clear the list. This module
//! holds the resulting client-side state and applies the packet effects.

use std::collections::BTreeMap;

use crate::{PacketEffect, Uuid};

/// `xaero/common/server/radar/tracker/SyncedTrackedPlayer`.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncedTrackedPlayer {
    id: Uuid,
    x: f64,
    y: f64,
    z: f64,
    /// Java `ResourceKey<Level>`; the Rust port keeps the identifier string.
    dimension: String,
}

impl SyncedTrackedPlayer {
    pub fn new(id: Uuid, x: f64, y: f64, z: f64, dimension: impl Into<String>) -> Self {
        SyncedTrackedPlayer {
            id,
            x,
            y,
            z,
            dimension: dimension.into(),
        }
    }

    /// `setPos(double, double, double)`.
    pub fn set_pos(&mut self, x: f64, y: f64, z: f64) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    /// `setDimension(ResourceKey)`.
    pub fn set_dimension(&mut self, dimension: impl Into<String>) {
        self.dimension = dimension.into();
    }

    /// `getId()`.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// `getX()`.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// `getY()`.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// `getZ()`.
    pub fn z(&self) -> f64 {
        self.z
    }

    /// `getDimensionKey()` / `SyncedTrackedPlayerReader.getDimension`.
    pub fn dimension(&self) -> &str {
        &self.dimension
    }

    /// `matchesEnough(Player, double)`: all three coordinates must be within
    /// `tolerance` and the dimension identifier must match.
    pub fn matches_enough(
        &self,
        x: f64,
        y: f64,
        z: f64,
        dimension: &str,
        tolerance: f64,
    ) -> bool {
        (x - self.x).abs() <= tolerance
            && (y - self.y).abs() <= tolerance
            && (z - self.z).abs() <= tolerance
            && dimension == self.dimension
    }

    /// `copyFrom(SyncedTrackedPlayer)`.
    pub fn copy_from(&mut self, other: &SyncedTrackedPlayer) {
        self.id = other.id;
        self.x = other.x;
        self.y = other.y;
        self.z = other.z;
        self.dimension = other.dimension.clone();
    }
}

/// `xaero/hud/minimap/player/tracker/synced/ClientSyncedTrackedPlayerManager`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientSyncedTrackedPlayerManager {
    tracked_players: BTreeMap<Uuid, SyncedTrackedPlayer>,
}

impl ClientSyncedTrackedPlayerManager {
    pub fn new() -> Self {
        ClientSyncedTrackedPlayerManager::default()
    }

    /// `remove(UUID)`.
    pub fn remove(&mut self, id: Uuid) {
        self.tracked_players.remove(&id);
    }

    /// `update(UUID, double, double, double, ResourceKey)`: creates the entry
    /// on first sight and mutates it in place afterwards.
    pub fn update(&mut self, id: Uuid, x: f64, y: f64, z: f64, dimension: impl Into<String>) {
        let dimension = dimension.into();
        if let Some(player) = self.tracked_players.get_mut(&id) {
            player.set_pos(x, y, z);
            player.set_dimension(dimension);
        } else {
            self.tracked_players
                .insert(id, SyncedTrackedPlayer::new(id, x, y, z, dimension));
        }
    }

    /// `getPlayers()`.
    pub fn get_players(&self) -> impl Iterator<Item = &SyncedTrackedPlayer> {
        self.tracked_players.values()
    }

    pub fn get(&self, id: Uuid) -> Option<&SyncedTrackedPlayer> {
        self.tracked_players.get(&id)
    }

    /// `reset()` — clears all tracked players.
    pub fn reset(&mut self) {
        self.tracked_players.clear();
    }

    pub fn count(&self) -> usize {
        self.tracked_players.len()
    }

    /// Applies a packet handler effect from the client handler of
    /// `ClientboundTrackedPlayerPacket` / `ClientboundPlayerTrackerResetPacket`.
    pub fn apply(&mut self, effect: &PacketEffect) -> bool {
        match effect {
            PacketEffect::UpdateTrackedPlayer(packet) => {
                if packet.remove {
                    if let Some(id) = packet.id {
                        self.remove(id);
                    }
                    true
                } else {
                    let Some(id) = packet.id else { return false };
                    let Some(dimension) = packet.dimension.as_deref() else {
                        return false;
                    };
                    self.update(id, packet.x, packet.y, packet.z, dimension);
                    true
                }
            }
            PacketEffect::ResetTrackedPlayers => {
                self.reset();
                true
            }
            _ => false,
        }
    }
}

/// `xaero/common/server/radar/tracker/SyncedPlayerTrackerSystemManager`:
/// systems are keyed by the string used by the team-mod integration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncedPlayerTrackerSystemManager {
    systems: BTreeMap<String, crate::SyncedTrackerSystem>,
}

impl SyncedPlayerTrackerSystemManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// `register(String, ISyncedPlayerTrackerSystem)`.
    pub fn register(&mut self, name: impl Into<String>, system: crate::SyncedTrackerSystem) {
        self.systems.insert(name.into(), system);
    }

    /// `getSystems()`.
    pub fn get_systems(&self) -> &BTreeMap<String, crate::SyncedTrackerSystem> {
        &self.systems
    }

    pub fn get(&self, name: &str) -> Option<crate::SyncedTrackerSystem> {
        self.systems.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TrackedPlayerPacket, TrackerResetPacket};

    fn id() -> Uuid {
        Uuid::parse("069a79f4-44e9-4726-a5be-fca90e38aaf5").unwrap()
    }

    #[test]
    fn synced_player_accessors_and_matching() {
        let mut player = SyncedTrackedPlayer::new(id(), 1.0, 64.0, 2.0, "minecraft:overworld");
        assert_eq!(player.id(), id());
        assert_eq!((player.x(), player.y(), player.z()), (1.0, 64.0, 2.0));
        assert_eq!(player.dimension(), "minecraft:overworld");

        assert!(player.matches_enough(1.5, 64.0, 2.5, "minecraft:overworld", 0.6));
        assert!(!player.matches_enough(1.5, 64.0, 2.5, "minecraft:the_nether", 0.6));
        assert!(!player.matches_enough(3.0, 64.0, 2.0, "minecraft:overworld", 0.6));

        player.set_pos(3.0, 4.0, 5.0);
        player.set_dimension("minecraft:the_end");
        let copy = player.clone();
        let mut target = SyncedTrackedPlayer::new(id(), 0.0, 0.0, 0.0, "minecraft:overworld");
        target.copy_from(&copy);
        assert_eq!(target, player);
    }

    #[test]
    fn client_manager_updates_removes_and_resets() {
        let mut manager = ClientSyncedTrackedPlayerManager::new();
        manager.update(id(), 1.0, 2.0, 3.0, "minecraft:overworld");
        assert_eq!(manager.count(), 1);
        manager.update(id(), 4.0, 5.0, 6.0, "minecraft:the_nether");
        assert_eq!(manager.count(), 1);
        assert_eq!(
            (
                manager.get(id()).unwrap().x(),
                manager.get(id()).unwrap().dimension()
            ),
            (4.0, "minecraft:the_nether")
        );
        manager.remove(id());
        assert!(manager.get_players().next().is_none());
        manager.update(id(), 1.0, 2.0, 3.0, "minecraft:overworld");
        manager.reset();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn packet_effects_are_applied_to_the_manager() {
        let mut manager = ClientSyncedTrackedPlayerManager::new();
        let update = TrackedPlayerPacket::update(
            id(), 1.0, 2.0, 3.0, "minecraft:overworld", 3,
        );
        let effect = crate::PacketKind::TrackedPlayer(update)
            .effect(crate::Side::Client)
            .unwrap();
        assert!(manager.apply(&effect));
        assert_eq!(manager.count(), 1);

        let removal = TrackedPlayerPacket::remove(id(), 3);
        let effect = crate::PacketKind::TrackedPlayer(removal)
            .effect(crate::Side::Client)
            .unwrap();
        assert!(manager.apply(&effect));
        assert_eq!(manager.count(), 0);

        let effect = crate::PacketKind::TrackerReset(TrackerResetPacket)
            .effect(crate::Side::Client)
            .unwrap();
        assert!(manager.apply(&effect));
    }

    #[test]
    fn tracker_system_manager_keeps_registered_systems() {
        let mut manager = SyncedPlayerTrackerSystemManager::new();
        manager.register("ftb_teams", crate::SyncedTrackerSystem::FtbTeams);
        manager.register("argonauts", crate::SyncedTrackerSystem::Argonauts);
        assert_eq!(manager.get_systems().len(), 2);
        assert_eq!(
            manager.get("ftb_teams"),
            Some(crate::SyncedTrackerSystem::FtbTeams)
        );
    }
}
