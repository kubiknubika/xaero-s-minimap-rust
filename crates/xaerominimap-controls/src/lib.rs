//! Key mappings and controller ticking, ported from
//! `xaero/hud/controls` and `xaero/hud/minimap/controls/key` of the original
//! mod.
//!
//! The Java mod registers `KeyMapping`s, wraps them in
//! `KeyMappingController`s, attaches `KeyMappingFunction`s and polls them
//! every client tick. The GPU/input layer is game-specific, so the Rust port
//! takes key state through [`KeyStateSource`] and returns [`ControlEvent`]s
//! for the game glue to apply:
//!
//! | Java class | Rust type |
//! | --- | --- |
//! | `xaero/hud/controls/key/KeyMappingController` | [`KeyMappingController`] |
//! | `xaero/hud/controls/key/KeyMappingControllerManager` | [`KeyMappingControllerManager`] |
//! | `xaero/hud/controls/key/KeyMappingTickHandler` | [`KeyMappingTickHandler`] |
//! | `xaero/hud/controls/key/function/KeyMappingFunction` | [`KeyFunction`] |
//! | `xaero/hud/minimap/controls/key/MinimapKeyMappings` | [`MinimapKeyMappings`] |

use std::collections::BTreeMap;

/// Category id registered by `MinimapKeyMappings.CATEGORY`.
pub const CATEGORY: &str = "xaerominimap:controls";

/// The actions the minimap's key mappings trigger (`MinimapKeyMappingFunctions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MinimapAction {
    ZoomIn,
    ZoomOut,
    AddWaypoint,
    WaypointMenu,
    HeldEnlargeMap,
    ToggledEnlargeMap,
    ToggleMap,
    ToggleWaypointsWorld,
    ToggleWaypointsMap,
    ToggleSlimeChunks,
    ToggleGrid,
    TemporaryWaypoint,
    SwitchWaypointSet,
    RenderAllSets,
    LightOverlay,
    ToggleRadar,
    ReverseEntityRadar,
    ManualCaveMode,
    AlternativeListPlayers,
    ToggleTrackedPlayersMap,
    ToggleTrackedPlayersWorld,
    ToggleOpacClaims,
}

impl MinimapAction {
    pub const ALL: [MinimapAction; 22] = [
        MinimapAction::ZoomIn,
        MinimapAction::ZoomOut,
        MinimapAction::AddWaypoint,
        MinimapAction::WaypointMenu,
        MinimapAction::HeldEnlargeMap,
        MinimapAction::ToggledEnlargeMap,
        MinimapAction::ToggleMap,
        MinimapAction::ToggleWaypointsWorld,
        MinimapAction::ToggleWaypointsMap,
        MinimapAction::ToggleSlimeChunks,
        MinimapAction::ToggleGrid,
        MinimapAction::TemporaryWaypoint,
        MinimapAction::SwitchWaypointSet,
        MinimapAction::RenderAllSets,
        MinimapAction::LightOverlay,
        MinimapAction::ToggleRadar,
        MinimapAction::ReverseEntityRadar,
        MinimapAction::ManualCaveMode,
        MinimapAction::AlternativeListPlayers,
        MinimapAction::ToggleTrackedPlayersMap,
        MinimapAction::ToggleTrackedPlayersWorld,
        MinimapAction::ToggleOpacClaims,
    ];
}

/// A registered key (the Java `net.minecraft.client.KeyMapping`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMapping {
    /// Stable id used by the controller manager (the Java object identity).
    pub id: &'static str,
    /// Translation key shown in the controls screen.
    pub translation_key: &'static str,
    /// `KeyMapping.Category` the mapping belongs to.
    pub category: &'static str,
    /// Default key code (`InputConstants.Key.getValue()` in Java). `-1` is
    /// `UNKNOWN`, i.e. unbound until the player binds it.
    pub default_code: i32,
    /// `xaeroKey` in `KeyMappingController`: mod-owned keys use
    /// `consumeClick()` edges instead of the vanilla `isDown()` state.
    pub xaero_key: bool,
}

impl KeyMapping {
    pub fn new(
        id: &'static str,
        translation_key: &'static str,
        default_code: i32,
        xaero_key: bool,
    ) -> Self {
        KeyMapping {
            id,
            translation_key,
            category: CATEGORY,
            default_code,
            xaero_key,
        }
    }

    /// Assigns a different controls category (used by add-on mappings).
    pub fn with_category(mut self, category: &'static str) -> Self {
        self.category = category;
        self
    }
}

/// `xaero/hud/controls/key/function/KeyMappingFunction`: one action bound to
/// a key. `held` functions only fire on the initial press; the others fire on
/// every repeated tick while the key is down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyFunction {
    pub action: MinimapAction,
    pub held: bool,
}

impl KeyFunction {
    pub fn new(action: MinimapAction) -> Self {
        KeyFunction {
            action,
            held: false,
        }
    }

    pub fn held(action: MinimapAction) -> Self {
        KeyFunction { action, held: true }
    }
}

/// `xaero/hud/controls/key/KeyMappingController`.
#[derive(Debug, Clone)]
pub struct KeyMappingController {
    mapping: KeyMapping,
    functions: Vec<KeyFunction>,
    pressed: bool,
}

impl KeyMappingController {
    pub fn new(mapping: KeyMapping) -> Self {
        KeyMappingController {
            mapping,
            functions: Vec::new(),
            pressed: false,
        }
    }

    pub fn get_key_mapping(&self) -> &KeyMapping {
        &self.mapping
    }

    pub fn get_functions(&self) -> &[KeyFunction] {
        &self.functions
    }

    pub fn is_xaero_key(&self) -> bool {
        self.mapping.xaero_key
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    pub fn add(&mut self, function: KeyFunction) {
        self.functions.push(function);
    }
}

/// `xaero/hud/controls/key/KeyMappingControllerManager`.
#[derive(Debug, Clone, Default)]
pub struct KeyMappingControllerManager {
    controllers: BTreeMap<String, KeyMappingController>,
}

impl KeyMappingControllerManager {
    pub fn new() -> Self {
        KeyMappingControllerManager::default()
    }

    /// `registerController(KeyMapping, boolean)`.
    pub fn register_controller(
        &mut self,
        mapping: KeyMapping,
    ) -> Result<&mut KeyMappingController, String> {
        let id = mapping.id.to_string();
        if self.controllers.contains_key(&id) {
            return Err("The key mapping is already registered!".into());
        }
        self.controllers
            .insert(id.clone(), KeyMappingController::new(mapping));
        Ok(self.controllers.get_mut(&id).unwrap())
    }

    /// `registerFunction(KeyMapping, KeyMappingFunction)`.
    pub fn register_function(
        &mut self,
        mapping_id: &str,
        function: KeyFunction,
    ) -> Result<(), String> {
        let controller = self.controllers.get_mut(mapping_id).ok_or_else(|| {
            "The key mapping needs to be registered with registerController first!".to_string()
        })?;
        controller.add(function);
        Ok(())
    }

    pub fn get_controller(&self, mapping_id: &str) -> Option<&KeyMappingController> {
        self.controllers.get(mapping_id)
    }

    pub fn controllers(&self) -> &BTreeMap<String, KeyMappingController> {
        &self.controllers
    }

    pub fn ids(&self) -> Vec<String> {
        self.controllers.keys().cloned().collect()
    }
}

/// Key state supplied by the game/input layer.
pub trait KeyStateSource {
    /// Whether the key is physically held down.
    fn is_down(&self, code: i32) -> bool;

    /// Consumes one click event (`KeyMapping.consumeClick()`).
    fn consume_click(&mut self, code: i32) -> bool;
}

/// What happened to a key-bound action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlEventKind {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ControlEvent {
    pub action: MinimapAction,
    pub kind: ControlEventKind,
}

/// `xaero/hud/controls/key/KeyMappingTickHandler`.
#[derive(Debug, Clone, Default)]
pub struct KeyMappingTickHandler {
    events: Vec<ControlEvent>,
}

impl KeyMappingTickHandler {
    pub fn new() -> Self {
        KeyMappingTickHandler::default()
    }

    /// Polls all controllers in the same three passes as the Java handler:
    /// extra presses, releases, then presses.
    pub fn tick(
        &mut self,
        manager: &mut KeyMappingControllerManager,
        state: &mut impl KeyStateSource,
    ) -> Vec<ControlEvent> {
        self.events.clear();
        for id in manager.ids() {
            if let Some(controller) = manager.controllers.get_mut(&id) {
                self.handle_extra_presses(controller, state);
            }
        }
        for id in manager.ids() {
            if let Some(controller) = manager.controllers.get_mut(&id) {
                self.handle_release(controller, state);
            }
        }
        for id in manager.ids() {
            if let Some(controller) = manager.controllers.get_mut(&id) {
                self.handle_press(controller, state);
            }
        }
        std::mem::take(&mut self.events)
    }

    fn handle_press(
        &mut self,
        controller: &mut KeyMappingController,
        state: &mut impl KeyStateSource,
    ) {
        let starting_press = if controller.is_pressed() {
            false
        } else {
            if !self.get_updated_pressed_value(controller, state) {
                return;
            }
            true
        };
        controller.set_pressed(true);
        self.handle_press_functions(controller, starting_press);
    }

    fn handle_extra_presses(
        &mut self,
        controller: &KeyMappingController,
        state: &mut impl KeyStateSource,
    ) {
        if !controller.is_pressed() {
            return;
        }
        // Only mod-owned keys have click events; vanilla key mappings are
        // handled entirely by `handlePress`/`handleRelease`. Every extra
        // click toggles non-held functions off (the Java loop repeats while
        // the click queue is non-empty).
        if !controller.is_xaero_key() {
            return;
        }
        let code = controller.get_key_mapping().default_code;
        while state.consume_click(code) {
            for function in controller.get_functions() {
                if !function.held {
                    self.events.push(ControlEvent {
                        action: function.action,
                        kind: ControlEventKind::Release,
                    });
                }
            }
        }
    }

    fn handle_release(
        &mut self,
        controller: &mut KeyMappingController,
        state: &impl KeyStateSource,
    ) {
        if !controller.is_pressed() || state.is_down(controller.get_key_mapping().default_code) {
            return;
        }
        self.handle_release_functions(controller);
        controller.set_pressed(false);
    }

    fn handle_press_functions(&mut self, controller: &KeyMappingController, starting_press: bool) {
        for function in controller.get_functions() {
            if starting_press || !function.held {
                self.events.push(ControlEvent {
                    action: function.action,
                    kind: ControlEventKind::Press,
                });
            }
        }
    }

    fn handle_release_functions(&mut self, controller: &KeyMappingController) {
        for function in controller.get_functions() {
            self.events.push(ControlEvent {
                action: function.action,
                kind: ControlEventKind::Release,
            });
        }
    }

    fn get_updated_pressed_value(
        &mut self,
        controller: &mut KeyMappingController,
        state: &mut impl KeyStateSource,
    ) -> bool {
        let code = controller.get_key_mapping().default_code;
        if !controller.is_xaero_key() {
            return state.is_down(code);
        }
        if !state.consume_click(code) {
            return false;
        }
        // Click queues may contain several events for one tick; flush them
        // and fire the press/release pair for each extra click.
        while state.consume_click(code) {
            self.handle_press_functions(controller, true);
            self.handle_release_functions(controller);
        }
        true
    }
}

/// `xaero/hud/minimap/controls/key/MinimapKeyMappings`: the built-in key
/// mappings and their default key codes.
pub struct MinimapKeyMappings;

impl MinimapKeyMappings {
    /// All mappings in the order of the Java class (21 entries).
    pub fn all() -> Vec<KeyMapping> {
        let unmapped = || -1;
        vec![
            KeyMapping::new("zoom_in", "gui.xaero_zoom_in", unmapped(), true),
            KeyMapping::new("zoom_out", "gui.xaero_zoom_out", unmapped(), true),
            KeyMapping::new("add_waypoint", "gui.xaero_new_waypoint", 5, true),
            KeyMapping::new("waypoint_menu", "gui.xaero_waypoints_key", 24, true),
            KeyMapping::new("enlarge_map", "gui.xaero_enlarge_map", 29, true),
            KeyMapping::new("toggle_map", "gui.xaero_toggle_map", unmapped(), true),
            KeyMapping::new(
                "toggle_world_waypoints",
                "gui.xaero_toggle_waypoints",
                unmapped(),
                true,
            ),
            KeyMapping::new(
                "toggle_map_waypoints",
                "gui.xaero_toggle_map_waypoints",
                unmapped(),
                true,
            ),
            KeyMapping::new("toggle_slime_chunks", "gui.xaero_toggle_slime", unmapped(), true),
            KeyMapping::new("toggle_grid", "gui.xaero_toggle_grid", unmapped(), true),
            KeyMapping::new("temporary_waypoint", "gui.xaero_instant_waypoint", 87, true),
            KeyMapping::new(
                "switch_waypoint_set",
                "gui.xaero_switch_waypoint_set",
                unmapped(),
                true,
            ),
            KeyMapping::new(
                "render_all_sets",
                "gui.xaero_display_all_sets",
                unmapped(),
                true,
            ),
            KeyMapping::new(
                "light_overlay",
                "gui.xaero_toggle_light_overlay",
                unmapped(),
                true,
            ),
            KeyMapping::new("toggle_radar", "gui.xaero_toggle_entity_radar", unmapped(), true),
            KeyMapping::new(
                "reverse_entity_radar",
                "gui.xaero_reverse_entity_radar",
                unmapped(),
                true,
            ),
            KeyMapping::new(
                "manual_cave_mode",
                "gui.xaero_toggle_manual_cave_mode",
                unmapped(),
                true,
            ),
            KeyMapping::new(
                "alternative_list_players",
                "gui.xaero_alternative_list_players",
                unmapped(),
                true,
            ),
            KeyMapping::new(
                "toggle_tracked_players_map",
                "gui.xaero_toggle_tracked_players_on_map",
                unmapped(),
                true,
            ),
            KeyMapping::new(
                "toggle_tracked_players_world",
                "gui.xaero_toggle_tracked_players_in_world",
                unmapped(),
                true,
            ),
            KeyMapping::new(
                "toggle_opac_claims",
                "gui.xaero_toggle_pac_chunk_claims",
                unmapped(),
                true,
            ),
        ]
    }

    /// `registerAll(manager, registry, categoryRegistry)` — the manager side
    /// of registration (the Minecraft registry/category callbacks are
    /// game-specific and therefore omitted).
    pub fn register_all(manager: &mut KeyMappingControllerManager) -> Result<(), String> {
        for mapping in Self::all() {
            manager.register_controller(mapping)?;
        }
        Self::register_functions(manager)
    }

    /// `xaero/hud/minimap/controls/key/function/MinimapKeyMappingFunctions.registerAll`.
    /// The Java method registers 20 functions: `ENLARGE_MAP` has both a held
    /// and a toggled function, while `REVERSE_ENTITY_RADAR` and
    /// `ALTERNATIVE_LIST_PLAYERS` have mappings but no registered function in
    /// `MinimapKeyMappingFunctions`.
    pub fn register_functions(manager: &mut KeyMappingControllerManager) -> Result<(), String> {
        let bindings: [(&str, MinimapAction, bool); 20] = [
            ("zoom_in", MinimapAction::ZoomIn, false),
            ("zoom_out", MinimapAction::ZoomOut, false),
            ("toggle_radar", MinimapAction::ToggleRadar, false),
            (
                "toggle_tracked_players_map",
                MinimapAction::ToggleTrackedPlayersMap,
                false,
            ),
            (
                "toggle_tracked_players_world",
                MinimapAction::ToggleTrackedPlayersWorld,
                false,
            ),
            ("add_waypoint", MinimapAction::AddWaypoint, false),
            ("waypoint_menu", MinimapAction::WaypointMenu, false),
            ("enlarge_map", MinimapAction::HeldEnlargeMap, true),
            ("enlarge_map", MinimapAction::ToggledEnlargeMap, false),
            ("toggle_map", MinimapAction::ToggleMap, false),
            (
                "toggle_map_waypoints",
                MinimapAction::ToggleWaypointsMap,
                false,
            ),
            (
                "toggle_world_waypoints",
                MinimapAction::ToggleWaypointsWorld,
                false,
            ),
            ("toggle_slime_chunks", MinimapAction::ToggleSlimeChunks, false),
            ("toggle_grid", MinimapAction::ToggleGrid, false),
            ("temporary_waypoint", MinimapAction::TemporaryWaypoint, false),
            ("switch_waypoint_set", MinimapAction::SwitchWaypointSet, false),
            ("render_all_sets", MinimapAction::RenderAllSets, false),
            ("light_overlay", MinimapAction::LightOverlay, false),
            ("manual_cave_mode", MinimapAction::ManualCaveMode, false),
            ("toggle_opac_claims", MinimapAction::ToggleOpacClaims, false),
        ];
        for (id, action, held) in bindings {
            let function = if held {
                KeyFunction::held(action)
            } else {
                KeyFunction::new(action)
            };
            manager.register_function(id, function)?;
        }
        Ok(())
    }
}

/// `xaero/hud/controls/ControlsRegister`: owns the controllers of the
/// minimap's key mappings.
#[derive(Debug, Clone)]
pub struct ControlsRegister {
    controllers: KeyMappingControllerManager,
}

impl Default for ControlsRegister {
    fn default() -> Self {
        let mut controllers = KeyMappingControllerManager::new();
        // Registration is a static table in the port; a failure here would
        // mean a duplicate id in `MinimapKeyMappings`.
        MinimapKeyMappings::register_all(&mut controllers)
            .expect("built-in minimap key mappings must register uniquely");
        ControlsRegister { controllers }
    }
}

impl ControlsRegister {
    pub fn new() -> Self {
        ControlsRegister::default()
    }

    pub fn key_mapping_controllers(&self) -> &KeyMappingControllerManager {
        &self.controllers
    }

    pub fn key_mapping_controllers_mut(&mut self) -> &mut KeyMappingControllerManager {
        &mut self.controllers
    }

    /// Runs one tick of the key handler and returns the actions to apply.
    pub fn tick(&mut self, state: &mut impl KeyStateSource) -> Vec<ControlEvent> {
        KeyMappingTickHandler::new().tick(&mut self.controllers, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct TestKeyState {
        down: BTreeMap<i32, bool>,
        clicks: BTreeMap<i32, usize>,
    }

    impl TestKeyState {
        fn press(&mut self, code: i32) {
            self.down.insert(code, true);
            *self.clicks.entry(code).or_insert(0) += 1;
        }

        fn release(&mut self, code: i32) {
            self.down.insert(code, false);
        }
    }

    impl KeyStateSource for TestKeyState {
        fn is_down(&self, code: i32) -> bool {
            self.down.get(&code).copied().unwrap_or(false)
        }

        fn consume_click(&mut self, code: i32) -> bool {
            let entry = self.clicks.entry(code).or_insert(0);
            if *entry > 0 {
                *entry -= 1;
                true
            } else {
                false
            }
        }
    }

    #[test]
    fn registers_all_mappings_and_functions() {
        let controls = ControlsRegister::new();
        let manager = controls.key_mapping_controllers();
        assert_eq!(manager.controllers().len(), 21);
        assert_eq!(manager.get_controller("add_waypoint").unwrap().get_key_mapping().default_code, 5);
        assert_eq!(manager.get_controller("enlarge_map").unwrap().get_key_mapping().default_code, 29);
        assert_eq!(manager.get_controller("zoom_in").unwrap().get_key_mapping().default_code, -1);
        assert_eq!(
            manager.get_controller("enlarge_map").unwrap().get_functions(),
            &[
                KeyFunction::held(MinimapAction::HeldEnlargeMap),
                KeyFunction::new(MinimapAction::ToggledEnlargeMap),
            ]
        );

        let mut empty = KeyMappingControllerManager::new();
        let mapping = KeyMapping::new("test", "key.test", -1, true);
        empty.register_controller(mapping.clone()).unwrap();
        assert!(empty.register_controller(mapping).is_err());
        assert!(empty.register_function("missing", KeyFunction::new(MinimapAction::ZoomIn)).is_err());
    }

    #[test]
    fn xaero_key_press_and_release() {
        let mut controls = ControlsRegister::new();
        let mut state = TestKeyState::default();
        let code = 5; // add_waypoint

        // No click yet: nothing happens.
        assert!(controls.tick(&mut state).is_empty());

        // A click starts the press; non-held functions fire on press.
        state.press(code);
        let events = controls.tick(&mut state);
        assert_eq!(
            events,
            vec![ControlEvent {
                action: MinimapAction::AddWaypoint,
                kind: ControlEventKind::Press
            }]
        );
        assert!(controls
            .key_mapping_controllers()
            .get_controller("add_waypoint")
            .unwrap()
            .is_pressed());

        // Still held down: an extra click toggles the non-held function off,
        // then `handlePressFunctions(false)` presses it again.
        state.press(code);
        let events = controls.tick(&mut state);
        assert_eq!(
            events,
            vec![
                ControlEvent {
                    action: MinimapAction::AddWaypoint,
                    kind: ControlEventKind::Release
                },
                ControlEvent {
                    action: MinimapAction::AddWaypoint,
                    kind: ControlEventKind::Press
                },
            ]
        );

        // Another extra click behaves the same way.
        state.press(code);
        let events = controls.tick(&mut state);
        assert_eq!(
            events,
            vec![
                ControlEvent {
                    action: MinimapAction::AddWaypoint,
                    kind: ControlEventKind::Release
                },
                ControlEvent {
                    action: MinimapAction::AddWaypoint,
                    kind: ControlEventKind::Press
                },
            ]
        );

        // Physical release fires all release callbacks.
        state.release(code);
        let events = controls.tick(&mut state);
        assert_eq!(
            events,
            vec![ControlEvent {
                action: MinimapAction::AddWaypoint,
                kind: ControlEventKind::Release
            }]
        );
        assert!(!controls
            .key_mapping_controllers()
            .get_controller("add_waypoint")
            .unwrap()
            .is_pressed());
    }

    #[test]
    fn multiple_clicks_on_press_are_flushed() {
        let mut controls = ControlsRegister::new();
        let mut state = TestKeyState::default();
        state.press(87); // temporary_waypoint: two clicks in one tick
        state.press(87);

        let events = controls.tick(&mut state);
        assert_eq!(
            events,
            vec![
                ControlEvent {
                    action: MinimapAction::TemporaryWaypoint,
                    kind: ControlEventKind::Press
                },
                ControlEvent {
                    action: MinimapAction::TemporaryWaypoint,
                    kind: ControlEventKind::Release
                },
                ControlEvent {
                    action: MinimapAction::TemporaryWaypoint,
                    kind: ControlEventKind::Press
                },
            ]
        );
    }

    #[test]
    fn held_functions_only_fire_on_start() {
        let mut manager = KeyMappingControllerManager::new();
        manager
            .register_controller(KeyMapping::new("zoom_in", "gui.xaero_zoom_in", -1, true))
            .unwrap();
        manager
            .register_function("zoom_in", KeyFunction::held(MinimapAction::ZoomIn))
            .unwrap();
        manager
            .register_function("zoom_in", KeyFunction::new(MinimapAction::ZoomIn))
            .unwrap();
        let mut state = TestKeyState::default();
        state.press(-1);

        let mut handler = KeyMappingTickHandler::new();
        let first = handler.tick(&mut manager, &mut state);
        assert_eq!(first.len(), 2);
        assert!(first.iter().all(|e| e.kind == ControlEventKind::Press));

        // Second tick: only the non-held function is re-pressed.
        let second = handler.tick(&mut manager, &mut state);
        assert_eq!(
            second,
            vec![ControlEvent {
                action: MinimapAction::ZoomIn,
                kind: ControlEventKind::Press
            }]
        );
    }

    #[test]
    fn non_xaero_mapping_uses_physical_key_state() {
        let mut manager = KeyMappingControllerManager::new();
        manager
            .register_controller(KeyMapping::new("vanilla", "key.test", 42, false))
            .unwrap();
        manager
            .register_function("vanilla", KeyFunction::new(MinimapAction::ToggleMap))
            .unwrap();
        let mut state = TestKeyState::default();
        let mut handler = KeyMappingTickHandler::new();

        state.down.insert(42, true);
        assert_eq!(
            handler.tick(&mut manager, &mut state),
            vec![ControlEvent {
                action: MinimapAction::ToggleMap,
                kind: ControlEventKind::Press
            }]
        );
        state.down.insert(42, false);
        assert_eq!(
            handler.tick(&mut manager, &mut state),
            vec![ControlEvent {
                action: MinimapAction::ToggleMap,
                kind: ControlEventKind::Release
            }]
        );
    }
}
