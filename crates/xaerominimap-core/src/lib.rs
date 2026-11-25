//! Core domain model of the Xaero's Minimap Rust rewrite.
//!
//! Mirrors the data model of the Java mod (Xaero's Minimap 26.6.2 for Minecraft
//! 26.3):
//!
//! | Java package (original mod) | Rust module |
//! | --- | --- |
//! | `xaero/hud/path/XaeroPath` | [`path::XaeroPath`] |
//! | `xaero/common/minimap/waypoints/Waypoint` | [`waypoints::Waypoint`] |
//! | `xaero/common/minimap/waypoints/WaypointUtil` | [`waypoints::WaypointUtil`] |
//! | `xaero/hud/minimap/waypoint/set/WaypointSet` | [`waypoints::WaypointSet`] |
//! | `xaero/hud/minimap/waypoint/io/WaypointIO` | [`waypoints::WaypointIO`] |
//! | `xaero/hud/minimap/waypoint/WaypointsSort` | [`waypoints::sort_waypoints`] |
//! | `xaero/hud/minimap/world/MinimapWorld` | [`world::MinimapWorld`] |
//! | `xaero/hud/minimap/world/container/MinimapWorldContainer` | [`world::MinimapWorldContainer`] |
//! | `xaero/hud/minimap/world/container/MinimapWorldRootContainer` | [`world::MinimapWorldRootContainer`] |
//! | `xaero/hud/minimap/world/state/MinimapWorldState` | [`world::MinimapWorldState`] |
//! | `xaero/common/pool/MapPool` / `PoolUnit` | [`pool::MapPool`] / [`pool::PoolUnit`] |
//! | `xaero/hud/minimap/world/MinimapDimensionHelper` | [`world::MinimapDimensionHelper`] |
//! | `xaero/common/minimap/region/MinimapTile` | [`tile::MinimapTile`] |
//! | `xaero/hud/minimap/radar/state/RadarState` / `RadarList` | [`radar::RadarState`] / [`radar::RadarList`] |
//! | `xaero/hud/preset` (presets and placement action) | [`preset`] |
//! | `xaero/common/minimap/region/MinimapChunk` | [`chunk::MinimapChunk`] |
//! | `xaero/common/minimap/write/biome/BiomeBlendCalculator` | [`biome_blend`] |
//! | `xaero/common/misc/OptimizedMath.myFloor` | [`optimized_math::my_floor`] |

pub mod biome_blend;
pub mod chunk;
pub mod optimized_math;
pub mod path;
pub mod pool;
pub mod preset;
pub mod radar;
pub mod radar_color;
pub mod tile;
pub mod waypoint_handlers;
pub mod waypoints;
pub mod world;

pub use biome_blend::{blend_biome_color, BiomeBlendOffsets};
pub use chunk::MinimapChunk;
pub use optimized_math::my_floor;
pub use path::XaeroPath;
pub use pool::{MapPool, PoolArg, PoolUnit};
pub use preset::{
    add_builtin_presets, builtin_presets, HudPreset, HudPresetManager, ModulePreset,
    ModuleTransform, PlacementPresetAction, PresetAction,
};
pub use tile::MinimapTile;
pub use waypoint_handlers::{
    DeathpointHandler, DeathpointSettings, DestinationHandler, TemporaryWaypointHandler,
    WaypointCollector,
};
pub use waypoints::{
    sort_by_distance, sort_waypoints, Waypoint, WaypointColor, WaypointIO, WaypointPurpose,
    WaypointSet, WaypointUtil, WaypointVisibilityType, WaypointsSort,
};
pub use world::{
    DimensionKey, MinimapDimensionHelper, MinimapWorld, MinimapWorldContainer,
    MinimapWorldManager, MinimapWorldRootContainer, MinimapWorldState, RootConfig,
};
pub use radar::{RadarEntity, RadarList, RadarState};
pub use radar_color::{
    entity_color, entity_height_fade, radar_color_hex, RadarColor,
};
