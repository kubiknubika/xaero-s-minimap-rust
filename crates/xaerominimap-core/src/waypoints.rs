//! Waypoints: the `Waypoint` model, the `WaypointSet` groups, waypoint file
//! IO and sorting (mirrors `xaero/common/minimap/waypoints` and
//! `xaero/hud/minimap/waypoint(set|io)` of the original mod).

use std::cmp::Ordering;
use std::io::{self, Write};

use crate::world::MinimapWorld;

/// `xaero/hud/minimap/waypoint/WaypointVisibilityType`: where a waypoint is
/// visible.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub enum WaypointVisibilityType {
    /// Visible on the minimap only.
    #[default]
    Local,
    /// Visible everywhere (the global map).
    Global,
    /// Visible on the world map only.
    WorldMapLocal,
    /// Visible on the world map everywhere.
    WorldMapGlobal,
}

impl WaypointVisibilityType {
    /// All variants, in the order of the Java enum (`values()`).
    pub const ALL: [WaypointVisibilityType; 4] = [
        WaypointVisibilityType::Local,
        WaypointVisibilityType::Global,
        WaypointVisibilityType::WorldMapLocal,
        WaypointVisibilityType::WorldMapGlobal,
    ];

    /// Parses a visibility name as written by [`Self::name`].
    pub fn from_name(name: &str) -> Option<WaypointVisibilityType> {
        WaypointVisibilityType::ALL
            .into_iter()
            .find(|v| v.name() == name)
    }

    /// Lower-case name of the variant (the Java `toString`).
    pub fn name(self) -> &'static str {
        match self {
            WaypointVisibilityType::Local => "local",
            WaypointVisibilityType::Global => "global",
            WaypointVisibilityType::WorldMapLocal => "world_map_local",
            WaypointVisibilityType::WorldMapGlobal => "world_map_global",
        }
    }

    /// The global variants, i.e. the ones [`Self::is_global`] accepts.
    pub fn is_global(self) -> bool {
        matches!(
            self,
            WaypointVisibilityType::Global | WaypointVisibilityType::WorldMapGlobal
        )
    }
}

/// `xaero/hud/minimap/waypoint/WaypointColor`: the 16 named colors of the
/// original mod.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaypointColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    Purple,
    Yellow,
    White,
}

impl WaypointColor {
    /// All colors in the original mod's order.
    pub const ALL: [WaypointColor; 16] = [
        WaypointColor::Black,
        WaypointColor::DarkBlue,
        WaypointColor::DarkGreen,
        WaypointColor::DarkAqua,
        WaypointColor::DarkRed,
        WaypointColor::DarkPurple,
        WaypointColor::Gold,
        WaypointColor::Gray,
        WaypointColor::DarkGray,
        WaypointColor::Blue,
        WaypointColor::Green,
        WaypointColor::Aqua,
        WaypointColor::Red,
        WaypointColor::Purple,
        WaypointColor::Yellow,
        WaypointColor::White,
    ];

    /// The color's display name (`getName`).
    pub fn name(self) -> &'static str {
        match self {
            WaypointColor::Black => "black",
            WaypointColor::DarkBlue => "dark_blue",
            WaypointColor::DarkGreen => "dark_green",
            WaypointColor::DarkAqua => "dark_aqua",
            WaypointColor::DarkRed => "dark_red",
            WaypointColor::DarkPurple => "dark_purple",
            WaypointColor::Gold => "gold",
            WaypointColor::Gray => "gray",
            WaypointColor::DarkGray => "dark_gray",
            WaypointColor::Blue => "blue",
            WaypointColor::Green => "green",
            WaypointColor::Aqua => "aqua",
            WaypointColor::Red => "red",
            WaypointColor::Purple => "purple",
            WaypointColor::Yellow => "yellow",
            WaypointColor::White => "white",
        }
    }

    /// The 0-based index of the color within [`Self::ALL`]
    /// (`fromIndex(I)`).
    pub fn from_index(index: i32) -> Option<WaypointColor> {
        WaypointColor::ALL.get(index as usize).copied()
    }

    /// The color for a given palette index (0-based), like the mod's
    /// `getRandom`-style indexing.
    pub fn index(self) -> i32 {
        WaypointColor::ALL
            .iter()
            .position(|c| *c == self)
            .map(|p| p as i32)
            .unwrap_or(-1)
    }

    /// Parses a color name as written by [`Self::name`].
    pub fn from_name(name: &str) -> Option<WaypointColor> {
        WaypointColor::ALL.into_iter().find(|c| c.name() == name)
    }

    /// The ARGB hex value of the color (`getHex`).
    pub fn hex(self) -> u32 {
        // Values taken from the 16 named colors the original mod defines.
        match self {
            WaypointColor::Black => 0xFF000000,
            WaypointColor::DarkBlue => 0xFF0000AA,
            WaypointColor::DarkGreen => 0xFF00AA00,
            WaypointColor::DarkAqua => 0xFF00AAAA,
            WaypointColor::DarkRed => 0xFFAA0000,
            WaypointColor::DarkPurple => 0xFFAA00AA,
            WaypointColor::Gold => 0xFFFFAA00,
            WaypointColor::Gray => 0xFFAAAAAA,
            WaypointColor::DarkGray => 0xFF555555,
            WaypointColor::Blue => 0xFF5555FF,
            WaypointColor::Green => 0xFF55FF55,
            WaypointColor::Aqua => 0xFF55FFFF,
            WaypointColor::Red => 0xFFFF5555,
            WaypointColor::Purple => 0xFFFF55FF,
            WaypointColor::Yellow => 0xFFFFFF55,
            WaypointColor::White => 0xFFFFFFFF,
        }
    }
}

/// `xaero/hud/minimap/waypoint/WaypointPurpose`: what a waypoint is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaypointPurpose {
    Normal,
    Death,
    /// Legacy waypoints persisted as "death" in very old versions.
    OldDeath,
    Destination,
}

impl WaypointPurpose {
    /// All variants in the original mod's order (`values()`).
    pub const ALL: [WaypointPurpose; 4] = [
        WaypointPurpose::Normal,
        WaypointPurpose::Death,
        WaypointPurpose::OldDeath,
        WaypointPurpose::Destination,
    ];

    /// `isDeath()`.
    pub fn is_death(self) -> bool {
        matches!(self, WaypointPurpose::Death | WaypointPurpose::OldDeath)
    }

    /// `isDestination()`.
    pub fn is_destination(self) -> bool {
        self.is_death() || matches!(self, WaypointPurpose::Destination)
    }

    /// Parses a purpose name as written by [`Self::name`].
    pub fn from_name(name: &str) -> Option<WaypointPurpose> {
        WaypointPurpose::ALL
            .into_iter()
            .find(|p| p.name() == name)
    }

    /// Name used when persisting purposes.
    pub fn name(self) -> &'static str {
        match self {
            WaypointPurpose::Normal => "normal",
            WaypointPurpose::Death => "death",
            WaypointPurpose::OldDeath => "old_death",
            WaypointPurpose::Destination => "destination",
        }
    }
}

/// `xaero/common/minimap/waypoints/Waypoint`: a single minimap waypoint.
///
/// Coordinates are block coordinates in the current dimension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waypoint {
    /// `xI` — block X.
    pub x: i32,
    /// `yI` — block Y (`0` if [`Waypoint::y_included`] is false).
    pub y: i32,
    /// `zI` — block Z.
    pub z: i32,
    /// `nameLjava/lang/String;`.
    pub name: String,
    /// `symbolLjava/lang/String;` (`getSymbol`/`setSymbol`).
    pub symbol: Option<String>,
    /// `colorLjava/lang/Integer;` (`getColor`/`setColor`).
    pub color: Option<WaypointColor>,
    /// Legacy type field (`getWaypointType`/`setType`).
    pub waypoint_type: i32,
    /// `purposeLxaero/hud/minimap/waypoint/WaypointPurpose;`.
    pub purpose: WaypointPurpose,
    /// `rotationZ`.
    pub rotation: bool,
    /// `yawI`.
    pub yaw: i32,
    /// `temporaryZ` (deathpoints are temporary).
    pub temporary: bool,
    /// `yIncludedZ`.
    pub y_included: bool,
    /// `createdAtJ` (milliseconds since the epoch).
    pub created_at: i64,
    /// `disabledLjava/lang/Boolean;`.
    pub disabled: Option<bool>,
    /// `visibility` (`getVisibility`/`setVisibility`).
    pub visibility: WaypointVisibilityType,
    /// `oneoffDestinationZ`.
    pub oneoff_destination: bool,
    /// Third-party origin marker (`setThirdPartyOrigin` / `isThirdParty`).
    pub third_party: bool,
    /// Origin id passed to `Waypoint.setThirdPartyOrigin(Identifier)`.
    pub third_party_origin: Option<String>,
    /// Marker for waypoints deleted by a third party.
    pub third_party_deleted: bool,
    /// `isEffectivelyDeleted`.
    pub effectively_deleted: bool,
}

impl Default for Waypoint {
    fn default() -> Self {
        Waypoint::new(0, 0, 0, "", "")
    }
}

impl Waypoint {
    /// Legacy-style constructor: name and symbol only
    /// (`Waypoint(IIILjava/lang/String;Ljava/lang/String;I)V` style).
    pub fn new(x: i32, y: i32, z: i32, name: &str, symbol: &str) -> Self {
        Waypoint {
            x,
            y,
            z,
            name: name.to_string(),
            symbol: if symbol.is_empty() { None } else { Some(symbol.to_string()) },
            color: None,
            waypoint_type: 0,
            third_party: false,
            third_party_origin: None,
            purpose: WaypointPurpose::Normal,
            rotation: false,
            yaw: 0,
            temporary: false,
            y_included: true,
            created_at: now_millis(),
            disabled: None,
            visibility: WaypointVisibilityType::Local,
            oneoff_destination: false,
            third_party_deleted: false,
            effectively_deleted: false,
        }
    }

    /// Constructor with a color.
    pub fn new_with_color(
        x: i32,
        y: i32,
        z: i32,
        name: &str,
        symbol: &str,
        color: WaypointColor,
    ) -> Self {
        Waypoint::new(x, y, z, name, symbol).with_color(color)
    }

    /// Constructor with color and purpose (used by deathpoints).
    pub fn with_color_and_purpose(
        x: i32,
        y: i32,
        z: i32,
        name: &str,
        symbol: &str,
        color: WaypointColor,
        purpose: WaypointPurpose,
    ) -> Self {
        Waypoint::new(x, y, z, name, symbol).with_color(color).with_purpose(purpose)
    }

    /// Constructor with color, purpose and temporary flag.
    pub fn with_color_purpose_and_temporary(
        x: i32,
        y: i32,
        z: i32,
        name: &str,
        symbol: &str,
        color: WaypointColor,
        purpose: WaypointPurpose,
        temporary: bool,
    ) -> Self {
        Waypoint::with_color_and_purpose(x, y, z, name, symbol, color, purpose)
            .with_temporary(temporary)
    }

    /// `getColor()I`.
    pub fn get_color(&self) -> i32 {
        self.color.map(|c| c.index()).unwrap_or(-1)
    }

    /// `getActualColor()` — the waypoint's color or the default.
    pub fn actual_color(&self) -> WaypointColor {
        self.color.unwrap_or(WaypointColor::DarkGray)
    }

    /// `isGlobal()`.
    pub fn is_global(&self) -> bool {
        self.visibility.is_global()
    }

    /// `getWaypointType()`.
    pub fn get_waypoint_type(&self) -> i32 {
        self.waypoint_type
    }

    /// `setType(I)V`.
    pub fn set_waypoint_type(&mut self, waypoint_type: i32) {
        self.waypoint_type = waypoint_type;
    }

    /// `isThirdParty()`.
    pub fn is_third_party(&self) -> bool {
        self.third_party
    }

    /// Fluent `with_third_party`.
    pub fn with_third_party(mut self, third_party: bool) -> Self {
        self.third_party = third_party;
        self
    }

    /// `setThirdPartyOrigin(Identifier)`.
    pub fn set_third_party_origin(&mut self, origin: impl Into<String>) {
        self.third_party = true;
        self.third_party_origin = Some(origin.into());
    }

    /// `getThirdPartyOrigin()`.
    pub fn third_party_origin(&self) -> Option<&str> {
        self.third_party_origin.as_deref()
    }

    /// `getSymbol()`.
    pub fn symbol(&self) -> Option<&str> {
        self.symbol.as_deref()
    }

    /// `getName()`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `getInitials()` — the uppercase initials of the name's words.
    pub fn initials(&self) -> String {
        let mut out = String::new();
        let mut prev_was_word_char = false;
        for c in self.name.chars() {
            if c.is_alphanumeric() {
                if !prev_was_word_char {
                    out.extend(c.to_uppercase());
                }
                prev_was_word_char = true;
            } else {
                prev_was_word_char = false;
            }
        }
        out
    }

    /// `getDistanceSq(DDD)D` — squared distance from the given block
    /// coordinates.
    pub fn distance_sq(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = (self.x as f64) - x;
        let dy = (self.y as f64) - y;
        let dz = (self.z as f64) - z;
        dx * dx + dy * dy + dz * dz
    }

    /// `isDestination()`.
    pub fn is_destination(&self) -> bool {
        self.purpose.is_destination()
    }

    /// `isOneoffDestination()`.
    pub fn is_oneoff_destination(&self) -> bool {
        self.oneoff_destination
    }

    /// `isEffectivelyDeleted()`.
    pub fn is_effectively_deleted(&self) -> bool {
        self.effectively_deleted || self.third_party_deleted
    }

    /// Fluent `with_color`.
    pub fn with_color(mut self, color: WaypointColor) -> Self {
        self.color = Some(color);
        self
    }

    /// Fluent `with_purpose`.
    pub fn with_purpose(mut self, purpose: WaypointPurpose) -> Self {
        self.purpose = purpose;
        self
    }

    /// Fluent `with_temporary`.
    pub fn with_temporary(mut self, temporary: bool) -> Self {
        self.temporary = temporary;
        self
    }

    /// Fluent `with_visibility`.
    pub fn with_visibility(mut self, visibility: WaypointVisibilityType) -> Self {
        self.visibility = visibility;
        self
    }
}

/// `xaero/hud/minimap/waypoint/WaypointUtil` — small waypoint helpers.
pub struct WaypointUtil;

impl WaypointUtil {
    /// Safe name used when a name is missing or would be blank
    /// (`getNameSafe`).
    pub fn name_safe(name: Option<&str>, default: &str) -> String {
        match name {
            Some(n) if !n.trim().is_empty() => n.to_string(),
            _ => default.to_string(),
        }
    }
}

/// How [`sort_waypoints`] orders a waypoint list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointsSort {
    /// Alphabetical by waypoint name.
    Name,
    /// Oldest waypoint first (`createdAt`).
    CreationTime,
}

/// `xaero/hud/minimap/waypoint/WaypointsSort`: sorts a waypoint list in place
/// by name, then by creation time.
pub fn sort_waypoints(waypoints: &mut [Waypoint], sort: WaypointsSort) {
    match sort {
        WaypointsSort::Name => waypoints.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.name.cmp(&b.name))
        }),
        WaypointsSort::CreationTime => waypoints.sort_by_key(|w| w.created_at),
    }
}

/// Sorts a waypoint list by squared distance from the given block
/// coordinates (like the mod's render sorting, which uses the player's
/// position).
pub fn sort_by_distance(waypoints: &mut [Waypoint], x: i32, y: i32, z: i32) {
    waypoints.sort_by(|a, b| {
        let da = a.distance_sq(x as f64, y as f64, z as f64);
        let db = b.distance_sq(x as f64, y as f64, z as f64);
        da.partial_cmp(&db).unwrap_or(Ordering::Equal)
    });
}

/// `xaero/hud/minimap/waypoint/set/WaypointSet`: a named group of waypoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointSet {
    name: String,
    waypoints: Vec<Waypoint>,
}

impl Default for WaypointSet {
    fn default() -> Self {
        WaypointSet::new("")
    }
}

impl WaypointSet {
    pub fn new(name: &str) -> Self {
        WaypointSet {
            name: name.to_string(),
            waypoints: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn get_waypoints(&self) -> &[Waypoint] {
        &self.waypoints
    }

    pub fn get_waypoints_mut(&mut self) -> &mut Vec<Waypoint> {
        &mut self.waypoints
    }

    pub fn add(&mut self, waypoint: Waypoint) {
        self.waypoints.push(waypoint);
    }

    pub fn add_all(&mut self, waypoints: &[Waypoint]) {
        self.waypoints.extend_from_slice(waypoints);
    }

    pub fn remove(&mut self, waypoint: &Waypoint) {
        self.waypoints.retain(|w| w != waypoint);
    }

    pub fn remove_at(&mut self, index: usize) -> Option<Waypoint> {
        if index < self.waypoints.len() {
            Some(self.waypoints.remove(index))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }

    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    pub fn get(&self, index: usize) -> Option<&Waypoint> {
        self.waypoints.get(index)
    }

    pub fn set(&mut self, index: usize, waypoint: Waypoint) -> Option<Waypoint> {
        self.waypoints.get_mut(index).map(|w| std::mem::replace(w, waypoint))
    }
}

/// `xaero/hud/minimap/waypoint/io/WaypointIO`: writes the waypoints of a
/// minimap world to a text stream.
///
/// The Rust port uses its own documented line format
/// (`xaerom-waypoints` v1, see [`WaypointIO::save_waypoints`]), one line per
/// set (`set=<name>`) and one per waypoint
/// (`wp=<name>;<x>;<y>;<z>;<color>;<visibility>;<purpose>;<flags>`),
/// mirroring the legacy file layout where each line describes one waypoint.
#[derive(Debug, Default)]
pub struct WaypointIO;

impl WaypointIO {
    pub fn new() -> Self {
        WaypointIO
    }

    /// `saveWaypoints(MinimapWorld, OutputStreamWriter)`: writes every
    /// waypoint set of `world` to `out`.
    pub fn save_waypoints(&self, world: &MinimapWorld, out: &mut dyn Write) -> io::Result<()> {
        for set in world.iter_waypoint_sets() {
            writeln!(out, "set={}", escape(set.get_name()))?;
            for wp in set.get_waypoints() {
                writeln!(
                    out,
                    "wp={};{};{};{};{};{};{};{};{};{}",
                    escape(&wp.name),
                    wp.x,
                    wp.y,
                    wp.z,
                    wp.color.map(|c| c.name()).unwrap_or(""),
                    wp.visibility.name(),
                    wp.purpose.name(),
                    i32::from(wp.rotation),
                    i32::from(wp.y_included),
                    wp.created_at,
                )?;
            }
        }
        out.flush()
    }

    /// Reads the format written by [`Self::save_waypoints`] back into `world`.
    pub fn load_waypoints(&self, world: &mut MinimapWorld, text: &str) -> io::Result<usize> {
        let mut added = 0;
        let mut current_set: Option<String> = None;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            match line.strip_prefix("set=") {
                Some(name) => {
                    current_set = Some(unescape(name));
                    world.add_waypoint_set(current_set.as_deref().unwrap());
                }
                None => match line.strip_prefix("wp=") {
                    Some(fields) => {
                        let parts: Vec<&str> = fields.split(';').collect();
                        if parts.len() != 10 {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("malformed waypoint line: {line}"),
                            ));
                        }
                        let name = unescape(parts[0]);
                        let x: i32 = parts[1].parse().map_err(|e| {
                            io::Error::new(io::ErrorKind::InvalidData, format!("x: {e}"))
                        })?;
                        let y: i32 = parts[2].parse().map_err(|e| {
                            io::Error::new(io::ErrorKind::InvalidData, format!("y: {e}"))
                        })?;
                        let z: i32 = parts[3].parse().map_err(|e| {
                            io::Error::new(io::ErrorKind::InvalidData, format!("z: {e}"))
                        })?;
                        let mut wp = Waypoint::new(x, y, z, &name, "");
                        wp.color = if parts[4].is_empty() {
                            None
                        } else {
                            WaypointColor::from_name(parts[4])
                        };
                        wp.visibility = WaypointVisibilityType::from_name(parts[5])
                            .unwrap_or(WaypointVisibilityType::Local);
                        wp.purpose = WaypointPurpose::from_name(parts[6])
                            .unwrap_or(WaypointPurpose::Normal);
                        wp.rotation = parts[7] != "0";
                        wp.y_included = parts[8] != "0";
                        wp.created_at = parts[9].parse().unwrap_or(0);
                        if let Some(set) = current_set.as_deref() {
                            if let Some(set_obj) = world.get_waypoint_set_mut(set) {
                                set_obj.add(wp);
                                added += 1;
                            }
                        }
                    }
                    None => { /* ignore unknown lines */ }
                },
            }
        }
        Ok(added)
    }
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ';' => out.push_str("\\;"),
            '=' => out.push_str("\\="),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(';') => out.push(';'),
                Some('=') => out.push('='),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                other => {
                    out.push('\\');
                    if let Some(o) = other {
                        out.push(o);
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Millisecond timestamp (used for `Waypoint::created_at`).
pub fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility_variants_round_trip_by_name() {
        for v in WaypointVisibilityType::ALL {
            assert_eq!(WaypointVisibilityType::from_name(v.name()), Some(v));
        }
    }

    #[test]
    fn purpose_is_death_and_destination() {
        assert!(WaypointPurpose::Death.is_death());
        assert!(WaypointPurpose::OldDeath.is_death());
        assert!(WaypointPurpose::Destination.is_destination());
        assert!(!WaypointPurpose::Normal.is_destination());
    }

    #[test]
    fn color_from_index_and_name() {
        assert_eq!(WaypointColor::from_index(2), Some(WaypointColor::DarkGreen));
        assert_eq!(WaypointColor::Red.name(), "red");
        assert_eq!(WaypointColor::from_name("red"), Some(WaypointColor::Red));
        assert_eq!(WaypointColor::from_name("nope"), None);
        assert_eq!(WaypointColor::Red.hex(), 0xFFFF5555);
    }

    #[test]
    fn waypoint_distance_sq_and_initials() {
        let wp = Waypoint::new(1, 2, 3, "my way point", "");
        assert_eq!(wp.distance_sq(1.0, 2.0, 3.0), 0.0);
        assert_eq!(wp.initials(), "MWP");
        assert_eq!(WaypointUtil::name_safe(Some(""), "fallback"), "fallback");
    }

    #[test]
    fn waypoint_set_basics() {
        let mut set = WaypointSet::new("set1");
        assert!(set.is_empty());
        set.add(Waypoint::new(1, 2, 3, "a", ""));
        set.add(Waypoint::new(4, 5, 6, "b", ""));
        assert_eq!(set.len(), 2);
        assert_eq!(set.get_waypoints()[0].name, "a");
        set.remove_at(0);
        assert_eq!(set.len(), 1);
        assert_eq!(set.get_waypoints()[0].name, "b");
        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn sort_waypoints_by_name_and_time() {
        let mut wps = vec![
            Waypoint::new(1, 0, 0, "zeta", ""),
            Waypoint::new(1, 0, 0, "alpha", ""),
        ];
        sort_waypoints(&mut wps, WaypointsSort::Name);
        assert_eq!(wps[0].name, "alpha");
        assert_eq!(wps[1].name, "zeta");
    }
}
