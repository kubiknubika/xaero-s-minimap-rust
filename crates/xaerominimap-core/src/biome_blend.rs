//! Biome colour blending from
//! `xaero/common/minimap/write/biome/BiomeBlendCalculator` of the original
//! mod.
//!
//! The Java class implements Minecraft's `BlockAndTintGetter` so it can
//! sample neighbouring biome colours at render time. The engine-facing
//! methods stay in the game client; this module ports the blend calculation
//! itself: sum the red/green/blue channels of the sampled colours, divide by
//! the number of samples and force an opaque alpha byte.

/// `BiomeBlendCalculator.prepare(level, biomeBlend)`: the offsets sampled
/// for each pixel when biome blending is on/off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeBlendOffsets {
    pub start_i: i32,
    pub end_i: i32,
    pub start_j: i32,
    pub end_j: i32,
}

impl BiomeBlendOffsets {
    pub fn prepare(blend: bool) -> Self {
        if blend {
            BiomeBlendOffsets {
                start_i: -1,
                end_i: 1,
                start_j: -1,
                end_j: 1,
            }
        } else {
            BiomeBlendOffsets {
                start_i: 0,
                end_i: 0,
                start_j: 0,
                end_j: 0,
            }
        }
    }
}

/// `BiomeBlendCalculator.getBlockTint(pos, resolver)` with the world and
/// resolver replaced by a callback. `sample(x, y, z, offset_i, offset_j)`
/// returns the packed ARGB colour of the biome at the offset position, or
/// `None` when that position has no biome.
pub fn blend_biome_color<F>(
    x: i32,
    y: i32,
    z: i32,
    offsets: BiomeBlendOffsets,
    mut sample: F,
) -> i32
where
    F: FnMut(i32, i32, i32, i32, i32) -> Option<i32>,
{
    let mut red = 0i32;
    let mut green = 0i32;
    let mut blue = 0i32;
    let mut count = 0i32;

    let mut i = offsets.start_i;
    while i <= offsets.end_i {
        let mut j = offsets.start_j;
        while j <= offsets.end_j {
            if let Some(color) = sample(x, y, z, i, j) {
                red += color & 0x00ff_0000;
                green += color & 0x0000_ff00;
                blue += color & 0x0000_00ff;
                count += 1;
            }
            j += 1;
        }
        i += 1;
    }

    if count == 0 {
        return 0;
    }
    0xff00_0000u32 as i32 | (red / count) | (green / count) | (blue / count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_selects_one_or_nine_samples() {
        let none = BiomeBlendOffsets::prepare(false);
        assert_eq!((none.start_i, none.end_i, none.start_j, none.end_j), (0, 0, 0, 0));
        let blend = BiomeBlendOffsets::prepare(true);
        assert_eq!((blend.start_i, blend.end_i, blend.start_j, blend.end_j), (-1, 1, -1, 1));
    }

    #[test]
    fn single_sample_keeps_rgb_and_forces_opaque_alpha() {
        let color = blend_biome_color(1, 2, 3, BiomeBlendOffsets::prepare(false), |_, _, _, _, _| {
            Some(0x1234_5678)
        });
        assert_eq!(color, 0xff34_5678u32 as i32);
    }

    #[test]
    fn blends_nine_neighbours_channel_wise() {
        let color = blend_biome_color(0, 0, 0, BiomeBlendOffsets::prepare(true), |x, _, z, i, j| {
            Some(((x + i + 8) << 16) | ((z + j + 8) << 8) | 4)
        });
        // Red and green channels average to 8; blue is constant.
        assert_eq!(color, 0xff08_0804u32 as i32);
    }

    #[test]
    fn missing_samples_are_skipped() {
        let color = blend_biome_color(0, 0, 0, BiomeBlendOffsets::prepare(true), |_, _, _, i, j| {
            if i == 0 && j == 0 {
                Some(0x00ff_00ff)
            } else {
                None
            }
        });
        assert_eq!(color, 0xffff_00ffu32 as i32);
    }
}
