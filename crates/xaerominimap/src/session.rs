//! `xaero/hud/minimap/module/MinimapSession`-style session state: the current
//! world and the settings of the minimap.

use xaerominimap_core::world::{DimensionKey, MinimapWorld};
use xaerominimap_settings::Settings;

/// Session state of the minimap: the world it is shown in and its settings.
#[derive(Debug, Clone, Default)]
pub struct MinimapSession {
    world: Option<MinimapWorld>,
    settings: Settings,
}

impl MinimapSession {
    pub fn new() -> Self {
        MinimapSession::default()
    }

    /// Session for a world that already exists.
    pub fn with_world(world: MinimapWorld) -> Self {
        MinimapSession {
            world: Some(world),
            ..Default::default()
        }
    }

    /// The current world, if any.
    pub fn world(&self) -> Option<&MinimapWorld> {
        self.world.as_ref()
    }

    /// Mutable access to the current world.
    pub fn world_mut(&mut self) -> Option<&mut MinimapWorld> {
        self.world.as_mut()
    }

    /// Sets the session's world.
    pub fn set_world(&mut self, world: MinimapWorld) {
        self.world = Some(world);
    }

    /// The session settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Mutable access to the session settings.
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Returns the world, creating one for `dim_id` if there is none yet
    /// (`getOrInsert`-style, like the Java session helpers that make a
    /// world on first use).
    pub fn ensure_world(&mut self, dim_id: DimensionKey) -> &mut MinimapWorld {
        self.world
            .get_or_insert_with(|| MinimapWorld::new("", dim_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_ensure_world_and_settings() {
        let mut session = MinimapSession::new();
        assert!(session.world().is_none());
        let world = session.ensure_world(DimensionKey::overworld());
        world.add_waypoint_set("Default");
        assert_eq!(session.world().unwrap().get_set_count(), 1);
        session.settings_mut().set("minimapScale", 0.5f32);
        assert_eq!(session.settings().float("minimapScale", 0.0), 0.5);
    }
}
