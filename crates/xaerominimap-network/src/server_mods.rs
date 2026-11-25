//! Detection of server-side mods the minimap integrates with, ported from
//! `xaero/common/server/mods` (`SupportServerMods` and its per-mod helpers).
//!
//! Java uses `Class.forName` on each integration class to decide which mods
//! are installed. A Rust server has no JVM reflection, so `detect` takes the
//! names of classes/features available in the running server instead.

use std::collections::BTreeSet;

/// Class names probed by `SupportServerMods.check`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ServerMod {
    FtbTeams,
    Argonauts,
    Opac,
    WorldMap,
}

impl ServerMod {
    pub const ALL: [ServerMod; 4] = [
        ServerMod::FtbTeams,
        ServerMod::Argonauts,
        ServerMod::Opac,
        ServerMod::WorldMap,
    ];

    /// `Class.forName` argument in the Java `check` method.
    pub fn detection_class(self) -> &'static str {
        match self {
            ServerMod::FtbTeams => "dev.ftb.mods.ftbteams.api.FTBTeamsAPI",
            ServerMod::Argonauts => "earth.terrarium.argonauts.api.ApiHelper",
            ServerMod::Opac => "xaero.pac.common.server.api.OpenPACServerAPI",
            ServerMod::WorldMap => "xaero.map.WorldMap",
        }
    }
}

/// The synced player tracker system chosen by each team mod.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncedTrackerSystem {
    FtbTeams,
    Argonauts,
}

impl SyncedTrackerSystem {
    pub fn class_name(self) -> &'static str {
        match self {
            SyncedTrackerSystem::FtbTeams => {
                "xaero/common/server/mods/ftbteams/FTBTeamsSyncedPlayerTrackerSystem"
            }
            SyncedTrackerSystem::Argonauts => {
                "xaero/common/server/mods/argonauts/ArgonautsSyncedPlayerTrackerSystem"
            }
        }
    }
}

/// `xaero/common/server/mods/SupportWorldMapServer`. `compatibilityVersion`
/// is copied from `xaero.map.WorldMap.MINIMAP_COMPATIBILITY_VERSION` in Java;
/// since that class belongs to the WorldMap mod, the Rust side passes it in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportWorldMap {
    pub compatibility_version: i32,
}

impl SupportWorldMap {
    /// `supportsTrackedPlayers()` — the Java constant is 22.
    pub const TRACKED_PLAYERS_VERSION: i32 = 22;

    pub fn new(compatibility_version: i32) -> Self {
        SupportWorldMap {
            compatibility_version,
        }
    }

    pub fn supports_tracked_players(&self) -> bool {
        self.compatibility_version >= Self::TRACKED_PLAYERS_VERSION
    }
}

/// `ServerPlayerOpacData`: the two OPAC config values that gate location
/// sharing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerPlayerOpacData {
    pub share_location_with_party: bool,
    pub share_location_with_mutual_allies: bool,
}

/// Relation levels accepted by `SupportOPACServer.isPositionSyncAllowed(int, …)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum OpacRelation {
    MutualAlly = 1,
    Party = 2,
}

/// `SupportOPACServer.isPositionSyncAllowed(int, ServerPlayerData, boolean)`.
pub fn is_position_sync_allowed(
    relation: i32,
    data: &ServerPlayerOpacData,
    check_relation: bool,
) -> bool {
    if !check_relation || relation <= 0 {
        return false;
    }
    if relation == OpacRelation::MutualAlly as i32 {
        return data.share_location_with_mutual_allies;
    }
    if relation > OpacRelation::Party as i32 - 1 {
        return data.share_location_with_party;
    }
    true
}

/// `xaero/common/server/mods/SupportServerMods`: the result of the mod
/// detection pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerModSupport {
    ftb_teams: bool,
    argonauts: bool,
    opac: bool,
    worldmap: Option<SupportWorldMap>,
}

impl ServerModSupport {
    /// `check()` — `available_classes` contains the class names the host
    /// server can resolve. `worldmap_compatibility` is only used when the
    /// WorldMap class is present.
    pub fn detect(available_classes: &BTreeSet<String>, worldmap_compatibility: i32) -> Self {
        let has = |class_name: &str| available_classes.contains(class_name);
        ServerModSupport {
            ftb_teams: has(ServerMod::FtbTeams.detection_class()),
            argonauts: has(ServerMod::Argonauts.detection_class()),
            opac: has(ServerMod::Opac.detection_class()),
            worldmap: has(ServerMod::WorldMap.detection_class())
                .then(|| SupportWorldMap::new(worldmap_compatibility)),
        }
    }

    pub fn has(&self, mod_: ServerMod) -> bool {
        match mod_ {
            ServerMod::FtbTeams => self.ftb_teams,
            ServerMod::Argonauts => self.argonauts,
            ServerMod::Opac => self.opac,
            ServerMod::WorldMap => self.worldmap.is_some(),
        }
    }

    /// `hasFtbTeams()`.
    pub fn has_ftb_teams(&self) -> bool {
        self.ftb_teams
    }

    /// `hasArgonauts()`.
    pub fn has_argonauts(&self) -> bool {
        self.argonauts
    }

    /// `hasOpac()`.
    pub fn has_opac(&self) -> bool {
        self.opac
    }

    /// `hasWorldmap()`.
    pub fn has_worldmap(&self) -> bool {
        self.worldmap.is_some()
    }

    /// The tracker system supplied by the first available team mod, matching
    /// the order in which the Java handlers are registered.
    pub fn synced_tracker_system(&self) -> Option<SyncedTrackerSystem> {
        if self.ftb_teams {
            Some(SyncedTrackerSystem::FtbTeams)
        } else if self.argonauts {
            Some(SyncedTrackerSystem::Argonauts)
        } else {
            None
        }
    }

    /// `getWorldmap()`.
    pub fn worldmap(&self) -> Option<SupportWorldMap> {
        self.worldmap
    }

    /// `registerEvents()` is a no-op in the Java implementation.
    pub fn register_events(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_mods_by_their_java_class_names() {
        let mut classes = BTreeSet::new();
        classes.insert(ServerMod::FtbTeams.detection_class().to_string());
        classes.insert(ServerMod::WorldMap.detection_class().to_string());
        let support = ServerModSupport::detect(&classes, 25);

        assert!(support.has_ftb_teams());
        assert!(support.has_worldmap());
        assert!(!support.has_argonauts());
        assert!(!support.has_opac());
        assert!(support.has(ServerMod::FtbTeams));
        assert_eq!(
            support.synced_tracker_system(),
            Some(SyncedTrackerSystem::FtbTeams)
        );
        assert_eq!(
            support.worldmap(),
            Some(SupportWorldMap::new(25))
        );
        assert!(support.worldmap().unwrap().supports_tracked_players());
        support.register_events();
    }

    #[test]
    fn worldmap_tracked_player_compatibility_boundary() {
        assert!(!SupportWorldMap::new(21).supports_tracked_players());
        assert!(SupportWorldMap::new(22).supports_tracked_players());
    }

    #[test]
    fn argonauts_is_the_fallback_tracker_system() {
        let mut classes = BTreeSet::new();
        classes.insert(ServerMod::Argonauts.detection_class().to_string());
        let support = ServerModSupport::detect(&classes, 0);
        assert_eq!(
            support.synced_tracker_system(),
            Some(SyncedTrackerSystem::Argonauts)
        );
        assert_eq!(
            SyncedTrackerSystem::Argonauts.class_name(),
            "xaero/common/server/mods/argonauts/ArgonautsSyncedPlayerTrackerSystem"
        );
    }

    #[test]
    fn opac_position_sync_respects_both_sharing_settings() {
        let data = ServerPlayerOpacData {
            share_location_with_party: true,
            share_location_with_mutual_allies: false,
        };
        assert!(is_position_sync_allowed(2, &data, true));
        assert!(!is_position_sync_allowed(1, &data, true));
        assert!(!is_position_sync_allowed(2, &data, false));
        assert!(!is_position_sync_allowed(0, &data, true));

        let allies = ServerPlayerOpacData {
            share_location_with_party: false,
            share_location_with_mutual_allies: true,
        };
        assert!(is_position_sync_allowed(1, &allies, true));
        assert!(!is_position_sync_allowed(2, &allies, true));
    }
}
