//! Rendering core of the Xaero's Minimap Rust rewrite
//! (mirrors `xaero/common/minimap/render` and the waypoint/radar renderers of
//! the original mod).
//!
//! The Java mod draws straight onto Minecraft's GL context; this crate
//! abstracts the drawing surface behind the [`Canvas`] trait so the map math
//! is testable, and implements [`MinimapRenderer`] on top of it.

pub mod canvas;
pub mod minimap;
pub mod waypoint_render;

pub use canvas::{Canvas, Color, RasterCanvas};
pub use minimap::{Camera, MinimapRenderer, TileSource, WaypointMarker};
pub use waypoint_render::WaypointRenderInfo;
