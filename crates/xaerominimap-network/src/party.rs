//! `xaero/common/mods/pac/party` — Open Parties and Claims party tracking.
//!
//! `OPACPlayerTrackerSystem` delegates the ally iterator to
//! `SupportOpenPartiesAndClaims.getAllyIterator()` and hands
//! `OPACTrackedPlayerReader` to the generic rendered-tracker machinery. The
//! reader pulls `getPlayerId`, `getX/Y/Z` and `getDimension` from each party
//! member and caches the `ResourceKey<Level>` created in the
//! `Registries.DIMENSION` registry. The Rust port models the identifiers as
//! strings and caches them by value.

use crate::{SyncedTrackedPlayer, Uuid};
use std::collections::HashMap;
use std::sync::Mutex;

/// One ally supplied by OPAC's party API (`IPartyMemberDynamicInfoSyncableAPI`).
pub trait PartyMember {
    fn id(&self) -> Uuid;
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn z(&self) -> f64;
    /// The dimension identifier from `getDimension()`.
    fn dimension(&self) -> Option<&str>;
}

/// A plain snapshot used by tests and by mixin bridges that read OPAC packets.
#[derive(Debug, Clone, PartialEq)]
pub struct PartyMemberSnapshot {
    pub id: Uuid,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub dimension: Option<String>,
}

impl PartyMember for PartyMemberSnapshot {
    fn id(&self) -> Uuid {
        self.id
    }
    fn x(&self) -> f64 {
        self.x
    }
    fn y(&self) -> f64 {
        self.y
    }
    fn z(&self) -> f64 {
        self.z
    }
    fn dimension(&self) -> Option<&str> {
        self.dimension.as_deref()
    }
}

/// `xaero/common/mods/pac/party/OPACTrackedPlayerReader`.
///
/// The Java reader keeps `Map<Identifier, ResourceKey<Level>>` so repeated
/// members in the same dimension share one key object. Here the same cache is
/// a `HashMap<String, String>` holding the resolved dimension path.
#[derive(Debug, Default)]
pub struct OpacTrackedPlayerReader {
    dimension_key_cache: Mutex<HashMap<String, String>>,
}

impl OpacTrackedPlayerReader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves a party member's dimension identifier to the stable key used
    /// by the port's tracked-player model (`Registries.DIMENSION.create`).
    pub fn dimension_key(&self, dimension: &str) -> String {
        let mut cache = self.dimension_key_cache.lock().unwrap();
        cache
            .entry(dimension.to_string())
            .or_insert_with(|| dimension.to_string())
            .clone()
    }

    pub fn cache_size(&self) -> usize {
        self.dimension_key_cache.lock().unwrap().len()
    }

    pub fn read<M: PartyMember>(&self, member: &M) -> SyncedTrackedPlayer {
        SyncedTrackedPlayer::new(
            member.id(),
            member.x(),
            member.y(),
            member.z(),
            member
                .dimension()
                .map(|dimension| self.dimension_key(dimension))
                .unwrap_or_default(),
        )
    }
}

/// `OPACPlayerTrackerSystem`: the allies are snapshotted at construction
/// (`getAllyIterator()` on the Java side) and exposed through the reader.
#[derive(Debug)]
pub struct OpacPlayerTrackerSystem<M> {
    allies: Vec<M>,
    reader: OpacTrackedPlayerReader,
}

impl<M> OpacPlayerTrackerSystem<M> {
    pub fn new(allies: Vec<M>) -> Self {
        OpacPlayerTrackerSystem {
            allies,
            reader: OpacTrackedPlayerReader::new(),
        }
    }

    pub fn reader(&self) -> &OpacTrackedPlayerReader {
        &self.reader
    }

    /// The Java `getTrackedPlayerIterator()` result, as a slice.
    pub fn allies(&self) -> &[M] {
        &self.allies
    }

    pub fn is_empty(&self) -> bool {
        self.allies.is_empty()
    }
}

impl<M: PartyMember> OpacPlayerTrackerSystem<M> {
    /// Applies the reader to every ally, as the rendered-tracker manager does.
    pub fn tracked_players(&self) -> Vec<SyncedTrackedPlayer> {
        self.allies.iter().map(|ally| self.reader.read(ally)).collect()
    }
}

/// The shared shape of `IRenderedPlayerTracker` for OPAC's system.
pub trait PartyTracker {
    fn reader(&self) -> &OpacTrackedPlayerReader;
    fn tracked_players(&self) -> Vec<SyncedTrackedPlayer>;
}

impl<M: PartyMember> PartyTracker for OpacPlayerTrackerSystem<M> {
    fn reader(&self) -> &OpacTrackedPlayerReader {
        self.reader()
    }

    fn tracked_players(&self) -> Vec<SyncedTrackedPlayer> {
        self.tracked_players()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(name: &str, dimension: Option<&str>) -> PartyMemberSnapshot {
        PartyMemberSnapshot {
            id: Uuid::parse(name).unwrap(),
            x: 1.0,
            y: 64.0,
            z: -3.0,
            dimension: dimension.map(str::to_string),
        }
    }

    #[test]
    fn reader_extracts_the_opac_member_fields() {
        let system = OpacPlayerTrackerSystem::new(vec![member(
            "00000000-0000-0000-0000-000000000001",
            Some("minecraft:the_nether"),
        )]);
        let tracked = system.tracked_players();
        assert_eq!(tracked.len(), 1);
        assert_eq!(tracked[0].id(), Uuid::parse("00000000-0000-0000-0000-000000000001").unwrap());
        assert_eq!(tracked[0].x(), 1.0);
        assert_eq!(tracked[0].y(), 64.0);
        assert_eq!(tracked[0].z(), -3.0);
        assert_eq!(tracked[0].dimension(), "minecraft:the_nether");
    }

    #[test]
    fn dimension_keys_are_cached_across_members() {
        let reader = OpacTrackedPlayerReader::new();
        let first = reader.dimension_key("minecraft:overworld");
        let second = reader.dimension_key("minecraft:overworld");
        assert_eq!(first, second);
        assert_eq!(reader.cache_size(), 1);
        assert_ne!(reader.dimension_key("minecraft:the_end"), first);
        assert_eq!(reader.cache_size(), 2);
    }

    #[test]
    fn members_without_a_dimension_get_an_empty_key() {
        let system = OpacPlayerTrackerSystem::new(vec![member(
            "00000000-0000-0000-0000-000000000002",
            None,
        )]);
        assert_eq!(system.tracked_players()[0].dimension(), "");
        assert_eq!(system.reader().cache_size(), 0);
        assert!(!system.is_empty());
    }

    #[test]
    fn empty_system_has_no_tracked_players() {
        let system = OpacPlayerTrackerSystem::<PartyMemberSnapshot>::new(Vec::new());
        assert!(system.is_empty());
        assert!(system.tracked_players().is_empty());
    }
}
