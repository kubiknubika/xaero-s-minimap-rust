//! Chunk colour sources for the Horizon HUD layer.
//!
//! A mixin bundle fills a [`ChunkColorGrid`] from the loaded chunk map (the
//! Java mod's `MinimapTile` lookup). The grid implements the port's
//! [`TileSource`] so the renderer does not depend on Horizon types.

use std::collections::HashMap;
use std::sync::RwLock;

use xaerominimap_render::{Color, TileSource};

/// A thread-safe chunk → colour map, with an optional fallback for chunks that
/// are known but have no colour yet.
#[derive(Debug)]
pub struct ChunkColorGrid {
    colors: RwLock<HashMap<(i32, i32), Color>>,
    fallback: Option<Color>,
}

impl ChunkColorGrid {
    pub fn new() -> Self {
        Self {
            colors: RwLock::new(HashMap::new()),
            fallback: None,
        }
    }

    pub fn with_fallback(fallback: Color) -> Self {
        Self {
            colors: RwLock::new(HashMap::new()),
            fallback: Some(fallback),
        }
    }

    pub fn set(&self, chunk_x: i32, chunk_z: i32, color: Color) {
        self.colors
            .write()
            .unwrap()
            .insert((chunk_x, chunk_z), color);
    }

    pub fn remove(&self, chunk_x: i32, chunk_z: i32) -> Option<Color> {
        self.colors.write().unwrap().remove(&(chunk_x, chunk_z))
    }

    pub fn clear(&self) {
        self.colors.write().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.colors.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for ChunkColorGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl TileSource for ChunkColorGrid {
    fn chunk_color(&self, chunk_x: i32, chunk_z: i32) -> Option<Color> {
        self.colors
            .read()
            .unwrap()
            .get(&(chunk_x, chunk_z))
            .copied()
            .or(self.fallback)
    }
}

/// A constant tile source, useful for cave-mode overlays and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolidTiles(pub Color);

impl TileSource for SolidTiles {
    fn chunk_color(&self, _chunk_x: i32, _chunk_z: i32) -> Option<Color> {
        Some(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_returns_coloured_chunks_and_fallback() {
        let grid = ChunkColorGrid::with_fallback(Color::rgb(9, 9, 9));
        assert_eq!(grid.chunk_color(0, 0), Some(Color::rgb(9, 9, 9)));
        grid.set(1, -2, Color::rgb(1, 2, 3));
        assert_eq!(grid.chunk_color(1, -2), Some(Color::rgb(1, 2, 3)));
        assert_eq!(grid.remove(1, -2), Some(Color::rgb(1, 2, 3)));
        assert_eq!(grid.chunk_color(1, -2), Some(Color::rgb(9, 9, 9)));
        grid.clear();
        assert!(grid.is_empty());
    }

    #[test]
    fn solid_tiles_covers_every_chunk() {
        let tiles = SolidTiles(Color::rgb(1, 2, 3));
        assert_eq!(tiles.chunk_color(100, -100), Some(Color::rgb(1, 2, 3)));
    }
}
