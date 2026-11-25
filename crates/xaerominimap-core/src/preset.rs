//! Data side of `xaero/hud/preset` from the original mod: HUD presets and
//! the placement action of a module preset.
//!
//! Java presets target live `HudModule` objects; the Rust port identifies
//! modules by string id and stores their transform in a map. Building the
//! actual widgets/screens stays engine glue.

use std::collections::{BTreeMap, BTreeSet};

/// `xaero/hud/module/ModuleTransform` fields written by placement actions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModuleTransform {
    pub x: i32,
    pub y: i32,
    pub centered: bool,
    pub from_right: bool,
    pub from_bottom: bool,
    pub flipped_hor: bool,
    pub flipped_ver: bool,
    /// The Java module keeps confirmed and unconfirmed transforms apart;
    /// this flag records that `confirmTransform` was called.
    pub confirmed: bool,
}

/// `xaero/hud/preset/action/PlacementPresetAction`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlacementPresetAction {
    pub x: i32,
    pub y: i32,
    pub centered: bool,
    pub from_right: bool,
    pub from_bottom: bool,
    pub flipped_hor: bool,
    pub flipped_ver: bool,
}

impl PlacementPresetAction {
    pub fn new(
        x: i32,
        y: i32,
        centered: bool,
        from_right: bool,
        from_bottom: bool,
        flipped_hor: bool,
        flipped_ver: bool,
    ) -> Self {
        PlacementPresetAction {
            x,
            y,
            centered,
            from_right,
            from_bottom,
            flipped_hor,
            flipped_ver,
        }
    }

    /// Java `apply(HudModule)` writes the unconfirmed transform of the
    /// module. `confirm` and `cancel` are no-ops for this action.
    pub fn apply(&self, transform: &mut ModuleTransform) {
        transform.x = self.x;
        transform.y = self.y;
        transform.centered = self.centered;
        transform.from_right = self.from_right;
        transform.from_bottom = self.from_bottom;
        transform.flipped_hor = self.flipped_hor;
        transform.flipped_ver = self.flipped_ver;
    }
}

/// One action of a module preset. Only the built-in placement action is
/// represented; other mods can add their own actions in Java.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetAction {
    Placement(PlacementPresetAction),
}

impl PresetAction {
    pub fn apply(&self, transform: &mut ModuleTransform) {
        match self {
            PresetAction::Placement(action) => action.apply(transform),
        }
    }
}

/// `xaero/hud/preset/ModulePreset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePreset {
    module_id: String,
    actions: Vec<PresetAction>,
}

impl ModulePreset {
    pub fn new(module_id: &str, actions: Vec<PresetAction>) -> Self {
        ModulePreset {
            module_id: module_id.to_string(),
            actions,
        }
    }

    /// Java `ModulePreset.Builder.setPlacement(int, int, boolean...)`.
    pub fn with_placement(module_id: &str, action: PlacementPresetAction) -> Self {
        ModulePreset::new(module_id, vec![PresetAction::Placement(action)])
    }

    pub fn get_module(&self) -> &str {
        &self.module_id
    }

    pub fn get_actions(&self) -> &[PresetAction] {
        &self.actions
    }

    /// Java `apply()` — apply all actions to the module's unconfirmed
    /// transform in the given map.
    pub fn apply(&self, transforms: &mut BTreeMap<String, ModuleTransform>) {
        let transform = transforms.entry(self.module_id.clone()).or_default();
        for action in &self.actions {
            action.apply(transform);
        }
    }

    /// `confirm()` / `cancel()` are action callbacks; placement has none,
    /// so these only matter for custom actions in the Java mod.
    pub fn confirm(&self) {}

    pub fn cancel(&self) {}
}

/// `xaero/hud/preset/HudPreset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudPreset {
    id: String,
    name: String,
    module_presets: BTreeSet<String>,
    /// Java stores `ModulePreset` objects in a `Set`; the Rust port keeps a
    /// vector so multiple presets for one module are still applied in order.
    presets: Vec<ModulePreset>,
    applied: bool,
}

impl HudPreset {
    pub fn new(id: &str, name: &str, module_presets: Vec<ModulePreset>) -> Self {
        let ids = module_presets
            .iter()
            .map(|preset| preset.get_module().to_string())
            .collect();
        HudPreset {
            id: id.to_string(),
            name: name.to_string(),
            module_presets: ids,
            presets: module_presets,
            applied: false,
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_module_presets(&self) -> &BTreeSet<String> {
        &self.module_presets
    }

    pub fn is_applied(&self) -> bool {
        self.applied
    }

    /// Java `apply()` — sets `applied` and applies every module preset.
    pub fn apply(&mut self, transforms: &mut BTreeMap<String, ModuleTransform>) {
        self.applied = true;
        for preset in &self.presets {
            preset.apply(transforms);
        }
    }

    /// Java `confirm()` / `cancel()` — ignored unless a preset was applied.
    pub fn confirm(&mut self) {
        if self.applied {
            self.applied = false;
            for preset in &self.presets {
                preset.confirm();
            }
        }
    }

    pub fn cancel(&mut self) {
        if self.applied {
            self.applied = false;
            for preset in &self.presets {
                preset.cancel();
            }
        }
    }

    /// Java `applyAndConfirm()` — apply, confirm and commit every module's
    /// unconfirmed transform.
    pub fn apply_and_confirm(&mut self, transforms: &mut BTreeMap<String, ModuleTransform>) {
        self.applied = true;
        self.apply(transforms);
        self.confirm();
        for id in &self.module_presets {
            if let Some(transform) = transforms.get_mut(id) {
                transform.confirmed = true;
            }
        }
    }
}

/// `xaero/hud/xminimap/preset/BuiltInHudPresets`: the four corner presets
/// registered by the minimap module.
pub fn builtin_presets() -> Vec<HudPreset> {
    let corners = [
        ("top_left", "gui.xaero_preset_topleft", false, false),
        ("top_right", "gui.xaero_preset_topright", true, false),
        ("bottom_left", "gui.xaero_preset_bottom_left", false, true),
        ("bottom_right", "gui.xaero_preset_bottom_right", true, true),
    ];
    corners
        .iter()
        .map(|(path, name, from_right, from_bottom)| {
            HudPreset::new(
                &format!("xaerominimap:{path}"),
                name,
                vec![ModulePreset::with_placement(
                    "minimap",
                    PlacementPresetAction::new(0, 0, false, *from_right, *from_bottom, false, false),
                )],
            )
        })
        .collect()
}

/// `BuiltInHudPresets.addAll(HudPresetManager)`.
pub fn add_builtin_presets(manager: &mut HudPresetManager) {
    for preset in builtin_presets() {
        manager.register(preset);
    }
}

/// `xaero/hud/preset/HudPresetManager`.
#[derive(Debug, Clone, Default)]
pub struct HudPresetManager {
    presets: BTreeMap<String, HudPreset>,
}

impl HudPresetManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `register(HudPreset)`.
    pub fn register(&mut self, preset: HudPreset) {
        self.presets.insert(preset.get_id().to_string(), preset);
    }

    /// Java `getPresets()`.
    pub fn get_presets(&self) -> impl Iterator<Item = &HudPreset> {
        self.presets.values()
    }

    pub fn get(&self, id: &str) -> Option<&HudPreset> {
        self.presets.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preset() -> HudPreset {
        HudPreset::new(
            "xaerominimap:default",
            "Default",
            vec![ModulePreset::with_placement(
                "minimap",
                PlacementPresetAction::new(10, 20, true, false, true, false, false),
            )],
        )
    }

    #[test]
    fn placement_action_writes_every_transform_field() {
        let mut transform = ModuleTransform::default();
        PlacementPresetAction::new(1, 2, true, true, false, false, true).apply(&mut transform);
        assert_eq!(
            transform,
            ModuleTransform {
                x: 1,
                y: 2,
                centered: true,
                from_right: true,
                from_bottom: false,
                flipped_hor: false,
                flipped_ver: true,
                confirmed: false,
            }
        );
    }

    #[test]
    fn preset_applies_to_its_module() {
        let mut preset = preset();
        assert_eq!(preset.get_id(), "xaerominimap:default");
        assert_eq!(preset.get_name(), "Default");
        assert!(preset.get_module_presets().contains("minimap"));

        let mut transforms = BTreeMap::new();
        preset.apply(&mut transforms);
        assert!(preset.is_applied());
        let transform = &transforms["minimap"];
        assert_eq!((transform.x, transform.y), (10, 20));
        assert!(transform.centered && transform.from_bottom);
        assert!(!transform.confirmed);
    }

    #[test]
    fn apply_and_confirm_commits_the_transform() {
        let mut preset = preset();
        let mut transforms = BTreeMap::new();
        preset.apply_and_confirm(&mut transforms);
        assert!(transforms["minimap"].confirmed);
        assert!(!preset.is_applied());
    }

    #[test]
    fn manager_registers_and_lists_presets() {
        let mut manager = HudPresetManager::new();
        manager.register(preset());
        assert!(manager.get("xaerominimap:default").is_some());
        assert_eq!(manager.get_presets().count(), 1);
    }

    #[test]
    fn builtin_corner_presets_match_java() {
        let presets = builtin_presets();
        assert_eq!(presets.len(), 4);
        assert_eq!(presets[0].get_id(), "xaerominimap:top_left");
        assert_eq!(presets[0].get_name(), "gui.xaero_preset_topleft");
        assert_eq!(presets[3].get_id(), "xaerominimap:bottom_right");
        assert_eq!(presets[3].get_name(), "gui.xaero_preset_bottom_right");

        let mut manager = HudPresetManager::new();
        add_builtin_presets(&mut manager);
        assert_eq!(manager.get_presets().count(), 4);

        let mut transforms = BTreeMap::new();
        manager
            .get("xaerominimap:bottom_right")
            .unwrap()
            .clone()
            .apply_and_confirm(&mut transforms);
        let transform = &transforms["minimap"];
        assert!(transform.from_right && transform.from_bottom);
        assert!(!transform.centered);
        assert!(transform.confirmed);
    }
}
