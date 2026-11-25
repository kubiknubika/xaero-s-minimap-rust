//! Entity radar icon definitions (the data half of the Java mod's
//! `xaero/hud/minimap/radar/icon/definition` package).
//!
//! The Java mod loads one JSON file per entity type from
//! `assets/xaerominimap/entity/icon/definition/<namespace>/<path>.json`, parses
//! it with Gson into a `RadarIconDefinition` and resolves each `variants`
//! entry into a `RadarIconForm`:
//!
//! | Java class | Rust type |
//! | --- | --- |
//! | `.../definition/RadarIconDefinition` | [`IconDefinition`] |
//! | `.../definition/RadarIconDefinitionManager` | [`IconDefinitionManager`] |
//! | `.../definition/form/RadarIconForm` | [`IconForm`] |
//! | `.../definition/form/model/config/RadarIconModelConfig` | [`ModelConfig`] |
//! | `.../definition/form/type/RadarIconFormTypes` | [`IconFormType`] |
//!
//! The engine-side drawing is game-specific; this module ports the resource
//! format, form selection and fallbacks so that a Horizon-side renderer can
//! consume the definitions. Form capabilities, model field references and
//! render traces live in [`crate::radar_icon_prerender`].

use std::collections::BTreeMap;

use xaerominimap_json::{get as json_get, Json};

/// `xaero/hud/minimap/radar/icon/definition/form/type/RadarIconFormType`:
/// the kind of icon to draw for one entity type variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconFormType {
    /// `model`: render the entity's Minecraft model into the icon.
    Model,
    /// `normal_sprite`: flat sprite, no outline.
    NormalSprite,
    /// `outlined_sprite`: sprite outlined in black.
    OutlinedSprite,
    /// `sprite`: the legacy sprite prerenderer.
    Sprite,
    /// `item`: render an item (or the entity's own item when unspecified).
    Item,
    /// `dot`: no icon; the coloured radar dot is drawn instead.
    Dot,
}

impl IconFormType {
    /// All types, in the order registered by `RadarIconFormTypes`.
    pub const ALL: [IconFormType; 6] = [
        IconFormType::Model,
        IconFormType::NormalSprite,
        IconFormType::OutlinedSprite,
        IconFormType::Sprite,
        IconFormType::Item,
        IconFormType::Dot,
    ];

    /// `getId()` — the name used in definition JSON values.
    pub fn name(self) -> &'static str {
        match self {
            IconFormType::Model => "model",
            IconFormType::NormalSprite => "normal_sprite",
            IconFormType::OutlinedSprite => "outlined_sprite",
            IconFormType::Sprite => "sprite",
            IconFormType::Item => "item",
            IconFormType::Dot => "dot",
        }
    }

    /// `RadarIconFormTypes.readType(String)`.
    pub fn from_name(name: &str) -> Option<IconFormType> {
        IconFormType::ALL.into_iter().find(|t| t.name() == name)
    }

    /// Whether this type uses a sprite resource (`RadarIconSpriteForm`).
    pub fn is_sprite(self) -> bool {
        matches!(
            self,
            IconFormType::NormalSprite | IconFormType::OutlinedSprite | IconFormType::Sprite
        )
    }
}

/// `xaero/.../definition/form/model/config/RadarIconModelConfig`: the
/// transform and model-part fields used by the model-form prerenderer.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelConfig {
    pub base_scale: f32,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub rotation_z: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub model_parts_rotation_reset: bool,
    /// `Boolean` in Java: `null` means "decide from the entity model".
    pub rendering_full_model: Option<bool>,
    pub model_main_part_field_aliases: Vec<String>,
    pub model_parts_fields: Vec<String>,
    /// Root-model field paths; each inner list is one alternative path.
    pub model_root_path: Vec<Vec<String>>,
    pub layers_allowed: bool,
}

impl Default for ModelConfig {
    /// The Java mod's `new RadarIconModelConfig()` defaults (all zero/false).
    fn default() -> Self {
        ModelConfig {
            base_scale: 0.0,
            rotation_y: 0.0,
            rotation_x: 0.0,
            rotation_z: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            model_parts_rotation_reset: false,
            rendering_full_model: None,
            model_main_part_field_aliases: Vec::new(),
            model_parts_fields: Vec::new(),
            model_root_path: Vec::new(),
            layers_allowed: false,
        }
    }
}

impl ModelConfig {
    /// Reads a Gson-style config object. Missing numeric/boolean fields keep
    /// their Java defaults; wrong types are reported as errors.
    pub fn from_json(value: &Json) -> Result<ModelConfig, String> {
        let mut config = ModelConfig::default();
        config.base_scale = read_float(value, "baseScale")?;
        config.rotation_y = read_float(value, "rotationY")?;
        config.rotation_x = read_float(value, "rotationX")?;
        config.rotation_z = read_float(value, "rotationZ")?;
        config.offset_x = read_float(value, "offsetX")?;
        config.offset_y = read_float(value, "offsetY")?;
        config.model_parts_rotation_reset = read_bool(value, "modelPartsRotationReset")?;
        config.layers_allowed = read_bool(value, "layersAllowed")?;
        config.rendering_full_model = match json_get(value, "renderingFullModel") {
            None | Some(Json::Null) => None,
            Some(Json::Bool(b)) => Some(*b),
            Some(_) => return Err("`renderingFullModel` must be a boolean".into()),
        };
        config.model_main_part_field_aliases = read_string_list(value, "modelMainPartFieldAliases")?;
        config.model_parts_fields = read_string_list(value, "modelPartsFields")?;
        config.model_root_path = read_string_list_list(value, "modelRootPath")?;
        Ok(config)
    }
}

fn read_float(obj: &Json, key: &str) -> Result<f32, String> {
    match json_get(obj, key) {
        None | Some(Json::Null) => Ok(0.0),
        Some(Json::Num(n)) => Ok(*n as f32),
        Some(_) => Err(format!("`{key}` must be a number")),
    }
}

fn read_bool(obj: &Json, key: &str) -> Result<bool, String> {
    match json_get(obj, key) {
        None | Some(Json::Null) => Ok(false),
        Some(Json::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("`{key}` must be a boolean")),
    }
}

fn read_string_list(obj: &Json, key: &str) -> Result<Vec<String>, String> {
    match json_get(obj, key) {
        None | Some(Json::Null) => Ok(Vec::new()),
        Some(Json::Arr(items)) => items
            .iter()
            .map(|item| match item {
                Json::Str(s) => Ok(s.clone()),
                _ => Err(format!("`{key}` must contain only strings")),
            })
            .collect(),
        Some(_) => Err(format!("`{key}` must be a list of strings")),
    }
}

fn read_string_list_list(obj: &Json, key: &str) -> Result<Vec<Vec<String>>, String> {
    match json_get(obj, key) {
        None | Some(Json::Null) => Ok(Vec::new()),
        Some(Json::Arr(paths)) => paths
            .iter()
            .map(|path| match path {
                Json::Arr(items) => items
                    .iter()
                    .map(|item| match item {
                        Json::Str(s) => Ok(s.clone()),
                        _ => Err(format!("`{key}` must contain only strings")),
                    })
                    .collect(),
                _ => Err(format!("`{key}` must contain lists of strings")),
            })
            .collect(),
        Some(_) => Err(format!("`{key}` must be a list of lists")),
    }
}

/// The parsed data of a `RadarIconForm`. The form type is kept separately
/// because three sprite types share one Java class but different
/// prerenderers.
#[derive(Debug, Clone, PartialEq)]
pub enum IconFormKind {
    /// Model form. `None` means the default model renderer (no config).
    Model { config: Option<ModelConfig> },
    /// Sprite form; the value is the full icon resource identifier, e.g.
    /// `xaerominimap:entity/icon/sprite/pig.png`.
    Sprite { texture: String },
    /// Item form. `None` means the entity's own item (`SELF_ITEM`).
    Item { item: Option<String> },
    /// Coloured radar dot, no icon.
    Dot,
}

/// `xaero/.../definition/form/RadarIconForm`: one resolved variant form.
#[derive(Debug, Clone, PartialEq)]
pub struct IconForm {
    pub form_type: IconFormType,
    pub kind: IconFormKind,
}

impl IconForm {
    /// The default model form (`RadarIconBasicForms.DEFAULT_MODEL`).
    pub fn default_model() -> Self {
        IconForm::model(None)
    }

    pub fn model(config: Option<ModelConfig>) -> Self {
        IconForm {
            form_type: IconFormType::Model,
            kind: IconFormKind::Model { config },
        }
    }

    /// Builds one of the three sprite forms.
    pub fn sprite(form_type: IconFormType, texture: impl Into<String>) -> Result<Self, String> {
        if !form_type.is_sprite() {
            return Err(format!("`{}` is not a sprite form type", form_type.name()));
        }
        Ok(IconForm {
            form_type,
            kind: IconFormKind::Sprite {
                texture: texture.into(),
            },
        })
    }

    pub fn item(item: Option<String>) -> Self {
        IconForm {
            form_type: IconFormType::Item,
            kind: IconFormKind::Item { item },
        }
    }

    pub fn dot() -> Self {
        IconForm {
            form_type: IconFormType::Dot,
            kind: IconFormKind::Dot,
        }
    }

    /// `getConfig()` — only set for an indexed `model:N` form.
    pub fn model_config(&self) -> Option<&ModelConfig> {
        match &self.kind {
            IconFormKind::Model { config } => config.as_ref(),
            _ => None,
        }
    }
}

/// `xaero/.../definition/RadarIconDefinition`: the icon forms of one entity
/// type, loaded from a JSON resource.
#[derive(Debug, Clone, PartialEq)]
pub struct IconDefinition {
    /// Entity type id, e.g. `minecraft:pig`.
    pub entity_id: String,
    /// Raw `variants` map from the JSON file (variant id → form string).
    pub variants: BTreeMap<String, String>,
    /// `modelConfigs` from the JSON file.
    pub model_configs: Vec<ModelConfig>,
    /// Parsed forms, keyed by variant id (filled by [`Self::construct`]).
    pub variant_forms: BTreeMap<String, IconForm>,
    /// Optional fully-qualified static method name for custom variants.
    pub variant_method: Option<String>,
    /// Optional legacy variant-id method name.
    pub variant_id_method: Option<String>,
    /// Optional variant-id builder method name.
    pub variant_id_builder_method: Option<String>,
}

impl IconDefinition {
    pub fn new(
        entity_id: &str,
        variants: BTreeMap<String, String>,
        model_configs: Vec<ModelConfig>,
        variant_method: Option<String>,
        variant_id_method: Option<String>,
        variant_id_builder_method: Option<String>,
    ) -> Self {
        let mut definition = IconDefinition {
            entity_id: entity_id.to_string(),
            variants,
            model_configs,
            variant_forms: BTreeMap::new(),
            variant_method,
            variant_id_method,
            variant_id_builder_method,
        };
        definition.construct();
        definition
    }

    /// Gson's `fromJson` followed by `construct(Identifier)`, as done by
    /// `RadarIconDefinitionReloader`.
    pub fn from_json(entity_id: &str, text: &str) -> Result<IconDefinition, String> {
        let root = xaerominimap_json::parse(text)?;
        let variants = match json_get(&root, "variants") {
            None | Some(Json::Null) => BTreeMap::new(),
            Some(Json::Obj(entries)) => entries
                .iter()
                .map(|(key, value)| match value {
                    Json::Str(s) => Ok((key.clone(), s.clone())),
                    _ => Err("`variants` values must be strings".to_string()),
                })
                .collect::<Result<BTreeMap<_, _>, _>>()?,
            Some(_) => return Err("`variants` must be an object".into()),
        };
        let model_configs = match json_get(&root, "modelConfigs") {
            None | Some(Json::Null) => Vec::new(),
            Some(Json::Arr(items)) => items
                .iter()
                .map(ModelConfig::from_json)
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => return Err("`modelConfigs` must be a list".into()),
        };
        Ok(IconDefinition::new(
            entity_id,
            variants,
            model_configs,
            optional_string(&root, "variantMethod"),
            optional_string(&root, "variantIdMethod"),
            optional_string(&root, "variantIdBuilderMethod"),
        ))
    }

    /// `construct(Identifier)`: parses every variant form and makes sure a
    /// `default` entry exists.
    pub fn construct(&mut self) {
        self.variant_forms.clear();
        let variants: Vec<(String, String)> = self
            .variants
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        for (variant, form) in variants {
            // The Java mod logs unparseable forms and skips them.
            if let Ok(form) = Self::construct_form(&form, self) {
                self.variant_forms.insert(variant, form);
            }
        }
        self.variant_forms
            .entry("default".to_string())
            .or_insert_with(IconForm::default_model);
    }

    /// `constructForm(String)`: `type[:args...]`.
    pub fn construct_form(text: &str, definition: &IconDefinition) -> Result<IconForm, String> {
        let args: Vec<&str> = text.split(':').collect();
        let type_name = args.first().copied().unwrap_or("");
        let form_type = IconFormType::from_name(type_name)
            .ok_or_else(|| format!("unknown radar icon form type `{type_name}`"))?;
        match form_type {
            IconFormType::Model => {
                if args.len() == 1 {
                    return Ok(IconForm::default_model());
                }
                if args.len() != 2 {
                    return Err(format!("`{text}` has too many arguments"));
                }
                let index = args[1].parse::<usize>().unwrap_or(0);
                // An invalid/undefined config falls back to the default
                // renderer, like the Java mod does after logging the error.
                Ok(IconForm::model(
                    definition.get_model_config(index).cloned(),
                ))
            }
            IconFormType::NormalSprite | IconFormType::OutlinedSprite | IconFormType::Sprite => {
                if args.len() != 2 {
                    return Err(format!("`{text}` needs exactly one sprite name"));
                }
                IconForm::sprite(
                    form_type,
                    format!("xaerominimap:entity/icon/sprite/{}", args[1]),
                )
            }
            IconFormType::Item => match args.len() {
                1 => Ok(IconForm::item(None)),
                2 => Ok(IconForm::item(Some(args[1].to_string()))),
                3 => Ok(IconForm::item(Some(format!("{}:{}", args[1], args[2])))),
                _ => Err(format!("`{text}` has too many arguments")),
            },
            IconFormType::Dot => {
                if args.len() == 1 {
                    Ok(IconForm::dot())
                } else {
                    Err("`dot` takes no arguments".into())
                }
            }
        }
    }

    /// `getVariantForm(String)` — exact match only.
    pub fn get_variant_form(&self, variant: &str) -> Option<&IconForm> {
        self.variant_forms.get(variant)
    }

    /// The form the manager uses: exact variant, then `default`, then the
    /// built-in default model form.
    pub fn resolve_variant_form(&self, variant: &str) -> IconForm {
        self.get_variant_form(variant)
            .or_else(|| self.get_variant_form("default"))
            .cloned()
            .unwrap_or_else(IconForm::default_model)
    }

    /// `getModelConfig(int)`.
    pub fn get_model_config(&self, index: usize) -> Option<&ModelConfig> {
        self.model_configs.get(index)
    }
}

fn optional_string(obj: &Json, key: &str) -> Option<String> {
    match json_get(obj, key) {
        Some(Json::Str(s)) => Some(s.clone()),
        _ => None,
    }
}

/// The definition resources built into the mod jar. These are the JSON files
/// under `assets/xaerominimap/entity/icon/definition/minecraft` with the
/// `_comment*` keys removed (the comments only document the format).
pub const BUILTIN_DEFINITIONS: &[(&str, &str)] = &[
    (
        "minecraft:axolotl",
        include_str!("../assets/entity/icon/definition/minecraft/axolotl.json"),
    ),
    (
        "minecraft:cat",
        include_str!("../assets/entity/icon/definition/minecraft/cat.json"),
    ),
    (
        "minecraft:enderman",
        include_str!("../assets/entity/icon/definition/minecraft/enderman.json"),
    ),
    (
        "minecraft:horse",
        include_str!("../assets/entity/icon/definition/minecraft/horse.json"),
    ),
    (
        "minecraft:iron_golem",
        include_str!("../assets/entity/icon/definition/minecraft/iron_golem.json"),
    ),
    (
        "minecraft:llama",
        include_str!("../assets/entity/icon/definition/minecraft/llama.json"),
    ),
    (
        "minecraft:pig",
        include_str!("../assets/entity/icon/definition/minecraft/pig.json"),
    ),
    (
        "minecraft:strider",
        include_str!("../assets/entity/icon/definition/minecraft/strider.json"),
    ),
    (
        "minecraft:trader_llama",
        include_str!("../assets/entity/icon/definition/minecraft/trader_llama.json"),
    ),
    (
        "minecraft:tropical_fish",
        include_str!("../assets/entity/icon/definition/minecraft/tropical_fish.json"),
    ),
    (
        "minecraft:villager",
        include_str!("../assets/entity/icon/definition/minecraft/villager.json"),
    ),
    (
        "minecraft:wolf",
        include_str!("../assets/entity/icon/definition/minecraft/wolf.json"),
    ),
    (
        "minecraft:zombie_villager",
        include_str!("../assets/entity/icon/definition/minecraft/zombie_villager.json"),
    ),
];

/// `xaero/.../definition/RadarIconDefinitionManager`: definitions by entity
/// type id, populated from resources at startup (`reloadResources`).
#[derive(Debug, Clone, Default)]
pub struct IconDefinitionManager {
    definitions: BTreeMap<String, IconDefinition>,
}

impl IconDefinitionManager {
    pub fn new() -> Self {
        IconDefinitionManager::default()
    }

    /// Manager with the mod's built-in definition resources loaded.
    pub fn with_builtins() -> Self {
        let mut manager = Self::new();
        manager.reload_builtins();
        manager
    }

    /// `reloadResources()` — reloads (and replaces) the built-in definitions.
    pub fn reload_builtins(&mut self) {
        self.definitions.clear();
        for (entity_id, json) in BUILTIN_DEFINITIONS {
            match IconDefinition::from_json(entity_id, json) {
                Ok(definition) => {
                    self.insert(definition);
                }
                Err(error) => {
                    // Resource files are part of the port; a parse failure
                    // here means a packaging bug, so keep the manager usable
                    // but leave that definition out.
                    eprintln!("invalid icon definition for {entity_id}: {error}");
                }
            }
        }
    }

    pub fn insert(&mut self, definition: IconDefinition) {
        self.definitions.insert(definition.entity_id.clone(), definition);
    }

    pub fn get(&self, entity_id: &str) -> Option<&IconDefinition> {
        self.definitions.get(entity_id)
    }

    pub fn definitions(&self) -> &BTreeMap<String, IconDefinition> {
        &self.definitions
    }

    /// Form used by the radar for `variant` of `entity_id`.
    pub fn form_for(&self, entity_id: &str, variant: &str) -> IconForm {
        self.get(entity_id)
            .map(|definition| definition.resolve_variant_form(variant))
            .unwrap_or_else(IconForm::default_model)
    }

    /// Form for a typed variant id from the cache/handler side.
    pub fn form_for_variant(
        &self,
        entity_id: &str,
        variant: &crate::radar_icon_cache::IconVariant,
    ) -> IconForm {
        self.form_for(entity_id, &variant.as_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"{
        "variants": {
            "default": "model",
            "axolotl_blue_baby": "model:0",
            "flat": "normal_sprite:pig.png",
            "outline": "outlined_sprite:pig.png",
            "legacy": "sprite:pig.png",
            "item": "item:minecraft:diamond",
            "self": "item",
            "none": "dot",
            "broken": "unknown:thing"
        },
        "modelConfigs": [
            {
                "baseScale": 0.5,
                "rotationY": 90,
                "offsetX": 5,
                "offsetY": -5,
                "renderingFullModel": true,
                "modelPartsRotationReset": true,
                "layersAllowed": true,
                "modelPartsFields": ["some.Model;head", "some.Model;body"],
                "modelRootPath": [["some.Model;root", "other.Model;root"]]
            }
        ],
        "variantMethod": "example.Mod.getVariant",
        "variantIdMethod": "example.Mod.getVariantId"
    }"#;

    #[test]
    fn form_type_names_round_trip() {
        for form_type in IconFormType::ALL {
            assert_eq!(IconFormType::from_name(form_type.name()), Some(form_type));
        }
        assert_eq!(IconFormType::from_name("nope"), None);
    }

    #[test]
    fn parses_all_form_kinds() {
        let definition = IconDefinition::from_json("example:pig", EXAMPLE).unwrap();
        assert_eq!(definition.entity_id, "example:pig");
        assert_eq!(definition.variant_method.as_deref(), Some("example.Mod.getVariant"));
        assert_eq!(definition.variant_id_method.as_deref(), Some("example.Mod.getVariantId"));
        assert_eq!(definition.get_model_config(0).unwrap().base_scale, 0.5);
        assert_eq!(definition.get_model_config(0).unwrap().rotation_y, 90.0);
        assert_eq!(definition.get_model_config(0).unwrap().offset_y, -5.0);
        assert_eq!(definition.get_model_config(0).unwrap().rendering_full_model, Some(true));
        assert_eq!(definition.get_model_config(0).unwrap().model_parts_fields.len(), 2);
        assert_eq!(definition.get_model_config(0).unwrap().model_root_path[0].len(), 2);

        assert_eq!(definition.get_variant_form("default").unwrap().form_type, IconFormType::Model);
        let model = definition.resolve_variant_form("axolotl_blue_baby");
        assert_eq!(model.form_type, IconFormType::Model);
        assert_eq!(model.model_config().unwrap().base_scale, 0.5);

        let sprite = definition.resolve_variant_form("flat");
        assert_eq!(sprite.form_type, IconFormType::NormalSprite);
        assert_eq!(
            sprite.kind,
            IconFormKind::Sprite {
                texture: "xaerominimap:entity/icon/sprite/pig.png".into()
            }
        );
        assert_eq!(definition.resolve_variant_form("outline").form_type, IconFormType::OutlinedSprite);
        assert_eq!(definition.resolve_variant_form("legacy").form_type, IconFormType::Sprite);
        assert_eq!(
            definition.resolve_variant_form("item").kind,
            IconFormKind::Item {
                item: Some("minecraft:diamond".into())
            }
        );
        assert_eq!(definition.resolve_variant_form("self").kind, IconFormKind::Item { item: None });
        assert_eq!(definition.resolve_variant_form("none").form_type, IconFormType::Dot);

        // Unparseable variants are skipped, and the exact-variant → default
        // fallback applies to unknown variants.
        assert!(definition.get_variant_form("broken").is_none());
        assert_eq!(definition.resolve_variant_form("broken").form_type, IconFormType::Model);
        assert_eq!(definition.resolve_variant_form("unknown").form_type, IconFormType::Model);
    }

    #[test]
    fn builtin_definitions_are_loaded() {
        let manager = IconDefinitionManager::with_builtins();
        assert_eq!(manager.definitions().len(), BUILTIN_DEFINITIONS.len());
        let pig = manager.get("minecraft:pig").unwrap();
        assert_eq!(pig.variants.get("default").map(String::as_str), Some("model"));
        assert_eq!(pig.resolve_variant_form("anything").form_type, IconFormType::Model);

        let axolotl = manager.get("minecraft:axolotl").unwrap();
        assert_eq!(axolotl.model_configs.len(), 2);
        let baby = axolotl.resolve_variant_form("minecraft:textures/entity/axolotl/axolotl_blue_baby.png");
        assert_eq!(baby.model_config().unwrap().base_scale, axolotl.model_configs[1].base_scale);

        assert_eq!(manager.form_for("minecraft:zombie", "default"), IconForm::default_model());
    }

    #[test]
    fn unparseable_forms_are_skipped() {
        assert!(IconDefinition::from_json("example:pig", "{").is_err());
        // A bad form string is skipped, just like the Java mod does after
        // logging it, and the `default` fallback is still installed.
        let definition = IconDefinition::from_json(
            "example:pig",
            r#"{"variants":{"broken":"nope"}}"#,
        )
        .unwrap();
        assert!(definition.get_variant_form("broken").is_none());
        assert_eq!(definition.get_variant_form("default").unwrap().form_type, IconFormType::Model);

        let empty = IconDefinition::new("example:pig", BTreeMap::new(), Vec::new(), None, None, None);
        assert!(IconDefinition::construct_form("item:a:b:c", &empty).is_err());
        assert!(IconForm::sprite(IconFormType::Item, "pig.png").is_err());
    }
}
