//! Rendering core of the Xaero's Minimap Rust rewrite
//! (mirrors `xaero/common/minimap/render` and the waypoint/radar renderers of
//! the original mod).
//!
//! The Java mod draws straight onto Minecraft's GL context; this crate
//! abstracts the drawing surface behind the [`Canvas`] trait so the map math
//! is testable, and implements [`MinimapRenderer`] on top of it.

pub mod canvas;
pub mod highlight;
pub mod minimap;
pub mod radar_icon_cache;
pub mod radar_icon_prerender;
pub mod radar_icons;
pub mod third_party_waypoints;
pub mod waypoint_render;

pub use canvas::{Canvas, Color, RasterCanvas};
pub use highlight::{ChunkHighlighter, Highlighter, HighlighterRegistry};
pub use minimap::{Camera, MinimapRenderer, TileSource, WaypointMarker};
pub use radar_icon_cache::{
    ArmorHandler, BuiltinVariantResolver, CachedIcon, EntityIconCache, EntityVariantData,
    EquippedItem, Equipment, HorseMarkings, IconArmor, IconCache, IconKey, IconVariant, VariantKind,
};
pub use radar_icon_prerender::{
    FieldIndex, FieldReference, FieldReferenceType, FormPrerenderer, ModelFieldResolver,
    ModelPartRenderTrace, ModelRenderTrace,
};
pub use radar_icons::{
    IconDefinition, IconDefinitionManager, IconForm, IconFormKind, IconFormType, ModelConfig,
    BUILTIN_DEFINITIONS,
};
pub use third_party_waypoints::{ThirdPartyWaypointManager, ThirdPartyWaypoints};
pub use waypoint_render::WaypointRenderInfo;
