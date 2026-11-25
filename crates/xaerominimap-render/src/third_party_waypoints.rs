//! Third-party waypoint origins, ported from
//! `xaero/hud/minimap/waypoint/thirdparty` of the original mod.
//!
//! Mods that expose waypoints on the minimap register an origin id, then add
//! waypoint instances to it. The Java code stores the waypoints both by id
//! and in a list for slot-based access, keeps render-info overrides per id and
//! lets the origin decide whether its waypoints are enabled. Saving the root
//! config after a removal (`ThirdPartyWaypoints.remove`) is a game-side side
//! effect and is intentionally left to the Horizon glue; this module owns
//! only the in-memory state.

use std::collections::BTreeMap;

use xaerominimap_core::waypoints::Waypoint;

use crate::waypoint_render::WaypointRenderInfo;

/// `xaero/hud/minimap/waypoint/thirdparty/ThirdPartyWaypoints`: the
/// waypoints contributed by one mod (origin).
#[derive(Debug, Clone)]
pub struct ThirdPartyWaypoints {
    origin_id: String,
    waypoints: BTreeMap<String, Waypoint>,
    /// Insertion order, used by `getBySlot`.
    order: Vec<String>,
    render_info_overrides: BTreeMap<String, WaypointRenderInfo>,
    /// Java `Supplier<Boolean>`; a plain function pointer keeps the type
    /// clonable and independent of the game runtime.
    enabled_state_getter: Option<fn() -> bool>,
}

impl ThirdPartyWaypoints {
    pub fn new(origin_id: impl Into<String>) -> Self {
        ThirdPartyWaypoints {
            origin_id: origin_id.into(),
            waypoints: BTreeMap::new(),
            order: Vec::new(),
            render_info_overrides: BTreeMap::new(),
            enabled_state_getter: None,
        }
    }

    /// `clear()` — clears waypoints and ids but keeps the render overrides
    /// and the enabled-state getter, as the Java method does.
    pub fn clear(&mut self) {
        self.waypoints.clear();
        self.order.clear();
    }

    /// `remove(String)`: forgets the waypoint, its slot and its override.
    pub fn remove(&mut self, id: &str) {
        if self.waypoints.remove(id).is_none() {
            return;
        }
        self.order.retain(|entry| entry != id);
        self.render_info_overrides.remove(id);
    }

    /// `add(String, Waypoint)`: replaces a waypoint with the same id,
    /// marks it as third-party, and records the render override (either the
    /// one supplied by the origin or the waypoint's existing one).
    pub fn add(&mut self, id: impl Into<String>, waypoint: Waypoint) {
        let id = id.into();
        let mut waypoint = waypoint;
        waypoint.set_third_party_origin(self.origin_id.clone());
        // An override registered by the origin takes precedence; otherwise
        // the waypoint's current display info becomes the stored override.
        if !self.render_info_overrides.contains_key(&id) {
            self.render_info_overrides
                .insert(id.clone(), render_info_from_waypoint(&waypoint));
        }
        // Replacing an id keeps its slot so later `getBySlot` lookups stay
        // stable (the Java list keeps slot order too).
        if !self.order.contains(&id) {
            self.order.push(id.clone());
        }
        self.waypoints.insert(id, waypoint);
    }

    /// `get(String)`.
    pub fn get(&self, id: &str) -> Option<&Waypoint> {
        self.waypoints.get(id)
    }

    /// `getBySlot(int)` — slot in insertion order.
    pub fn get_by_slot(&self, slot: usize) -> Option<&Waypoint> {
        self.order.get(slot).and_then(|id| self.waypoints.get(id))
    }

    /// `getIds()` — the ids in insertion order.
    pub fn get_ids(&self) -> &[String] {
        &self.order
    }

    /// `addRenderInfoOverride(String, WaypointRenderInfo)`.
    pub fn add_render_info_override(&mut self, id: impl Into<String>, info: WaypointRenderInfo) {
        self.render_info_overrides.insert(id.into(), info);
    }

    /// The render info the minimap should use for a waypoint from this
    /// origin (`Waypoint.getThirdPartyRenderOverride`).
    pub fn render_info_for(&self, id: &str) -> Option<WaypointRenderInfo> {
        self.render_info_overrides.get(id).cloned()
    }

    /// `isEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }

    /// `getWaypoints()`.
    pub fn waypoints(&self) -> &BTreeMap<String, Waypoint> {
        &self.waypoints
    }

    /// `getCount()`.
    pub fn count(&self) -> usize {
        self.waypoints.len()
    }

    /// `getOriginId()`.
    pub fn origin_id(&self) -> &str {
        &self.origin_id
    }

    /// `getRenderInfoOverrides()`.
    pub fn render_info_overrides(&self) -> &BTreeMap<String, WaypointRenderInfo> {
        &self.render_info_overrides
    }

    /// `setEnabledStateGetter(Supplier<Boolean>)`.
    pub fn set_enabled_state_getter(&mut self, getter: fn() -> bool) {
        self.enabled_state_getter = Some(getter);
    }

    /// `isEnabled()` — `true` when no getter is installed.
    pub fn is_enabled(&self) -> bool {
        self.enabled_state_getter.map(|getter| getter()).unwrap_or(true)
    }

    /// `hasEnabledStateGetter()`.
    pub fn has_enabled_state_getter(&self) -> bool {
        self.enabled_state_getter.is_some()
    }
}

/// Small helper mirroring fields of `WaypointRenderInfo` that a third-party
/// override can supply.
fn render_info_from_waypoint(waypoint: &Waypoint) -> WaypointRenderInfo {
    let mut info = WaypointRenderInfo::new();
    info.set_initials(&waypoint.initials());
    if let Some(color) = waypoint.color {
        info.set_waypoint_color(color);
    }
    info.set_visibility(waypoint.visibility);
    info.set_disabled(waypoint.disabled);
    info
}

/// `xaero/hud/minimap/waypoint/thirdparty/ThirdPartyWaypointManager`.
#[derive(Debug, Clone, Default)]
pub struct ThirdPartyWaypointManager {
    storage: BTreeMap<String, ThirdPartyWaypoints>,
    ordered_storage: Vec<String>,
}

impl ThirdPartyWaypointManager {
    pub fn new() -> Self {
        ThirdPartyWaypointManager::default()
    }

    /// `clear()` — clears every origin's waypoints but keeps the origins
    /// registered, like the Java implementation.
    pub fn clear(&mut self) {
        for origin in self.storage.values_mut() {
            origin.clear();
        }
    }

    /// `clearOrigin(Identifier)`.
    pub fn clear_origin(&mut self, origin_id: &str) {
        if let Some(origin) = self.storage.get_mut(origin_id) {
            origin.clear();
        }
    }

    /// `getOriginCount()`.
    pub fn origin_count(&self) -> usize {
        self.storage.len()
    }

    /// `get(Identifier)` — the Java `computeIfAbsent` creates a new origin.
    pub fn get_or_create(&mut self, origin_id: impl Into<String>) -> &mut ThirdPartyWaypoints {
        let origin_id = origin_id.into();
        if !self.storage.contains_key(&origin_id) {
            self.ordered_storage.push(origin_id.clone());
            self.storage
                .insert(origin_id.clone(), ThirdPartyWaypoints::new(origin_id.clone()));
        }
        self.storage.get_mut(&origin_id).unwrap()
    }

    /// Non-creating lookup.
    pub fn get(&self, origin_id: &str) -> Option<&ThirdPartyWaypoints> {
        self.storage.get(origin_id)
    }

    /// `getByIndex(int)`.
    pub fn get_by_index(&self, index: usize) -> Option<&ThirdPartyWaypoints> {
        self.ordered_storage
            .get(index)
            .and_then(|origin_id| self.storage.get(origin_id))
    }

    /// `getCount()` — the sum over all origins.
    pub fn count(&self) -> usize {
        self.storage.values().map(ThirdPartyWaypoints::count).sum()
    }

    /// `getAll()` — origins in registration order.
    pub fn get_all(&self) -> Vec<&ThirdPartyWaypoints> {
        self.ordered_storage
            .iter()
            .filter_map(|origin_id| self.storage.get(origin_id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xaerominimap_core::waypoints::WaypointColor;

    fn waypoint(name: &str) -> Waypoint {
        Waypoint::new(1, 2, 3, name, "")
            .with_color(WaypointColor::Red)
    }

    #[test]
    fn waypoints_are_stored_by_id_and_by_slot() {
        let mut origin = ThirdPartyWaypoints::new("mymod:waystones");
        assert!(origin.is_empty());
        assert!(origin.is_enabled());

        origin.add("first", waypoint("Alpha"));
        origin.add("second", waypoint("Beta"));
        assert_eq!(origin.count(), 2);
        assert_eq!(origin.get_ids(), &["first", "second"]);
        assert_eq!(origin.get("first").unwrap().name(), "Alpha");
        assert_eq!(origin.get_by_slot(1).unwrap().name(), "Beta");
        assert_eq!(
            origin.get("first").unwrap().third_party_origin(),
            Some("mymod:waystones")
        );

        // Re-adding an id replaces the waypoint without growing the list.
        origin.add("first", waypoint("Alpha 2"));
        assert_eq!(origin.count(), 2);
        assert_eq!(origin.get_by_slot(0).unwrap().name(), "Alpha 2");

        origin.remove("first");
        assert_eq!(origin.count(), 1);
        assert_eq!(origin.get("first"), None);
        assert_eq!(origin.get_by_slot(0).unwrap().name(), "Beta");
        assert!(origin.get_by_slot(1).is_none());
    }

    #[test]
    fn render_overrides_and_enabled_state() {
        let mut origin = ThirdPartyWaypoints::new("mymod:waystones");
        let mut override_info = WaypointRenderInfo::new();
        override_info.set_initials("OW");
        origin.add_render_info_override("one", override_info);
        origin.add("one", waypoint("Origin Waypoint"));
        assert_eq!(
            origin.render_info_for("one").unwrap().get_initials(),
            Some("OW")
        );

        origin.clear();
        assert!(origin.is_empty());
        // Overrides survive `clear()`, mirroring the Java implementation.
        assert!(origin.render_info_overrides().contains_key("one"));

        assert!(!origin.has_enabled_state_getter());
        origin.set_enabled_state_getter(|| false);
        assert!(origin.has_enabled_state_getter());
        assert!(!origin.is_enabled());
    }

    #[test]
    fn manager_keeps_origins_in_registration_order() {
        let mut manager = ThirdPartyWaypointManager::new();
        manager.get_or_create("mymod:a").add("a", waypoint("A"));
        manager.get_or_create("mymod:b").add("b", waypoint("B"));
        manager.get_or_create("mymod:a").add("c", waypoint("C"));
        assert_eq!(manager.origin_count(), 2);
        assert_eq!(manager.count(), 3);
        assert_eq!(manager.get_by_index(0).unwrap().origin_id(), "mymod:a");
        assert_eq!(manager.get_by_index(1).unwrap().origin_id(), "mymod:b");
        assert_eq!(manager.get_all().len(), 2);

        manager.clear_origin("mymod:b");
        assert_eq!(manager.get("mymod:b").unwrap().count(), 0);
        assert_eq!(manager.count(), 2);

        manager.clear();
        assert_eq!(manager.count(), 0);
        assert_eq!(manager.origin_count(), 2);
    }
}
