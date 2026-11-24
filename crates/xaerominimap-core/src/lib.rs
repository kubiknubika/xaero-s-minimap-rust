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
//! | `xaero/hud/minimap/world/MinimapDimensionHelper` | [`world::MinimapDimensionHelper`] |
//! | `xaero/common/minimap/region/MinimapTile` | [`tile::MinimapTile`] |
//! | `xaero/hud/minimap/radar/state/RadarState` / `RadarList` | [`radar::RadarState`] / [`radar::RadarList`] |

pub mod path;
pub mod radar;
pub mod tile;
pub mod waypoints;
pub mod world;

pub use path::XaeroPath;
pub use tile::MinimapTile;
pub use waypoints::{
    sort_by_distance, sort_waypoints, Waypoint, WaypointColor, WaypointIO, WaypointPurpose,
    WaypointSet, WaypointUtil, WaypointVisibilityType, WaypointsSort,
};
pub use world::{
    DimensionKey, MinimapDimensionHelper, MinimapWorld, MinimapWorldContainer,
    MinimapWorldManager, MinimapWorldRootContainer, MinimapWorldState, RootConfig,
};
pub use radar::{RadarEntity, RadarList, RadarState};
