//! Radar colours and colour/height-fade helpers from
//! `xaero/hud/minimap/radar/color` of the original mod.
//!
//! `RadarColor` mirrors the 16-value Minecraft colour enum with its
//! formatting char, packed ARGB hex and translation key. The helpers port
//! `RadarColorHelper`; entity/team lookups stay in the game client.

/// `xaero/hud/minimap/radar/color/RadarColor`.
// Keep the Java enum spellings (e.g. `DARK_GRAY`) for parity with the mod.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadarColor {
    BLACK,
    DARK_BLUE,
    DARK_GREEN,
    DARK_AQUA,
    DARK_RED,
    DARK_PURPLE,
    GOLD,
    GRAY,
    DARK_GRAY,
    BLUE,
    GREEN,
    AQUA,
    RED,
    PURPLE,
    YELLOW,
    WHITE,
}

impl RadarColor {
    /// Java `values()`, in enum declaration order.
    pub const ALL: [RadarColor; 16] = [RadarColor::BLACK, RadarColor::DARK_BLUE, RadarColor::DARK_GREEN, RadarColor::DARK_AQUA, RadarColor::DARK_RED, RadarColor::DARK_PURPLE, RadarColor::GOLD, RadarColor::GRAY, RadarColor::DARK_GRAY, RadarColor::BLUE, RadarColor::GREEN, RadarColor::AQUA, RadarColor::RED, RadarColor::PURPLE, RadarColor::YELLOW, RadarColor::WHITE];

    /// Enum ordinal.
    pub fn index(self) -> usize {
        match self {
            RadarColor::BLACK => 0,
            RadarColor::DARK_BLUE => 1,
            RadarColor::DARK_GREEN => 2,
            RadarColor::DARK_AQUA => 3,
            RadarColor::DARK_RED => 4,
            RadarColor::DARK_PURPLE => 5,
            RadarColor::GOLD => 6,
            RadarColor::GRAY => 7,
            RadarColor::DARK_GRAY => 8,
            RadarColor::BLUE => 9,
            RadarColor::GREEN => 10,
            RadarColor::AQUA => 11,
            RadarColor::RED => 12,
            RadarColor::PURPLE => 13,
            RadarColor::YELLOW => 14,
            RadarColor::WHITE => 15,
        }
    }

    /// `getFormat()` — Minecraft formatting code character.
    pub fn format(self) -> char {
        match self {
            RadarColor::BLACK => '0',
            RadarColor::DARK_BLUE => '1',
            RadarColor::DARK_GREEN => '2',
            RadarColor::DARK_AQUA => '3',
            RadarColor::DARK_RED => '4',
            RadarColor::DARK_PURPLE => '5',
            RadarColor::GOLD => '6',
            RadarColor::GRAY => '7',
            RadarColor::DARK_GRAY => '8',
            RadarColor::BLUE => '9',
            RadarColor::GREEN => 'a',
            RadarColor::AQUA => 'b',
            RadarColor::RED => 'c',
            RadarColor::PURPLE => 'd',
            RadarColor::YELLOW => 'e',
            RadarColor::WHITE => 'f',
        }
    }

    /// `getHex()` — packed ARGB colour (including the alpha byte).
    pub fn hex(self) -> i32 {
        match self {
            RadarColor::BLACK => -16777216,
            RadarColor::DARK_BLUE => -16777046,
            RadarColor::DARK_GREEN => -16733696,
            RadarColor::DARK_AQUA => -16733526,
            RadarColor::DARK_RED => -5636096,
            RadarColor::DARK_PURPLE => -5635926,
            RadarColor::GOLD => -22016,
            RadarColor::GRAY => -5592406,
            RadarColor::DARK_GRAY => -11184811,
            RadarColor::BLUE => -11184641,
            RadarColor::GREEN => -11141291,
            RadarColor::AQUA => -11141121,
            RadarColor::RED => -65536,
            RadarColor::PURPLE => -43521,
            RadarColor::YELLOW => -171,
            RadarColor::WHITE => -1,
        }
    }

    /// Translation key of `getName()`.
    pub fn translation_key(self) -> &'static str {
        match self {
            RadarColor::BLACK => "gui.xaero_black",
            RadarColor::DARK_BLUE => "gui.xaero_dark_blue",
            RadarColor::DARK_GREEN => "gui.xaero_dark_green",
            RadarColor::DARK_AQUA => "gui.xaero_dark_aqua",
            RadarColor::DARK_RED => "gui.xaero_dark_red",
            RadarColor::DARK_PURPLE => "gui.xaero_dark_purple",
            RadarColor::GOLD => "gui.xaero_gold",
            RadarColor::GRAY => "gui.xaero_gray",
            RadarColor::DARK_GRAY => "gui.xaero_dark_gray",
            RadarColor::BLUE => "gui.xaero_blue",
            RadarColor::GREEN => "gui.xaero_green",
            RadarColor::AQUA => "gui.xaero_aqua",
            RadarColor::RED => "gui.xaero_red",
            RadarColor::PURPLE => "gui.xaero_purple",
            RadarColor::YELLOW => "gui.xaero_yellow",
            RadarColor::WHITE => "gui.xaero_white",
        }
    }

    /// `fromIndex(int)` — `-1` maps to `None`, out-of-range values clamp.
    pub fn from_index(index: i32) -> Option<RadarColor> {
        if index < 0 {
            None
        } else {
            Some(RadarColor::ALL[(index as usize).min(15)])
        }
    }

    /// `getRandom()` with the random fraction supplied by the caller.
    pub fn from_fraction(fraction: f64) -> RadarColor {
        let clamped = fraction.clamp(0.0, 1.0);
        RadarColor::ALL[((clamped * 16.0) as usize).min(15)]
    }
}

/// `RadarColorHelper.getRadarColorHex`: explicit colour wins, then the
/// team colour (`-1` means no team colour), then the fallback.
pub fn radar_color_hex(
    explicit: Option<RadarColor>,
    team_color: Option<i32>,
    fallback: RadarColor,
) -> i32 {
    if let Some(color) = explicit {
        return color.hex();
    }
    match team_color {
        Some(color) if color != -1 => 0xff00_0000u32 as i32 | color,
        _ => fallback.hex(),
    }
}

/// `RadarColorHelper.getEntityHeightFade(entityY, playerY, startFadeAt)`.
/// The Java version treats `startFadeAt == 0` as three quarters of the
/// player height.
pub fn entity_height_fade(entity_y: f32, player_y: i32, start_fade_at: i32) -> f32 {
    let mut diff = player_y as f32 - entity_y;
    if diff < 0.0 {
        diff = 0.0;
    }
    let threshold = if start_fade_at == 0 {
        player_y * 3 / 4
    } else {
        player_y - start_fade_at
    };
    let mut alpha = 1.0f32;
    if diff > threshold as f32 && threshold != 0 {
        alpha = 0.25 + 0.5 * diff / threshold as f32;
    }
    alpha
}

/// `RadarColorHelper.getEntityColor`: applies the height fade to the
/// colour channels. When `fade_alpha` is set the alpha channel is faded
/// too; otherwise the faded colour stays opaque.
pub fn entity_color(
    entity_y: f32,
    height_fade: bool,
    fade_alpha: bool,
    player_y: i32,
    start_fade_at: i32,
    explicit: Option<RadarColor>,
    team_color: Option<i32>,
    fallback: RadarColor,
) -> i32 {
    let base = radar_color_hex(explicit, team_color, fallback);
    let alpha = if height_fade {
        entity_height_fade(entity_y, player_y, start_fade_at)
    } else {
        1.0
    };
    if alpha >= 1.0 {
        return base;
    }
    let mut r = (base >> 16) & 0xff;
    let mut g = (base >> 8) & 0xff;
    let mut b = base & 0xff;
    let mut a = 255i32;
    if fade_alpha {
        a = (a as f32 * alpha) as i32;
    } else {
        r = (r as f32 * alpha) as i32;
        g = (g as f32 * alpha) as i32;
        b = (b as f32 * alpha) as i32;
    }
    (a << 24) | (r << 16) | (g << 8) | b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_values_match_minecraft_codes() {
        assert_eq!(RadarColor::ALL.len(), 16);
        assert_eq!(RadarColor::BLACK.index(), 0);
        assert_eq!(RadarColor::WHITE.index(), 15);
        assert_eq!(RadarColor::BLACK.format(), '0');
        assert_eq!(RadarColor::GREEN.format(), 'a');
        assert_eq!(RadarColor::WHITE.format(), 'f');
        assert_eq!(RadarColor::RED.hex(), 0xffff_0000u32 as i32);
        assert_eq!(RadarColor::WHITE.hex(), -1);
        assert_eq!(RadarColor::GOLD.translation_key(), "gui.xaero_gold");
    }

    #[test]
    fn from_index_clamps_and_random_fraction_selects() {
        assert_eq!(RadarColor::from_index(-1), None);
        assert_eq!(RadarColor::from_index(0), Some(RadarColor::BLACK));
        assert_eq!(RadarColor::from_index(99), Some(RadarColor::WHITE));
        assert_eq!(RadarColor::from_fraction(0.0), RadarColor::BLACK);
        assert_eq!(RadarColor::from_fraction(0.999), RadarColor::WHITE);
    }

    #[test]
    fn color_resolution_prefers_explicit_then_team_then_fallback() {
        assert_eq!(
            radar_color_hex(Some(RadarColor::RED), Some(0x00ff00), RadarColor::BLUE),
            RadarColor::RED.hex()
        );
        assert_eq!(
            radar_color_hex(None, Some(0x00ff00), RadarColor::BLUE),
            0xff00_ff00u32 as i32
        );
        assert_eq!(
            radar_color_hex(None, Some(-1), RadarColor::BLUE),
            RadarColor::BLUE.hex()
        );
    }

    #[test]
    fn height_fade_defaults_to_three_quarters() {
        assert_eq!(entity_height_fade(100.0, 100, 0), 1.0);
        let fade = entity_height_fade(0.0, 100, 0);
        assert!(fade > 0.75 && fade < 1.0, "fade = {fade}");
        assert!(entity_height_fade(-100.0, 100, 0) > 1.0);
        // Java would divide by zero when the threshold is 0; the port
        // treats that as no fade.
        assert_eq!(entity_height_fade(0.0, 100, 100), 1.0);
    }

    #[test]
    fn entity_color_scales_rgb_or_alpha() {
        let red = RadarColor::RED.hex();
        assert_eq!(entity_color(100.0, true, false, 100, 0, None, None, RadarColor::RED), red);
        let faded = entity_color(0.0, true, false, 100, 0, None, None, RadarColor::RED);
        assert!(faded != red);
        assert_eq!(faded >> 24, -1);
        let alpha_faded = entity_color(0.0, true, true, 100, 0, None, None, RadarColor::RED);
        assert_eq!(alpha_faded & 0x00ff_ffff, 0x00ff_0000);
        assert!((alpha_faded >> 24) < 255);
    }
}

