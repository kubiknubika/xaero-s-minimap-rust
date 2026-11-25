//! Data side of `xaero/common/minimap/region/MinimapChunk` from the original
//! mod: the 4×4 tile grid of a chunk and its refresh bookkeeping. GPU
//! textures/byte buffers and `cleanup(Minimap)` stay in the game client.

use crate::tile::MinimapTile;

/// `xaero/common/minimap/region/MinimapChunk`.
#[derive(Debug, Clone, Default)]
pub struct MinimapChunk {
    /// Java `SIZE_TILES`.
    pub size_tiles: usize,
    /// Java `LIGHT_LEVELS` (also the number of refresh/buffer slots).
    pub light_levels: usize,
    x: i32,
    z: i32,
    has_something: bool,
    tiles: Vec<Vec<Option<MinimapTile>>>,
    refresh_required: Vec<bool>,
    changed: bool,
    refreshed: bool,
    levels_buffered: usize,
    block_texture_upload: bool,
}

impl MinimapChunk {
    /// Java `MinimapChunk(int x, int z)`.
    pub fn new(x: i32, z: i32) -> Self {
        let mut chunk = MinimapChunk::default();
        chunk.reset(x, z);
        chunk
    }

    /// Java `reset(int x, int z)` — also recycles the tiles and clears the
    /// GPU-side slots (represented by clearing the flags here).
    pub fn reset(&mut self, x: i32, z: i32) {
        self.size_tiles = 4;
        self.light_levels = 5;
        self.x = x;
        self.z = z;
        self.has_something = false;
        self.tiles = vec![vec![None; 4]; 4];
        self.refresh_required = vec![false; 5];
        self.changed = false;
        self.refreshed = false;
        self.levels_buffered = 0;
        self.block_texture_upload = false;
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    /// `getTile(int, int)`.
    pub fn get_tile(&self, x: usize, y: usize) -> Option<&MinimapTile> {
        self.tiles.get(y).and_then(|row| row.get(x)).and_then(|tile| tile.as_ref())
    }

    /// `setTile(int, int, MinimapTile)`.
    pub fn set_tile(&mut self, x: usize, y: usize, tile: MinimapTile) {
        if let Some(slot) = self.tiles.get_mut(y).and_then(|row| row.get_mut(x)) {
            *slot = Some(tile);
        }
    }

    /// Java `recycleTiles()` — drop all tile references for reuse.
    pub fn recycle_tiles(&mut self) {
        for row in &mut self.tiles {
            for tile in row {
                *tile = None;
            }
        }
    }

    pub fn is_has_something(&self) -> bool {
        self.has_something
    }

    pub fn set_has_something(&mut self, value: bool) {
        self.has_something = value;
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }

    pub fn set_changed(&mut self, value: bool) {
        self.changed = value;
    }

    pub fn is_block_texture_upload(&self) -> bool {
        self.block_texture_upload
    }

    pub fn set_block_texture_upload(&mut self, value: bool) {
        self.block_texture_upload = value;
    }

    pub fn get_levels_buffered(&self) -> usize {
        self.levels_buffered
    }

    pub fn set_levels_buffered(&mut self, value: usize) {
        self.levels_buffered = value.min(self.light_levels);
    }

    pub fn is_refresh_required(&self, level: usize) -> bool {
        self.refresh_required.get(level).copied().unwrap_or(false)
    }

    pub fn set_refresh_required(&mut self, level: usize, value: bool) {
        if let Some(slot) = self.refresh_required.get_mut(level) {
            *slot = value;
        }
    }

    /// `getLevelToRefresh(int lastLevel)`: scans forward from `lastLevel`
    /// through the buffered levels, stopping just before `lastLevel - 1`
    /// (the previously processed level). Returns the first dirty level or
    /// `-1` when none needs refreshing.
    pub fn get_level_to_refresh(&mut self, last_level: i32) -> i32 {
        if self.refreshed || last_level == -1 || self.levels_buffered == 0 {
            return -1;
        }
        let count = self.levels_buffered as i32;
        let mut stop = last_level - 1;
        if stop < 0 {
            stop = count - 1;
        }
        let mut current = last_level % count;
        loop {
            if self.is_refresh_required(current as usize) {
                return current;
            }
            if current == stop {
                break;
            }
            current = (current + 1) % count;
        }
        self.refreshed = true;
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(height: i16) -> MinimapTile {
        let mut tile = MinimapTile::new(0, 0);
        tile.set_height(0, 0, 0, height);
        tile
    }

    #[test]
    fn chunk_has_four_by_four_tiles_and_five_levels() {
        let mut chunk = MinimapChunk::new(3, -2);
        assert_eq!((chunk.get_x(), chunk.get_z()), (3, -2));
        assert_eq!(chunk.size_tiles, 4);
        assert_eq!(chunk.light_levels, 5);
        assert!(chunk.get_tile(0, 0).is_none());
        chunk.set_tile(1, 2, tile(7));
        assert_eq!(chunk.get_tile(1, 2).unwrap().height(0, 0, 0), Some(7));
        chunk.recycle_tiles();
        assert!(chunk.get_tile(1, 2).is_none());
    }

    #[test]
    fn reset_clears_flags() {
        let mut chunk = MinimapChunk::new(0, 0);
        chunk.set_has_something(true);
        chunk.set_changed(true);
        chunk.set_block_texture_upload(true);
        chunk.set_levels_buffered(3);
        chunk.set_refresh_required(2, true);
        chunk.set_tile(0, 0, tile(1));
        chunk.reset(5, 6);
        assert_eq!((chunk.get_x(), chunk.get_z()), (5, 6));
        assert!(!chunk.is_has_something());
        assert!(!chunk.is_changed());
        assert!(!chunk.is_block_texture_upload());
        assert_eq!(chunk.get_levels_buffered(), 0);
        assert!(!chunk.is_refresh_required(2));
        assert!(chunk.get_tile(0, 0).is_none());
    }

    #[test]
    fn refresh_walks_levels_cyclically() {
        let mut chunk = MinimapChunk::new(0, 0);
        assert_eq!(chunk.get_level_to_refresh(-1), -1);
        chunk.set_levels_buffered(5);
        chunk.set_refresh_required(4, true);
        chunk.set_refresh_required(3, true);
        chunk.set_refresh_required(0, true);
        // The Java code rejects -1 outright when levels are buffered.
        assert_eq!(chunk.get_level_to_refresh(-1), -1);
        // From last level 2 the scan checks 2, 3, ... and finds 3.
        assert_eq!(chunk.get_level_to_refresh(2), 3);
        assert_eq!(chunk.get_level_to_refresh(4), 4);
        assert_eq!(chunk.get_level_to_refresh(0), 0);
        // Nothing left to refresh: -1 and the chunk is marked refreshed.
        for level in 0..5 {
            chunk.set_refresh_required(level, false);
        }
        assert_eq!(chunk.get_level_to_refresh(1), -1);
        assert_eq!(chunk.get_level_to_refresh(0), -1);
    }
}
