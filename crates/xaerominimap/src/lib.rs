//! Facade crate of the Xaero's Minimap Rust rewrite.
//!
//! Re-exports the domain model of the rewrite; see the crate README for the
//! module-by-module mapping to the original Java mod.

pub mod session;

pub use session::MinimapSession;
pub use xaerominimap_controls as controls;
pub use xaerominimap_controls::{
    ControlEvent, ControlEventKind, ControlsRegister, KeyFunction, KeyMapping,
    KeyMappingController, KeyMappingControllerManager, KeyMappingTickHandler, KeyStateSource,
    MinimapAction, MinimapKeyMappings, CATEGORY,
};
pub use xaerominimap_network as network;
pub use xaerominimap_network::{
    info_display_from_nbt, info_display_to_nbt, is_position_sync_allowed, HandshakePacket,
    LevelMapProperties, NbtValue, PacketEffect, PacketId,
    PacketKind, PacketReader, PacketWriter, RulesPacket, ServerMod, ServerModSupport,
    ServerPlayerOpacData, Side, SupportWorldMap, SyncedTrackerSystem,
    ClientSyncedTrackedPlayerManager, ClientWorldData, ClientWorldDataHelper,
    OpacPlayerTrackerSystem, OpacTrackedPlayerReader, PartyMember, PartyMemberSnapshot,
    PartyTracker,
    SyncedPlayerTrackerSystemManager, SyncedTrackedPlayer,
    TrackerResetPacket, TrackedPlayerPacket, Uuid, LEVEL_MAP_PROPERTIES_FILE,
    NETWORK_COMPATIBILITY,
};
pub use xaerominimap_core::{
    add_builtin_presets, blend_biome_color, builtin_presets, entity_color, entity_height_fade,
    my_floor, radar_color_hex,
    BiomeBlendOffsets, DeathpointHandler, DeathpointSettings,
    DestinationHandler, DimensionKey, HudPreset, HudPresetManager, MinimapChunk,
    MapPool, MinimapDimensionHelper, ModulePreset, ModuleTransform, PlacementPresetAction, PoolArg,
    PoolUnit, PresetAction,
    RadarColor,
    MinimapTile, RadarEntity, RadarList, RadarState, TemporaryWaypointHandler, Waypoint,
    WaypointCollector, WaypointColor, WaypointIO, WaypointPurpose, WaypointSet, WaypointUtil,
    WaypointVisibilityType, XaeroPath, MinimapWorld, MinimapWorldContainer,
    MinimapWorldRootContainer, MinimapWorldState,
};
pub use xaerominimap_render::{
    ArmorHandler, BuiltinVariantResolver, Camera, Canvas, CachedIcon, Color, EntityIconCache,
    EntityVariantData, EquippedItem, Equipment, FieldIndex, FieldReference, FieldReferenceType,
    ChunkHighlighter, FormPrerenderer, Highlighter, HighlighterRegistry, HorseMarkings, IconArmor,
    IconCache, IconDefinition, IconDefinitionManager, IconForm, IconFormKind, IconKey, IconVariant,
    IconFormType, MinimapRenderer, ModelConfig,
    ModelFieldResolver, ModelPartRenderTrace, ModelRenderTrace, RasterCanvas,
    ThirdPartyWaypointManager, ThirdPartyWaypoints, TileSource, VariantKind, WaypointMarker,
    WaypointRenderInfo,
};
pub use xaerominimap_settings::{
    adapt_display_order, all_settings as radar_setting_definitions,
    default_settings as default_radar_settings, decode_info_display, encode_info_display,
    for_each_builtin_info_display, setting_by_id as radar_setting_by_id, BuiltInInfoDisplay,
    BUILTIN_INFO_DISPLAYS, ConfigUiBinding, ConfigUiType, ConfigUiTypeManager, InfoDisplayCodec,
    InfoDisplayWidgetKind,
    InfoDisplayConfigData, InfoDisplayManagerConfigData, InfoDisplayManagerConfigDataBuilder,
    InfoDisplayState,
    LegacyCommonConfigIO, RadarCategory, RadarCategoryData, RadarCategoryManager, RadarEntityInfo,
    FilterObjectCategoryData, RadarHardRule, RadarListRuleType, RadarSetting, SettingValue,
    Settings, UI_BINDINGS,
};
