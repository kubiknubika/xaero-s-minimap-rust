//! Map highlighter registry and colour blending, ported from
//! `xaero/common/minimap/highlight` of the original mod.
//!
//! The Java classes ask an external mod (Xaero's WorldMap) for per-chunk
//! highlight colours and blend them into the map texture. The GPU-facing
//! callbacks stay in the game client; this module ports the colour blending
//! and the 16×16 per-block result store used by the renderers.

/// `xaero/common/minimap/highlight/AbstractHighlighter`: the 16×16 result
/// store and the ARGB blend helpers.
#[derive(Debug, Clone)]
pub struct Highlighter {
    covering_outside_discovered: bool,
    result_store: Vec<u32>,
}

impl Highlighter {
    /// `AbstractHighlighter(boolean coveringOutsideDiscovered)`.
    pub fn new(covering_outside_discovered: bool) -> Self {
        Highlighter {
            covering_outside_discovered,
            result_store: vec![0; 256],
        }
    }

    /// `setResult(int x, int y, int color)` — index is `y << 4 | x`.
    pub fn set_result(&mut self, x: usize, y: usize, color: u32) {
        if x < 16 && y < 16 {
            let index = (y << 4) | x;
            self.result_store[index] = color;
        }
    }

    /// The value previously stored for a block position.
    pub fn get_result(&self, x: usize, y: usize) -> u32 {
        if x < 16 && y < 16 {
            self.result_store[(y << 4) | x]
        } else {
            0
        }
    }

    /// `clear()`-style reset of the per-block results (the Java code
    /// overwrites the array during a render pass).
    pub fn clear_results(&mut self) {
        self.result_store.fill(0);
    }

    /// `getBlend(int, int)`: averages the four ARGB channels, rounding down.
    pub fn get_blend(first: u32, second: u32) -> u32 {
        if first == second {
            return first;
        }
        let a = Self::blend_channel(first >> 24, second >> 24);
        let r = Self::blend_channel(first >> 16, second >> 16);
        let g = Self::blend_channel(first >> 8, second >> 8);
        let b = Self::blend_channel(first, second);
        (a << 24) | (r << 16) | (g << 8) | b
    }

    fn blend_channel(first: u32, second: u32) -> u32 {
        ((first & 0xff) + (second & 0xff)) >> 1
    }

    /// `isCoveringOutsideDiscovered()`.
    pub fn is_covering_outside_discovered(&self) -> bool {
        self.covering_outside_discovered
    }
}

/// `xaero/common/minimap/highlight/ChunkHighlighter`: blends the two visible
/// neighbouring chunk colours with the highlight colour itself.
#[derive(Debug, Clone, Default)]
pub struct ChunkHighlighter;

impl ChunkHighlighter {
    /// `getSideBlend(int, int, int)` — the Java method picks one neighbour
    /// when the two sides agree and blends the other pair otherwise.
    pub fn get_side_blend(first: u32, second: u32, third: u32) -> u32 {
        if first == third {
            second
        } else if first == second {
            first
        } else {
            Highlighter::get_blend(second, third)
        }
    }
}

/// `xaero/common/minimap/highlight/HighlighterRegistry`.
#[derive(Debug, Clone, Default)]
pub struct HighlighterRegistry {
    highlighters: Vec<Highlighter>,
    frozen: bool,
}

impl HighlighterRegistry {
    pub fn new() -> Self {
        HighlighterRegistry::default()
    }

    /// `register(AbstractHighlighter)`.
    pub fn register(&mut self, highlighter: Highlighter) {
        if !self.frozen {
            self.highlighters.push(highlighter);
        }
    }

    /// `end()` makes the registered list unmodifiable in Java; the Rust port
    /// rejects later registrations instead.
    pub fn end(&mut self) {
        self.frozen = true;
    }

    /// `getHighlighters()`.
    pub fn get_highlighters(&self) -> &[Highlighter] {
        &self.highlighters
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_store_uses_the_four_bit_indices() {
        let mut highlighter = Highlighter::new(true);
        highlighter.set_result(3, 5, 0xff00ff00);
        assert_eq!(highlighter.get_result(3, 5), 0xff00ff00);
        assert_eq!(highlighter.get_result(5, 3), 0);
        assert_eq!(highlighter.get_result(16, 0), 0);
        assert!(highlighter.is_covering_outside_discovered());
        highlighter.clear_results();
        assert_eq!(highlighter.get_result(3, 5), 0);
    }

    #[test]
    fn blends_argb_channels() {
        let white = 0xffff_ffff;
        let black = 0xff00_0000;
        assert_eq!(Highlighter::get_blend(white, white), white);
        assert_eq!(
            Highlighter::get_blend(white, black),
            0xff7f_7f7f
        );
        assert_eq!(
            Highlighter::get_blend(0xff00_00ff, 0x8000_00ff),
            0xbf00_00ff
        );
    }

    #[test]
    fn side_blend_reuses_equal_neighbours() {
        assert_eq!(ChunkHighlighter::get_side_blend(1, 2, 1), 2);
        assert_eq!(ChunkHighlighter::get_side_blend(1, 1, 3), 1);
        assert_eq!(
            ChunkHighlighter::get_side_blend(0xff00_00ff, 0xff00_ff00, 0xff00_00ff),
            0xff00_ff00
        );
    }

    #[test]
    fn registry_freezes_after_end() {
        let mut registry = HighlighterRegistry::new();
        registry.register(Highlighter::new(false));
        assert_eq!(registry.get_highlighters().len(), 1);
        registry.end();
        registry.register(Highlighter::new(true));
        assert!(registry.is_frozen());
        assert_eq!(registry.get_highlighters().len(), 1);
    }
}
