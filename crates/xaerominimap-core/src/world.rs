//! Minimap worlds and world containers (mirrors
//! `xaero/hud/minimap/world(container|state)` of the original mod) plus the
//! dimension helper (`xaero/hud/minimap/world/MinimapDimensionHelper`).

use std::collections::BTreeMap;
use std::path::Path;

use crate::path::XaeroPath;
use crate::waypoints::{WaypointsSort, WaypointSet};

/// `net/minecraft/resources/ResourceKey` — the minimap works with dimension
/// keys; the Rust port uses plain strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DimensionKey {
    /// e.g. `minecraft:overworld`, `minecraft:the_nether`, `minecraft:the_end`.
    pub path: String,
}

impl DimensionKey {
    pub fn new(path: &str) -> Self {
        DimensionKey {
            path: path.to_string(),
        }
    }

    /// `OVERWORLD` in the original mod.
    pub fn overworld() -> Self {
        DimensionKey::new("minecraft:overworld")
    }

    pub fn the_nether() -> Self {
        DimensionKey::new("minecraft:the_nether")
    }

    pub fn the_end() -> Self {
        DimensionKey::new("minecraft:the_end")
    }

    /// The key's resource path without the namespace
    /// (`the_nether`, `overworld`, …).
    pub fn name(&self) -> &str {
        self.path.rsplit(':').next().unwrap_or(&self.path)
    }
}

impl Default for DimensionKey {
    fn default() -> Self {
        DimensionKey::overworld()
    }
}

impl std::fmt::Display for DimensionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path)
    }
}

/// `xaero/hud/minimap/world/container/config/RootConfig`: the config of a
/// world root container.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RootConfig {
    loaded: bool,
    using_multiworld_detection: bool,
    ignore_server_level_id: bool,
    default_multiworld_id: String,
    teleportation_enabled: bool,
    using_default_teleport_command: bool,
    server_teleport_command_format: String,
    server_teleport_command_rotation_format: String,
    sort_type: Option<crate::waypoints::WaypointsSort>,
    sort_reversed: bool,
    ignore_heightmaps: bool,
}

impl RootConfig {
    pub fn is_using_multiworld_detection(&self) -> bool {
        self.using_multiworld_detection
    }

    pub fn set_using_multiworld_detection(&mut self, value: bool) {
        self.using_multiworld_detection = value;
    }

    pub fn get_default_multiworld_id(&self) -> &str {
        &self.default_multiworld_id
    }

    pub fn set_default_multiworld_id(&mut self, id: &str) {
        self.default_multiworld_id = id.to_string();
    }

    pub fn is_teleportation_enabled(&self) -> bool {
        self.teleportation_enabled
    }

    pub fn set_teleportation_enabled(&mut self, value: bool) {
        self.teleportation_enabled = value;
    }

    pub fn is_using_default_teleport_command(&self) -> bool {
        self.using_default_teleport_command
    }

    pub fn set_using_default_teleport_command(&mut self, value: bool) {
        self.using_default_teleport_command = value;
    }

    pub fn get_server_teleport_command_format(&self) -> &str {
        &self.server_teleport_command_format
    }

    pub fn set_server_teleport_command_format(&mut self, format: &str) {
        self.server_teleport_command_format = format.to_string();
    }

    pub fn get_server_teleport_command_rotation_format(&self) -> &str {
        &self.server_teleport_command_rotation_format
    }

    pub fn set_server_teleport_command_rotation_format(&mut self, format: &str) {
        self.server_teleport_command_rotation_format = format.to_string();
    }

    /// `getSortType()` / `setSortType(...)`.
    pub fn get_sort_type(&self) -> Option<crate::waypoints::WaypointsSort> {
        self.sort_type
    }

    pub fn set_sort_type(&mut self, sort: crate::waypoints::WaypointsSort) {
        self.sort_type = Some(sort);
    }

    /// `toggleSortType()`.
    pub fn toggle_sort_type(&mut self) {
        self.sort_type = Some(match self.sort_type {
            Some(WaypointsSort::Name) => WaypointsSort::CreationTime,
            Some(WaypointsSort::CreationTime) => WaypointsSort::Name,
            None => WaypointsSort::Name,
        });
    }

    pub fn is_sort_reversed(&self) -> bool {
        self.sort_reversed
    }

    pub fn set_sort_reversed(&mut self, value: bool) {
        self.sort_reversed = value;
    }

    pub fn toggle_sort_reversed(&mut self) {
        self.sort_reversed = !self.sort_reversed;
    }

    pub fn is_ignore_server_level_id(&self) -> bool {
        self.ignore_server_level_id
    }

    pub fn set_ignore_server_level_id(&mut self, value: bool) {
        self.ignore_server_level_id = value;
    }

    pub fn is_ignore_heightmaps(&self) -> bool {
        self.ignore_heightmaps
    }

    pub fn set_ignore_heightmaps(&mut self, value: bool) {
        self.ignore_heightmaps = value;
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn set_loaded(&mut self, loaded: bool) {
        self.loaded = loaded;
    }
}

/// `xaero/hud/minimap/world/container/MinimapWorldRootContainer`: the root of
/// a container tree: it holds the [`RootConfig`], the per-dimension
/// coordinate scales (read by `MinimapDimensionHelper`) and the worlds of
/// the root.
#[derive(Debug, Clone, Default)]
pub struct MinimapWorldRootContainer {
    /// `configLxaero/hud/minimap/world/container/config/RootConfig;`.
    pub config: RootConfig,
    /// Dimension path -> coordinate scale (the Java `getDimensionScale`
    /// reads from the root config).
    pub dimension_scales: BTreeMap<String, f64>,
    /// The worlds of the root container.
    pub worlds: BTreeMap<String, MinimapWorld>,
}

impl MinimapWorldRootContainer {
    /// `getDimensionScale(ResourceKey)`.
    pub fn get_dimension_scale(&self, dim_id: &DimensionKey) -> f64 {
        self.dimension_scales
            .get(dim_id.path.as_str())
            .copied()
            .unwrap_or_else(|| MinimapDimensionHelper::get_dim_scale_default(dim_id))
    }

    /// `addWorld(MinimapWorld)` (Java containers hold worlds in sub trees;
    /// the port keeps them in a map here).
    pub fn add_world(&mut self, world: MinimapWorld) {
        self.worlds.insert(world.node.clone(), world);
    }

    /// `getWorld(String)`.
    pub fn get_world(&self, name: &str) -> Option<&MinimapWorld> {
        self.worlds.get(name)
    }
}

/// `xaero/hud/minimap/world/MinimapWorldManager`: the minimap's world
/// manager — one root container per path plus the current world.
#[derive(Debug, Clone, Default)]
pub struct MinimapWorldManager {
    root_containers: BTreeMap<XaeroPath, MinimapWorldRootContainer>,
    /// `MinimapClientWorldData.serverLevelId`, set by
    /// `MinimapWorldStateUpdater.onServerLevelId`.
    server_level_id: Option<i32>,
}

impl MinimapWorldManager {
    pub fn new() -> Self {
        MinimapWorldManager::default()
    }

    /// `MinimapWorldStateUpdater.onServerLevelId(int)` — stores the id sent
    /// by the server (the Java code also logs it).
    pub fn set_server_level_id(&mut self, id: i32) {
        self.server_level_id = Some(id);
    }

    pub fn get_server_level_id(&self) -> Option<i32> {
        self.server_level_id
    }

    /// `MinimapWorldStateUpdater.hasServerLevelId(root)` — the id is ignored
    /// when the root config opts out.
    pub fn has_server_level_id(&self, root: &MinimapWorldRootContainer) -> bool {
        self.server_level_id.is_some() && !root.config.is_ignore_server_level_id()
    }

    /// `getWorldContainer(XaeroPath)`.
    pub fn get_world_container(&self, path: &XaeroPath) -> Option<&MinimapWorldRootContainer> {
        self.root_containers.get(path)
    }

    /// `getWorldContainerNullable`.
    pub fn get_world_container_nullable(&self, path: &XaeroPath) -> Option<&MinimapWorldRootContainer> {
        self.get_world_container(path)
    }

    /// `getRootWorldContainer(XaeroPath)`.
    pub fn get_root_world_container(&self, path: &XaeroPath) -> Option<&MinimapWorldRootContainer> {
        self.get_world_container(path)
    }

    /// `addWorldContainer(XaeroPath)` — creates the root container at
    /// `path`.
    pub fn add_world_container(&mut self, path: XaeroPath) -> &mut MinimapWorldRootContainer {
        self.root_containers
            .entry(path)
            .or_insert_with(MinimapWorldRootContainer::default)
    }

    /// `addRootWorldContainer(MinimapWorldRootContainer)`.
    pub fn add_root_world_container(&mut self, path: XaeroPath, container: MinimapWorldRootContainer) {
        self.root_containers.insert(path, container);
    }

    /// `containerExists(XaeroPath)`.
    pub fn container_exists(&self, path: &XaeroPath) -> bool {
        self.root_containers.contains_key(path)
    }

    /// `removeContainer(XaeroPath)`.
    pub fn remove_container(&mut self, path: &XaeroPath) -> bool {
        self.root_containers.remove(path).is_some()
    }

    /// `getRootContainers()`.
    pub fn get_root_containers(&self) -> impl Iterator<Item = &MinimapWorldRootContainer> {
        self.root_containers.values()
    }

    /// `getWorld(XaeroPath)` — the world named by the path's last part,
    /// searched in the root containers.
    pub fn get_world(&self, path: &XaeroPath) -> Option<&MinimapWorld> {
        let name = path.file_name()?;
        self.root_containers.values().find_map(|c| c.get_world(name))
    }
}

/// `xaero/hud/minimap/world/container/MinimapWorldContainer`: a container
/// holds the minimap worlds of its dimension tree and its sub-containers.
#[derive(Debug, Clone, Default)]
pub struct MinimapWorldContainer {
    worlds: BTreeMap<String, MinimapWorld>,
    sub_containers: BTreeMap<XaeroPath, Box<MinimapWorldContainer>>,
    names: BTreeMap<String, String>,
}

impl MinimapWorldContainer {
    pub fn new() -> Self {
        MinimapWorldContainer::default()
    }

    /// `addWorld(MinimapWorld)`.
    pub fn add_world(&mut self, world: MinimapWorld) -> Option<MinimapWorld> {
        self.worlds.insert(world.node.clone(), world.clone())
    }

    /// `removeWorld(String)`.
    pub fn remove_world(&mut self, name: &str) -> Option<MinimapWorld> {
        self.worlds.remove(name)
    }

    /// All worlds in the container.
    pub fn get_worlds(&self) -> impl Iterator<Item = &MinimapWorld> {
        self.worlds.values()
    }

    /// First world in the container (`getFirstWorld`).
    pub fn get_first_world(&self) -> Option<&MinimapWorld> {
        self.worlds.values().next()
    }

    /// `getAllWorldsIterable` — worlds plus sub-container worlds.
    pub fn get_all_worlds(&self) -> Vec<&MinimapWorld> {
        let mut out: Vec<&MinimapWorld> = self.worlds.values().collect();
        for sub in self.sub_containers.values() {
            out.extend(sub.get_all_worlds());
        }
        out
    }

    /// `addSubContainer(XaeroPath)`.
    pub fn add_sub_container(&mut self, path: XaeroPath) -> &mut MinimapWorldContainer {
        self.sub_containers.entry(path).or_default()
    }

    /// `containsSubContainer(XaeroPath)`.
    pub fn contains_sub_container(&self, path: &XaeroPath) -> bool {
        self.sub_containers.contains_key(path)
    }

    /// `deleteSubContainer(XaeroPath)`.
    pub fn delete_sub_container(&mut self, path: &XaeroPath) -> bool {
        self.sub_containers.remove(path).is_some()
    }

    /// All sub-containers.
    pub fn get_sub_containers(&self) -> impl Iterator<Item = &MinimapWorldContainer> {
        self.sub_containers.values().map(|b| b.as_ref())
    }

    /// `setName(String, String)` — display name for a world node.
    pub fn set_name(&mut self, node: &str, name: &str) {
        self.names.insert(node.to_string(), name.to_string());
    }

    /// `getName(String)`.
    pub fn get_name(&self, node: &str) -> Option<&str> {
        self.names.get(node).map(|s| s.as_str())
    }

    /// `removeName(String)`.
    pub fn remove_name(&mut self, node: &str) {
        self.names.remove(node);
    }

    /// `getDirectoryPath` — `Path` of this container.
    pub fn get_directory_path(&self) -> &Path {
        Path::new(".")
    }
}

/// `xaero/hud/minimap/world/state/MinimapWorldState`: the paths a world
/// container lives at while the minimap is running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapWorldState {
    auto_root_container_path: Option<XaeroPath>,
    auto_world_path: XaeroPath,
    custom_world_path: XaeroPath,
    /// Java uses a fixed array of four slots.
    outdated_auto_root_container_paths: Vec<Option<XaeroPath>>,
    auto_container_path_ignore_case_cache: Option<XaeroPath>,
}

impl Default for MinimapWorldState {
    fn default() -> Self {
        MinimapWorldState {
            auto_root_container_path: None,
            auto_world_path: XaeroPath::new(),
            custom_world_path: XaeroPath::new(),
            outdated_auto_root_container_paths: vec![None; 4],
            auto_container_path_ignore_case_cache: None,
        }
    }
}

impl MinimapWorldState {
    pub fn new() -> Self {
        MinimapWorldState::default()
    }

    /// `setAutoRootContainerPath` — the Java version throws
    /// `IllegalStateException` when the path was already set; the Rust port
    /// returns an error instead.
    pub fn set_auto_root_container_path(&mut self, path: XaeroPath) -> Result<(), String> {
        if self.auto_root_container_path.is_some() {
            return Err("auto root container path is already set".into());
        }
        self.auto_root_container_path = Some(path);
        Ok(())
    }

    pub fn get_auto_root_container_path(&self) -> Option<&XaeroPath> {
        self.auto_root_container_path.as_ref()
    }

    /// `setOutdatedAutoRootContainerPath(int, XaeroPath)`.
    pub fn set_outdated_auto_root_container_path(
        &mut self,
        index: usize,
        path: XaeroPath,
    ) -> Result<(), String> {
        let slot = self
            .outdated_auto_root_container_paths
            .get_mut(index)
            .ok_or_else(|| format!("outdated path index {index} out of bounds"))?;
        if slot.is_some() {
            return Err("outdated auto root container path is already set".into());
        }
        *slot = Some(path);
        Ok(())
    }

    pub fn get_outdated_auto_root_container_path(&self, index: usize) -> Option<&XaeroPath> {
        self.outdated_auto_root_container_paths
            .get(index)
            .and_then(|slot| slot.as_ref())
    }

    pub fn set_auto_world_path(&mut self, path: XaeroPath) {
        self.auto_world_path = path;
    }

    pub fn get_auto_world_path(&self) -> &XaeroPath {
        &self.auto_world_path
    }

    pub fn set_custom_world_path(&mut self, path: XaeroPath) {
        self.custom_world_path = path;
    }

    pub fn get_custom_world_path(&self) -> &XaeroPath {
        &self.custom_world_path
    }

    /// `getCustomContainerPath()` — the parent of the custom world path.
    pub fn get_custom_container_path(&self) -> Option<XaeroPath> {
        if self.custom_world_path.is_empty() {
            None
        } else {
            self.custom_world_path.parent()
        }
    }

    pub fn get_auto_container_path_ignore_case_cache(&self) -> Option<&XaeroPath> {
        self.auto_container_path_ignore_case_cache.as_ref()
    }

    pub fn set_auto_container_path_ignore_case_cache(&mut self, path: XaeroPath) {
        self.auto_container_path_ignore_case_cache = Some(path);
    }

    /// `getCurrentWorldPath(XaeroPath)` — custom path wins over auto.
    pub fn current_world_path_for(&self, auto: &XaeroPath) -> XaeroPath {
        if self.custom_world_path.is_empty() {
            auto.clone()
        } else {
            self.custom_world_path.clone()
        }
    }

    /// Current world path: the custom one when set, else the auto one.
    pub fn get_current_world_path(&self) -> XaeroPath {
        self.current_world_path_for(&self.auto_world_path)
    }

    /// `getCurrentContainerPath()` — parent of the current world path.
    pub fn get_current_container_path(&self) -> Option<XaeroPath> {
        self.get_current_world_path().parent()
    }

    /// `getCurrentRootContainerPath()` — root of the current container path.
    pub fn get_current_root_container_path(&self) -> Option<XaeroPath> {
        self.get_current_container_path().and_then(|path| path.root())
    }
}

/// `xaero/hud/minimap/world/MinimapWorld`: a minimap world of a dimension,
/// with its waypoint sets.
#[derive(Debug, Clone)]
pub struct MinimapWorld {
    node: String,
    dim_id: DimensionKey,
    waypoint_sets: BTreeMap<String, WaypointSet>,
    current_waypoint_set_id: String,
    slime_chunk_seed: Option<i64>,
    root_container: MinimapWorldRootContainer,
}

impl Default for MinimapWorld {
    fn default() -> Self {
        MinimapWorld::new("", DimensionKey::overworld())
    }
}

impl MinimapWorld {
    pub fn new(node: &str, dim_id: DimensionKey) -> Self {
        MinimapWorld {
            node: node.to_string(),
            dim_id,
            waypoint_sets: BTreeMap::new(),
            current_waypoint_set_id: String::new(),
            slime_chunk_seed: None,
            root_container: MinimapWorldRootContainer::default(),
        }
    }

    /// `getContainer().getRoot()`: the root container of the world.
    pub fn get_container_root(&self) -> &MinimapWorldRootContainer {
        &self.root_container
    }

    /// Mutable access to the world's root container.
    pub fn get_container_root_mut(&mut self) -> &mut MinimapWorldRootContainer {
        &mut self.root_container
    }

    /// `DEFAULT_SET` in the original mod.
    pub const DEFAULT_SET: &'static str = "Default";

    /// `addWaypointSet(String)` — creates (or returns) the set.
    pub fn add_waypoint_set(&mut self, name: &str) {
        self.waypoint_sets
            .entry(name.to_string())
            .or_insert_with(|| WaypointSet::new(name));
        if self.current_waypoint_set_id.is_empty() {
            self.current_waypoint_set_id = name.to_string();
        }
    }

    /// `addWaypointSet(WaypointSet)`.
    pub fn add_waypoint_set_obj(&mut self, set: WaypointSet) -> WaypointSet {
        let name = set.get_name().to_string();
        self.waypoint_sets.insert(name.clone(), set);
        if self.current_waypoint_set_id.is_empty() {
            self.current_waypoint_set_id = name.clone();
        }
        self.waypoint_sets.get(&name).cloned().unwrap()
    }

    /// `getWaypointSet(String)`.
    pub fn get_waypoint_set(&self, name: &str) -> Option<&WaypointSet> {
        self.waypoint_sets.get(name)
    }

    /// Mutable variant (the IO and renderers need it).
    pub fn get_waypoint_set_mut(&mut self, name: &str) -> Option<&mut WaypointSet> {
        self.waypoint_sets.get_mut(name)
    }

    /// `removeWaypointSet(String)`.
    pub fn remove_waypoint_set(&mut self, name: &str) -> Option<WaypointSet> {
        let removed = self.waypoint_sets.remove(name);
        if self.current_waypoint_set_id == name {
            self.current_waypoint_set_id = String::new();
        }
        removed
    }

    /// `getIterableWaypointSets`.
    pub fn iter_waypoint_sets(&self) -> impl Iterator<Item = &WaypointSet> {
        self.waypoint_sets.values()
    }

    /// Mutable variant used by the waypoint handlers.
    pub fn iter_waypoint_sets_mut(&mut self) -> impl Iterator<Item = &mut WaypointSet> {
        self.waypoint_sets.values_mut()
    }

    /// `setCurrentWaypointSetId(String)`.
    pub fn set_current_waypoint_set_id(&mut self, id: &str) {
        self.current_waypoint_set_id = id.to_string();
    }

    /// `getCurrentWaypointSetId`.
    pub fn get_current_waypoint_set_id(&self) -> &str {
        &self.current_waypoint_set_id
    }

    /// `getCurrentWaypointSet`.
    pub fn get_current_waypoint_set(&self) -> Option<&WaypointSet> {
        if self.current_waypoint_set_id.is_empty() {
            self.waypoint_sets.values().next()
        } else {
            self.waypoint_sets.get(&self.current_waypoint_set_id)
        }
    }

    /// Mutable variant of [`Self::get_current_waypoint_set`], including the
    /// first-set fallback of the Java implementation.
    pub fn get_current_waypoint_set_mut(&mut self) -> Option<&mut WaypointSet> {
        if self.current_waypoint_set_id.is_empty() {
            self.waypoint_sets.values_mut().next()
        } else {
            let id = self.current_waypoint_set_id.clone();
            self.waypoint_sets.get_mut(&id)
        }
    }

    /// `getSetCount`.
    pub fn get_set_count(&self) -> usize {
        self.waypoint_sets.len()
    }

    /// `getNode`.
    pub fn get_node(&self) -> &str {
        &self.node
    }

    /// `setNode`.
    pub fn set_node(&mut self, node: &str) {
        self.node = node.to_string();
    }

    /// `getDimId`.
    pub fn get_dim_id(&self) -> &DimensionKey {
        &self.dim_id
    }

    /// `getSlimeChunkSeed` / `setSlimeChunkSeed`.
    pub fn get_slime_chunk_seed(&self) -> Option<i64> {
        self.slime_chunk_seed
    }

    pub fn set_slime_chunk_seed(&mut self, seed: i64) {
        self.slime_chunk_seed = Some(seed);
    }
}

/// `xaero/hud/minimap/world/MinimapDimensionHelper`: converts dimension keys
/// to the directory names the minimap uses on disk and reads coordinate
/// scales from a world's root container.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinimapDimensionHelper;

impl MinimapDimensionHelper {
    /// `getInstance()` (the Java helper is a singleton).
    pub fn get_instance() -> MinimapDimensionHelper {
        MinimapDimensionHelper
    }

    /// `getDimensionDirectoryName(ResourceKey)`: the dimension path with the
    /// namespace stripped (`overworld`, `the_nether`, `the_end`, …).
    pub fn dimension_directory_name(dim_id: &DimensionKey) -> String {
        let name = dim_id.name();
        name.rsplit_once('/').map(|(_, n)| n.to_string()).unwrap_or_else(|| name.to_string())
    }

    /// `getDimCoordinateScale(MinimapWorld)`: the world's dimension
    /// coordinate scale from the root container, or the default.
    pub fn get_dim_coordinate_scale(world: &MinimapWorld) -> f64 {
        world.get_container_root().get_dimension_scale(world.get_dim_id())
    }

    fn get_dim_scale_default(dim_id: &DimensionKey) -> f64 {
        // Mirrors DimensionType#getCoordinateScale defaults.
        match dim_id.name() {
            "the_nether" => 1.0,
            _ => 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_directory_names() {
        assert_eq!(
            MinimapDimensionHelper::dimension_directory_name(&DimensionKey::overworld()),
            "overworld"
        );
        assert_eq!(
            MinimapDimensionHelper::dimension_directory_name(&DimensionKey::the_nether()),
            "the_nether"
        );
    }

    #[test]
    fn world_waypoint_sets() {
        let mut world = MinimapWorld::new("test", DimensionKey::overworld());
        world.add_waypoint_set("Set1");
        world.add_waypoint_set("Set2");
        assert_eq!(world.get_set_count(), 2);
        assert!(world.get_waypoint_set("Set1").is_some());
        assert!(world.get_current_waypoint_set().is_some());
        assert!(world.get_current_waypoint_set_id().len() > 0);
        world.remove_waypoint_set("Set1");
        assert!(world.get_waypoint_set("Set1").is_none());
    }

    #[test]
    fn container_add_and_get_worlds() {
        let mut container = MinimapWorldContainer::new();
        container.add_world(MinimapWorld::new("w1", DimensionKey::overworld()));
        assert_eq!(container.get_worlds().count(), 1);
        container.set_name("w1", "Display name");
        assert_eq!(container.get_name("w1"), Some("Display name"));
        container.remove_name("w1");
        assert_eq!(container.get_name("w1"), None);
    }

    #[test]
    fn state_current_world_path() {
        let mut state = MinimapWorldState::new();
        state.set_auto_world_path(XaeroPath::of(&["worlds", "test"]));
        assert_eq!(
            state.get_current_world_path(),
            XaeroPath::of(&["worlds", "test"])
        );
        state.set_custom_world_path(XaeroPath::of(&["custom"]));
        assert_eq!(state.get_current_world_path(), XaeroPath::of(&["custom"]));
        assert_eq!(
            state.get_current_container_path(),
            Some(XaeroPath::new())
        );
    }

    #[test]
    fn state_paths_and_java_style_guards() {
        let mut state = MinimapWorldState::new();
        let root = XaeroPath::of(&["worlds", "root"]);
        assert!(state.set_auto_root_container_path(root.clone()).is_ok());
        assert!(state.set_auto_root_container_path(root.clone()).is_err());
        assert_eq!(state.get_auto_root_container_path(), Some(&root));

        let outdated = XaeroPath::of(&["old", "one"]);
        assert!(state.set_outdated_auto_root_container_path(0, outdated.clone()).is_ok());
        assert!(state.set_outdated_auto_root_container_path(0, outdated.clone()).is_err());
        assert!(state.set_outdated_auto_root_container_path(4, outdated).is_err());
        assert_eq!(
            state.get_outdated_auto_root_container_path(0),
            Some(&XaeroPath::of(&["old", "one"]))
        );
        assert_eq!(state.get_outdated_auto_root_container_path(1), None);

        state.set_auto_world_path(XaeroPath::of(&["worlds", "root", "dim", "world"]));
        assert_eq!(
            state.get_current_root_container_path(),
            Some(XaeroPath::of(&["worlds"]))
        );
        state.set_custom_world_path(XaeroPath::of(&["custom", "world"]));
        assert_eq!(
            state.get_custom_container_path(),
            Some(XaeroPath::of(&["custom"]))
        );

        state.set_auto_container_path_ignore_case_cache(XaeroPath::of(&["worlds", "root"]));
        assert_eq!(
            state.get_auto_container_path_ignore_case_cache(),
            Some(&XaeroPath::of(&["worlds", "root"]))
        );
    }

    #[test]
    fn path_parent_root_and_subpath() {
        let path = XaeroPath::of(&["worlds", "root", "dim"]);
        assert_eq!(path.parent(), Some(XaeroPath::of(&["worlds", "root"])));
        assert_eq!(path.root(), Some(XaeroPath::of(&["worlds"])));
        assert_eq!(path.sub_path(2), XaeroPath::of(&["worlds", "root"]));
        assert_eq!(path.node_count(), 3);
        assert!(path.is_sub_of(&XaeroPath::of(&["worlds"])));
        assert!(path.is_sub_of(&XaeroPath::of(&["worlds", "root"])));
    }

    #[test]
    fn manager_server_level_id_and_ignore_flag() {
        let mut manager = MinimapWorldManager::new();
        let root = MinimapWorldRootContainer::default();
        assert!(!manager.has_server_level_id(&root));
        manager.set_server_level_id(7);
        assert_eq!(manager.get_server_level_id(), Some(7));
        assert!(manager.has_server_level_id(&root));

        let mut ignoring = MinimapWorldRootContainer::default();
        ignoring.config.set_ignore_server_level_id(true);
        assert!(!manager.has_server_level_id(&ignoring));
    }

    #[test]
    fn dim_coordinate_scale_default() {
        let world = MinimapWorld::new("w", DimensionKey::overworld());
        assert_eq!(MinimapDimensionHelper::get_dim_coordinate_scale(&world), 0.5);
        let mut root = MinimapWorldRootContainer::default();
        root.dimension_scales
            .insert("minecraft:the_nether".to_string(), 1.0);
        assert_eq!(root.get_dimension_scale(&DimensionKey::the_nether()), 1.0);
        assert_eq!(root.get_dimension_scale(&DimensionKey::overworld()), 0.5);
    }

    #[test]
    fn root_config_flags() {
        let mut config = RootConfig::default();
        config.set_using_multiworld_detection(true);
        config.set_teleportation_enabled(true);
        config.set_ignore_heightmaps(true);
        assert!(config.is_using_multiworld_detection());
        assert!(config.is_teleportation_enabled());
        assert!(config.is_ignore_heightmaps());
        config.set_sort_type(WaypointsSort::Name);
        assert_eq!(config.get_sort_type(), Some(WaypointsSort::Name));
        config.toggle_sort_type();
        assert_eq!(config.get_sort_type(), Some(WaypointsSort::CreationTime));
        config.set_sort_reversed(true);
        assert!(config.is_sort_reversed());
        config.toggle_sort_reversed();
        assert!(!config.is_sort_reversed());
    }

    #[test]
    fn world_manager_containers() {
        let mut manager = MinimapWorldManager::new();
        let path = XaeroPath::of(&["root", "container"]);
        assert!(!manager.container_exists(&path));
        manager.add_root_world_container(path.clone(), MinimapWorldRootContainer::default());
        assert!(manager.container_exists(&path));
        manager
            .add_world_container(path.clone())
            .add_world(MinimapWorld::new("w", DimensionKey::overworld()));
        let world_path = XaeroPath::of(&["root", "container", "w"]);
        assert_eq!(
            manager.get_world(&world_path).map(|w| w.get_node()),
            Some("w")
        );
        assert_eq!(manager.get_root_containers().count(), 1);
        assert!(manager.remove_container(&path));
        assert!(!manager.container_exists(&path));
    }
}
