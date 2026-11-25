//! Port of the `xaero/hud/category` settings system:
//!
//! - `xaero/hud/category/ObjectCategory` → [`ObjectCategory`]
//! - `xaero/hud/category/setting/ObjectCategorySetting` → [`ObjectCategorySetting`]
//! - `xaero/hud/category/rule/*` → [`ObjectCategoryRule`], [`ObjectCategoryHardRule`],
//!   [`ObjectCategoryListRule`], [`ExcludeListMode`], [`FilterObjectCategory`]
//! - `xaero/hud/category/rule/resolver/ObjectCategoryRuleResolver` → [`ObjectCategoryRuleResolver`]
//! - `xaero/hud/category/serialization/data/ObjectCategoryData*` →
//!   [`ObjectCategoryData`] + [`json`] (the Java Gson/NBT serializers become
//!   JSON serialization here)
//!
//! The GUI-facing parts of the category system (`hud/category/ui`,
//! `hud/category/rule/resolver` helpers that only feed the editor) are out of
//! scope for the data model; see the README.

use std::collections::BTreeMap;

use crate::SettingValue;

/// `xaero/hud/category/ui/setting/EditorSettingType`: the kind of editor the
/// mod's category UI uses for a setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorSettingType {
    /// `ITERATION_BUTTON`.
    IterationButton,
    /// `SLIDER`.
    Slider,
    /// `EXPANDING`.
    Expanding,
}

impl EditorSettingType {
    /// Display name of the enum constant (`getName`).
    pub fn name(self) -> &'static str {
        match self {
            EditorSettingType::IterationButton => "iteration_button",
            EditorSettingType::Slider => "slider",
            EditorSettingType::Expanding => "expanding",
        }
    }

    /// Parses an editor type name.
    pub fn from_name(name: &str) -> Option<EditorSettingType> {
        match name {
            "iteration_button" => Some(EditorSettingType::IterationButton),
            "slider" => Some(EditorSettingType::Slider),
            "expanding" => Some(EditorSettingType::Expanding),
            _ => None,
        }
    }
}

/// `xaero/hud/category/setting/ObjectCategorySetting`: a setting that lives
/// in a category tree, with a display name, default value and editor type.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectCategorySetting {
    id: String,
    display_name: String,
    default_value: SettingValue,
    setting_ui_type: EditorSettingType,
    ui_first_option: i32,
    ui_last_option: i32,
}

impl ObjectCategorySetting {
    /// `getId()`.
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// `getDisplayName()`.
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    /// `getDefaultValue()`.
    pub fn get_default_value(&self) -> &SettingValue {
        &self.default_value
    }

    /// `getSettingUIType()`.
    pub fn get_setting_ui_type(&self) -> EditorSettingType {
        self.setting_ui_type
    }

    /// `getUiFirstOption()`.
    pub fn get_ui_first_option(&self) -> i32 {
        self.ui_first_option
    }

    /// `getUiLastOption()`.
    pub fn get_ui_last_option(&self) -> i32 {
        self.ui_last_option
    }

    /// Builder: `ObjectCategorySetting.builder()`.
    pub fn builder() -> ObjectCategorySettingBuilder {
        ObjectCategorySettingBuilder::new()
    }
}

/// `ObjectCategorySetting$Builder`.
#[derive(Debug, Clone, Default)]
pub struct ObjectCategorySettingBuilder {
    id: String,
    display_name: String,
    default_value: SettingValue,
    setting_ui_type: Option<EditorSettingType>,
    ui_first_option: i32,
    ui_last_option: i32,
}

impl ObjectCategorySettingBuilder {
    pub fn new() -> Self {
        ObjectCategorySettingBuilder::default()
    }

    /// `setDefault()`.
    pub fn set_default() -> Self {
        ObjectCategorySettingBuilder::new().set_setting_ui_type(EditorSettingType::IterationButton)
    }

    /// `setId(String)`.
    pub fn set_id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// `setDisplayName(...)` (the Java type is an MC component; the port uses
    /// a plain string).
    pub fn set_display_name(mut self, name: &str) -> Self {
        self.display_name = name.to_string();
        self
    }

    /// `setDefaultValue(Object)`.
    pub fn set_default_value(mut self, value: SettingValue) -> Self {
        self.default_value = value;
        self
    }

    /// `setSettingUIType(...)`.
    pub fn set_setting_ui_type(mut self, t: EditorSettingType) -> Self {
        self.setting_ui_type = Some(t);
        self
    }

    /// `setUiFirstOption(I)`.
    pub fn set_ui_first_option(mut self, v: i32) -> Self {
        self.ui_first_option = v;
        self
    }

    /// `setUiLastOption(I)`.
    pub fn set_ui_last_option(mut self, v: i32) -> Self {
        self.ui_last_option = v;
        self
    }

    /// `build()`.
    pub fn build(self) -> ObjectCategorySetting {
        ObjectCategorySetting {
            id: self.id,
            display_name: self.display_name,
            default_value: self.default_value,
            setting_ui_type: self.setting_ui_type.unwrap_or(EditorSettingType::IterationButton),
            ui_first_option: self.ui_first_option,
            ui_last_option: self.ui_last_option,
        }
    }
}

/// `xaero/hud/category/ObjectCategory`: a named node in the settings tree,
/// with a super category, setting overrides and sub categories.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectCategory {
    name: String,
    super_category: Option<Box<ObjectCategory>>,
    setting_overrides: BTreeMap<String, SettingValue>,
    sub_categories: Vec<ObjectCategory>,
    protection: bool,
}

impl ObjectCategory {
    /// `getName()`.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// `getSuperCategory()`.
    pub fn get_super_category(&self) -> Option<&ObjectCategory> {
        self.super_category.as_deref()
    }

    /// `setSuperCategory(...)`.
    pub fn set_super_category(&mut self, category: Option<ObjectCategory>) {
        self.super_category = category.map(Box::new);
    }

    /// `getSettingValue(ObjectCategorySetting)`.
    pub fn get_setting_value(&self, id: &str) -> Option<&SettingValue> {
        self.setting_overrides.get(id)
    }

    /// `setSettingValue(...)`.
    pub fn set_setting_value(&mut self, id: &str, value: SettingValue) {
        self.setting_overrides.insert(id.to_string(), value);
    }

    /// `getSettingOverridesIterator`.
    pub fn get_setting_overrides(&self) -> &BTreeMap<String, SettingValue> {
        &self.setting_overrides
    }

    /// `getDirectSubCategoryIterator`.
    pub fn get_sub_categories(&self) -> &[ObjectCategory] {
        &self.sub_categories
    }

    /// `getProtection()`.
    pub fn get_protection(&self) -> bool {
        self.protection
    }

    /// `ObjectCategory.builder()`.
    pub fn builder() -> ObjectCategoryBuilder {
        ObjectCategoryBuilder::new()
    }
}

/// `ObjectCategory$Builder` (chaining, by value like the Java builder).
#[derive(Debug, Clone, Default)]
pub struct ObjectCategoryBuilder {
    name: String,
    super_category: Option<Box<ObjectCategory>>,
    sub_categories: Vec<ObjectCategory>,
    protection: bool,
    setting_overrides: BTreeMap<String, SettingValue>,
}

impl ObjectCategoryBuilder {
    pub fn new() -> Self {
        ObjectCategoryBuilder::default()
    }

    /// `setDefault()`.
    pub fn set_default() -> Self {
        ObjectCategoryBuilder::new()
    }

    /// `setName(String)`.
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// `setSuperCategory(...)`.
    pub fn set_super_category(mut self, category: Option<ObjectCategory>) -> Self {
        self.super_category = category.map(Box::new);
        self
    }

    /// `addSubCategoryBuilder(ObjectCategory$Builder)` — adds a built
    /// sub category.
    pub fn add_sub_category(mut self, category: ObjectCategory) -> Self {
        self.sub_categories.push(category);
        self
    }

    /// `setSettingValue(...)`.
    pub fn set_setting_value(mut self, id: &str, value: SettingValue) -> Self {
        self.setting_overrides.insert(id.to_string(), value);
        self
    }

    /// `setProtection(Z)`.
    pub fn set_protection(mut self, protection: bool) -> Self {
        self.protection = protection;
        self
    }

    /// `build()`.
    pub fn build(self) -> ObjectCategory {
        ObjectCategory {
            name: self.name,
            super_category: self.super_category,
            setting_overrides: self.setting_overrides,
            sub_categories: self.sub_categories,
            protection: self.protection,
        }
    }
}

/// `xaero/hud/category/rule/ObjectCategoryRule`: a rule that decides whether
/// a (setting, value) pair is followed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectCategoryRule {
    name: String,
}

impl ObjectCategoryRule {
    pub fn new(name: &str) -> Self {
        ObjectCategoryRule {
            name: name.to_string(),
        }
    }

    /// `getName()`.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// `isFollowedBy(Object, Object)Z` — the base rule is followed.
    pub fn is_followed_by(&self, _setting: &ObjectCategorySetting, _value: &SettingValue) -> bool {
        true
    }
}

/// The predicate of an [`ObjectCategoryHardRule`]
/// (`ObjectCategoryHardRule$Predicate#test`).
pub type HardRulePredicate = Box<dyn Fn(&ObjectCategorySetting, &SettingValue) -> bool>;

/// `xaero/hud/category/rule/ObjectCategoryHardRule`: a rule whose predicate
/// either passes or is inverted with `reversed`.
pub struct ObjectCategoryHardRule {
    name: String,
    predicate: HardRulePredicate,
    reversed: bool,
}

impl ObjectCategoryHardRule {
    /// `getName()`.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// `isFollowedBy(Object, Object)Z`: the rule is followed when the
    /// predicate passes and not reversed.
    pub fn is_followed_by(&self, setting: &ObjectCategorySetting, value: &SettingValue) -> bool {
        let followed = (self.predicate)(setting, value);
        followed != self.reversed
    }

    /// Builder (`ObjectCategoryHardRule$Builder`).
    pub fn builder() -> HardRuleBuilder {
        HardRuleBuilder::default()
    }
}

#[derive(Default)]
pub struct HardRuleBuilder {
    name: String,
    predicate: Option<HardRulePredicate>,
    reversed: bool,
}

impl HardRuleBuilder {
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn set_predicate(mut self, predicate: HardRulePredicate) -> Self {
        self.predicate = Some(predicate);
        self
    }

    pub fn set_reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    pub fn build(self) -> ObjectCategoryHardRule {
        ObjectCategoryHardRule {
            name: self.name,
            predicate: self.predicate.unwrap_or_else(|| Box::new(|_, _| true)),
            reversed: self.reversed,
        }
    }
}

/// `xaero/hud/category/rule/ObjectCategoryListRuleType`: describes one
/// string-list rule (id + string handling).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObjectCategoryListRuleType {
    id: String,
}

impl ObjectCategoryListRuleType {
    pub fn new(id: &str) -> Self {
        ObjectCategoryListRuleType {
            id: id.to_string(),
        }
    }

    /// `getId()`.
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

/// `xaero/hud/category/rule/ExcludeListMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExcludeListMode {
    /// `ONLY`.
    Only,
    /// `ALL_BUT`.
    AllBut,
}

impl ExcludeListMode {
    /// Java `Enum.name()`.
    pub fn name(self) -> &'static str {
        match self {
            ExcludeListMode::Only => "ONLY",
            ExcludeListMode::AllBut => "ALL_BUT",
        }
    }

    /// Java `ExcludeListMode.valueOf(String)`.
    pub fn from_name(name: &str) -> Option<ExcludeListMode> {
        match name {
            "ONLY" => Some(ExcludeListMode::Only),
            "ALL_BUT" => Some(ExcludeListMode::AllBut),
            _ => None,
        }
    }
}

/// `xaero/hud/category/rule/ObjectCategoryListRule`: an include or exclude
/// list of strings for a setting.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObjectCategoryListRule {
    type_: ObjectCategoryListRuleType,
    strings: Vec<String>,
}

impl ObjectCategoryListRule {
    pub fn new(type_: ObjectCategoryListRuleType, strings: Vec<String>) -> Self {
        ObjectCategoryListRule { type_, strings }
    }

    /// `getType()`.
    pub fn get_type(&self) -> &ObjectCategoryListRuleType {
        &self.type_
    }

    /// The listed strings.
    pub fn get_strings(&self) -> &[String] {
        &self.strings
    }

    /// `inList(Object, Object)Z`: whether `value` is listed.
    pub fn in_list(&self, value: &str) -> bool {
        self.strings.iter().any(|s| s == value)
    }
}

/// `xaero/hud/category/FilterObjectCategory`: a filter combining a base rule
/// with include/exclude string lists.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterObjectCategory {
    base_rule: Option<ObjectCategoryRule>,
    include_lists: BTreeMap<String, ObjectCategoryListRule>,
    exclude_lists: BTreeMap<String, ObjectCategoryListRule>,
    exclude_mode: ExcludeListMode,
    include_in_super_category: bool,
}

impl Default for FilterObjectCategory {
    fn default() -> Self {
        FilterObjectCategory {
            base_rule: None,
            include_lists: BTreeMap::new(),
            exclude_lists: BTreeMap::new(),
            exclude_mode: ExcludeListMode::AllBut,
            include_in_super_category: true,
        }
    }
}

impl FilterObjectCategory {
    /// `getBaseRule()`.
    pub fn get_base_rule(&self) -> Option<&ObjectCategoryRule> {
        self.base_rule.as_ref()
    }

    /// `getIncludeList(ObjectCategoryListRuleType)`.
    pub fn get_include_list(&self, type_: &ObjectCategoryListRuleType) -> Option<&ObjectCategoryListRule> {
        self.include_lists.get(type_.get_id())
    }

    /// `getExcludeList(ObjectCategoryListRuleType)`.
    pub fn get_exclude_list(&self, type_: &ObjectCategoryListRuleType) -> Option<&ObjectCategoryListRule> {
        self.exclude_lists.get(type_.get_id())
    }

    /// `getExcludeMode()`.
    pub fn get_exclude_mode(&self) -> ExcludeListMode {
        self.exclude_mode
    }

    /// `getIncludeInSuperCategory()`.
    pub fn include_in_super_category(&self) -> bool {
        self.include_in_super_category
    }

    /// `followsRules(FilterObjectCategory, Object, Object)Z`: whether the
    /// (setting id, value) pair is followed by the include/exclude lists.
    pub fn follows_rules(&self, setting_id: &str, value: &str) -> bool {
        let included = match self.include_lists.get(setting_id) {
            Some(list) => list.in_list(value),
            None => true,
        };
        let excluded = match self.exclude_lists.get(setting_id) {
            Some(list) => list.in_list(value),
            None => false,
        };
        if excluded {
            return false;
        }
        included
    }

    /// Builder (`FilterObjectCategory$Builder`).
    pub fn builder() -> FilterObjectCategoryBuilder {
        FilterObjectCategoryBuilder::default()
    }
}

#[derive(Default)]
pub struct FilterObjectCategoryBuilder {
    base_rule: Option<ObjectCategoryRule>,
    include_lists: BTreeMap<String, ObjectCategoryListRule>,
    exclude_lists: BTreeMap<String, ObjectCategoryListRule>,
    exclude_mode: Option<ExcludeListMode>,
    include_in_super_category: bool,
}

impl FilterObjectCategoryBuilder {
    /// `setDefault()`.
    pub fn set_default() -> Self {
        FilterObjectCategoryBuilder::default()
    }

    pub fn set_base_rule(mut self, rule: ObjectCategoryRule) -> Self {
        self.base_rule = Some(rule);
        self
    }

    pub fn include_list(mut self, list: ObjectCategoryListRule) -> Self {
        self.include_lists.insert(list.get_type().get_id().to_string(), list);
        self
    }

    pub fn exclude_list(mut self, list: ObjectCategoryListRule) -> Self {
        self.exclude_lists.insert(list.get_type().get_id().to_string(), list);
        self
    }

    pub fn set_exclude_mode(mut self, mode: ExcludeListMode) -> Self {
        self.exclude_mode = Some(mode);
        self
    }

    pub fn set_include_in_super_category(mut self, value: bool) -> Self {
        self.include_in_super_category = value;
        self
    }

    pub fn build(self) -> FilterObjectCategory {
        FilterObjectCategory {
            base_rule: self.base_rule,
            include_lists: self.include_lists,
            exclude_lists: self.exclude_lists,
            exclude_mode: self.exclude_mode.unwrap_or(ExcludeListMode::AllBut),
            include_in_super_category: self.include_in_super_category,
        }
    }
}

/// `xaero/hud/category/rule/resolver/ObjectCategoryRuleResolver`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ObjectCategoryRuleResolver;

impl ObjectCategoryRuleResolver {
    pub fn new() -> Self {
        ObjectCategoryRuleResolver
    }

    /// `followsRules(FilterObjectCategory, Object, Object)Z`.
    pub fn follows_rules(&self, filter: &FilterObjectCategory, setting_id: &str, value: &str) -> bool {
        filter.follows_rules(setting_id, value)
    }
}

/// `xaero/hud/category/serialization/data/ObjectCategoryData`: the
/// serializable form of a category (name, setting overrides, sub categories,
/// protection).
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectCategoryData {
    name: String,
    setting_overrides: BTreeMap<String, SettingValue>,
    sub_categories: Vec<ObjectCategoryData>,
    protection: bool,
}

impl ObjectCategoryData {
    /// `ObjectCategoryData$Builder#setName`.
    pub fn new(
        name: &str,
        setting_overrides: BTreeMap<String, SettingValue>,
        sub_categories: Vec<ObjectCategoryData>,
        protection: bool,
    ) -> Self {
        ObjectCategoryData {
            name: name.to_string(),
            setting_overrides,
            sub_categories,
            protection,
        }
    }

    /// `getName()`.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// `getSettingOverrideIterator`.
    pub fn get_setting_overrides(&self) -> &BTreeMap<String, SettingValue> {
        &self.setting_overrides
    }

    /// `getSubCategoryIterator`.
    pub fn get_sub_categories(&self) -> &[ObjectCategoryData] {
        &self.sub_categories
    }

    /// `getProtection()`.
    pub fn get_protection(&self) -> bool {
        self.protection
    }

    /// `ObjectCategoryData.builder()`.
    pub fn builder() -> ObjectCategoryDataBuilder {
        ObjectCategoryDataBuilder::new()
    }

    /// Serializes the category to JSON — the port of
    /// `ObjectCategoryDataGsonSerializer` (and the NBT serializer's data
    /// layout).
    pub fn to_json(&self) -> String {
        json::to_string(&data_to_json(self))
    }

    /// Deserializes from JSON — the inverse of [`ObjectCategoryData::to_json`].
    pub fn from_json(text: &str) -> Result<ObjectCategoryData, String> {
        data_from_json(&json::parse(text)?)
    }
}

fn data_to_json(cat: &ObjectCategoryData) -> json::Json {
    let mut obj = Vec::new();
    obj.push(("name".to_string(), json::Json::Str(cat.name.clone())));
    let overrides: Vec<(String, json::Json)> = cat
        .setting_overrides
        .iter()
        .map(|(k, v)| (k.clone(), setting_value_to_json(v)))
        .collect();
    obj.push(("settingOverrides".to_string(), json::Json::Obj(overrides)));
    obj.push((
        "subCategories".to_string(),
        json::Json::Arr(cat.sub_categories.iter().map(data_to_json).collect()),
    ));
    obj.push(("protection".to_string(), json::Json::Bool(cat.protection)));
    json::Json::Obj(obj)
}

fn data_from_json(v: &json::Json) -> Result<ObjectCategoryData, String> {
    let name = match json::get(v, "name") {
        Some(json::Json::Str(s)) => s.clone(),
        Some(_) => return Err("category name must be a string".to_string()),
        None => String::new(),
    };
    let mut overrides = BTreeMap::new();
    if let Some(json::Json::Obj(pairs)) = json::get(v, "settingOverrides") {
        for (k, val) in pairs {
            if let Some(value) = setting_value_from_json(val) {
                overrides.insert(k.clone(), value);
            }
        }
    }
    let mut subs = Vec::new();
    if let Some(json::Json::Arr(items)) = json::get(v, "subCategories") {
        for item in items {
            subs.push(data_from_json(item)?);
        }
    }
    let protection = match json::get(v, "protection") {
        Some(json::Json::Bool(b)) => *b,
        _ => false,
    };
    Ok(ObjectCategoryData {
        name,
        setting_overrides: overrides,
        sub_categories: subs,
        protection,
    })
}

/// `ObjectCategoryData$Builder`.
#[derive(Debug, Clone, Default)]
pub struct ObjectCategoryDataBuilder {
    name: String,
    setting_overrides: BTreeMap<String, SettingValue>,
    sub_categories: Vec<ObjectCategoryData>,
    protection: bool,
}

impl ObjectCategoryDataBuilder {
    pub fn new() -> Self {
        ObjectCategoryDataBuilder::default()
    }

    /// `setDefault()`.
    pub fn set_default() -> Self {
        ObjectCategoryDataBuilder::new()
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn set_setting_override(mut self, id: &str, value: SettingValue) -> Self {
        self.setting_overrides.insert(id.to_string(), value);
        self
    }

    pub fn add_sub_category(mut self, category: ObjectCategoryData) -> Self {
        self.sub_categories.push(category);
        self
    }

    pub fn set_protection(mut self, protection: bool) -> Self {
        self.protection = protection;
        self
    }

    pub fn build(self) -> ObjectCategoryData {
        ObjectCategoryData {
            name: self.name,
            setting_overrides: self.setting_overrides,
            sub_categories: self.sub_categories,
            protection: self.protection,
        }
    }
}

pub(crate) fn setting_value_to_json(v: &SettingValue) -> json::Json {
    match v {
        SettingValue::Bool(b) => json::Json::Bool(*b),
        SettingValue::Int(i) => json::Json::Num(*i as f64),
        SettingValue::Long(l) => json::Json::Num(*l as f64),
        SettingValue::Float(f) => json::Json::Num(*f as f64),
        SettingValue::Str(s) => json::Json::Str(s.clone()),
    }
}

pub(crate) fn setting_value_from_json(v: &json::Json) -> Option<SettingValue> {
    match v {
        json::Json::Bool(b) => Some(SettingValue::Bool(*b)),
        json::Json::Str(s) => Some(SettingValue::Str(s.clone())),
        json::Json::Num(n) => {
            if n.fract() == 0.0 && n.abs() <= i32::MAX as f64 {
                Some(SettingValue::Int(*n as i32))
            } else {
                Some(SettingValue::Float(*n as f32))
            }
        }
        _ => None,
    }
}

/// Minimal JSON value and parser, shared with the other crates.
/// Re-exported from `xaerominimap-json`.
pub use xaerominimap_json as json;

/// `xaero/hud/category/serialization/data/FilterObjectCategoryData`: a
/// serializable category with hard-include, include/exclude lists and an
/// exclude mode. The base name/settings/sub-categories/protection fields are
/// stored in the embedded [`ObjectCategoryData`].
#[derive(Debug, Clone, PartialEq)]
pub struct FilterObjectCategoryData {
    base: ObjectCategoryData,
    hard_include: String,
    include_list: Vec<String>,
    include_list_in_super_category: bool,
    exclude_mode: ExcludeListMode,
    exclude_list: Vec<String>,
}

impl FilterObjectCategoryData {
    /// Java constructor order: name, hardInclude, includeList, excludeList,
    /// excludeMode, settingOverrides, subCategories,
    /// includeListInSuperCategory, protection.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        hard_include: &str,
        include_list: Vec<String>,
        exclude_list: Vec<String>,
        exclude_mode: ExcludeListMode,
        setting_overrides: BTreeMap<String, SettingValue>,
        sub_categories: Vec<ObjectCategoryData>,
        include_list_in_super_category: bool,
        protection: bool,
    ) -> Self {
        FilterObjectCategoryData {
            base: ObjectCategoryData::new(
                name,
                setting_overrides,
                sub_categories,
                protection,
            ),
            hard_include: hard_include.to_string(),
            include_list,
            include_list_in_super_category,
            exclude_mode,
            exclude_list,
        }
    }

    /// `getBase()`-style access to the inherited data.
    pub fn get_base(&self) -> &ObjectCategoryData {
        &self.base
    }

    pub fn get_hard_include(&self) -> &str {
        &self.hard_include
    }

    pub fn get_include_list(&self) -> &[String] {
        &self.include_list
    }

    pub fn get_exclude_list(&self) -> &[String] {
        &self.exclude_list
    }

    pub fn get_include_list_in_super_category(&self) -> bool {
        self.include_list_in_super_category
    }

    pub fn get_exclude_mode(&self) -> ExcludeListMode {
        self.exclude_mode
    }

    /// `FilterObjectCategoryDataNbtSerializer.serialize` as JSON (NBT keys
    /// `h`, `i`, `m`, `l`, `e` are kept verbatim for parity).
    pub fn to_json(&self) -> String {
        let json::Json::Obj(mut base) = data_to_json(&self.base) else {
            unreachable!()
        };
        base.push(("h".into(), json::Json::Str(self.hard_include.clone())));
        base.push((
            "i".into(),
            json::Json::Bool(self.include_list_in_super_category),
        ));
        base.push(("m".into(), json::Json::Str(self.exclude_mode.name().into())));
        base.push((
            "l".into(),
            json::Json::Arr(
                self.include_list
                    .iter()
                    .map(|s| json::Json::Str(s.clone()))
                    .collect(),
            ),
        ));
        base.push((
            "e".into(),
            json::Json::Arr(
                self.exclude_list
                    .iter()
                    .map(|s| json::Json::Str(s.clone()))
                    .collect(),
            ),
        ));
        json::to_string(&json::Json::Obj(base))
    }

    /// Inverse of [`FilterObjectCategoryData::to_json`].
    pub fn from_json(text: &str) -> Result<FilterObjectCategoryData, String> {
        let value = json::parse(text)?;
        let base = data_from_json(&value)?;
        let string_or_default = |key: &str| match json::get(&value, key) {
            Some(json::Json::Str(s)) => s.clone(),
            _ => String::new(),
        };
        let strings_or_default = |key: &str| match json::get(&value, key) {
            Some(json::Json::Arr(items)) => items
                .iter()
                .filter_map(|item| match item {
                    json::Json::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        let exclude_mode = match json::get(&value, "m") {
            Some(json::Json::Str(name)) => ExcludeListMode::from_name(name)
                .ok_or_else(|| format!("unknown exclude mode `{name}`"))?,
            _ => ExcludeListMode::AllBut,
        };
        Ok(FilterObjectCategoryData {
            base,
            hard_include: string_or_default("h"),
            include_list: strings_or_default("l"),
            include_list_in_super_category: matches!(
                json::get(&value, "i"),
                Some(json::Json::Bool(true))
            ),
            exclude_mode,
            exclude_list: strings_or_default("e"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_builder_and_getters() {
        let setting = ObjectCategorySetting::builder()
            .set_id("minimapScale")
            .set_display_name("Scale")
            .set_default_value(SettingValue::Float(0.5))
            .build();
        assert_eq!(setting.get_id(), "minimapScale");
        assert_eq!(setting.get_display_name(), "Scale");
        assert_eq!(setting.get_default_value(), &SettingValue::Float(0.5));
        assert_eq!(setting.get_setting_ui_type(), EditorSettingType::IterationButton);
    }

    #[test]
    fn category_tree_build_and_lookup() {
        let root = ObjectCategoryBuilder::new()
            .set_name("Root")
            .set_setting_value("scale", SettingValue::Bool(true))
            .add_sub_category(ObjectCategory::builder().set_name("Waypoints").build())
            .build();
        assert_eq!(root.get_name(), "Root");
        assert!(root.get_setting_value("scale").is_some());
        assert_eq!(root.get_sub_categories().len(), 1);
        assert_eq!(root.get_sub_categories()[0].get_name(), "Waypoints");
        assert!(root.get_setting_value("missing").is_none());
        assert!(root.get_super_category().is_none());
    }

    #[test]
    fn hard_rule_and_filter() {
        let setting = ObjectCategorySetting::builder().set_id("v").build();
        let rule = ObjectCategoryHardRule::builder()
            .set_predicate(Box::new(|s, v| matches!(v, SettingValue::Bool(true) if s.get_id() == "v")))
            .build();
        assert!(rule.is_followed_by(&setting, &SettingValue::Bool(true)));
        assert!(!rule.is_followed_by(&setting, &SettingValue::Bool(false)));
        let reversed = ObjectCategoryHardRule::builder()
            .set_predicate(Box::new(|_s, _v| false))
            .set_reversed(true)
            .build();
        assert!(reversed.is_followed_by(&setting, &SettingValue::Bool(true)));
    }

    #[test]
    fn list_rules_and_filter_follows() {
        let list = ObjectCategoryListRule::new(
            ObjectCategoryListRuleType::new("entityNames"),
            vec!["Zombie".to_string(), "Creeper".to_string()],
        );
        assert!(list.in_list("Zombie"));
        assert!(!list.in_list("Player"));
        let filter = FilterObjectCategory::builder()
            .exclude_list(ObjectCategoryListRule::new(
                ObjectCategoryListRuleType::new("entityNames"),
                vec!["Zombie".to_string()],
            ))
            .build();
        assert!(!filter.follows_rules("entityNames", "Zombie"));
        assert!(filter.follows_rules("entityNames", "Creeper"));
    }

    #[test]
    fn data_json_round_trip() {
        let mut overrides = BTreeMap::new();
        overrides.insert("scale".to_string(), SettingValue::Float(0.5));
        let data = ObjectCategoryData::new(
            "Minimap",
            overrides,
            vec![ObjectCategoryData::new(
                "Waypoints",
                BTreeMap::new(),
                vec![],
                false,
            )],
            true,
        );
        let json = data.to_json();
        assert!(json.starts_with('{'));
        let parsed = ObjectCategoryData::from_json(&json).unwrap();
        assert_eq!(parsed.get_name(), "Minimap");
        assert_eq!(parsed.get_setting_overrides().len(), 1);
        assert_eq!(parsed.get_setting_overrides().get("scale"), Some(&SettingValue::Float(0.5)));
        assert_eq!(parsed.get_sub_categories().len(), 1);
        assert_eq!(parsed.get_sub_categories()[0].get_name(), "Waypoints");
        assert!(parsed.get_protection());
        assert!(ObjectCategoryData::from_json("{ broken").is_err());
    }

    #[test]
    fn json_parse_and_encode() {
        let v = json::parse(r#"{"a": [1, 2.5, true, "x"], "b": {"c": null}}"#).unwrap();
        assert_eq!(json::get(&v, "a").map(json::to_string), Some("[1,2.5,true,\"x\"]".to_string()));
        assert!(matches!(json::get(&v, "b"), Some(json::Json::Obj(_))));
        assert_eq!(json::get(&v, "missing"), None);
    }

    #[test]
    fn editor_type_names() {
        assert_eq!(EditorSettingType::from_name("slider"), Some(EditorSettingType::Slider));
        assert_eq!(EditorSettingType::Slider.name(), "slider");
        assert_eq!(EditorSettingType::from_name("nope"), None);
    }

    #[test]
    fn exclude_mode_names() {
        assert_eq!(ExcludeListMode::Only.name(), "ONLY");
        assert_eq!(ExcludeListMode::AllBut.name(), "ALL_BUT");
        assert_eq!(ExcludeListMode::from_name("ONLY"), Some(ExcludeListMode::Only));
        assert_eq!(ExcludeListMode::from_name("nope"), None);
    }

    #[test]
    fn filter_category_data_json_round_trip() {
        let data = FilterObjectCategoryData::new(
            "Players",
            "player",
            vec!["zombie".into(), "skeleton".into()],
            vec!["creeper".into()],
            ExcludeListMode::Only,
            BTreeMap::new(),
            vec![],
            true,
            false,
        );
        let text = data.to_json();
        assert!(text.contains("\"h\":\"player\""));
        assert!(text.contains("\"m\":\"ONLY\""));
        let parsed = FilterObjectCategoryData::from_json(&text).unwrap();
        assert_eq!(parsed, data);
        assert_eq!(parsed.get_base().get_name(), "Players");
        assert_eq!(parsed.get_include_list().len(), 2);
        assert_eq!(parsed.get_exclude_list(), &["creeper".to_string()]);
        assert!(parsed.get_include_list_in_super_category());
        assert_eq!(parsed.get_exclude_mode(), ExcludeListMode::Only);
    }
}
