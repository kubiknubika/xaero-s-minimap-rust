//! Applies `xaerominimap-controls` key events to the Horizon HUD state.
//!
//! The input layer is platform-specific; the port already produces
//! [`ControlEvent`]s from `MinimapKeyMappings`. This bridge implements the
//! HUD-visible ones: map toggle, zoom, enlarge map, radar and cave mode.

use std::collections::BTreeSet;
use std::sync::Arc;

use xaerominimap_controls::{
    ControlEvent, ControlEventKind, ControlsRegister, KeyStateSource, MinimapAction,
};

use crate::{MinimapState, MinimapViewConfig};

/// Default map side used by `HeldEnlargeMap`/`ToggledEnlargeMap`.
pub const DEFAULT_MAP_SIZE: f32 = 160.0;
pub const ENLARGED_MAP_SIZE: f32 = 240.0;

pub struct ControlBridge {
    state: Arc<MinimapState>,
    held_enlarged: bool,
    toggled_enlarged: bool,
}

impl ControlBridge {
    pub fn new(state: Arc<MinimapState>) -> Self {
        ControlBridge {
            state,
            held_enlarged: false,
            toggled_enlarged: false,
        }
    }

    pub fn state(&self) -> &Arc<MinimapState> {
        &self.state
    }

    /// Applies one event. Returns `true` when it changed a HUD-visible option.
    pub fn apply(&mut self, event: ControlEvent) -> bool {
        let mut view = self.state.view();
        let mut changed = true;
        match (event.action, event.kind) {
            (MinimapAction::ToggleMap, ControlEventKind::Press) => view.enabled = !view.enabled,
            (MinimapAction::ToggleRadar, ControlEventKind::Press) => {
                view.radar_enabled = !view.radar_enabled
            }
            (MinimapAction::ZoomIn, ControlEventKind::Press) => {
                view.scale = (view.scale / 2.0).max(0.25)
            }
            (MinimapAction::ZoomOut, ControlEventKind::Press) => {
                view.scale = (view.scale * 2.0).min(64.0)
            }
            (MinimapAction::HeldEnlargeMap, ControlEventKind::Press) => {
                self.held_enlarged = true;
                view.map_size = ENLARGED_MAP_SIZE;
            }
            (MinimapAction::HeldEnlargeMap, ControlEventKind::Release) => {
                self.held_enlarged = false;
                view.map_size = self.default_size();
            }
            (MinimapAction::ToggledEnlargeMap, ControlEventKind::Press) => {
                self.toggled_enlarged = !self.toggled_enlarged;
                view.map_size = self.default_size();
            }
            (MinimapAction::ManualCaveMode, ControlEventKind::Press) => {
                view.cave_mode = !view.cave_mode
            }
            _ => changed = false,
        }
        if changed {
            self.state.set_view(view);
        }
        changed
    }

    pub fn apply_all(&mut self, events: &[ControlEvent]) -> usize {
        events
            .iter()
            .copied()
            .filter(|event| self.apply(*event))
            .count()
    }

    fn default_size(&self) -> f32 {
        if self.held_enlarged || self.toggled_enlarged {
            ENLARGED_MAP_SIZE
        } else {
            DEFAULT_MAP_SIZE
        }
    }
}

/// Maps a Horizon key name (`key.keyboard.o`, `key.keyboard.f11`) to the
/// LWJGL key code stored by the Java mod's `KeyMapping`s. This is the classic
/// LWJGL2 table Minecraft 26.3 uses for the ported default codes (4→5, O→24,
/// left Ctrl→29, F11→87). Unknown keys return `None` so a mixin can ignore
/// them instead of binding the wrong action.
pub fn horizon_key_code(name: &str) -> Option<i32> {
    let key = name.strip_prefix("key.keyboard.")?;
    if key.len() == 1 {
        let byte = key.as_bytes()[0];
        return match byte {
            b'0'..=b'9' => Some((byte - b'0') as i32 + 1),
            b'a'..=b'z' => Some(match byte {
                b'q' => 16,
                b'w' => 17,
                b'e' => 18,
                b'r' => 19,
                b't' => 20,
                b'y' => 21,
                b'u' => 22,
                b'i' => 23,
                b'o' => 24,
                b'p' => 25,
                b'a' => 30,
                b's' => 31,
                b'd' => 32,
                b'f' => 33,
                b'g' => 34,
                b'h' => 35,
                b'j' => 36,
                b'k' => 37,
                b'l' => 38,
                b'z' => 44,
                b'x' => 45,
                b'c' => 46,
                b'v' => 47,
                b'b' => 48,
                b'n' => 49,
                b'm' => 50,
                _ => return None,
            }),
            _ => None,
        };
    }
    match key {
        "space" => Some(57),
        "escape" => Some(1),
        "enter" | "kp_enter" => Some(28),
        "tab" => Some(15),
        "backspace" => Some(14),
        "left.shift" | "shift.left" => Some(42),
        "right.shift" | "shift.right" => Some(54),
        "left.control" | "control.left" => Some(29),
        "right.control" | "control.right" => Some(157),
        "left.alt" | "alt.left" => Some(56),
        "right.alt" | "alt.right" => Some(184),
        "arrow.left" | "left" => Some(203),
        "arrow.right" | "right" => Some(205),
        "arrow.up" | "up" => Some(200),
        "arrow.down" | "down" => Some(208),
        "insert" => Some(210),
        "delete" => Some(211),
        "home" => Some(199),
        "end" => Some(207),
        "page_up" | "pageup" => Some(201),
        "page_down" | "pagedown" => Some(209),
        _ => {
            if let Some(number) = key.strip_prefix('f') {
                let number: i32 = number.parse().ok()?;
                return match number {
                    1..=10 => Some(58 + number),
                    11 => Some(87),
                    12 => Some(88),
                    _ => None,
                };
            }
            None
        }
    }
}

/// Key state supplied by Horizon's `native_key` hook. Codes are the same
/// integers `MinimapKeyMappings` stores; an input mixin maps Horizon's key
/// names to those codes (options screens can rebind them).
#[derive(Debug, Default)]
pub struct HorizonKeyState {
    down: BTreeSet<i32>,
    clicks: BTreeSet<i32>,
}

impl HorizonKeyState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds a key press or release. Repeated presses do not queue a click.
    pub fn set(&mut self, code: i32, pressed: bool) {
        if pressed {
            if self.down.insert(code) {
                self.clicks.insert(code);
            }
        } else {
            self.down.remove(&code);
        }
    }

    pub fn clear(&mut self) {
        self.down.clear();
        self.clicks.clear();
    }
}

impl KeyStateSource for HorizonKeyState {
    fn is_down(&self, code: i32) -> bool {
        self.down.contains(&code)
    }

    fn consume_click(&mut self, code: i32) -> bool {
        self.clicks.remove(&code)
    }
}

/// End-to-end input path: Horizon key events → `ControlsRegister.tick` →
/// [`ControlBridge`] → HUD view options.
pub struct HorizonControls {
    keys: HorizonKeyState,
    register: ControlsRegister,
    bridge: ControlBridge,
}

impl HorizonControls {
    pub fn new(state: Arc<MinimapState>) -> Self {
        // `ControlsRegister::default()` already registers the built-in
        // `MinimapKeyMappings`; registering them again fails on duplicate ids.
        HorizonControls {
            keys: HorizonKeyState::new(),
            register: ControlsRegister::new(),
            bridge: ControlBridge::new(state),
        }
    }

    pub fn set_key(&mut self, code: i32, pressed: bool) {
        self.keys.set(code, pressed);
    }

    pub fn clear_keys(&mut self) {
        self.keys.clear();
    }

    /// Runs one input tick and applies the resulting events. Returns how many
    /// HUD options changed.
    pub fn tick(&mut self) -> usize {
        let events = self.register.tick(&mut self.keys);
        self.bridge.apply_all(&events)
    }

    pub fn bridge(&self) -> &ControlBridge {
        &self.bridge
    }
}

impl From<Arc<MinimapState>> for ControlBridge {
    fn from(state: Arc<MinimapState>) -> Self {
        Self::new(state)
    }
}

/// Kept public for mixin bridges that need the full view struct.
pub type HudView = MinimapViewConfig;

#[cfg(test)]
mod tests {
    use super::*;

    fn event(action: MinimapAction, kind: ControlEventKind) -> ControlEvent {
        ControlEvent { action, kind }
    }

    #[test]
    fn zoom_and_toggles_change_the_hud_view() {
        let state = Arc::new(MinimapState::new());
        let mut bridge = ControlBridge::new(state.clone());
        assert!(bridge.apply(event(MinimapAction::ToggleMap, ControlEventKind::Press)));
        assert!(!state.is_enabled());
        assert!(bridge.apply(event(MinimapAction::ToggleRadar, ControlEventKind::Press)));
        assert!(!state.is_radar_enabled());
        assert!(bridge.apply(event(MinimapAction::ZoomIn, ControlEventKind::Press)));
        assert_eq!(state.view().scale, 0.5);
        assert!(bridge.apply(event(MinimapAction::ZoomOut, ControlEventKind::Press)));
        assert_eq!(state.view().scale, 1.0);
        // Out-of-range keys are ignored.
        assert!(!bridge.apply(event(MinimapAction::AddWaypoint, ControlEventKind::Press)));
    }

    #[test]
    fn held_enlarge_restores_size_on_release() {
        let state = Arc::new(MinimapState::new());
        let mut bridge = ControlBridge::new(state.clone());
        assert!(bridge.apply(event(
            MinimapAction::HeldEnlargeMap,
            ControlEventKind::Press
        )));
        assert_eq!(state.view().map_size, ENLARGED_MAP_SIZE);
        assert!(bridge.apply(event(
            MinimapAction::HeldEnlargeMap,
            ControlEventKind::Release
        )));
        assert_eq!(state.view().map_size, DEFAULT_MAP_SIZE);
    }

    #[test]
    fn toggled_enlarge_and_cave_mode_are_independent() {
        let state = Arc::new(MinimapState::new());
        let mut bridge = ControlBridge::new(state.clone());
        bridge.apply_all(&[
            event(MinimapAction::ToggledEnlargeMap, ControlEventKind::Press),
            event(MinimapAction::ManualCaveMode, ControlEventKind::Press),
        ]);
        assert_eq!(state.view().map_size, ENLARGED_MAP_SIZE);
        assert!(state.is_cave_mode());
        bridge.apply(event(MinimapAction::ToggledEnlargeMap, ControlEventKind::Press));
        assert_eq!(state.view().map_size, DEFAULT_MAP_SIZE);
        assert!(state.is_cave_mode());
    }

    #[test]
    fn horizon_key_names_map_to_lwjgl_codes() {
        assert_eq!(horizon_key_code("key.keyboard.o"), Some(24));
        assert_eq!(horizon_key_code("key.keyboard.left.control"), Some(29));
        assert_eq!(horizon_key_code("key.keyboard.f11"), Some(87));
        assert_eq!(horizon_key_code("key.keyboard.4"), Some(5));
        assert_eq!(horizon_key_code("key.keyboard.a"), Some(30));
        assert_eq!(horizon_key_code("key.keyboard.z"), Some(44));
        assert_eq!(horizon_key_code("key.mouse.left"), None);
    }

    #[test]
    fn horizon_key_state_tracks_down_and_clicks() {
        let mut keys = HorizonKeyState::new();
        keys.set(29, true);
        keys.set(29, true); // auto-repeat must not queue another click
        assert!(keys.is_down(29));
        assert!(keys.consume_click(29));
        assert!(!keys.consume_click(29));
        keys.set(29, false);
        assert!(!keys.is_down(29));
    }

    #[test]
    fn horizon_controls_feed_key_events_into_the_hud() {
        let state = Arc::new(MinimapState::new());
        let mut controls = HorizonControls::new(state.clone());
        controls.set_key(29, true); // enlarge_map
        assert_eq!(controls.tick(), 2); // held + toggled enlarge functions
        assert_eq!(state.view().map_size, ENLARGED_MAP_SIZE);
        controls.set_key(29, false);
        controls.tick();
        // The same key also carries the toggled function, so the map stays
        // enlarged until the next press toggles it off.
        assert_eq!(state.view().map_size, ENLARGED_MAP_SIZE);
        controls.set_key(29, true);
        controls.tick();
        controls.set_key(29, false);
        controls.tick();
        assert_eq!(state.view().map_size, DEFAULT_MAP_SIZE);
    }

    #[test]
    fn zoom_bounds_match_the_port_scale_semantics() {
        let state = Arc::new(MinimapState::new());
        let mut bridge = ControlBridge::new(state.clone());
        let mut view = state.view();
        view.scale = 0.25;
        state.set_view(view);
        bridge.apply(event(MinimapAction::ZoomIn, ControlEventKind::Press));
        assert_eq!(state.view().scale, 0.25);
        let mut view = state.view();
        view.scale = 64.0;
        state.set_view(view);
        bridge.apply(event(MinimapAction::ZoomOut, ControlEventKind::Press));
        assert_eq!(state.view().scale, 64.0);
    }
}
