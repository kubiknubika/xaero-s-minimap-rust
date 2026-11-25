//! Entity radar category system (the data/config side of
//! `xaero/hud/minimap/radar/category` in the original mod).
//!
//! The Java mod builds a tree of `EntityRadarCategory` nodes. Each node has a
//! hard rule, include/exclude string lists and setting overrides; the deepest
//! node an entity matches supplies its radar settings. This module ports that
//! model on top of the generic [`crate::categories`] system:
//!
//! | Java class | Rust type |
//! | --- | --- |
//! | `.../category/EntityRadarCategory` | [`RadarCategory`] |
//! | `.../category/EntityRadarCategoryManager` | [`RadarCategoryManager`] |
//! | `.../category/setting/EntityRadarCategorySettings` | [`RadarSetting`] / [`all_settings`] |
//! | `.../category/rule/EntityRadarCategoryHardRules` | [`RadarHardRule`] |
//! | `.../category/rule/EntityRadarListRuleTypes` | [`RadarListRuleType`] |
//! | `.../category/serialization/data/EntityRadarCategoryData` | [`RadarCategoryData`] |
//!
//! The GUI editors and the config-channel plumbing are not ported.

use std::collections::BTreeMap;

use xaerominimap_json::{get as json_get, Json};

use crate::categories::{
    setting_value_from_json, setting_value_to_json, EditorSettingType, ExcludeListMode,
};
use crate::SettingValue;

/// `xaero/hud/minimap/radar/category/setting/EntityRadarCategorySettings`:
/// one setting of an entity radar category.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarSetting {
    pub id: &'static str,
    /// Translation key used by the Java GUI (`gui.xaero_...`).
    pub display_name: &'static str,
    pub default_value: SettingValue,
    pub editor_type: EditorSettingType,
    pub first_option: i32,
    pub last_option: i32,
    /// Names of the options, when the editor is an iteration button.
    pub options: &'static [&'static str],
}

impl RadarSetting {
    /// Converts the definition to the generic `ObjectCategorySetting`.
    pub fn to_object_setting(&self) -> crate::ObjectCategorySetting {
        crate::ObjectCategorySetting::builder()
            .set_id(self.id)
            .set_display_name(self.display_name)
            .set_default_value(self.default_value.clone())
            .set_setting_ui_type(self.editor_type)
            .set_ui_first_option(self.first_option)
            .set_ui_last_option(self.last_option)
            .build()
    }
}

const ICON_OPTIONS: &[&str] = &[
    "gui.xaero_icons_off",
    "gui.xaero_icons_list",
    "gui.xaero_icons_always",
    "-",
];
const NAME_OPTIONS: &[&str] = &[
    "gui.xaero_names_off",
    "gui.xaero_names_list",
    "gui.xaero_names_always",
    "-",
];
const OVER_MAP_OPTIONS: &[&str] = &[
    "gui.xaero_radar_over_map_never",
    "gui.xaero_radar_over_map_list",
    "gui.xaero_radar_over_map_always",
    "-",
];

/// All radar category settings, in the order of Java's `SETTINGS_LIST`.
pub fn all_settings() -> Vec<RadarSetting> {
    use EditorSettingType::{IterationButton as Button, Slider, Expanding};
    vec![
        RadarSetting {
            id: "displayed",
            display_name: "gui.xaero_radar_displayed",
            default_value: SettingValue::Bool(true),
            editor_type: Button,
            first_option: 0,
            last_option: 1,
            options: &[],
        },
        RadarSetting {
            id: "color",
            display_name: "gui.xaero_radar_dots_color",
            default_value: SettingValue::Int(13),
            editor_type: Expanding,
            first_option: -1,
            last_option: 15,
            options: &[],
        },
        RadarSetting {
            id: "icons",
            display_name: "gui.xaero_radar_icons_displayed",
            default_value: SettingValue::Int(1),
            editor_type: Button,
            first_option: 0,
            last_option: 2,
            options: ICON_OPTIONS,
        },
        RadarSetting {
            id: "names",
            display_name: "gui.xaero_radar_names_displayed",
            default_value: SettingValue::Int(0),
            editor_type: Button,
            first_option: 0,
            last_option: 2,
            options: NAME_OPTIONS,
        },
        RadarSetting {
            id: "dotSize",
            display_name: "gui.xaero_dots_size",
            default_value: SettingValue::Float(2.0),
            editor_type: Slider,
            first_option: 1,
            last_option: 4,
            options: &[],
        },
        RadarSetting {
            id: "iconScale",
            display_name: "gui.xaero_entity_heads_scale",
            default_value: SettingValue::Float(1.0),
            editor_type: Slider,
            first_option: 5,
            last_option: 40,
            options: &[],
        },
        RadarSetting {
            id: "heightLimit",
            display_name: "gui.xaero_height_limit",
            default_value: SettingValue::Float(20.0),
            editor_type: Slider,
            first_option: 2,
            last_option: 30,
            options: &[],
        },
        RadarSetting {
            id: "heightBasedFade",
            display_name: "gui.xaero_entity_depth",
            default_value: SettingValue::Bool(true),
            editor_type: Button,
            first_option: 0,
            last_option: 1,
            options: &[],
        },
        RadarSetting {
            id: "displayHeight",
            display_name: "gui.xaero_entity_display_height",
            default_value: SettingValue::Float(0.0),
            editor_type: Button,
            first_option: 0,
            last_option: 2,
            options: &[],
        },
        RadarSetting {
            id: "startFadingAt",
            display_name: "gui.xaero_start_fading_at",
            default_value: SettingValue::Float(0.0),
            editor_type: Slider,
            first_option: 0,
            last_option: 100,
            options: &[],
        },
        RadarSetting {
            id: "entityNumber",
            display_name: "gui.xaero_entity_amount",
            default_value: SettingValue::Float(1000.0),
            editor_type: Slider,
            first_option: 1,
            last_option: 1000,
            options: &[],
        },
        RadarSetting {
            id: "alwaysDisplayNametags",
            display_name: "gui.xaero_always_entity_nametags",
            default_value: SettingValue::Bool(false),
            editor_type: Button,
            first_option: 0,
            last_option: 1,
            options: &[],
        },
        RadarSetting {
            id: "displayNameWhenIconFails",
            display_name: "gui.xaero_entity_icon_name_fallback",
            default_value: SettingValue::Bool(true),
            editor_type: Button,
            first_option: 0,
            last_option: 1,
            options: &[],
        },
        RadarSetting {
            id: "renderOverMinimapFrame",
            display_name: "gui.xaero_radar_render_over_minimap",
            default_value: SettingValue::Int(1),
            editor_type: Button,
            first_option: 0,
            last_option: 2,
            options: OVER_MAP_OPTIONS,
        },
        RadarSetting {
            id: "renderOrder",
            display_name: "gui.xaero_radar_render_order",
            default_value: SettingValue::Int(0),
            editor_type: Slider,
            first_option: -10,
            last_option: 10,
            options: &[],
        },
    ]
}

/// `SETTINGS` as a map of id → default value.
pub fn default_settings() -> BTreeMap<String, SettingValue> {
    all_settings()
        .into_iter()
        .map(|s| (s.id.to_string(), s.default_value))
        .collect()
}

pub fn setting_by_id(id: &str) -> Option<RadarSetting> {
    all_settings().into_iter().find(|s| s.id == id)
}

/// `xaero/hud/minimap/radar/category/rule/EntityRadarCategoryHardRules`:
/// a named predicate an entity either satisfies or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadarHardRule {
    Nothing,
    Anything,
    Living,
    Player,
    SameTeam,
    Hostile,
    Tamed,
    Item,
    Friendly,
    OtherTeams,
    Baby,
    Vanilla,
    Modded,
    AboveGround,
    BelowGround,
    MyGround,
    NotMyGround,
    Lit,
    Unlit,
    HasCustomName,
    NoCustomName,
    Tracked,
    InTeam,
    Teamless,
}

impl RadarHardRule {
    pub const ALL: [RadarHardRule; 24] = [
        RadarHardRule::Nothing,
        RadarHardRule::Anything,
        RadarHardRule::Living,
        RadarHardRule::Player,
        RadarHardRule::SameTeam,
        RadarHardRule::Hostile,
        RadarHardRule::Tamed,
        RadarHardRule::Item,
        RadarHardRule::Friendly,
        RadarHardRule::OtherTeams,
        RadarHardRule::Baby,
        RadarHardRule::Vanilla,
        RadarHardRule::Modded,
        RadarHardRule::AboveGround,
        RadarHardRule::BelowGround,
        RadarHardRule::MyGround,
        RadarHardRule::NotMyGround,
        RadarHardRule::Lit,
        RadarHardRule::Unlit,
        RadarHardRule::HasCustomName,
        RadarHardRule::NoCustomName,
        RadarHardRule::Tracked,
        RadarHardRule::InTeam,
        RadarHardRule::Teamless,
    ];

    /// The serialized name (the Java rule's `getName()`).
    pub fn name(self) -> &'static str {
        match self {
            RadarHardRule::Nothing => "nothing",
            RadarHardRule::Anything => "anything",
            RadarHardRule::Living => "living",
            RadarHardRule::Player => "players",
            RadarHardRule::SameTeam => "same-team",
            RadarHardRule::Hostile => "hostile",
            RadarHardRule::Tamed => "tamed",
            RadarHardRule::Item => "items",
            RadarHardRule::Friendly => "friendly",
            RadarHardRule::OtherTeams => "other-teams",
            RadarHardRule::Baby => "baby",
            RadarHardRule::Vanilla => "vanilla",
            RadarHardRule::Modded => "modded",
            RadarHardRule::AboveGround => "above-ground",
            RadarHardRule::BelowGround => "below-ground",
            RadarHardRule::MyGround => "my-ground",
            RadarHardRule::NotMyGround => "not-my-ground",
            RadarHardRule::Lit => "block-lit",
            RadarHardRule::Unlit => "block-unlit",
            RadarHardRule::HasCustomName => "has-custom-name",
            RadarHardRule::NoCustomName => "no-custom-name",
            RadarHardRule::Tracked => "tracked",
            RadarHardRule::InTeam => "in-a-team",
            RadarHardRule::Teamless => "teamless",
        }
    }

    pub fn from_name(name: &str) -> Option<RadarHardRule> {
        RadarHardRule::ALL.into_iter().find(|r| r.name() == name)
    }

    /// `isFollowedBy(Entity, Player)` for the entity data the port receives.
    pub fn matches(self, entity: &RadarEntityInfo) -> bool {
        match self {
            RadarHardRule::Nothing => false,
            RadarHardRule::Anything => true,
            RadarHardRule::Living => entity.is_living,
            RadarHardRule::Player => entity.is_player,
            RadarHardRule::SameTeam => entity.is_player && entity.same_team,
            RadarHardRule::Hostile => entity.is_hostile,
            RadarHardRule::Tamed => entity.is_tamed,
            RadarHardRule::Item => entity.is_item,
            RadarHardRule::Friendly => entity.is_living && !entity.is_hostile && !entity.is_player,
            RadarHardRule::OtherTeams => entity.is_player && entity.in_team && !entity.same_team,
            RadarHardRule::Baby => entity.is_baby,
            RadarHardRule::Vanilla => entity.is_vanilla(),
            RadarHardRule::Modded => !entity.is_vanilla(),
            RadarHardRule::AboveGround => !entity.below_ground,
            RadarHardRule::BelowGround => entity.below_ground,
            RadarHardRule::MyGround => entity.my_ground,
            RadarHardRule::NotMyGround => !entity.my_ground,
            RadarHardRule::Lit => entity.is_lit,
            RadarHardRule::Unlit => !entity.is_lit,
            RadarHardRule::HasCustomName => !entity.custom_name.is_empty(),
            RadarHardRule::NoCustomName => entity.custom_name.is_empty(),
            RadarHardRule::Tracked => entity.is_tracked,
            RadarHardRule::InTeam => entity.in_team,
            RadarHardRule::Teamless => !entity.in_team,
        }
    }
}

/// Ids of `EntityRadarListRuleTypes` (`TYPE_MAP`).
pub mod list_rule_types {
    pub const ENTITY: &str = "entity";
    pub const ITEM: &str = "item";
    pub const PLAYER: &str = "player";
    pub const CUSTOM_NAME: &str = "custom-name";
    pub const LIVING: &str = "living";
    pub const HOSTILE: &str = "hostile";
    pub const TAMED: &str = "tamed";
    pub const SAME_TEAM: &str = "same-team";
    pub const BABY: &str = "baby";
    pub const VANILLA: &str = "vanilla";
    pub const ABOVE_GROUND: &str = "above-ground";
    pub const MY_GROUND: &str = "my-ground";
    pub const LIT: &str = "lit";
    pub const IN_TEAM: &str = "in-team";
    pub const TRACKED: &str = "tracked";

    pub const ALL: [&str; 15] = [
        ENTITY,
        ITEM,
        PLAYER,
        CUSTOM_NAME,
        LIVING,
        HOSTILE,
        TAMED,
        SAME_TEAM,
        BABY,
        VANILLA,
        ABOVE_GROUND,
        MY_GROUND,
        LIT,
        IN_TEAM,
        TRACKED,
    ];
}

/// `xaero/hud/category/rule/ObjectCategoryListRuleType` for radar categories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadarListRuleType {
    pub id: String,
}

impl RadarListRuleType {
    pub fn new(id: impl Into<String>) -> Self {
        RadarListRuleType { id: id.into() }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        list_rule_types::ALL
            .contains(&id)
            .then(|| RadarListRuleType::new(id))
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }
}

/// The entity state the category rules evaluate (the Java mod passes the
/// live `Entity` and `Player`; the port takes the values from game glue).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RadarEntityInfo {
    pub entity_type: String,
    pub player_name: String,
    pub custom_name: String,
    pub item_id: String,
    pub is_living: bool,
    pub is_player: bool,
    pub same_team: bool,
    pub is_hostile: bool,
    pub is_tamed: bool,
    pub is_item: bool,
    pub is_baby: bool,
    pub below_ground: bool,
    pub my_ground: bool,
    pub is_lit: bool,
    pub is_tracked: bool,
    pub in_team: bool,
}

impl RadarEntityInfo {
    pub fn new(entity_type: impl Into<String>) -> Self {
        RadarEntityInfo {
            entity_type: entity_type.into(),
            ..Default::default()
        }
    }

    pub fn is_vanilla(&self) -> bool {
        self.entity_type.starts_with("minecraft:")
    }

    /// Values produced by a list rule type for this entity. Hard-rule based
    /// list types contribute the matching rule name, as in the Java mod.
    pub fn list_values(&self, type_id: &str) -> Vec<String> {
        match type_id {
            list_rule_types::ENTITY => vec![self.entity_type.clone()],
            list_rule_types::ITEM if self.is_item => vec![self.item_id.clone()],
            list_rule_types::PLAYER if self.is_player => vec![self.player_name.clone()],
            list_rule_types::CUSTOM_NAME if !self.custom_name.is_empty() => {
                vec![self.custom_name.clone()]
            }
            other => RadarHardRule::ALL
                .into_iter()
                .find(|rule| matches!(rule.name(), "living" | "hostile" | "tamed" | "same-team" | "baby" | "vanilla" | "above-ground" | "my-ground" | "block-lit" | "in-a-team" | "tracked") && rule.name() == other)
                .filter(|rule| rule.matches(self))
                .map(|rule| vec![rule.name().to_string()])
                .unwrap_or_default(),
        }
    }
}

/// `xaero/hud/minimap/radar/category/EntityRadarCategory`: one node of the
/// category tree.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarCategory {
    pub name: String,
    pub setting_overrides: BTreeMap<String, SettingValue>,
    pub sub_categories: Vec<RadarCategory>,
    pub base_rule: Option<RadarHardRule>,
    pub include_lists: BTreeMap<String, Vec<String>>,
    pub exclude_lists: BTreeMap<String, Vec<String>>,
    pub exclude_mode: ExcludeListMode,
    pub include_in_super_category: bool,
    pub protection: bool,
}

impl RadarCategory {
    pub fn new(name: impl Into<String>) -> Self {
        RadarCategory {
            name: name.into(),
            setting_overrides: BTreeMap::new(),
            sub_categories: Vec::new(),
            base_rule: None,
            include_lists: BTreeMap::new(),
            exclude_lists: BTreeMap::new(),
            exclude_mode: ExcludeListMode::AllBut,
            include_in_super_category: true,
            protection: false,
        }
    }

    pub fn with_base_rule(mut self, rule: RadarHardRule) -> Self {
        self.base_rule = Some(rule);
        self
    }

    pub fn with_setting(mut self, id: &str, value: SettingValue) -> Self {
        self.setting_overrides.insert(id.to_string(), value);
        self
    }

    pub fn with_sub_category(mut self, category: RadarCategory) -> Self {
        self.sub_categories.push(category);
        self
    }

    pub fn with_include_list(mut self, type_id: &str, values: &[&str]) -> Self {
        self.include_lists.insert(
            type_id.to_string(),
            values.iter().map(|s| (*s).to_string()).collect(),
        );
        self
    }

    pub fn with_exclude_list(mut self, type_id: &str, values: &[&str]) -> Self {
        self.exclude_lists.insert(
            type_id.to_string(),
            values.iter().map(|s| (*s).to_string()).collect(),
        );
        self
    }

    pub fn with_protection(mut self, protection: bool) -> Self {
        self.protection = protection;
        self
    }

    /// `isFollowedBy(Entity, Player)` of the whole filter.
    pub fn matches(&self, entity: &RadarEntityInfo) -> bool {
        if let Some(rule) = self.base_rule {
            if !rule.matches(entity) {
                return false;
            }
        }
        for (type_id, values) in &self.include_lists {
            if values.is_empty() {
                continue;
            }
            let actual = entity.list_values(type_id);
            if !values.iter().any(|value| actual.contains(value)) {
                return false;
            }
        }
        for (type_id, values) in &self.exclude_lists {
            if values.is_empty() {
                continue;
            }
            let actual = entity.list_values(type_id);
            if values.iter().any(|value| actual.contains(value)) {
                return false;
            }
        }
        true
    }

    /// Setting values with the defaults applied, without parent overrides.
    pub fn own_settings(&self) -> BTreeMap<String, SettingValue> {
        let mut settings = default_settings();
        self.apply_overrides(&mut settings);
        settings
    }

    fn apply_overrides(&self, settings: &mut BTreeMap<String, SettingValue>) {
        for (id, value) in &self.setting_overrides {
            settings.insert(id.clone(), value.clone());
        }
    }
}

/// `xaero/hud/minimap/radar/category/serialization/data/EntityRadarCategoryData`:
/// the Gson representation of a category tree.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarCategoryData {
    pub name: String,
    pub hard_include: Option<String>,
    pub include_list: BTreeMap<String, Vec<String>>,
    pub exclude_list: BTreeMap<String, Vec<String>>,
    pub exclude_mode: ExcludeListMode,
    pub setting_overrides: BTreeMap<String, SettingValue>,
    pub sub_categories: Vec<RadarCategoryData>,
    pub protection: bool,
    pub include_list_in_super_category: bool,
}

impl RadarCategoryData {
    pub fn from_category(category: &RadarCategory) -> Self {
        RadarCategoryData {
            name: category.name.clone(),
            hard_include: category.base_rule.map(|rule| rule.name().to_string()),
            include_list: category.include_lists.clone(),
            exclude_list: category.exclude_lists.clone(),
            exclude_mode: category.exclude_mode,
            setting_overrides: category.setting_overrides.clone(),
            sub_categories: category
                .sub_categories
                .iter()
                .map(RadarCategoryData::from_category)
                .collect(),
            protection: category.protection,
            include_list_in_super_category: category.include_in_super_category,
        }
    }

    pub fn into_category(self) -> Result<RadarCategory, String> {
        let mut category = RadarCategory::new(self.name);
        category.base_rule = match self.hard_include {
            None => None,
            Some(name) => Some(
                RadarHardRule::from_name(&name)
                    .ok_or_else(|| format!("unknown hard rule `{name}`"))?,
            ),
        };
        category.include_lists = self.include_list;
        category.exclude_lists = self.exclude_list;
        category.exclude_mode = self.exclude_mode;
        category.setting_overrides = self.setting_overrides;
        category.sub_categories = self
            .sub_categories
            .into_iter()
            .map(RadarCategoryData::into_category)
            .collect::<Result<Vec<_>, _>>()?;
        category.protection = self.protection;
        category.include_in_super_category = self.include_list_in_super_category;
        Ok(category)
    }

    pub fn to_json(&self) -> String {
        xaerominimap_json::to_string(&self.to_json_value())
    }

    fn to_json_value(&self) -> Json {
        Json::Obj(vec![
            ("name".into(), Json::Str(self.name.clone())),
            (
                "hardInclude".into(),
                self.hard_include
                    .as_ref()
                    .map(|s| Json::Str(s.clone()))
                    .unwrap_or(Json::Null),
            ),
            ("includeList".into(), lists_to_json(&self.include_list)),
            ("excludeList".into(), lists_to_json(&self.exclude_list)),
            (
                "excludeMode".into(),
                Json::Str(exclude_mode_text(self.exclude_mode).to_string()),
            ),
            (
                "settingOverrides".into(),
                Json::Obj(
                    self.setting_overrides
                        .iter()
                        .map(|(k, v)| (k.clone(), setting_value_to_json(v)))
                        .collect(),
                ),
            ),
            (
                "subCategories".into(),
                Json::Obj(vec![(
                    "values".into(),
                    Json::Arr(
                        self.sub_categories
                            .iter()
                            .map(RadarCategoryData::to_json_value)
                            .collect(),
                    ),
                )]),
            ),
            ("protection".into(), Json::Bool(self.protection)),
            (
                "includeListInSuperCategory".into(),
                Json::Bool(self.include_list_in_super_category),
            ),
        ])
    }

    pub fn from_json(text: &str) -> Result<RadarCategoryData, String> {
        let root = xaerominimap_json::parse(text)?;
        let name = match json_get(&root, "name") {
            Some(Json::Str(s)) => s.clone(),
            _ => return Err("`name` must be a string".into()),
        };
        let hard_include = match json_get(&root, "hardInclude") {
            Some(Json::Str(s)) => Some(s.clone()),
            None | Some(Json::Null) => None,
            _ => return Err("`hardInclude` must be a string".into()),
        };
        let sub_categories = match json_get(&root, "subCategories") {
            None | Some(Json::Null) => Vec::new(),
            Some(Json::Arr(items)) => items
                .iter()
                .map(|v| match v {
                    Json::Obj(pairs) if pairs.len() == 1 && pairs[0].0 == "values" => {
                        RadarCategoryData::from_json(&xaerominimap_json::to_string(&pairs[0].1))
                    }
                    _ => RadarCategoryData::from_json(&xaerominimap_json::to_string(v)),
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(Json::Obj(pairs)) => match pairs.iter().find(|(k, _)| k == "values") {
                Some((_, Json::Arr(items))) => items
                    .iter()
                    .map(|v| RadarCategoryData::from_json(&xaerominimap_json::to_string(v)))
                    .collect::<Result<Vec<_>, _>>()?,
                _ => Vec::new(),
            },
            _ => return Err("`subCategories` must be a list".into()),
        };
        Ok(RadarCategoryData {
            name,
            hard_include,
            include_list: lists_from_json(json_get(&root, "includeList"))?,
            exclude_list: lists_from_json(json_get(&root, "excludeList"))?,
            exclude_mode: match json_get(&root, "excludeMode") {
                None | Some(Json::Null) => ExcludeListMode::AllBut,
                Some(Json::Str(s)) => parse_exclude_mode(s)?,
                _ => return Err("`excludeMode` must be a string".into()),
            },
            setting_overrides: match json_get(&root, "settingOverrides") {
                None | Some(Json::Null) => BTreeMap::new(),
                Some(Json::Obj(pairs)) => pairs
                    .iter()
                    .filter_map(|(k, v)| {
                        // JSON numbers do not carry an int/float marker; use
                        // the setting's declared default to restore the type.
                        let value = match (setting_by_id(k), v) {
                            (Some(def), Json::Num(n))
                                if matches!(def.default_value, SettingValue::Float(_)) =>
                            {
                                Some(SettingValue::Float(*n as f32))
                            }
                            _ => setting_value_from_json(v),
                        };
                        value.map(|value| (k.clone(), value))
                    })
                    .collect(),
                _ => return Err("`settingOverrides` must be an object".into()),
            },
            sub_categories,
            protection: match json_get(&root, "protection") {
                Some(Json::Bool(b)) => *b,
                _ => false,
            },
            include_list_in_super_category: match json_get(&root, "includeListInSuperCategory") {
                None | Some(Json::Null) => true,
                Some(Json::Bool(b)) => *b,
                _ => return Err("`includeListInSuperCategory` must be a boolean".into()),
            },
        })
    }
}

fn exclude_mode_text(mode: ExcludeListMode) -> &'static str {
    match mode {
        ExcludeListMode::Only => "ONLY",
        ExcludeListMode::AllBut => "ALL_BUT",
    }
}

fn parse_exclude_mode(text: &str) -> Result<ExcludeListMode, String> {
    match text {
        "ONLY" => Ok(ExcludeListMode::Only),
        "ALL_BUT" => Ok(ExcludeListMode::AllBut),
        _ => Err(format!("unknown exclude mode `{text}`")),
    }
}

fn lists_to_json(lists: &BTreeMap<String, Vec<String>>) -> Json {
    Json::Arr(
        lists
            .iter()
            .map(|(type_id, values)| {
                Json::Obj(vec![
                    ("type".into(), Json::Str(type_id.clone())),
                    (
                        "values".into(),
                        Json::Arr(values.iter().cloned().map(Json::Str).collect()),
                    ),
                ])
            })
            .collect(),
    )
}

fn lists_from_json(value: Option<&Json>) -> Result<BTreeMap<String, Vec<String>>, String> {
    let items = match value {
        None | Some(Json::Null) => return Ok(BTreeMap::new()),
        Some(Json::Arr(items)) => items,
        Some(_) => return Err("list rules must be a list".into()),
    };
    let mut result = BTreeMap::new();
    for item in items {
        let type_id = match json_get(item, "type") {
            Some(Json::Str(s)) => s.clone(),
            _ => return Err("list rule `type` must be a string".into()),
        };
        let values = match json_get(item, "values") {
            Some(Json::Arr(items)) => items
                .iter()
                .map(|v| match v {
                    Json::Str(s) => Ok(s.clone()),
                    _ => Err("list rule values must be strings".to_string()),
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => Vec::new(),
        };
        result.insert(type_id, values);
    }
    Ok(result)
}

/// `xaero/hud/minimap/radar/category/EntityRadarCategoryManager` (data side):
/// the root category tree and the rules used to select a node for an entity.
#[derive(Debug, Clone)]
pub struct RadarCategoryManager {
    root: RadarCategory,
}

impl RadarCategoryManager {
    pub fn new(root: RadarCategory) -> Self {
        RadarCategoryManager { root }
    }

    /// The tree from `EntityRadarDefaultCategories.setupDefault`.
    pub fn with_defaults() -> Self {
        let root = RadarCategory::new("gui.xaero_entity_category_root")
            .with_base_rule(RadarHardRule::Anything)
            .with_protection(true)
            .with_exclude_list(list_rule_types::ENTITY, &[
                "minecraft:item_frame",
                "minecraft:glow_item_frame",
                "minecraft:cushion",
            ])
            .with_setting("entityNumber", SettingValue::Float(1000.0))
            .with_setting("dotSize", SettingValue::Float(2.0))
            .with_setting("iconScale", SettingValue::Float(1.0))
            .with_setting("heightBasedFade", SettingValue::Bool(true))
            .with_setting("heightLimit", SettingValue::Float(20.0))
            .with_setting("alwaysDisplayNametags", SettingValue::Bool(false))
            .with_setting("displayNameWhenIconFails", SettingValue::Bool(true))
            .with_sub_category(
                RadarCategory::new("gui.xaero_entity_category_living")
                    .with_base_rule(RadarHardRule::Living)
                    .with_sub_category(
                        RadarCategory::new("gui.xaero_entity_category_hostile")
                            .with_base_rule(RadarHardRule::Hostile)
                            .with_setting("displayed", SettingValue::Bool(true))
                            .with_setting("color", SettingValue::Int(13))
                            .with_setting("icons", SettingValue::Int(1))
                            .with_setting("names", SettingValue::Int(0)),
                    )
                    .with_sub_category(
                        RadarCategory::new("gui.xaero_entity_category_friendly")
                            .with_base_rule(RadarHardRule::Friendly),
                    )
                    .with_sub_category(
                        RadarCategory::new("gui.xaero_entity_category_players")
                            .with_base_rule(RadarHardRule::Player)
                            .with_sub_category(
                                RadarCategory::new("gui.xaero_entity_category_friend")
                                    .with_base_rule(RadarHardRule::Nothing),
                            )
                            .with_sub_category(
                                RadarCategory::new("gui.xaero_entity_category_tracked")
                                    .with_base_rule(RadarHardRule::Tracked),
                            )
                            .with_sub_category(
                                RadarCategory::new("gui.xaero_entity_category_same_team")
                                    .with_base_rule(RadarHardRule::SameTeam),
                            )
                            .with_sub_category(
                                RadarCategory::new("gui.xaero_entity_category_other_teams")
                                    .with_base_rule(RadarHardRule::OtherTeams),
                            ),
                    )
                    .with_sub_category(
                        RadarCategory::new("gui.xaero_entity_category_hostile_tamed")
                            .with_base_rule(RadarHardRule::Tamed),
                    )
                    .with_sub_category(
                        RadarCategory::new("gui.xaero_entity_category_friendly_tamed")
                            .with_base_rule(RadarHardRule::Tamed),
                    ),
            )
            .with_sub_category(
                RadarCategory::new("gui.xaero_entity_category_items")
                    .with_base_rule(RadarHardRule::Item),
            )
            .with_sub_category(
                RadarCategory::new("gui.xaero_entity_category_other_entities")
                    .with_base_rule(RadarHardRule::Anything),
            );
        RadarCategoryManager::new(root)
    }

    pub fn root(&self) -> &RadarCategory {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut RadarCategory {
        &mut self.root
    }

    /// Loads a tree from the JSON written by the category editor.
    pub fn from_json(text: &str) -> Result<RadarCategoryManager, String> {
        Ok(RadarCategoryManager::new(
            RadarCategoryData::from_json(text)?.into_category()?,
        ))
    }

    pub fn to_json(&self) -> String {
        RadarCategoryData::from_category(&self.root).to_json()
    }

    /// Depth-first path to the deepest category matching `entity`.
    fn matching_path<'a>(&'a self, entity: &RadarEntityInfo) -> Vec<&'a RadarCategory> {
        let mut path = Vec::new();
        self.collect_path(&self.root, entity, &mut path);
        path
    }

    fn collect_path<'a>(
        &'a self,
        category: &'a RadarCategory,
        entity: &RadarEntityInfo,
        path: &mut Vec<&'a RadarCategory>,
    ) {
        if !category.matches(entity) {
            return;
        }
        path.push(category);
        for child in &category.sub_categories {
            if child.matches(entity) {
                self.collect_path(child, entity, path);
                break;
            }
        }
    }

    /// The deepest matching category (the root if nothing else matches).
    pub fn category_for(&self, entity: &RadarEntityInfo) -> &RadarCategory {
        self.matching_path(entity)
            .into_iter()
            .last()
            .unwrap_or(&self.root)
    }

    /// Default settings overridden along the matching path, parent first.
    pub fn settings_for(&self, entity: &RadarEntityInfo) -> BTreeMap<String, SettingValue> {
        let mut settings = default_settings();
        for category in self.matching_path(entity) {
            category.apply_overrides(&mut settings);
        }
        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_match_java_defaults() {
        let settings = all_settings();
        assert_eq!(settings.len(), 15);
        assert_eq!(settings[0].id, "displayed");
        assert_eq!(settings[0].default_value, SettingValue::Bool(true));
        assert_eq!(settings[1].default_value, SettingValue::Int(13));
        assert_eq!(settings[4].default_value, SettingValue::Float(2.0));
        assert_eq!(settings[10].default_value, SettingValue::Float(1000.0));
        assert_eq!(setting_by_id("renderOrder").unwrap().default_value, SettingValue::Int(0));
        assert_eq!(default_settings().len(), 15);
        assert_eq!(
            settings[2].to_object_setting().get_default_value(),
            &SettingValue::Int(1)
        );
    }

    #[test]
    fn hard_rules_and_list_rules() {
        for rule in RadarHardRule::ALL {
            assert_eq!(RadarHardRule::from_name(rule.name()), Some(rule));
        }
        let mut entity = RadarEntityInfo::new("minecraft:zombie");
        entity.is_living = true;
        entity.is_hostile = true;
        assert!(RadarHardRule::Hostile.matches(&entity));
        assert!(!RadarHardRule::Friendly.matches(&entity));
        assert!(RadarHardRule::Vanilla.matches(&entity));
        assert!(entity.list_values(list_rule_types::ENTITY).contains(&"minecraft:zombie".to_string()));
        assert!(entity.list_values(list_rule_types::HOSTILE).contains(&"hostile".to_string()));
        assert!(entity.list_values(list_rule_types::PLAYER).is_empty());

        entity.entity_type = "examplemod:robot".into();
        assert!(RadarHardRule::Modded.matches(&entity));
    }

    #[test]
    fn category_filters_entities() {
        let category = RadarCategory::new("mobs")
            .with_base_rule(RadarHardRule::Living)
            .with_include_list(list_rule_types::ENTITY, &["minecraft:zombie", "minecraft:creeper"])
            .with_exclude_list(list_rule_types::ENTITY, &["minecraft:creeper"]);
        let mut zombie = RadarEntityInfo::new("minecraft:zombie");
        zombie.is_living = true;
        assert!(category.matches(&zombie));
        zombie.entity_type = "minecraft:creeper".into();
        assert!(!category.matches(&zombie));
        zombie.entity_type = "minecraft:pig".into();
        assert!(!category.matches(&zombie));
        zombie.is_living = false;
        zombie.entity_type = "minecraft:zombie".into();
        assert!(!category.matches(&zombie));
    }

    #[test]
    fn default_manager_selects_deepest_category() {
        let manager = RadarCategoryManager::with_defaults();
        let mut player = RadarEntityInfo::new("minecraft:player");
        player.is_living = true;
        player.is_player = true;
        player.in_team = true;
        player.same_team = true;
        player.player_name = "Alex".into();
        let category = manager.category_for(&player);
        assert_eq!(category.name, "gui.xaero_entity_category_same_team");
        let settings = manager.settings_for(&player);
        assert_eq!(settings.get("color"), Some(&SettingValue::Int(13)));

        let mut zombie = RadarEntityInfo::new("minecraft:zombie");
        zombie.is_living = true;
        zombie.is_hostile = true;
        assert_eq!(manager.category_for(&zombie).name, "gui.xaero_entity_category_hostile");

        let mut item = RadarEntityInfo::new("minecraft:item");
        item.is_item = true;
        assert_eq!(manager.category_for(&item).name, "gui.xaero_entity_category_items");
        assert_eq!(manager.settings_for(&item).get("displayed"), Some(&SettingValue::Bool(true)));
    }

    #[test]
    fn category_json_round_trip() {
        let manager = RadarCategoryManager::with_defaults();
        let text = manager.to_json();
        assert!(text.contains("gui.xaero_entity_category_root"));
        let loaded = RadarCategoryManager::from_json(&text).unwrap();
        assert_eq!(loaded.root(), manager.root());
        assert_eq!(loaded.to_json(), text);

        let bad = RadarCategoryManager::from_json(r#"{"name":"x","hardInclude":"nope"}"#);
        assert!(bad.is_err());
    }
}
