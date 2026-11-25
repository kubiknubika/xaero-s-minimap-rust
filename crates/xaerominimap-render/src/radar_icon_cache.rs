//! Radar icon cache and variant ids (the data half of
//! `xaero/hud/minimap/radar/icon/cache` in the original mod).
//!
//! The Java mod keys its cached icons by a `RadarIconKey` of an entity
//! variant object and equipped armour, and obtains the variant object from
//! `BuiltInRadarIconDefinitions.getVariant`. Variants are strings such as
//! `minecraft:textures/entity/pig/pig.png%true`, so the Rust port models them
//! as typed values that render to exactly those strings. The game-side glue
//! supplies the entity state ([`EntityVariantData`], [`Equipment`]); the GPU
//! icon itself is represented by an id ([`CachedIcon`]).

use std::collections::{BTreeMap, HashMap};
use std::fmt;

/// Markings of a horse, mapped to the overlay textures used by
/// `HorseVariant.toString()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HorseMarkings {
    None,
    White,
    WhiteField,
    WhiteDots,
    BlackDots,
}

impl HorseMarkings {
    /// The string appended to the entity texture by `HorseVariant`.
    pub fn variant_text(self) -> &'static str {
        match self {
            HorseMarkings::None => "null",
            HorseMarkings::White => "minecraft:textures/entity/horse/horse_markings_white.png",
            HorseMarkings::WhiteField => {
                "minecraft:textures/entity/horse/horse_markings_whitefield.png"
            }
            HorseMarkings::WhiteDots => {
                "minecraft:textures/entity/horse/horse_markings_whitedots.png"
            }
            HorseMarkings::BlackDots => {
                "minecraft:textures/entity/horse/horse_markings_blackdots.png"
            }
        }
    }
}

/// `xaero/.../cache/id/variant/*`: a stable id for one variant of an entity
/// type. `Display` produces the string used as a key in definition JSONs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IconVariant {
    /// Plain identifier/variant string (also used for item entities).
    Texture(String),
    /// Horse with a markings overlay.
    Horse {
        texture: String,
        markings: HorseMarkings,
    },
    /// Villager (`texture%baby%type%profession%level`).
    Villager {
        texture: String,
        baby: bool,
        villager_type: String,
        profession: String,
        level: i32,
    },
    /// Cat/wolf (`texture%tamed`).
    Tamable { texture: String, tamed: bool },
    /// Iron golem crackiness (`texture%level`).
    IronGolem { texture: String, crackiness: String },
    /// Llama (`texture%trader%body armour`).
    Llama {
        texture: String,
        trader: bool,
        body_armor: String,
    },
    /// Pig/strider (`texture%saddled`).
    Saddle { texture: String, saddled: bool },
    /// Tropical fish (`texture%pattern%base colour%pattern colour`).
    TropicalFish {
        texture: String,
        pattern: String,
        base_color: String,
        pattern_color: String,
    },
    /// Enderman (`texture%creepy`).
    Enderman { texture: String, creepy: bool },
    /// Cushion map colour, or `default` when it has no colour.
    Cushion { color: Option<String> },
}

impl IconVariant {
    pub fn texture(texture: impl Into<String>) -> Self {
        IconVariant::Texture(texture.into())
    }

    /// The `toString()` of the Java variant object.
    pub fn as_id(&self) -> String {
        match self {
            IconVariant::Texture(texture) => texture.clone(),
            IconVariant::Horse { texture, markings } => {
                format!("{texture}%{}", markings.variant_text())
            }
            IconVariant::Villager {
                texture,
                baby,
                villager_type,
                profession,
                level,
            } => format!("{texture}%{baby}%{villager_type}%{profession}%{level}"),
            IconVariant::Tamable { texture, tamed } => format!("{texture}%{tamed}"),
            IconVariant::IronGolem {
                texture,
                crackiness,
            } => format!("{texture}%{crackiness}"),
            IconVariant::Llama {
                texture,
                trader,
                body_armor,
            } => format!("{texture}%{trader}%{body_armor}"),
            IconVariant::Saddle { texture, saddled } => format!("{texture}%{saddled}"),
            IconVariant::TropicalFish {
                texture,
                pattern,
                base_color,
                pattern_color,
            } => format!("{texture}%{pattern}%{base_color}%{pattern_color}"),
            IconVariant::Enderman { texture, creepy } => format!("{texture}%{creepy}"),
            IconVariant::Cushion { color } => color.clone().unwrap_or_else(|| "default".into()),
        }
    }
}

impl fmt::Display for IconVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_id())
    }
}

/// Entity state needed by [`BuiltinVariantResolver`], mirroring the checks in
/// `BuiltInRadarIconDefinitions.getVariant`.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityVariantData {
    /// Main entity texture (`Identifier.toString()` in Java).
    pub texture: String,
    pub kind: VariantKind,
}

impl EntityVariantData {
    pub fn new(texture: impl Into<String>, kind: VariantKind) -> Self {
        EntityVariantData {
            texture: texture.into(),
            kind,
        }
    }

    /// `BuiltInRadarIconDefinitions.getVariant(Identifier, ..., Entity)`.
    pub fn variant(&self) -> IconVariant {
        let texture = self.texture.clone();
        match &self.kind {
            VariantKind::Other => IconVariant::texture(texture),
            VariantKind::Horse { markings } => IconVariant::Horse {
                texture,
                markings: *markings,
            },
            VariantKind::Villager {
                baby,
                villager_type,
                profession,
                level,
            } => IconVariant::Villager {
                texture,
                baby: *baby,
                villager_type: villager_type.clone(),
                profession: profession.clone(),
                level: *level,
            },
            VariantKind::Tamable { tamed } => IconVariant::Tamable {
                texture,
                tamed: *tamed,
            },
            VariantKind::IronGolem { crackiness } => IconVariant::IronGolem {
                texture,
                crackiness: crackiness.clone(),
            },
            VariantKind::Llama {
                trader,
                body_armor,
            } => IconVariant::Llama {
                texture,
                trader: *trader,
                body_armor: body_armor.clone().unwrap_or_else(|| "minecraft:air".into()),
            },
            VariantKind::Saddle { saddled } => IconVariant::Saddle {
                texture,
                saddled: *saddled,
            },
            VariantKind::TropicalFish {
                pattern,
                base_color,
                pattern_color,
            } => IconVariant::TropicalFish {
                texture,
                pattern: pattern.clone(),
                base_color: base_color.clone(),
                pattern_color: pattern_color.clone(),
            },
            VariantKind::Enderman { creepy } => IconVariant::Enderman {
                texture,
                creepy: *creepy,
            },
            VariantKind::Item { item } => IconVariant::texture(item.clone()),
            VariantKind::ItemFrame { item } => {
                IconVariant::texture(item.clone().unwrap_or_else(|| "minecraft:air".into()))
            }
            VariantKind::Cushion { color } => IconVariant::Cushion { color: color.clone() },
        }
    }
}

/// Which built-in variant rule applies to an entity.
#[derive(Debug, Clone, PartialEq)]
pub enum VariantKind {
    Other,
    Horse { markings: HorseMarkings },
    Villager {
        baby: bool,
        villager_type: String,
        profession: String,
        level: i32,
    },
    Tamable {
        tamed: bool,
    },
    IronGolem {
        crackiness: String,
    },
    Llama {
        trader: bool,
        body_armor: Option<String>,
    },
    Saddle {
        saddled: bool,
    },
    TropicalFish {
        pattern: String,
        base_color: String,
        pattern_color: String,
    },
    Enderman {
        creepy: bool,
    },
    Item {
        item: String,
    },
    ItemFrame {
        item: Option<String>,
    },
    Cushion {
        color: Option<String>,
    },
}

/// `xaero/.../definition/BuiltInRadarIconDefinitions`: stateless helper that
/// builds the variant id from supplied entity state.
#[derive(Debug, Clone, Copy, Default)]
pub struct BuiltinVariantResolver;

impl BuiltinVariantResolver {
    pub fn new() -> Self {
        BuiltinVariantResolver
    }

    pub fn resolve(&self, data: &EntityVariantData) -> IconVariant {
        data.variant()
    }
}

/// One equipped item and its armor trim, as used by `RadarIconArmorHandler`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EquippedItem {
    pub item: String,
    pub trim_material: Option<String>,
    pub trim_pattern: Option<String>,
}

impl EquippedItem {
    pub fn new(item: impl Into<String>) -> Self {
        EquippedItem {
            item: item.into(),
            trim_material: None,
            trim_pattern: None,
        }
    }

    pub fn trimmed(
        item: impl Into<String>,
        trim_material: impl Into<String>,
        trim_pattern: impl Into<String>,
    ) -> Self {
        EquippedItem {
            item: item.into(),
            trim_material: Some(trim_material.into()),
            trim_pattern: Some(trim_pattern.into()),
        }
    }
}

/// The equipment slots the Java handler looks at.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Equipment {
    pub head: Option<EquippedItem>,
    pub body: Option<EquippedItem>,
}

/// `xaero/.../cache/id/armor/RadarIconArmor`: part of the icon cache key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IconArmor {
    pub armor: String,
    pub trim_material: Option<String>,
    pub trim_pattern: Option<String>,
}

impl IconArmor {
    pub fn new(
        armor: impl Into<String>,
        trim_material: Option<String>,
        trim_pattern: Option<String>,
    ) -> Self {
        IconArmor {
            armor: armor.into(),
            trim_material,
            trim_pattern,
        }
    }

    pub fn is_trimmed(&self) -> bool {
        self.trim_material.is_some() && self.trim_pattern.is_some()
    }
}

impl fmt::Display for IconArmor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let material = self.trim_material.as_deref().unwrap_or("null");
        let pattern = self.trim_pattern.as_deref().unwrap_or("null");
        write!(
            f,
            "RadarIconArmor{{{}, {}, {}}}",
            self.armor, material, pattern
        )
    }
}

/// `xaero/.../cache/id/armor/RadarIconArmorHandler`. Mods/Horizon provide the
/// item and trim ids in [`Equipment`]; the Java item tags/data components
/// check happens before that data is supplied.
#[derive(Debug, Clone, Copy, Default)]
pub struct ArmorHandler;

impl ArmorHandler {
    pub fn new() -> Self {
        ArmorHandler
    }

    /// Horses and nautili use the BODY slot, all other living entities HEAD.
    pub fn get_armor(&self, equipment: &Equipment, equine: bool) -> Option<IconArmor> {
        let stack = if equine {
            equipment.body.as_ref()
        } else {
            equipment.head.as_ref()
        }?;
        Some(IconArmor::new(
            stack.item.clone(),
            stack.trim_material.clone(),
            stack.trim_pattern.clone(),
        ))
    }
}

/// `xaero/.../cache/id/RadarIconKey`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IconKey {
    pub variant: IconVariant,
    pub armor: Option<IconArmor>,
}

impl IconKey {
    pub fn new(variant: IconVariant, armor: Option<IconArmor>) -> Self {
        IconKey { variant, armor }
    }

    /// The variant string used to look the form up in an `IconDefinition`.
    pub fn variant_string(&self) -> String {
        self.variant.as_id()
    }
}

impl fmt::Display for IconKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let armor = self
            .armor
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "null".into());
        write!(f, "RadarIconKey{{{}, {}}}", self.variant, armor)
    }
}

/// A cached icon, represented by its resource id (the Java type is a
/// `XaeroIcon` holding a renderable texture).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CachedIcon(String);

impl CachedIcon {
    pub fn new(id: impl Into<String>) -> Self {
        CachedIcon(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `xaero/.../cache/RadarIconEntityCache`: cached icons of one entity type.
#[derive(Debug, Clone)]
pub struct EntityIconCache {
    entity_type: String,
    storage: HashMap<IconKey, CachedIcon>,
    variant_string_cache: HashMap<IconKey, String>,
    invalid_variant_class: bool,
}

impl EntityIconCache {
    pub fn new(entity_type: &str) -> Self {
        EntityIconCache {
            entity_type: entity_type.to_string(),
            storage: HashMap::new(),
            variant_string_cache: HashMap::new(),
            invalid_variant_class: false,
        }
    }

    pub fn entity_type(&self) -> &str {
        &self.entity_type
    }

    /// `get(RadarIconKey)`.
    pub fn get(&self, key: &IconKey) -> Option<&CachedIcon> {
        self.storage.get(key)
    }

    /// `add(RadarIconKey, XaeroIcon)`: returns the previous icon, if any.
    pub fn add(&mut self, key: IconKey, icon: CachedIcon) -> Option<CachedIcon> {
        self.variant_string_cache
            .entry(key.clone())
            .or_insert_with(|| key.variant_string());
        self.storage.insert(key, icon)
    }

    /// `getVariantString(RadarIconKey)`: caches `variant.toString()` so a
    /// definition lookup does not repeat it for every frame.
    pub fn get_variant_string(&mut self, key: &IconKey) -> Option<String> {
        if let Some(value) = self.variant_string_cache.get(key) {
            return Some(value.clone());
        }
        let value = key.variant_string();
        if value.is_empty() {
            // Java rejects null/empty variant ids and logs an error.
            self.invalid_variant_class = true;
            return None;
        }
        self.variant_string_cache
            .insert(key.clone(), value.clone());
        Some(value)
    }

    /// `isInvalidVariantClass()` — set when a variant cannot be identified.
    pub fn is_invalid_variant_class(&self) -> bool {
        self.invalid_variant_class
    }

    pub fn clear(&mut self) {
        self.storage.clear();
        self.variant_string_cache.clear();
        self.invalid_variant_class = false;
    }
}

/// `xaero/.../cache/RadarIconCache`: entity type → its icon cache.
#[derive(Debug, Clone, Default)]
pub struct IconCache {
    caches: BTreeMap<String, EntityIconCache>,
}

impl IconCache {
    pub fn new() -> Self {
        IconCache::default()
    }

    /// `getEntityCache(EntityType)`, creating the per-type cache on first use.
    pub fn get_entity_cache(&mut self, entity_type: &str) -> &mut EntityIconCache {
        self.caches
            .entry(entity_type.to_string())
            .or_insert_with(|| EntityIconCache::new(entity_type))
    }

    pub fn peek_entity_cache(&self, entity_type: &str) -> Option<&EntityIconCache> {
        self.caches.get(entity_type)
    }

    /// `clear()`.
    pub fn clear(&mut self) {
        self.caches.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.caches.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_ids_match_definition_keys() {
        let texture = "minecraft:textures/entity/pig/pig.png";
        assert_eq!(
            EntityVariantData::new(texture, VariantKind::Saddle { saddled: true })
                .variant()
                .as_id(),
            format!("{texture}%true")
        );
        assert_eq!(
            EntityVariantData::new(texture, VariantKind::Saddle { saddled: false })
                .variant()
                .as_id(),
            format!("{texture}%false")
        );
        let horse = EntityVariantData::new(
            "minecraft:textures/entity/horse/horse.png",
            VariantKind::Horse {
                markings: HorseMarkings::WhiteDots,
            },
        );
        assert_eq!(
            horse.variant().as_id(),
            "minecraft:textures/entity/horse/horse.png%minecraft:textures/entity/horse/horse_markings_whitedots.png"
        );
        assert!(horse.variant().to_string().contains("%minecraft:textures/"));
        assert_eq!(
            EntityVariantData::new(
                "minecraft:textures/entity/horse/horse.png",
                VariantKind::Horse {
                    markings: HorseMarkings::None
                }
            )
            .variant()
            .as_id(),
            "minecraft:textures/entity/horse/horse.png%null"
        );
    }

    #[test]
    fn builtin_variants_for_villager_llama_and_others() {
        let villager = EntityVariantData::new(
            "minecraft:textures/entity/villager/villager.png",
            VariantKind::Villager {
                baby: true,
                villager_type: "minecraft:desert".into(),
                profession: "minecraft:libarian".into(),
                level: 2,
            },
        );
        assert_eq!(
            BuiltinVariantResolver::new().resolve(&villager).as_id(),
            "minecraft:textures/entity/villager/villager.png%true%minecraft:desert%minecraft:libarian%2"
        );

        let llama = EntityVariantData::new(
            "minecraft:textures/entity/llama/llama.png",
            VariantKind::Llama {
                trader: true,
                body_armor: Some("minecraft:diamond_horse_armor".into()),
            },
        );
        assert_eq!(
            llama.variant().as_id(),
            "minecraft:textures/entity/llama/llama.png%true%minecraft:diamond_horse_armor"
        );
        assert_eq!(
            EntityVariantData::new(
                "minecraft:textures/entity/llama/llama.png",
                VariantKind::Llama {
                    trader: false,
                    body_armor: None
                }
            )
            .variant()
            .as_id(),
            "minecraft:textures/entity/llama/llama.png%false%minecraft:air"
        );

        assert_eq!(
            EntityVariantData::new("ignored", VariantKind::Item { item: "minecraft:diamond".into() })
                .variant()
                .as_id(),
            "minecraft:diamond"
        );
        assert_eq!(
            EntityVariantData::new("ignored", VariantKind::Cushion { color: None })
                .variant()
                .as_id(),
            "default"
        );
    }

    #[test]
    fn armor_key_uses_body_slot_for_equines() {
        let equipment = Equipment {
            head: Some(EquippedItem::trimmed(
                "minecraft:iron_helmet",
                "minecraft:emerald",
                "minecraft:ward",
            )),
            body: Some(EquippedItem::new("minecraft:diamond_horse_armor")),
        };
        let handler = ArmorHandler::new();
        let equine = handler.get_armor(&equipment, true).unwrap();
        assert_eq!(equine.armor, "minecraft:diamond_horse_armor");
        assert!(!equine.is_trimmed());
        let human = handler.get_armor(&equipment, false).unwrap();
        assert!(human.is_trimmed());
        assert_eq!(human.to_string(), "RadarIconArmor{minecraft:iron_helmet, minecraft:emerald, minecraft:ward}");
    }

    #[test]
    fn icon_cache_stores_icons_and_variant_strings_per_entity() {
        let mut cache = IconCache::new();
        assert!(cache.is_empty());
        let key = IconKey::new(
            EntityVariantData::new("minecraft:textures/entity/pig/pig.png", VariantKind::Saddle { saddled: true })
                .variant(),
            None,
        );
        let entity = cache.get_entity_cache("minecraft:pig");
        assert!(entity.get(&key).is_none());
        assert_eq!(
            entity.get_variant_string(&key).as_deref(),
            Some("minecraft:textures/entity/pig/pig.png%true")
        );
        assert_eq!(
            entity.add(key.clone(), CachedIcon::new("model:default")),
            None
        );
        assert_eq!(
            entity.add(key.clone(), CachedIcon::new("model:1")),
            Some(CachedIcon::new("model:default"))
        );
        assert_eq!(entity.get(&key).unwrap().as_str(), "model:1");
        assert!(!entity.is_invalid_variant_class());

        cache.get_entity_cache("minecraft:wolf").add(
            IconKey::new(IconVariant::texture("wolf"), None),
            CachedIcon::new("dot"),
        );
        assert_eq!(cache.peek_entity_cache("minecraft:pig").unwrap().get(&key).unwrap().as_str(), "model:1");
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn empty_variant_is_rejected() {
        let mut entity = EntityIconCache::new("minecraft:pig");
        let key = IconKey::new(IconVariant::texture(""), None);
        assert_eq!(entity.get_variant_string(&key), None);
        assert!(entity.is_invalid_variant_class());
    }
}
