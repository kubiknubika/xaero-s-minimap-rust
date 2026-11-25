//! Info-display configuration data, ported from
//! `xaero/hud/minimap/common/config/info/config` of the original mod.
//!
//! `InfoDisplayManagerConfigData` holds an ordered list of info-display ids
//! plus a map of per-display settings. The rendering of the info lines stays
//! in the game client; this module is the data store and its builder.

use std::collections::BTreeMap;

/// `xaero/hud/minimap/common/config/info/config/InfoDisplayConfigData`:
/// optional background/text colours and an optional state string.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InfoDisplayConfigData {
    pub background_color: Option<i32>,
    pub text_color: Option<i32>,
    pub state: Option<String>,
}

impl InfoDisplayConfigData {
    pub fn new(
        background_color: Option<i32>,
        text_color: Option<i32>,
        state: Option<String>,
    ) -> Self {
        InfoDisplayConfigData {
            background_color,
            text_color,
            state,
        }
    }
}

/// `xaero/hud/minimap/common/config/info/config/InfoDisplayManagerConfigData`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InfoDisplayManagerConfigData {
    order: Vec<String>,
    configs: BTreeMap<String, InfoDisplayConfigData>,
}

impl InfoDisplayManagerConfigData {
    /// The Java `EMPTY` singleton value.
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn new(order: Vec<String>, configs: BTreeMap<String, InfoDisplayConfigData>) -> Self {
        InfoDisplayManagerConfigData { order, configs }
    }

    /// `getFromOrder(int)`.
    pub fn get_from_order(&self, index: usize) -> Option<&str> {
        self.order.get(index).map(|s| s.as_str())
    }

    /// Java `getOrderStream()` — iteration in display order.
    pub fn order_iter(&self) -> impl Iterator<Item = &str> {
        self.order.iter().map(|s| s.as_str())
    }

    /// `copyOrder()`.
    pub fn copy_order(&self) -> Vec<String> {
        self.order.clone()
    }

    /// `get(String)`.
    pub fn get(&self, id: &str) -> Option<&InfoDisplayConfigData> {
        self.configs.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut InfoDisplayConfigData> {
        self.configs.get_mut(id)
    }

    /// `Builder.put(id, config)` — adds the id to the end of the order and
    /// replaces any previous config for it.
    pub fn put(&mut self, id: &str, config: InfoDisplayConfigData) {
        if !self.configs.contains_key(id) {
            self.order.push(id.to_string());
        }
        self.configs.insert(id.to_string(), config);
    }

    /// `Builder.remove(id)`.
    pub fn remove(&mut self, id: &str) -> Option<InfoDisplayConfigData> {
        let removed = self.configs.remove(id);
        if removed.is_some() {
            self.order.retain(|entry| entry != id);
        }
        removed
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty() && self.configs.is_empty()
    }
}

/// `InfoDisplayManagerConfigData.Builder`.
#[derive(Debug, Clone, Default)]
pub struct InfoDisplayManagerConfigDataBuilder {
    data: InfoDisplayManagerConfigData,
}

impl InfoDisplayManagerConfigDataBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// `begin()`-style builder from existing data.
    pub fn from_data(data: InfoDisplayManagerConfigData) -> Self {
        InfoDisplayManagerConfigDataBuilder { data }
    }

    pub fn put(&mut self, id: &str, config: InfoDisplayConfigData) -> &mut Self {
        self.data.put(id, config);
        self
    }

    pub fn remove(&mut self, id: &str) -> &mut Self {
        self.data.remove(id);
        self
    }

    pub fn build(self) -> InfoDisplayManagerConfigData {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manager_has_no_order_or_configs() {
        let data = InfoDisplayManagerConfigData::empty();
        assert!(data.is_empty());
        assert_eq!(data.get_from_order(0), None);
        assert_eq!(data.get("coords"), None);
    }

    #[test]
    fn builder_keeps_insertion_order_and_replaces_values() {
        let mut builder = InfoDisplayManagerConfigDataBuilder::new();
        builder.put(
            "coords",
            InfoDisplayConfigData::new(Some(0x2200_0000), Some(-1), Some("short".into())),
        );
        builder.put("biome", InfoDisplayConfigData::default());
        builder.put(
            "coords",
            InfoDisplayConfigData::new(None, None, Some("long".into())),
        );
        let data = builder.build();
        assert_eq!(data.copy_order(), vec!["coords", "biome"]);
        assert_eq!(data.order_iter().collect::<Vec<_>>(), vec!["coords", "biome"]);
        assert_eq!(data.get("coords").unwrap().state.as_deref(), Some("long"));
        assert_eq!(data.get("coords").unwrap().background_color, None);
    }

    #[test]
    fn remove_drops_both_order_and_config() {
        let mut data = InfoDisplayManagerConfigData::new(
            vec!["a".into(), "b".into()],
            BTreeMap::from([
                ("a".into(), InfoDisplayConfigData::default()),
                ("b".into(), InfoDisplayConfigData::default()),
            ]),
        );
        assert!(data.remove("a").is_some());
        assert_eq!(data.copy_order(), vec!["b"]);
        assert!(data.remove("missing").is_none());
    }
}
