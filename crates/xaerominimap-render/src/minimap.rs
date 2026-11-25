//! `MinimapRenderer`: draws the map square, waypoint markers and radar dots.
//! (Mirrors the drawing core of `xaero/common/minimap/render` and the
//! waypoint/radar renderers.)

use std::sync::Arc;

use xaerominimap_core::radar::RadarEntity;

use crate::canvas::{Canvas, Color};

/// Player position the map is centered on (the port's [`Camera`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Camera {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Camera {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Camera { x, y, z }
    }
}

/// `xaero/common/minimap/region/MinimapTile` lookups the map draws with.
pub trait TileSource {
    /// Color of the tile of the chunk at `(chunk_x, chunk_z)`; `None` means
    /// "no tile drawn" (the chunk is outside the loaded region).
    fn chunk_color(&self, chunk_x: i32, chunk_z: i32) -> Option<Color>;
}

/// A waypoint shown on the map (the port's [`WaypointMarker`]).
#[derive(Debug, Clone, PartialEq)]
pub struct WaypointMarker {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub name: String,
    pub color: Color,
}

/// `MinimapRenderer`: draws the minimap around [`Camera`].
///
/// Scale semantics mirror the Java mod: `units_per_pixel` is `scale` world
/// units per pixel of map space, so the map spans `map_size * scale` world
/// units across.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapRenderer {
    /// Side of the map, in pixels.
    pub map_size: f32,
    /// Scale: world units (blocks) per map pixel.
    pub scale: f32,
    /// Chunk size in blocks (16, as in Minecraft).
    pub chunk_size: i32,
}

impl MinimapRenderer {
    pub fn new(map_size: f32, scale: f32, chunk_size: i32) -> Self {
        MinimapRenderer {
            map_size,
            scale,
            chunk_size,
        }
    }

    /// Center of the map, in pixels.
    pub fn center(&self) -> f32 {
        self.map_size / 2.0
    }

    /// World units per map pixel.
    pub fn units_per_pixel(&self) -> f32 {
        if self.scale > 0.0 {
            self.scale
        } else {
            0.0
        }
    }

    /// Map pixel position of a world offset from the player.
    /// Map Y grows upward, so the z offset is negated.
    pub fn map_position(&self, camera: &Camera, dx_block: i32, dz_block: i32) -> (f32, f32) {
        let u = self.units_per_pixel();
        (
            self.center() + (dx_block - camera.x) as f32 * u,
            self.center() - (dz_block - camera.z) as f32 * u,
        )
    }

    /// `draw_map`-style map drawing: fills a rect per visible chunk using
    /// the [`TileSource`].
    pub fn draw_map(&self, canvas: &mut impl Canvas, camera: &Camera, tiles: &dyn TileSource) {
        let center = self.center();
        let u = self.units_per_pixel();
        if u <= 0.0 {
            return;
        }
        let radius_blocks = (self.map_size / 2.0) * self.scale;
        let chunk_radius = (radius_blocks / self.chunk_size as f32).ceil() as i32 + 1;
        let cx0 = camera.x.div_euclid(self.chunk_size) - chunk_radius;
        let cx1 = camera.x.div_euclid(self.chunk_size) + chunk_radius;
        let cz0 = camera.z.div_euclid(self.chunk_size) - chunk_radius;
        let cz1 = camera.z.div_euclid(self.chunk_size) + chunk_radius;
        for cx in cx0..=cx1 {
            for cz in cz0..=cz1 {
                let Some(color) = tiles.chunk_color(cx, cz) else {
                    continue;
                };
                let left = center + (cx * self.chunk_size - camera.x) as f32 * u;
                let top = center - (((cz + 1) * self.chunk_size) - camera.z) as f32 * u;
                let w = (self.chunk_size as f32 * u) as u32;
                let h = (self.chunk_size as f32 * u) as u32;
                canvas.fill_rect(left as i32, top as i32, w, h, color);
            }
        }
    }

    /// Draws the waypoint markers of `markers`: in-map markers as 2x2 rects
    /// and off-map ones as arrows on the map edge.
    pub fn draw_waypoints(&self, canvas: &mut impl Canvas, camera: &Camera, markers: &[WaypointMarker]) {
        for marker in markers {
            let (x, y) = self.map_position(camera, marker.x, marker.z);
            if (0.0..self.map_size).contains(&x) && (0.0..self.map_size).contains(&y) {
                canvas.fill_rect(x as i32, y as i32, 2, 2, marker.color);
            } else {
                self.draw_offscreen_arrow(canvas, x, y, marker.color);
            }
        }
    }

    fn draw_offscreen_arrow(&self, canvas: &mut impl Canvas, x: f32, y: f32, color: Color) {
        // Arrow at the map edge along the line from the center to the marker.
        let ex = x.clamp(0.0, self.map_size);
        let ey = y.clamp(0.0, self.map_size);
        canvas.fill_rect(
            (ex as i32).saturating_sub(1),
            (ey as i32).saturating_sub(1),
            3,
            3,
            color,
        );
        canvas.draw_line(ex, ey, x, y, color);
    }

    /// Draws radar dots for `entities` within `max_range` blocks of the
    /// camera.
    pub fn draw_radar(
        &self,
        canvas: &mut impl Canvas,
        camera: &Camera,
        entities: &[RadarEntity],
        max_range: f64,
        color: Color,
    ) {
        for entity in entities {
            let dx = entity.x - camera.x as f64;
            let dz = entity.z - camera.z as f64;
            if dx.abs() > max_range || dz.abs() > max_range {
                continue;
            }
            let (x, y) = self.map_position(camera, entity.x as i32, entity.z as i32);
            if (0.0..self.map_size).contains(&x) && (0.0..self.map_size).contains(&y) {
                canvas.draw_point(x as i32, y as i32, color, 1);
            }
        }
    }
}

/// Shared tile sources work with the HUD layer, which keeps an `Arc` to the
/// chunk grid a mixin fills.
impl<T: TileSource + ?Sized> TileSource for Arc<T> {
    fn chunk_color(&self, chunk_x: i32, chunk_z: i32) -> Option<Color> {
        (**self).chunk_color(chunk_x, chunk_z)
    }
}

/// A [`TileSource`] that returns a color for every chunk (test helper).
#[derive(Debug, Clone, Copy, Default)]
pub struct AllChunks;

impl TileSource for AllChunks {
    fn chunk_color(&self, _chunk_x: i32, _chunk_z: i32) -> Option<Color> {
        Some(Color::rgb(10, 20, 30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::RasterCanvas;

    #[test]
    fn draw_map_fills_pixels_inside_canvas() {
        let renderer = MinimapRenderer::new(64.0, 0.5, 16);
        let mut canvas = RasterCanvas::new(64, 64);
        renderer.draw_map(&mut canvas, &Camera::new(0, 0, 0), &AllChunks);
        assert!(canvas.drawn_pixels() > 0);
    }

    #[test]
    fn waypoint_at_camera_position_draws_at_center() {
        let renderer = MinimapRenderer::new(64.0, 0.5, 16);
        let mut canvas = RasterCanvas::new(64, 64);
        renderer.draw_waypoints(
            &mut canvas,
            &Camera::new(0, 0, 0),
            &[WaypointMarker {
                x: 0,
                y: 0,
                z: 0,
                name: "home".to_string(),
                color: Color::rgb(255, 0, 0),
            }],
        );
        assert!(canvas.get(32, 32).is_some());
    }

    #[test]
    fn offscreen_waypoint_draws_arrow_at_edge() {
        let renderer = MinimapRenderer::new(64.0, 0.5, 16);
        let mut canvas = RasterCanvas::new(64, 64);
        renderer.draw_waypoints(
            &mut canvas,
            &Camera::new(0, 0, 0),
            &[WaypointMarker {
                x: 1000,
                y: 0,
                z: 0,
                name: "far".to_string(),
                color: Color::rgb(0, 255, 0),
            }],
        );
        assert!(canvas.get(63, 32).is_some());
    }

    #[test]
    fn radar_draws_dots_at_entities() {
        let renderer = MinimapRenderer::new(64.0, 0.5, 16);
        let mut canvas = RasterCanvas::new(64, 64);
        let entity = RadarEntity::new("Zed", 0.0, 0.0, 0.0, "zombie");
        renderer.draw_radar(&mut canvas, &Camera::new(0, 0, 0), &[entity], 128.0, Color::rgb(0, 255, 255));
        assert!(canvas.get(32, 32).is_some());
    }
}
