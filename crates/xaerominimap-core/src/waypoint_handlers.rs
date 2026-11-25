//! Waypoint lifecycle helpers from `xaero/hud/minimap/waypoint`: deathpoints,
//! temporary waypoints, destination reach handling and waypoint collection.
//!
//! The Java handlers reach into `Minecraft`, `HudMod` and the config manager
//! for player coordinates and settings. The Rust port takes those values as
//! parameters and keeps the deterministic parts: which waypoints are created,
//! renamed, collected or deleted. IO (`saveWorld`) and GUI checks stay in the
//! game glue.

use crate::waypoints::{Waypoint, WaypointColor, WaypointPurpose};
use crate::world::MinimapWorld;

/// Effective config flags used by [`DeathpointHandler`]
/// (`MinimapProfiledConfigOptions.DEATHPOINTS` / `OLD_DEATHPOINTS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeathpointSettings {
    pub enabled: bool,
    pub old_deathpoints: bool,
}

/// `xaero/hud/minimap/waypoint/DeathpointHandler`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeathpointHandler;

impl DeathpointHandler {
    /// The Java translation keys used for the created/renamed deathpoints.
    pub const DEATHPOINT_NAME: &'static str = "gui.xaero_deathpoint";
    pub const OLD_DEATHPOINT_NAME: &'static str = "gui.xaero_deathpoint_old";
    pub const DEATHPOINT_SYMBOL: &'static str = "D";

    /// `createDeathpoint(Player, MinimapWorld, boolean)` — creates a deathpoint
    /// at the player's floored coordinates and returns it. `dim_div` is the
    /// dimension division applied to X/Z (`MinimapDimensionHelper`).
    pub fn create_deathpoint(
        world: &mut MinimapWorld,
        x: f64,
        y: f64,
        z: f64,
        dim_div: f64,
        settings: DeathpointSettings,
    ) -> Option<Waypoint> {
        if !settings.enabled || world.get_current_waypoint_set().is_none() {
            return None;
        }

        // Old deathpoints are demoted to OLD_DEATH and renamed before the
        // new one is added. A disabled old deathpoint stays disabled.
        let mut old_disabled = false;
        if settings.old_deathpoints {
            for set in world.iter_waypoint_sets_mut() {
                for waypoint in set.get_waypoints_mut() {
                    if waypoint.purpose == WaypointPurpose::Death {
                        old_disabled = waypoint.disabled.unwrap_or(false);
                        waypoint.purpose = WaypointPurpose::OldDeath;
                        waypoint.name = Self::OLD_DEATHPOINT_NAME.to_string();
                    }
                }
            }
        }

        let mut waypoint = Waypoint::new(
            (x / dim_div).floor() as i32,
            y.floor() as i32,
            (z / dim_div).floor() as i32,
            Self::DEATHPOINT_NAME,
            Self::DEATHPOINT_SYMBOL,
        )
        .with_color(WaypointColor::Black)
        .with_purpose(WaypointPurpose::Death);
        waypoint.temporary = true;
        waypoint.disabled = Some(old_disabled);
        // Death waypoints are global (`Waypoint` constructor behavior).
        waypoint.visibility = crate::waypoints::WaypointVisibilityType::Global;

        world
            .get_current_waypoint_set_mut()
            .expect("checked above")
            .add(waypoint.clone());
        Some(waypoint)
    }
}

/// `xaero/hud/minimap/waypoint/TemporaryWaypointHandler`.
#[derive(Debug, Clone, Copy, Default)]
pub struct TemporaryWaypointHandler;

impl TemporaryWaypointHandler {
    pub const WAYPOINT_NAME: &'static str = "Waypoint";
    pub const WAYPOINT_SYMBOL: &'static str = "X";

    /// `createTemporaryWaypoint(MinimapWorld, int, int, int, boolean, double)`.
    /// `coordinate_scale` is the dimension's coordinate scale divided by the
    /// world's coordinate scale (the Java `dimDiv` calculation).
    pub fn create_temporary_waypoint(
        world: &mut MinimapWorld,
        x: i32,
        y: i32,
        z: i32,
        y_included: bool,
        coordinate_scale: f64,
        to_bottom: bool,
    ) -> Option<Waypoint> {
        let scale = coordinate_scale.max(f64::MIN_POSITIVE);
        let mut waypoint = Waypoint::new(
            (x as f64 * scale).floor() as i32,
            y,
            (z as f64 * scale).floor() as i32,
            Self::WAYPOINT_NAME,
            Self::WAYPOINT_SYMBOL,
        )
        .with_color(
            WaypointColor::from_index(
                (x as i64 + y as i64 + z as i64).rem_euclid(WaypointColor::ALL.len() as i64) as i32,
            )
            .unwrap_or(WaypointColor::Red),
        )
        .with_purpose(WaypointPurpose::Normal);
        waypoint.temporary = true;
        waypoint.y_included = y_included;

        let set = world.get_current_waypoint_set_mut()?;
        if to_bottom {
            set.add(waypoint.clone());
        } else {
            set.get_waypoints_mut().insert(0, waypoint.clone());
        }
        Some(waypoint)
    }
}

/// `xaero/hud/minimap/waypoint/WaypointCollector`: gathers the waypoints the
/// minimap renderer should draw.
#[derive(Debug, Clone, Copy, Default)]
pub struct WaypointCollector;

impl WaypointCollector {
    /// `collect(List<Waypoint>)`. Pass the third-party/custom waypoints as
    /// `extra`; the collector skips effectively-deleted entries from every
    /// source, like the Java implementation.
    pub fn collect(
        world: &MinimapWorld,
        all_sets: bool,
        extra: &[Waypoint],
    ) -> Vec<Waypoint> {
        let mut result = Vec::new();
        if all_sets {
            for set in world.iter_waypoint_sets() {
                result.extend(
                    set.get_waypoints()
                        .iter()
                        .filter(|waypoint| !waypoint.is_effectively_deleted())
                        .cloned(),
                );
            }
        } else if let Some(set) = world.get_current_waypoint_set() {
            result.extend(
                set.get_waypoints()
                    .iter()
                    .filter(|waypoint| !waypoint.is_effectively_deleted())
                    .cloned(),
            );
        }
        result.extend(
            extra
                .iter()
                .filter(|waypoint| !waypoint.third_party_deleted && !waypoint.is_effectively_deleted())
                .cloned(),
        );
        result
    }
}

/// `xaero/hud/minimap/waypoint/DestinationHandler`: collects destinations the
/// player has reached so they can be deleted (`WaypointDeleter`).
#[derive(Debug, Clone, Default)]
pub struct DestinationHandler {
    dim_div: f64,
    collected: Vec<Waypoint>,
}

impl DestinationHandler {
    /// `WaypointDeleter` reach radius in blocks.
    pub const REACH_DISTANCE: f64 = 4.0;
    /// Death waypoints are only deleted shortly after being created.
    pub const DEATHPOINT_GRACE_MS: i64 = 5000;

    pub fn new(dim_div: f64) -> Self {
        DestinationHandler {
            dim_div,
            collected: Vec::new(),
        }
    }

    /// `begin(...)` starts a collection pass with the dimension division.
    pub fn begin(&mut self, dim_div: f64) {
        self.dim_div = dim_div;
        self.collected.clear();
    }

    /// `handle(Waypoint)`: returns whether the waypoint was reached and added
    /// to the deletion list. `now_millis` is `System.currentTimeMillis()`.
    pub fn handle(
        &mut self,
        waypoint: &Waypoint,
        entity_x: f64,
        entity_y: f64,
        entity_z: f64,
        now_millis: i64,
    ) -> bool {
        if !waypoint.is_destination() || waypoint.disabled.unwrap_or(false) {
            return false;
        }
        if waypoint.purpose.is_death()
            && now_millis.saturating_sub(waypoint.created_at) > Self::DEATHPOINT_GRACE_MS
        {
            return false;
        }
        let dx = (waypoint.x as f64) * self.dim_div - entity_x;
        let dy = if waypoint.y_included {
            (waypoint.y as f64) - entity_y
        } else {
            0.0
        };
        let dz = (waypoint.z as f64) * self.dim_div - entity_z;
        if (dx * dx + dy * dy + dz * dz).sqrt() >= Self::REACH_DISTANCE {
            return false;
        }
        if !self.collected.iter().any(|entry| entry == waypoint) {
            self.collected.push(waypoint.clone());
        }
        true
    }

    /// `end()`: the waypoints collected during this pass, for the glue to
    /// delete (`WaypointDeleter.deleteCollected`).
    pub fn end(&mut self) -> Vec<Waypoint> {
        std::mem::take(&mut self.collected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_deathpoints_and_demotes_old_ones() {
        let mut world = MinimapWorld::new("overworld", Default::default());
        world.add_waypoint_set("Default");
        let mut old = Waypoint::new(5, 0, 5, "old", "D").with_purpose(WaypointPurpose::Death);
        old.disabled = Some(true);
        world.get_waypoint_set_mut("Default").unwrap().add(old);

        let settings = DeathpointSettings {
            enabled: true,
            old_deathpoints: true,
        };
        let created = DeathpointHandler::create_deathpoint(
            &mut world, 10.9, 64.7, -20.2, 8.0, settings,
        )
        .unwrap();
        assert_eq!((created.x, created.y, created.z), (1, 64, -3));
        assert_eq!(created.name, DeathpointHandler::DEATHPOINT_NAME);
        assert_eq!(created.symbol, Some(DeathpointHandler::DEATHPOINT_SYMBOL.to_string()));
        assert_eq!(created.color, Some(WaypointColor::Black));
        assert!(created.temporary);
        assert_eq!(created.disabled, Some(true));
        assert_eq!(created.visibility, crate::waypoints::WaypointVisibilityType::Global);

        let set = world.get_current_waypoint_set().unwrap();
        let old = &set.get_waypoints()[0];
        assert_eq!(old.purpose, WaypointPurpose::OldDeath);
        assert_eq!(old.name, DeathpointHandler::OLD_DEATHPOINT_NAME);
        assert_eq!(set.get_waypoints().len(), 2);

        // Disabled creation leaves the world untouched.
        let count = set.get_waypoints().len();
        assert!(DeathpointHandler::create_deathpoint(
            &mut world,
            1.0,
            1.0,
            1.0,
            1.0,
            DeathpointSettings { enabled: false, old_deathpoints: false },
        )
        .is_none());
        assert_eq!(world.get_current_waypoint_set().unwrap().get_waypoints().len(), count);
    }

    #[test]
    fn temporary_waypoint_respects_scale_and_insertion_order() {
        let mut world = MinimapWorld::new("overworld", Default::default());
        world.add_waypoint_set("Default");
        world.get_waypoint_set_mut("Default").unwrap().add(Waypoint::new(0, 0, 0, "existing", ""));

        let first = TemporaryWaypointHandler::create_temporary_waypoint(
            &mut world, 10, 70, -4, true, 0.5, false,
        )
        .unwrap();
        assert_eq!((first.x, first.y, first.z), (5, 70, -2));
        assert!(first.temporary && first.y_included);
        assert_eq!(first.name, TemporaryWaypointHandler::WAYPOINT_NAME);
        assert_eq!(first.symbol, Some(TemporaryWaypointHandler::WAYPOINT_SYMBOL.to_string()));
        assert_eq!(world.get_current_waypoint_set().unwrap().get_waypoints()[0], first);

        TemporaryWaypointHandler::create_temporary_waypoint(
            &mut world, 2, 3, 4, false, 1.0, true,
        );
        let waypoints = world.get_current_waypoint_set().unwrap().get_waypoints();
        assert_eq!(waypoints.len(), 3);
        assert_eq!(waypoints[0].name, "Waypoint");
        assert_eq!(waypoints[1].name, "existing");
        assert_eq!(waypoints[2].name, "Waypoint");
        assert!(!waypoints[2].y_included);
    }

    #[test]
    fn collects_from_all_sets_or_only_the_current_one() {
        let mut world = MinimapWorld::new("overworld", Default::default());
        world.add_waypoint_set("Default");
        world.add_waypoint_set("Other");
        world.get_waypoint_set_mut("Default").unwrap().add(Waypoint::new(1, 0, 1, "one", ""));
        world.get_waypoint_set_mut("Other").unwrap().add(Waypoint::new(2, 0, 2, "two", ""));
        world.set_current_waypoint_set_id("Default");

        assert_eq!(WaypointCollector::collect(&world, false, &[]).len(), 1);
        assert_eq!(WaypointCollector::collect(&world, true, &[]).len(), 2);

        let mut external = Waypoint::new(3, 0, 3, "external", "");
        external.third_party = true;
        let mut deleted = Waypoint::new(4, 0, 4, "deleted", "");
        deleted.third_party_deleted = true;
        let collected = WaypointCollector::collect(&world, false, &[external, deleted]);
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[1].name, "external");
    }

    #[test]
    fn destination_handler_collects_reached_destinations() {
        let mut handler = DestinationHandler::new(8.0);
        let mut destination = Waypoint::new(1, 0, 1, "home", "")
            .with_purpose(WaypointPurpose::Destination);
        destination.created_at = 1000;
        let now = 3000;

        // Player at x=8, z=8 (same block as the waypoint at 1,1).
        assert!(handler.handle(&destination, 8.0, 0.0, 8.0, now));
        assert!(!handler.handle(&destination, 100.0, 0.0, 100.0, now));
        assert_eq!(handler.end(), vec![destination.clone()]);
        assert!(handler.end().is_empty());

        // Recent deathpoints are eligible; old ones are not.
        let mut deathpoint = Waypoint::new(1, 0, 1, "death", "").with_purpose(WaypointPurpose::Death);
        deathpoint.created_at = now - 6000;
        assert!(!handler.handle(&deathpoint, 8.0, 0.0, 8.0, now));
        deathpoint.created_at = now - 4000;
        assert!(handler.handle(&deathpoint, 8.0, 0.0, 8.0, now));
    }
}
