//! Packet registration and codecs of the Xaero's Minimap Rust rewrite
//! (`xaero/hud/packet` and `xaero/common/server/level` in the Java mod).
//!
//! Five packets are registered by `MinimapPacketRegister`, with ids 0–4. The
//! Java code uses `FriendlyByteBuf` and NBT `CompoundTag`s; this crate uses a
//! small byte buffer ([`codec`]) and the NBT subset ([`nbt`]) the packets
//! actually need, so the wire format stays identical.
//!
//! | Java class | Rust type |
//! | --- | --- |
//! | `xaero/common/server/level/LevelMapProperties` | [`LevelMapProperties`] |
//! | `xaero/hud/packet/basic/HandshakePacket` | [`HandshakePacket`] |
//! | `xaero/hud/packet/tracker/ClientboundTrackedPlayerPacket` | [`TrackedPlayerPacket`] |
//! | `xaero/hud/packet/tracker/ClientboundPlayerTrackerResetPacket` | [`TrackerResetPacket`] |
//! | `xaero/hud/packet/basic/ClientboundRulesPacket` | [`RulesPacket`] |

pub mod codec;
pub mod party;
pub mod config_sync;
pub mod nbt;
pub mod world_data;
pub mod server_mods;
pub mod tracked_players;

pub use codec::{PacketReader, PacketWriter};
pub use config_sync::{info_display_from_nbt, info_display_to_nbt};
pub use nbt::NbtValue;
pub use world_data::{ClientWorldData, ClientWorldDataHelper};
pub use server_mods::{
    is_position_sync_allowed, ServerMod, ServerModSupport, ServerPlayerOpacData,
    SupportWorldMap, SyncedTrackerSystem,
};
pub use party::{
    OpacPlayerTrackerSystem, OpacTrackedPlayerReader, PartyMember, PartyMemberSnapshot,
    PartyTracker,
};
pub use tracked_players::{
    ClientSyncedTrackedPlayerManager, SyncedPlayerTrackerSystemManager, SyncedTrackedPlayer,
};

use nbt::{get_bool, get_double, get_string, get_uuid, put_bool, put_double, put_string};

/// `HandshakePacket.NETWORK_COMPATIBILITY`.
pub const NETWORK_COMPATIBILITY: i32 = 3;

/// `LevelMapPropertiesIO.FILE_NAME`.
pub const LEVEL_MAP_PROPERTIES_FILE: &str = "xaeromap.txt";

/// A `java.util.UUID`, stored either as an int[4] (`XaeroNbtUtils.putUUID`)
/// or a long[2] (`putUUIDAsLongArray`) in the packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uuid {
    pub most_significant_bits: u64,
    pub least_significant_bits: u64,
}

impl Uuid {
    pub fn new(most_significant_bits: u64, least_significant_bits: u64) -> Self {
        Uuid {
            most_significant_bits,
            least_significant_bits,
        }
    }

    /// `UUIDUtil.uuidFromIntArray`.
    pub fn from_int_array(values: &[i32]) -> Option<Self> {
        if values.len() != 4 {
            return None;
        }
        let high = ((values[0] as u32) as u64) << 32 | (values[1] as u32) as u64;
        let low = ((values[2] as u32) as u64) << 32 | (values[3] as u32) as u64;
        Some(Uuid::new(high, low))
    }

    /// `UUIDUtil.uuidToIntArray`.
    pub fn to_int_array(self) -> [i32; 4] {
        [
            (self.most_significant_bits >> 32) as i32,
            self.most_significant_bits as i32,
            (self.least_significant_bits >> 32) as i32,
            self.least_significant_bits as i32,
        ]
    }

    pub fn from_long_array(values: &[i64]) -> Option<Self> {
        if values.len() != 2 {
            return None;
        }
        Some(Uuid::new(values[0] as u64, values[1] as u64))
    }

    pub fn to_long_array(self) -> [i64; 2] {
        [self.most_significant_bits as i64, self.least_significant_bits as i64]
    }

    /// Parses the canonical hyphenated form.
    pub fn parse(text: &str) -> Option<Self> {
        let hex: String = text.chars().filter(|c| *c != '-').collect();
        if hex.len() != 32 {
            return None;
        }
        let high = u64::from_str_radix(&hex[..16], 16).ok()?;
        let low = u64::from_str_radix(&hex[16..], 16).ok()?;
        Some(Uuid::new(high, low))
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits = self.most_significant_bits;
        let low = self.least_significant_bits;
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (bits >> 32) as u32,
            (bits >> 16) as u16,
            bits as u16,
            (low >> 48) as u16,
            low & 0xffff_ffff_ffff
        )
    }
}

/// Registered packet ids in `MinimapPacketRegister.register` order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PacketId {
    LevelMapProperties = 0,
    Handshake = 1,
    TrackedPlayer = 2,
    TrackerReset = 3,
    Rules = 4,
}

impl PacketId {
    pub const ALL: [PacketId; 5] = [
        PacketId::LevelMapProperties,
        PacketId::Handshake,
        PacketId::TrackedPlayer,
        PacketId::TrackerReset,
        PacketId::Rules,
    ];

    pub fn from_u8(value: u8) -> Option<PacketId> {
        PacketId::ALL.into_iter().find(|id| *id as u8 == value)
    }

    /// Names used by the Java packet classes.
    pub fn name(self) -> &'static str {
        match self {
            PacketId::LevelMapProperties => "LevelMapProperties",
            PacketId::Handshake => "HandshakePacket",
            PacketId::TrackedPlayer => "ClientboundTrackedPlayerPacket",
            PacketId::TrackerReset => "ClientboundPlayerTrackerResetPacket",
            PacketId::Rules => "ClientboundRulesPacket",
        }
    }
}

/// A packet type that knows its id and wire representation.
pub trait Packet: Sized {
    const ID: PacketId;

    fn encode(&self, out: &mut PacketWriter);
    fn decode(input: &mut PacketReader<'_>) -> Result<Self, String>;
}

/// `LevelMapProperties` sent by the server. Only `id` is written to the
/// buffer; `usable` is server-side configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelMapProperties {
    pub id: i32,
    pub usable: bool,
}

impl LevelMapProperties {
    pub fn new(id: i32) -> Self {
        LevelMapProperties { id, usable: false }
    }

    /// `LevelMapProperties.write(PrintWriter)` / `read(BufferedReader)`: the
    /// `xaeromap.txt` line format is `id:<number>`.
    pub fn to_line(&self) -> String {
        format!("id:{}", self.id)
    }

    pub fn from_line(line: &str) -> Option<Self> {
        let (key, value) = line.split_once(':')?;
        if key != "id" {
            return None;
        }
        Some(LevelMapProperties::new(value.trim().parse().ok()?))
    }

    /// `LevelMapPropertiesIO.load(Path, LevelMapProperties)`.
    pub fn load(path: &std::path::Path, dest: &mut LevelMapProperties) -> std::io::Result<()> {
        let text = std::fs::read_to_string(path)?;
        for line in text.lines() {
            if let Some(properties) = LevelMapProperties::from_line(line) {
                *dest = properties;
            }
        }
        Ok(())
    }

    /// `LevelMapPropertiesIO.save(Path, LevelMapProperties)`.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_line())
    }
}

impl Default for LevelMapProperties {
    fn default() -> Self {
        LevelMapProperties::new(0)
    }
}

impl Packet for LevelMapProperties {
    const ID: PacketId = PacketId::LevelMapProperties;

    fn encode(&self, out: &mut PacketWriter) {
        out.write_i32(self.id);
    }

    fn decode(input: &mut PacketReader<'_>) -> Result<Self, String> {
        Ok(LevelMapProperties::new(input.read_i32()?))
    }
}

/// `HandshakePacket`: the client and server exchange their network versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandshakePacket {
    pub network_version: i32,
}

impl HandshakePacket {
    /// The Java no-arg constructor sends the compatibility version.
    pub fn compatible() -> Self {
        HandshakePacket {
            network_version: NETWORK_COMPATIBILITY,
        }
    }

    pub fn is_compatible(self) -> bool {
        self.network_version == NETWORK_COMPATIBILITY
    }
}

impl Packet for HandshakePacket {
    const ID: PacketId = PacketId::Handshake;

    fn encode(&self, out: &mut PacketWriter) {
        out.write_i32(self.network_version);
    }

    fn decode(input: &mut PacketReader<'_>) -> Result<Self, String> {
        Ok(HandshakePacket {
            network_version: input.read_i32()?,
        })
    }
}

/// `ClientboundRulesPacket`: NBT keys `cm`, `ncm` and `r`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RulesPacket {
    pub allow_cave_mode_on_server: bool,
    pub allow_nether_cave_mode_on_server: bool,
    pub allow_radar_on_server: bool,
}

/// Java's no-packet default: all three rules allowed.
impl Default for RulesPacket {
    fn default() -> Self {
        RulesPacket {
            allow_cave_mode_on_server: true,
            allow_nether_cave_mode_on_server: true,
            allow_radar_on_server: true,
        }
    }
}

impl RulesPacket {
    fn from_entries(entries: &[(String, NbtValue)]) -> RulesPacket {
        RulesPacket {
            allow_cave_mode_on_server: get_bool(entries, "cm"),
            allow_nether_cave_mode_on_server: get_bool(entries, "ncm"),
            allow_radar_on_server: get_bool(entries, "r"),
        }
    }
}

impl Packet for RulesPacket {
    const ID: PacketId = PacketId::Rules;

    fn encode(&self, out: &mut PacketWriter) {
        let mut entries = Vec::new();
        put_bool(&mut entries, "cm", self.allow_cave_mode_on_server);
        put_bool(&mut entries, "ncm", self.allow_nether_cave_mode_on_server);
        put_bool(&mut entries, "r", self.allow_radar_on_server);
        nbt::write_root(&NbtValue::Compound(entries), out);
    }

    fn decode(input: &mut PacketReader<'_>) -> Result<Self, String> {
        let NbtValue::Compound(entries) = nbt::read_root(input)? else {
            return Err("rules packet needs a compound tag".into());
        };
        Ok(RulesPacket::from_entries(&entries))
    }
}

/// `ClientboundTrackedPlayerPacket`. A removal packet only carries the
/// player's uuid; coordinates and dimension are omitted and read back as
/// `0.0` / `None`, matching the Java `remove` branches.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackedPlayerPacket {
    pub remove: bool,
    pub id: Option<Uuid>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub dimension: Option<String>,
    /// The client version the server encodes for; `read` always receives 3.
    pub client_network_version: i32,
}

impl TrackedPlayerPacket {
    pub fn remove(id: Uuid, client_network_version: i32) -> Self {
        TrackedPlayerPacket {
            remove: true,
            id: Some(id),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            dimension: None,
            client_network_version,
        }
    }

    pub fn update(
        id: Uuid,
        x: f64,
        y: f64,
        z: f64,
        dimension: impl Into<String>,
        client_network_version: i32,
    ) -> Self {
        TrackedPlayerPacket {
            remove: false,
            id: Some(id),
            x,
            y,
            z,
            dimension: Some(dimension.into()),
            client_network_version,
        }
    }
}

impl Packet for TrackedPlayerPacket {
    const ID: PacketId = PacketId::TrackedPlayer;

    fn encode(&self, out: &mut PacketWriter) {
        let mut entries = Vec::new();
        put_bool(&mut entries, "r", self.remove);
        if let Some(id) = self.id {
            // Compatibility: versions before 3 stored UUIDs as long arrays.
            if self.client_network_version < NETWORK_COMPATIBILITY {
                entries.push(("i".into(), NbtValue::LongArray(id.to_long_array().to_vec())));
            } else {
                entries.push(("i".into(), NbtValue::IntArray(id.to_int_array().to_vec())));
            }
        }
        if !self.remove {
            put_double(&mut entries, "x", self.x);
            put_double(&mut entries, "y", self.y);
            put_double(&mut entries, "z", self.z);
            if let Some(dimension) = &self.dimension {
                put_string(&mut entries, "d", dimension);
            }
        }
        nbt::write_root(&NbtValue::Compound(entries), out);
    }

    fn decode(input: &mut PacketReader<'_>) -> Result<Self, String> {
        let NbtValue::Compound(entries) = nbt::read_root(input)? else {
            return Err("tracked player packet needs a compound tag".into());
        };
        let remove = get_bool(&entries, "r");
        Ok(TrackedPlayerPacket {
            remove,
            id: get_uuid(&entries, "i"),
            x: if remove { 0.0 } else { get_double(&entries, "x") },
            y: if remove { 0.0 } else { get_double(&entries, "y") },
            z: if remove { 0.0 } else { get_double(&entries, "z") },
            dimension: if remove {
                None
            } else {
                get_string(&entries, "d").map(str::to_string)
            },
            // The Java reader constructs packets with `NETWORK_COMPATIBILITY`.
            client_network_version: NETWORK_COMPATIBILITY,
        })
    }
}

/// `ClientboundPlayerTrackerResetPacket`: an empty packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrackerResetPacket;

impl Packet for TrackerResetPacket {
    const ID: PacketId = PacketId::TrackerReset;

    fn encode(&self, _out: &mut PacketWriter) {}

    fn decode(_input: &mut PacketReader<'_>) -> Result<Self, String> {
        Ok(TrackerResetPacket)
    }
}

/// One of the five registered packets, with its id byte on the wire.
#[derive(Debug, Clone, PartialEq)]
pub enum PacketKind {
    LevelMapProperties(LevelMapProperties),
    Handshake(HandshakePacket),
    TrackedPlayer(TrackedPlayerPacket),
    TrackerReset(TrackerResetPacket),
    Rules(RulesPacket),
}

impl PacketKind {
    pub fn id(&self) -> PacketId {
        match self {
            PacketKind::LevelMapProperties(_) => PacketId::LevelMapProperties,
            PacketKind::Handshake(_) => PacketId::Handshake,
            PacketKind::TrackedPlayer(_) => PacketId::TrackedPlayer,
            PacketKind::TrackerReset(_) => PacketId::TrackerReset,
            PacketKind::Rules(_) => PacketId::Rules,
        }
    }

    /// Writes the id byte followed by the packet payload.
    pub fn encode(&self, out: &mut PacketWriter) {
        out.write_u8(self.id() as u8);
        match self {
            PacketKind::LevelMapProperties(packet) => packet.encode(out),
            PacketKind::Handshake(packet) => packet.encode(out),
            PacketKind::TrackedPlayer(packet) => packet.encode(out),
            PacketKind::TrackerReset(packet) => packet.encode(out),
            PacketKind::Rules(packet) => packet.encode(out),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new();
        self.encode(&mut writer);
        writer.into_bytes()
    }

    pub fn decode(input: &mut PacketReader<'_>) -> Result<Self, String> {
        let id = PacketId::from_u8(input.read_u8()?).ok_or_else(|| "unknown packet id".to_string())?;
        Ok(match id {
            PacketId::LevelMapProperties => {
                PacketKind::LevelMapProperties(LevelMapProperties::decode(input)?)
            }
            PacketId::Handshake => PacketKind::Handshake(HandshakePacket::decode(input)?),
            PacketId::TrackedPlayer => {
                PacketKind::TrackedPlayer(TrackedPlayerPacket::decode(input)?)
            }
            PacketId::TrackerReset => PacketKind::TrackerReset(TrackerResetPacket::decode(input)?),
            PacketId::Rules => PacketKind::Rules(RulesPacket::decode(input)?),
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut reader = PacketReader::new(bytes);
        let packet = PacketKind::decode(&mut reader)?;
        if !reader.is_empty() {
            return Err("trailing bytes after packet".into());
        }
        Ok(packet)
    }

    /// The side a packet is sent to, used by the packet handlers.
    pub fn target(&self) -> Side {
        match self {
            PacketKind::Handshake(_) => Side::Both,
            _ => Side::Client,
        }
    }
}

/// Which handler receives a packet in the Java registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Client,
    Server,
    Both,
}

/// The effect the Java packet handlers apply on the receiving side.
#[derive(Debug, Clone, PartialEq)]
pub enum PacketEffect {
    /// `ServerPlayerData.setClientModNetworkVersion`.
    SetClientNetworkVersion(i32),
    /// `MinimapProcessor.setServerModNetworkVersion`.
    SetServerNetworkVersion(i32),
    /// `MinimapClientWorldData.setSyncedRules`.
    SetSyncedRules(RulesPacket),
    /// `ClientSyncedTrackedPlayerManager.update`.
    UpdateTrackedPlayer(TrackedPlayerPacket),
    /// `ClientSyncedTrackedPlayerManager.reset`.
    ResetTrackedPlayers,
    /// `MinimapWorldStateUpdater.onServerLevelId`.
    SetServerLevelId(i32),
}

impl PacketKind {
    /// Resolves the handler effect for a side (`null` handlers in the Java
    /// registration simply return `None`).
    pub fn effect(&self, side: Side) -> Option<PacketEffect> {
        match (self, side) {
            (PacketKind::Handshake(packet), Side::Server | Side::Both) => {
                Some(PacketEffect::SetClientNetworkVersion(packet.network_version))
            }
            (PacketKind::Handshake(packet), Side::Client) => {
                Some(PacketEffect::SetServerNetworkVersion(packet.network_version))
            }
            (PacketKind::Rules(packet), Side::Client | Side::Both) => {
                Some(PacketEffect::SetSyncedRules(*packet))
            }
            (PacketKind::TrackedPlayer(packet), Side::Client | Side::Both) => {
                Some(PacketEffect::UpdateTrackedPlayer(packet.clone()))
            }
            (PacketKind::TrackerReset(_), Side::Client | Side::Both) => {
                Some(PacketEffect::ResetTrackedPlayers)
            }
            (PacketKind::LevelMapProperties(packet), Side::Client | Side::Both) => {
                Some(PacketEffect::SetServerLevelId(packet.id))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_ids_match_the_java_registration() {
        assert_eq!(PacketId::from_u8(0), Some(PacketId::LevelMapProperties));
        assert_eq!(PacketId::from_u8(4), Some(PacketId::Rules));
        assert_eq!(PacketId::from_u8(5), None);
        assert_eq!(NETWORK_COMPATIBILITY, 3);
    }

    #[test]
    fn handshake_and_properties_are_plain_ints() {
        let packet = PacketKind::Handshake(HandshakePacket::compatible());
        assert_eq!(packet.to_bytes(), vec![1, 0, 0, 0, 3]);
        assert_eq!(PacketKind::from_bytes(&packet.to_bytes()).unwrap(), packet);

        let packet = PacketKind::LevelMapProperties(LevelMapProperties::new(42));
        assert_eq!(packet.to_bytes(), vec![0, 0, 0, 0, 42]);
        assert_eq!(PacketKind::from_bytes(&packet.to_bytes()).unwrap(), packet);
    }

    #[test]
    fn rules_round_trip_uses_the_java_nbt_keys() {
        let rules = RulesPacket {
            allow_cave_mode_on_server: true,
            allow_nether_cave_mode_on_server: false,
            allow_radar_on_server: true,
        };
        let bytes = PacketKind::Rules(rules).to_bytes();
        assert_eq!(bytes[0], 4);
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("cm"));
        assert!(text.contains("ncm"));
        assert!(text.contains('r'));
        let decoded = PacketKind::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, PacketKind::Rules(rules));
    }

    #[test]
    fn tracked_player_uuid_form_depends_on_client_version() {
        let id = Uuid::parse("069a79f4-44e9-4726-a5be-fca90e38aaf5").unwrap();
        let old = TrackedPlayerPacket::update(id, 1.0, 2.0, 3.0, "minecraft:overworld", 2);
        let old_bytes = PacketKind::TrackedPlayer(old.clone()).to_bytes();
        assert!(String::from_utf8_lossy(&old_bytes).len() > 0);
        assert_eq!(PacketKind::from_bytes(&old_bytes).unwrap(), PacketKind::TrackedPlayer(TrackedPlayerPacket {
            client_network_version: NETWORK_COMPATIBILITY,
            ..old
        }));

        let current = TrackedPlayerPacket::update(id, 1.0, 2.0, 3.0, "minecraft:overworld", 3);
        let bytes = PacketKind::TrackedPlayer(current.clone()).to_bytes();
        let decoded = PacketKind::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, PacketKind::TrackedPlayer(current));

        let removal = PacketKind::TrackedPlayer(TrackedPlayerPacket::remove(id, 3));
        let bytes = removal.to_bytes();
        // Removal packets omit coordinates and the dimension.
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("minecraft:overworld"));
        assert_eq!(PacketKind::from_bytes(&bytes).unwrap(), removal);
    }

    #[test]
    fn uuid_array_forms_round_trip() {
        let id = Uuid::parse("00000000-0000-0000-0000-000000000001").unwrap();
        assert_eq!(Uuid::from_int_array(&id.to_int_array()), Some(id));
        assert_eq!(Uuid::from_long_array(&id.to_long_array()), Some(id));
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000001");
    }

    #[test]
    fn level_map_properties_file_format() {
        let properties = LevelMapProperties::new(-12);
        assert_eq!(properties.to_line(), "id:-12");
        assert_eq!(LevelMapProperties::from_line("id:7"), Some(LevelMapProperties::new(7)));
        assert_eq!(LevelMapProperties::from_line("other:7"), None);
    }

    #[test]
    fn handlers_resolve_effects() {
        let rules = PacketKind::Rules(RulesPacket {
            allow_cave_mode_on_server: false,
            allow_nether_cave_mode_on_server: false,
            allow_radar_on_server: true,
        });
        assert!(matches!(
            rules.effect(Side::Client),
            Some(PacketEffect::SetSyncedRules(_))
        ));
        assert_eq!(rules.effect(Side::Server), None);

        let handshake = PacketKind::Handshake(HandshakePacket::compatible());
        assert_eq!(
            handshake.effect(Side::Server),
            Some(PacketEffect::SetClientNetworkVersion(3))
        );
        assert_eq!(
            handshake.effect(Side::Client),
            Some(PacketEffect::SetServerNetworkVersion(3))
        );
        assert_eq!(
            PacketKind::TrackerReset(TrackerResetPacket).effect(Side::Client),
            Some(PacketEffect::ResetTrackedPlayers)
        );
    }
}
