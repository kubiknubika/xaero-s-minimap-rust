//! Horizon adapter for the Rust port of Xaero's Minimap.
//!
//! Horizon native mods are compile-time source/bundle additions, not runtime
//! plugins. This crate implements the platform's `horizon_hooks` extension
//! points: a HUD [`Layer`](horizon_hooks::hud::Layer) that drives the ported
//! [`MinimapRenderer`] through Horizon's `Canvas`, and a small [`Hooks`]
//! implementation that keeps per-connection state.
//!
//! World/chunk colour lookup and camera updates are supplied by the mod's
//! mixin bundle through [`MinimapState`] and [`TileSource`]; this crate has
//! no dependency on Horizon's engine crates. `ChunkColorGrid` is a ready
//! thread-safe tile source, and `PacketBridge` applies the ported packet
//! effects to the HUD state.

pub mod controls;
pub mod packets;
pub mod settings_page;
pub mod tiles;

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use horizon_hooks::hud::{Canvas as HorizonCanvas, Corner, Layer};
use horizon_hooks::players::{Frame, SpriteStore};
use horizon_hooks::{Hooks, TooLate};
use xaerominimap_core::radar::RadarEntity;
use xaerominimap_render::{Camera, Canvas, TileSource, WaypointMarker};

pub use controls::{
    horizon_key_code, ControlBridge, HorizonControls, HorizonKeyState,
};
pub use packets::PacketBridge;
pub use settings_page::{register_settings_page, MinimapSettingsPage};
pub use xaerominimap_settings::Settings;
pub use xaerominimap_controls::{ControlEvent, ControlEventKind, MinimapAction};
pub use xaerominimap_core::Waypoint;
pub use xaerominimap_render::{Color, MinimapRenderer};
pub use tiles::{ChunkColorGrid, SolidTiles};

/// `Color` → Horizon's packed `0xAARRGGBB`.
fn argb(color: Color) -> u32 {
    ((color.a as u32) << 24)
        | ((color.r as u32) << 16)
        | ((color.g as u32) << 8)
        | color.b as u32
}

/// Horizon's packed `0xAARRGGBB` back to the port's RGBA `Color`.
fn color_from_argb(value: u32) -> Color {
    Color::rgba(
        ((value >> 16) & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        (value & 0xff) as u8,
        ((value >> 24) & 0xff) as u8,
    )
}

/// Adapts Horizon's retained-mode HUD canvas to the port's pixel canvas.
/// Coordinates are offset by the corner position Horizon assigned us.
struct OffsetCanvas<'a> {
    inner: &'a mut dyn HorizonCanvas,
    ox: f32,
    oy: f32,
    width: f32,
    height: f32,
}

impl Canvas for OffsetCanvas<'_> {
    fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        self.fill_rect(x, y, 1, 1, color);
    }

    fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color) {
        if width == 0 || height == 0 {
            return;
        }
        // Clip to the corner box Horizon assigned to us; map drawing walks
        // chunks slightly outside the visible square.
        let x0 = (x as f32).max(0.0);
        let y0 = (y as f32).max(0.0);
        let x1 = (x as f32 + width as f32).min(self.width);
        let y1 = (y as f32 + height as f32).min(self.height);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        self.inner.fill(
            self.ox + x0,
            self.oy + y0,
            x1 - x0,
            y1 - y0,
            argb(color),
        );
    }

    fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
        let mut x0 = x0.round() as i32;
        let mut y0 = y0.round() as i32;
        let x1 = x1.round() as i32;
        let y1 = y1.round() as i32;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_point(&mut self, x: i32, y: i32, color: Color, size: u8) {
        let size = size.max(1) as i32;
        self.fill_rect(
            x - size / 2,
            y - size / 2,
            size as u32,
            size as u32,
            color,
        );
    }
}

/// State shared between the mixin bundle (writes) and the HUD layer (reads).
/// HUD-side view options a mixin/settings page can change at runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapViewConfig {
    pub enabled: bool,
    pub map_size: f32,
    /// World blocks per map pixel.
    pub scale: f32,
    pub corner: Corner,
    pub show_labels: bool,
    pub radar_enabled: bool,
    pub cave_mode: bool,
}

impl Default for MinimapViewConfig {
    fn default() -> Self {
        MinimapViewConfig {
            enabled: true,
            map_size: 160.0,
            scale: 1.0,
            corner: Corner::TopLeft,
            show_labels: true,
            radar_enabled: true,
            cave_mode: false,
        }
    }
}

pub struct MinimapState {
    camera_x: AtomicI32,
    camera_y: AtomicI32,
    camera_z: AtomicI32,
    enabled: AtomicBool,
    map_size_bits: AtomicU32,
    scale_bits: AtomicU32,
    corner: AtomicU8,
    show_labels: AtomicBool,
    radar_enabled: AtomicBool,
    cave_mode: AtomicBool,
    connection: AtomicU64,
    waypoints: Mutex<Vec<WaypointMarker>>,
    radar: Mutex<Vec<RadarEntity>>,
}

impl Default for MinimapState {
    fn default() -> Self {
        let view = MinimapViewConfig::default();
        let state = MinimapState {
            camera_x: AtomicI32::new(0),
            camera_y: AtomicI32::new(0),
            camera_z: AtomicI32::new(0),
            enabled: AtomicBool::new(view.enabled),
            map_size_bits: AtomicU32::new(view.map_size.to_bits()),
            scale_bits: AtomicU32::new(view.scale.to_bits()),
            corner: AtomicU8::new(corner_index(view.corner)),
            show_labels: AtomicBool::new(view.show_labels),
            radar_enabled: AtomicBool::new(view.radar_enabled),
            cave_mode: AtomicBool::new(view.cave_mode),
            connection: AtomicU64::new(0),
            waypoints: Mutex::new(Vec::new()),
            radar: Mutex::new(Vec::new()),
        };
        state.set_view(view);
        state
    }
}

fn corner_index(corner: Corner) -> u8 {
    match corner {
        Corner::TopLeft => 0,
        Corner::TopRight => 1,
        Corner::BottomLeft => 2,
        Corner::BottomRight => 3,
    }
}

fn corner_from_index(index: u8) -> Corner {
    match index {
        1 => Corner::TopRight,
        2 => Corner::BottomLeft,
        3 => Corner::BottomRight,
        _ => Corner::TopLeft,
    }
}

impl MinimapState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates every runtime view option in one call.
    pub fn set_view(&self, view: MinimapViewConfig) {
        self.enabled.store(view.enabled, Ordering::Relaxed);
        self.map_size_bits
            .store(view.map_size.max(1.0).to_bits(), Ordering::Relaxed);
        self.scale_bits
            .store(view.scale.max(0.0).to_bits(), Ordering::Relaxed);
        self.corner
            .store(corner_index(view.corner), Ordering::Relaxed);
        self.show_labels
            .store(view.show_labels, Ordering::Relaxed);
        self.radar_enabled
            .store(view.radar_enabled, Ordering::Relaxed);
        self.cave_mode.store(view.cave_mode, Ordering::Relaxed);
    }

    pub fn view(&self) -> MinimapViewConfig {
        MinimapViewConfig {
            enabled: self.is_enabled(),
            map_size: f32::from_bits(self.map_size_bits.load(Ordering::Relaxed)),
            scale: f32::from_bits(self.scale_bits.load(Ordering::Relaxed)),
            corner: corner_from_index(self.corner.load(Ordering::Relaxed)),
            show_labels: self.show_labels.load(Ordering::Relaxed),
            radar_enabled: self.radar_enabled.load(Ordering::Relaxed),
            cave_mode: self.cave_mode.load(Ordering::Relaxed),
        }
    }

    /// Called by the world/tick mixin with the local player's block position.
    pub fn set_camera(&self, x: i32, y: i32, z: i32) {
        self.camera_x.store(x, Ordering::Relaxed);
        self.camera_y.store(y, Ordering::Relaxed);
        self.camera_z.store(z, Ordering::Relaxed);
    }

    pub fn camera(&self) -> Camera {
        Camera::new(
            self.camera_x.load(Ordering::Relaxed),
            self.camera_y.load(Ordering::Relaxed),
            self.camera_z.load(Ordering::Relaxed),
        )
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_connection(&self, connection: u64) {
        if self.connection.swap(connection, Ordering::Relaxed) != connection {
            self.set_waypoints(Vec::new());
            self.set_radar(Vec::new());
        }
    }

    pub fn connection(&self) -> u64 {
        self.connection.load(Ordering::Relaxed)
    }

    /// Waypoint markers to draw, normally synced from `MinimapSession`.
    pub fn set_waypoints(&self, waypoints: Vec<WaypointMarker>) {
        *self.waypoints.lock().unwrap() = waypoints;
    }

    /// Converts the port's `Waypoint` list into HUD markers. Deleted and
    /// disabled waypoints are skipped, as in the Java minimap element
    /// collector.
    pub fn sync_waypoints(&self, waypoints: &[Waypoint]) {
        let markers = waypoints
            .iter()
            .filter(|waypoint| !waypoint.effectively_deleted && !waypoint.disabled.unwrap_or(false))
            .map(|waypoint| WaypointMarker {
                x: waypoint.x,
                y: waypoint.y,
                z: waypoint.z,
                name: waypoint.name.clone(),
                color: color_from_argb(waypoint.actual_color().hex()),
            })
            .collect();
        self.set_waypoints(markers);
    }

    /// Nearby radar entities to draw.
    pub fn set_radar(&self, entities: Vec<RadarEntity>) {
        *self.radar.lock().unwrap() = entities;
    }

    pub fn radar_entities(&self) -> Vec<RadarEntity> {
        self.radar.lock().unwrap().clone()
    }

    pub fn set_radar_enabled(&self, radar_enabled: bool) {
        self.radar_enabled.store(radar_enabled, Ordering::Relaxed);
    }

    pub fn is_radar_enabled(&self) -> bool {
        self.radar_enabled.load(Ordering::Relaxed)
    }

    pub fn set_cave_mode(&self, cave_mode: bool) {
        self.cave_mode.store(cave_mode, Ordering::Relaxed);
    }

    pub fn is_cave_mode(&self) -> bool {
        self.cave_mode.load(Ordering::Relaxed)
    }
}

/// The HUD layer: draws the map square, waypoints, radar dots and labels in
/// one of Horizon's HUD corners.
pub struct MinimapLayer<S> {
    state: Arc<MinimapState>,
    tiles: S,
    chunk_size: i32,
    radar_color: Color,
    radar_range: f64,
}

impl<S> MinimapLayer<S> {
    pub fn new(
        state: Arc<MinimapState>,
        renderer: MinimapRenderer,
        tiles: S,
        _corner: Corner,
    ) -> Self {
        MinimapLayer {
            state,
            chunk_size: renderer.chunk_size,
            tiles,
            radar_color: Color::rgb(255, 255, 255),
            radar_range: 128.0,
        }
    }

    pub fn with_radar(mut self, color: Color, range: f64) -> Self {
        self.radar_color = color;
        self.radar_range = range;
        self
    }

    pub fn state(&self) -> &Arc<MinimapState> {
        &self.state
    }

    /// The renderer with the current view settings applied.
    fn renderer(&self) -> MinimapRenderer {
        let view = self.state.view();
        MinimapRenderer::new(view.map_size, view.scale, self.chunk_size)
    }
}

impl<S: TileSource + Sync> Layer for MinimapLayer<S> {
    fn draw(&self, canvas: &mut dyn HorizonCanvas) {
        let view = self.state.view();
        if !view.enabled {
            return;
        }
        let renderer = self.renderer();
        let size = renderer.map_size;
        let Some((x, y)) = canvas.place(view.corner, size, size) else {
            return;
        };
        let camera = self.state.camera();
        {
            let mut target = OffsetCanvas {
                inner: canvas,
                ox: x,
                oy: y,
                width: size,
                height: size,
            };
            renderer.draw_map(&mut target, &camera, &self.tiles);

            let waypoints = self.state.waypoints.lock().unwrap();
            renderer.draw_waypoints(&mut target, &camera, &waypoints);
            drop(waypoints);

            if view.radar_enabled {
                let radar = self.state.radar.lock().unwrap();
                renderer.draw_radar(
                    &mut target,
                    &camera,
                    &radar,
                    self.radar_range,
                    self.radar_color,
                );
            }
        }

        // Labels use Horizon's text API directly, after the pixel adapter
        // has released its mutable borrow of the canvas.
        if view.show_labels {
            let waypoints = self.state.waypoints.lock().unwrap();
            for marker in waypoints.iter() {
                let (mx, my) = renderer.map_position(&camera, marker.x, marker.z);
                if (0.0..size).contains(&mx) && (0.0..size).contains(&my) {
                    canvas.text(x + mx + 3.0, y + my, &marker.name, argb(marker.color), true);
                }
            }
        }
    }
}

/// `Hooks` implementation that clears transient radar data on a new join.
pub struct MinimapHooks {
    state: Arc<MinimapState>,
}

impl MinimapHooks {
    pub fn new(state: Arc<MinimapState>) -> Self {
        MinimapHooks { state }
    }
}

impl Hooks for MinimapHooks {
    fn frame(&self, frame: &Frame, _sprites: &mut dyn SpriteStore) {
        self.state.set_connection(frame.connection);
    }
}

/// Registers the HUD layer. Horizon requires registration before the game
/// starts; `TooLate` is returned otherwise.
pub fn register_layer<S: TileSource + Sync + 'static>(
    state: Arc<MinimapState>,
    renderer: MinimapRenderer,
    tiles: S,
    corner: Corner,
) -> Result<Arc<MinimapState>, TooLate> {
    let mut view = state.view();
    view.corner = corner;
    view.map_size = renderer.map_size;
    view.scale = renderer.scale;
    state.set_view(view);
    let layer = Box::leak(Box::new(MinimapLayer::new(
        state.clone(),
        renderer,
        tiles,
        corner,
    )));
    horizon_hooks::hud::register(layer)?;
    Ok(state)
}

/// Registers the matching `Hooks` implementation.
pub fn register_hooks(state: Arc<MinimapState>) -> Result<(), TooLate> {
    let hooks = Box::leak(Box::new(MinimapHooks::new(state)));
    horizon_hooks::register(hooks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingCanvas {
        fills: Vec<(f32, f32, f32, f32, u32)>,
        texts: Vec<(f32, f32, String)>,
        placed: Vec<(Corner, f32, f32)>,
    }

    impl HorizonCanvas for RecordingCanvas {
        fn size(&self) -> (f32, f32) {
            (160.0, 100.0)
        }
        fn tr(&self, key: &str) -> String {
            key.to_string()
        }
        fn text_width(&self, text: &str) -> f32 {
            text.len() as f32 * 6.0
        }
        fn text(&mut self, x: f32, y: f32, text: &str, _argb: u32, _shadow: bool) {
            self.texts.push((x, y, text.to_string()));
        }
        fn fill(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
            self.fills.push((x, y, w, h, color));
        }
        fn place(&mut self, corner: Corner, w: f32, h: f32) -> Option<(f32, f32)> {
            self.placed.push((corner, w, h));
            Some((10.0, 20.0))
        }
    }

    #[derive(Default)]
    struct OneChunk;

    impl TileSource for OneChunk {
        fn chunk_color(&self, _chunk_x: i32, _chunk_z: i32) -> Option<Color> {
            Some(Color::rgb(1, 2, 3))
        }
    }

    #[test]
    fn color_conversion_uses_argb_order() {
        assert_eq!(argb(Color::rgba(1, 2, 3, 4)), 0x0401_0203);
    }

    #[test]
    fn waypoints_are_converted_and_hidden_ones_skipped() {
        use xaerominimap_core::waypoints::WaypointColor;

        let state = MinimapState::new();
        let visible = Waypoint::new(3, 64, -5, "home", "*")
            .with_color(WaypointColor::Gold);
        let deleted = Waypoint {
            effectively_deleted: true,
            ..Waypoint::new(1, 64, 1, "gone", "*")
        };
        state.sync_waypoints(&[visible, deleted]);
        let markers = state.waypoints.lock().unwrap();
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].name, "home");
        assert_eq!(markers[0].color, Color::rgba(255, 0xaa, 0, 255));
    }

    #[test]
    fn offset_canvas_translates_and_packs_colors() {
        let mut horizon = RecordingCanvas::default();
        let mut canvas = OffsetCanvas {
            inner: &mut horizon,
            ox: 10.0,
            oy: 20.0,
            width: 64.0,
            height: 64.0,
        };
        canvas.fill_rect(1, 2, 3, 4, Color::rgb(1, 2, 3));
        canvas.set_pixel(5, 6, Color::rgba(9, 8, 7, 6));
        assert_eq!(
            horizon.fills,
            vec![
                (11.0, 22.0, 3.0, 4.0, 0xff01_0203),
                (15.0, 26.0, 1.0, 1.0, 0x0609_0807),
            ]
        );
    }

    #[test]
    fn hud_layer_draws_map_waypoints_and_radar_in_its_corner() {
        let state = Arc::new(MinimapState::new());
        state.set_enabled(true);
        state.set_camera(0, 64, 0);
        state.set_waypoints(vec![WaypointMarker {
            x: 1,
            y: 64,
            z: 1,
            name: "home".into(),
            color: Color::rgb(255, 0, 0),
        }]);
        state.set_radar(vec![RadarEntity::new("mob", 2.0, 64.0, 2.0, "zombie")]);

        let layer = MinimapLayer::new(
            state.clone(),
            MinimapRenderer::new(64.0, 1.0, 16),
            OneChunk,
            Corner::TopLeft,
        );
        let mut canvas = RecordingCanvas::default();
        Layer::draw(&layer, &mut canvas);

        assert!(!canvas.fills.is_empty());
        assert!(canvas.fills.iter().all(|(x, y, _, _, _)| *x >= 10.0 && *y >= 20.0));
        assert!(canvas.texts.iter().any(|(_, _, text)| text == "home"));
    }

    #[test]
    fn disabled_layer_draws_nothing() {
        let state = Arc::new(MinimapState::new());
        state.set_enabled(false);
        let layer = MinimapLayer::new(
            state,
            MinimapRenderer::new(64.0, 1.0, 16),
            OneChunk,
            Corner::TopLeft,
        );
        let mut canvas = RecordingCanvas::default();
        Layer::draw(&layer, &mut canvas);
        assert!(canvas.fills.is_empty());
    }

    #[test]
    fn view_config_controls_corner_size_labels_and_radar() {
        let state = Arc::new(MinimapState::new());
        state.set_camera(0, 0, 0);
        state.set_waypoints(vec![WaypointMarker {
            x: 1,
            y: 0,
            z: 1,
            name: "home".into(),
            color: Color::rgb(1, 1, 1),
        }]);
        state.set_radar(vec![RadarEntity::new("mob", 2.0, 0.0, 2.0, "zombie")]);
        state.set_view(MinimapViewConfig {
            map_size: 32.0,
            scale: 2.0,
            corner: Corner::BottomRight,
            show_labels: false,
            radar_enabled: false,
            cave_mode: false,
            enabled: true,
        });

        let layer = MinimapLayer::new(
            state.clone(),
            MinimapRenderer::new(160.0, 1.0, 16),
            OneChunk,
            Corner::TopLeft,
        );
        let mut canvas = RecordingCanvas::default();
        Layer::draw(&layer, &mut canvas);
        assert_eq!(canvas.placed, vec![(Corner::BottomRight, 32.0, 32.0)]);
        assert!(canvas.texts.is_empty(), "labels must be hidden");

        // Turning radar back on adds a dot without changing the label count.
        let mut view = state.view();
        view.radar_enabled = true;
        state.set_view(view);
        let mut canvas = RecordingCanvas::default();
        Layer::draw(&layer, &mut canvas);
        assert!(canvas.texts.is_empty());
        assert!(!canvas.fills.is_empty());
    }

    #[test]
    fn new_connection_clears_transient_data() {
        let state = MinimapState::new();
        state.set_connection(1);
        state.set_waypoints(vec![WaypointMarker {
            x: 0,
            y: 0,
            z: 0,
            name: "old".into(),
            color: Color::rgb(1, 1, 1),
        }]);
        state.set_connection(2);
        assert!(state.waypoints.lock().unwrap().is_empty());
        assert_eq!(state.connection(), 2);
    }
}
