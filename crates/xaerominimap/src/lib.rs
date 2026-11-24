//! Facade crate of the Xaero's Minimap Rust rewrite.
//!
//! Re-exports the domain model of the rewrite; see the crate README for the
//! module-by-module mapping to the original Java mod.

pub mod session;

pub use session::MinimapSession;
pub use xaerominimap_core::{
    DimensionKey, MinimapDimensionHelper, MinimapTile, RadarEntity, RadarList, RadarState,
    Waypoint, WaypointColor, WaypointIO, WaypointPurpose, WaypointSet, WaypointUtil,
    WaypointVisibilityType, XaeroPath, MinimapWorld, MinimapWorldContainer,
    MinimapWorldRootContainer, MinimapWorldState,
};
pub use xaerominimap_settings::{LegacyCommonConfigIO, SettingValue, Settings};
