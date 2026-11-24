//! Port of `xaero/hud/minimap/waypoint/WaypointRenderInfo`: the display info
//! a waypoint renderer uses on the map (initials, color, visibility,
//! disabled state) — the data layer of the waypoint renderers; the actual
//! GL drawing stays in the game client.

use xaerominimap_core::waypoints::{Waypoint, WaypointColor, WaypointVisibilityType};

/// `xaero/hud/minimap/waypoint/WaypointRenderInfo`.
#[derive(Debug, Clone, Default)]
pub struct WaypointRenderInfo {
    pub initials: Option<String>,
    pub color: Option<WaypointColor>,
    pub visibility: WaypointVisibilityType,
    pub disabled: Option<bool>,
    /// `actualColorI`.
    pub actual_color: i32,
    pub third_party_deleted: bool,
}

impl WaypointRenderInfo {
    pub fn new() -> Self {
        WaypointRenderInfo::default()
    }

    /// `getInitials()`.
    pub fn get_initials(&self) -> Option<&str> {
        self.initials.as_deref()
    }

    /// `setInitials(...)`.
    pub fn set_initials(&mut self, initials: &str) {
        self.initials = Some(initials.to_string());
    }

    /// `getInitialsSafe(String)` — the initials or the safe default.
    pub fn get_initials_safe(&self, default: &str) -> String {
        self.initials
            .clone()
            .filter(|i| !i.trim().is_empty())
            .unwrap_or_else(|| default.to_string())
    }

    /// `getWaypointColor()` — the info's color or the waypoint's default.
    pub fn get_waypoint_color(&self) -> WaypointColor {
        self.color.unwrap_or(WaypointColor::DarkGray)
    }

    /// `setWaypointColor(...)`.
    pub fn set_waypoint_color(&mut self, color: WaypointColor) {
        self.color = Some(color);
    }

    /// `getVisibility()`.
    pub fn get_visibility(&self) -> WaypointVisibilityType {
        self.visibility
    }

    /// `setVisibility(...)`.
    pub fn set_visibility(&mut self, visibility: WaypointVisibilityType) {
        self.visibility = visibility;
    }

    /// `getDisabled()`.
    pub fn get_disabled(&self) -> Option<bool> {
        self.disabled
    }

    /// `setDisabled(...)`.
    pub fn set_disabled(&mut self, disabled: Option<bool>) {
        self.disabled = disabled;
    }

    /// `isThirdPartyDeleted()`.
    pub fn is_third_party_deleted(&self) -> bool {
        self.third_party_deleted
    }

    /// `setThirdPartyDeleted(Z)`.
    pub fn set_third_party_deleted(&mut self, deleted: bool) {
        self.third_party_deleted = deleted;
    }

    /// `nullEverythingMatching(Waypoint)` — clears fields that match the
    /// given waypoint (used when refreshing rendering info).
    pub fn null_everything_matching(&mut self, waypoint: &Waypoint) {
        if self.initials.is_some() && self.get_initials() == Some(waypoint.name()) {
            self.initials = None;
        }
        if self.color == waypoint.color {
            self.color = None;
        }
        self.clear();
    }

    /// `clear()`.
    pub fn clear(&mut self) {
        self.initials = None;
        self.color = None;
        self.visibility = WaypointVisibilityType::Local;
        self.disabled = None;
        self.actual_color = 0;
        self.third_party_deleted = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_info_basics() {
        let mut info = WaypointRenderInfo::new();
        info.set_initials("W");
        assert_eq!(info.get_initials(), Some("W"));
        assert_eq!(info.get_initials_safe("D"), "W");
        assert_eq!(WaypointRenderInfo::new().get_initials_safe("D"), "D");
        info.set_waypoint_color(WaypointColor::Green);
        assert_eq!(info.get_waypoint_color(), WaypointColor::Green);
        info.set_visibility(WaypointVisibilityType::Global);
        assert_eq!(info.get_visibility(), WaypointVisibilityType::Global);
        info.set_disabled(Some(false));
        assert_eq!(info.get_disabled(), Some(false));
        info.set_third_party_deleted(true);
        assert!(info.is_third_party_deleted());
        info.clear();
        assert_eq!(info.get_initials(), None);
        assert_eq!(info.get_disabled(), None);
    }

    #[test]
    fn null_everything_matching() {
        let mut info = WaypointRenderInfo::new();
        info.set_initials("test");
        let wp = Waypoint::new(1, 2, 3, "test", "");
        info.null_everything_matching(&wp);
        assert_eq!(info.get_initials(), None);
    }
}
