//! Minimap tiles: the map-region data the minimap draws from
//! (mirrors `xaero/common/minimap/region/MinimapTile` of the original mod).
//!
//! A tile covers one chunk of the map: heights per level and per pixel, plus
//! the `chunkGrid` and `slimeChunk` flags.

/// `xaero/common/minimap/region/MinimapTile`.
#[derive(Debug, Clone, Default)]
pub struct MinimapTile {
    x: i32,
    z: i32,
    /// `heights[[S` — per level, per pixel height grids (`levels[level][x][z]`).
    heights: Vec<Vec<Vec<i16>>>,
    /// `red[[B` / `green[[B` / `blue[[B` — per level, per pixel map colors.
    red: Vec<Vec<Vec<u8>>>,
    green: Vec<Vec<Vec<u8>>>,
    blue: Vec<Vec<Vec<u8>>>,
    /// `chunkGridZ`.
    chunk_grid: bool,
    /// `slimeChunkZ`.
    slime_chunk: bool,
    /// `successZ`.
    success: bool,
    /// `hasSomethingZ`.
    has_something: bool,
    /// `hasTerrainZ`.
    has_terrain: bool,
}

impl MinimapTile {
    /// `getANewTile` / `create`: a new, empty tile.
    pub fn new(x: i32, z: i32) -> MinimapTile {
        MinimapTile {
            x,
            z,
            ..Default::default()
        }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    /// `isSlimeChunk()`.
    pub fn is_slime_chunk(&self) -> bool {
        self.slime_chunk
    }

    pub fn set_slime_chunk(&mut self, slime_chunk: bool) {
        self.slime_chunk = slime_chunk;
    }

    /// `isChunkGrid()`.
    pub fn is_chunk_grid(&self) -> bool {
        self.chunk_grid
    }

    pub fn set_chunk_grid(&mut self, chunk_grid: bool) {
        self.chunk_grid = chunk_grid;
    }

    /// `isSuccess()`.
    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }

    /// `isHasSomething()`.
    pub fn is_has_something(&self) -> bool {
        self.has_something
    }

    pub fn set_has_something(&mut self, has_something: bool) {
        self.has_something = has_something;
    }

    /// `hasTerrain()`.
    pub fn has_terrain(&self) -> bool {
        self.has_terrain
    }

    pub fn set_has_terrain(&mut self, has_terrain: bool) {
        self.has_terrain = has_terrain;
    }

    /// Number of levels with stored data.
    pub fn levels(&self) -> usize {
        self.heights.len()
    }

    /// `getHeight(III)` — the stored height at `(level, x, z)`.
    pub fn height(&self, level: usize, x: usize, z: usize) -> Option<i16> {
        self.heights.get(level)?.get(x)?.get(z).copied()
    }

    /// `setHeight(III)` — stores the height; grows the arrays as needed.
    pub fn set_height(&mut self, level: usize, x: usize, z: usize, height: i16) {
        while self.heights.len() <= level {
            self.heights.push(Vec::new());
        }
        let grid = &mut self.heights[level];
        while grid.len() <= x {
            grid.push(Vec::new());
        }
        let row = &mut grid[x];
        while row.len() <= z {
            row.push(0);
        }
        row[z] = height;
    }

    /// `getRed(III)`.
    pub fn red(&self, level: usize, x: usize, z: usize) -> Option<u8> {
        self.red.get(level)?.get(x)?.get(z).copied()
    }

    /// `setRGB(IIIIII)` — stores red/green/blue at `(level, x, z)`.
    pub fn set_rgb(&mut self, level: usize, x: usize, z: usize, r: u8, g: u8, b: u8) {
        while self.red.len() <= level {
            self.red.push(Vec::new());
            self.green.push(Vec::new());
            self.blue.push(Vec::new());
        }
        ensure_column(&mut self.red[level], x, z);
        ensure_column(&mut self.green[level], x, z);
        ensure_column(&mut self.blue[level], x, z);
        self.red[level][x][z] = r;
        self.green[level][x][z] = g;
        self.blue[level][x][z] = b;
    }
}

/// Grows one colour plane so that `(x, z)` exists, zero-filling new cells.
fn ensure_column(plane: &mut Vec<Vec<u8>>, x: usize, z: usize) {
    while plane.len() <= x {
        plane.push(Vec::new());
    }
    let row = &mut plane[x];
    while row.len() <= z {
        row.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_set_height_and_rgb() {
        let mut tile = MinimapTile::new(3, 4);
        assert_eq!(tile.get_x(), 3);
        assert_eq!(tile.get_z(), 4);
        assert!(!tile.is_slime_chunk());
        tile.set_height(0, 1, 2, 7);
        assert_eq!(tile.height(0, 1, 2), Some(7));
        tile.set_rgb(0, 1, 2, 10, 20, 30);
        assert_eq!(tile.red(0, 1, 2), Some(10));
        tile.set_slime_chunk(true);
        assert!(tile.is_slime_chunk());
        tile.set_chunk_grid(true);
        assert!(tile.is_chunk_grid());
        tile.set_success(true);
        assert!(tile.is_success());
        tile.set_has_something(true);
        assert!(tile.is_has_something());
        tile.set_has_terrain(true);
        assert!(tile.has_terrain());
    }
}
