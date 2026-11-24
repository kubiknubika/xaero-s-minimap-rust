//! Entity radar state (mirrors `xaero/hud/minimap/radar/state` of the
//! original mod).
//!
//! The Java mod reads live entities from the Minecraft client; the Rust port
//! keeps the state containers and lets the game-side glue feed entities in.

/// A tracked entity in the radar (the port's stand-in for
/// `net.minecraft.world.entity.Entity`).
#[derive(Debug, Clone, PartialEq)]
pub struct RadarEntity {
    /// Entity name/display name.
    pub name: String,
    /// Block X position.
    pub x: f64,
    /// Block Y position.
    pub y: f64,
    /// Block Z position.
    pub z: f64,
    /// Entity type (e.g. `zombie`, `creeper`, …).
    pub entity_type: String,
    /// UUID, used as a stable identity (the Java mod uses the `Entity`).
    pub uuid: String,
}

impl RadarEntity {
    pub fn new(name: &str, x: f64, y: f64, z: f64, entity_type: &str) -> Self {
        RadarEntity {
            name: name.to_string(),
            x,
            y,
            z,
            entity_type: entity_type.to_string(),
            uuid: String::new(),
        }
    }
}

/// `xaero/hud/minimap/radar/state/RadarList`: the entities of one radar
/// category (client or synced).
#[derive(Debug, Clone, Default)]
pub struct RadarList {
    entities: Vec<RadarEntity>,
}

impl RadarList {
    pub fn new() -> Self {
        RadarList::default()
    }

    /// `add(Entity)`.
    pub fn add(&mut self, entity: RadarEntity) -> bool {
        if self.entities.iter().any(|e| e.uuid == entity.uuid && !entity.uuid.is_empty()) {
            return false;
        }
        self.entities.push(entity);
        true
    }

    /// `clearEntities()`.
    pub fn clear_entities(&mut self) {
        self.entities.clear();
    }

    pub fn entities(&self) -> &[RadarEntity] {
        &self.entities
    }

    pub fn size(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

/// `xaero/hud/minimap/radar/state/RadarState`: the radar lists the minimap
/// draws from.
#[derive(Debug, Clone, Default)]
pub struct RadarState {
    radar_lists: Vec<RadarList>,
    lists_reversed_order: bool,
}

impl RadarState {
    pub fn new() -> Self {
        RadarState::default()
    }

    /// `getRadarLists()`.
    pub fn get_radar_lists(&self) -> &[RadarList] {
        &self.radar_lists
    }

    /// `getUpdatableLists()` — all lists, used to refresh entity positions.
    pub fn get_updatable_lists(&self) -> &[RadarList] {
        &self.radar_lists
    }

    pub fn push(&mut self, list: RadarList) {
        self.radar_lists.push(list);
    }

    pub fn clear(&mut self) {
        self.radar_lists.clear();
    }

    /// `setListsReversedOrder(Z)`.
    pub fn set_lists_reversed_order(&mut self, reversed: bool) {
        self.lists_reversed_order = reversed;
    }

    /// `getListsReversedOrder()`.
    pub fn lists_reversed_order(&self) -> bool {
        self.lists_reversed_order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radar_list_adds_and_clears() {
        let mut list = RadarList::new();
        assert!(list.is_empty());
        assert!(list.add(RadarEntity::new("Zed", 1.0, 2.0, 3.0, "zombie")));
        assert_eq!(list.size(), 1);
        assert_eq!(list.entities()[0].name, "Zed");
        list.clear_entities();
        assert!(list.is_empty());
    }

    #[test]
    fn radar_state_holds_lists() {
        let mut state = RadarState::new();
        state.push(RadarList::new());
        assert_eq!(state.get_radar_lists().len(), 1);
        state.set_lists_reversed_order(true);
        assert!(state.lists_reversed_order());
    }
}
