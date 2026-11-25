//! Data-side port of the entity radar icon prerenderers and the model field
//! resolver (`xaero/hud/minimap/radar/icon/creator/render`).
//!
//! The Java classes in this package drive Minecraft's `EntityRenderer`,
//! `EntityModel` and render pipelines directly. A Horizon-side renderer has
//! its own model types, so this module ports the parts that are independent
//! of the engine: form prerenderer capabilities, the field-reference syntax
//! used by custom model configs, model render traces and the selection of
//! model part fields.
//!
//! | Java class | Rust type |
//! | --- | --- |
//! | `.../form/IRadarIconFormPrerenderer` | [`FormPrerenderer`] |
//! | `.../resolver/RadarIconModelFieldResolver` | [`ModelFieldResolver`] |
//! | `.../resolver/RadarIconModelFieldResolver$FieldReferenceType` | [`FieldReferenceType`] |
//! | `.../trace/ModelRenderTrace` | [`ModelRenderTrace`] |
//! | `.../trace/ModelPartRenderTrace` | [`ModelPartRenderTrace`] |

use std::collections::BTreeMap;

use xaerominimap_json::Json;

use crate::radar_icons::{IconFormType, ModelConfig};

/// Looks up a key in the JSON crate's `Obj(Vec<(String, Json)>)` shape.
fn object_get<'a>(entries: &'a [(String, Json)], key: &str) -> Option<&'a Json> {
    entries
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value)
}

/// Capabilities of an `IRadarIconFormPrerenderer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormPrerenderer {
    /// `requiresEntityModel()` — the model form needs the entity's model.
    pub requires_entity_model: bool,
    /// `isFlipped()` — the legacy `sprite` form is drawn mirrored.
    pub flipped: bool,
    /// `isOutlined()` — model, outlined sprite and item forms draw an outline.
    pub outlined: bool,
    /// `getFailureResult()` icon id (`FAILED`, `DOT` or none for `dot`).
    pub failure_icon: Option<&'static str>,
}

impl FormPrerenderer {
    /// `RadarIconFormTypes` registration for a form type. `dot` has no
    /// prerenderer: the coloured radar dot is drawn instead.
    pub fn for_type(form_type: IconFormType) -> Option<FormPrerenderer> {
        match form_type {
            IconFormType::Model => Some(FormPrerenderer {
                requires_entity_model: true,
                flipped: false,
                outlined: true,
                failure_icon: Some("FAILED"),
            }),
            IconFormType::NormalSprite => Some(FormPrerenderer {
                requires_entity_model: false,
                flipped: false,
                outlined: false,
                failure_icon: Some("FAILED"),
            }),
            IconFormType::OutlinedSprite => Some(FormPrerenderer {
                requires_entity_model: false,
                flipped: false,
                outlined: true,
                failure_icon: Some("FAILED"),
            }),
            IconFormType::Sprite => Some(FormPrerenderer {
                requires_entity_model: false,
                flipped: true,
                outlined: false,
                failure_icon: Some("FAILED"),
            }),
            IconFormType::Item => Some(FormPrerenderer {
                requires_entity_model: false,
                flipped: false,
                outlined: true,
                failure_icon: Some("DOT"),
            }),
            IconFormType::Dot => None,
        }
    }
}

/// `FieldReferenceType` in `RadarIconModelFieldResolver`: how a referenced
/// Java object is indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldReferenceType {
    Single,
    Array,
    Collection,
    Map,
}

impl FieldReferenceType {
    /// `getReferenceType(Object)` — the Java test order is single, array,
    /// collection, map.
    pub fn of(value: &Json) -> FieldReferenceType {
        match value {
            Json::Arr(_) => FieldReferenceType::Array,
            Json::Obj(_) => FieldReferenceType::Map,
            _ => FieldReferenceType::Single,
        }
    }
}

/// One index inside a field reference: `field[0]` or `field['key']`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldIndex {
    Index(usize),
    Key(String),
}

/// A parsed model field path, e.g. `root[0]['head']` from the Java reflection
/// resolver's `field[index]` syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldReference {
    pub field: String,
    pub indices: Vec<FieldIndex>,
}

impl FieldReference {
    pub fn new(field: impl Into<String>) -> Self {
        FieldReference {
            field: field.into(),
            indices: Vec::new(),
        }
    }

    /// Rebuilds the Java field-name form used by filters and config strings.
    pub fn as_string(&self) -> String {
        let mut result = self.field.clone();
        for index in &self.indices {
            match index {
                FieldIndex::Index(i) => result.push_str(&format!("[{i}]")),
                FieldIndex::Key(key) => result.push_str(&format!("['{key}']")),
            }
        }
        result
    }
}

/// `RadarIconModelFieldResolver`: resolves reflection-style field references
/// against a generic JSON tree supplied by the game glue.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModelFieldResolver;

impl ModelFieldResolver {
    /// `handleDeclaredField(String, ...)`: splits the field name from its
    /// optional `[...]` selectors. Keys may be separated by `,` or `;` and
    /// may be quoted, mirroring `SeparatedKeysParser`.
    pub fn parse(reference: &str) -> Result<FieldReference, String> {
        let trimmed = reference.trim();
        let Some(open) = trimmed.find('[') else {
            if trimmed.ends_with(']') {
                return Err(format!(
                    "Field name {trimmed} ends with ] but is missing [!"
                ));
            }
            return Ok(FieldReference::new(trimmed));
        };
        if !trimmed.ends_with(']') {
            return Err(format!("Invalid field reference {trimmed}!"));
        }
        let field = trimmed[..open].trim();
        if field.is_empty() {
            return Err(format!("Invalid field reference {trimmed}!"));
        }
        let mut indices = Vec::new();
        // A path can chain several bracket groups (`root[0]['head']`).
        let mut rest = &trimmed[open..];
        while let Some(close) = rest.find(']') {
            let inner = &rest[1..close];
            for key in Self::split_keys(inner) {
                if let Ok(index) = key.parse::<usize>() {
                    indices.push(FieldIndex::Index(index));
                } else {
                    indices.push(FieldIndex::Key(key));
                }
            }
            rest = rest[close + 1..].trim_start();
            if rest.is_empty() {
                break;
            }
            if !rest.starts_with('[') {
                return Err(format!("Invalid field reference {trimmed}!"));
            }
        }
        Ok(FieldReference {
            field: field.to_string(),
            indices,
        })
    }

    fn split_keys(text: &str) -> Vec<String> {
        let mut keys = Vec::new();
        let mut current = String::new();
        let mut quote: Option<char> = None;
        let mut escaped = false;
        for ch in text.chars() {
            if escaped {
                current.push(ch);
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if let Some(q) = quote {
                if ch == q {
                    quote = None;
                } else {
                    current.push(ch);
                }
            } else if ch == '\'' || ch == '"' {
                quote = Some(ch);
            } else if ch == ',' || ch == ';' {
                let key = current.trim();
                if !key.is_empty() {
                    keys.push(key.to_string());
                }
                current.clear();
            } else {
                current.push(ch);
            }
        }
        let key = current.trim();
        if !key.is_empty() {
            keys.push(key.to_string());
        }
        keys
    }

    /// Resolves a field reference against the supplied model data. Objects
    /// are treated as Java maps and arrays as Java arrays/collections.
    pub fn resolve(root: &Json, reference: &str) -> Result<Vec<Json>, String> {
        let reference = Self::parse(reference)?;
        let mut values = match root {
            Json::Obj(entries) => object_get(entries, &reference.field)
                .cloned()
                .into_iter()
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        for index in &reference.indices {
            let mut next = Vec::new();
            for value in values {
                match (index, value) {
                    (FieldIndex::Index(i), Json::Arr(items)) => {
                        if let Some(item) = items.get(*i) {
                            next.push(item.clone());
                        }
                    }
                    (FieldIndex::Key(key), Json::Obj(entries)) => {
                        if let Some(item) = object_get(&entries, key) {
                            next.push(item.clone());
                        }
                    }
                    (FieldIndex::Key(key), Json::Arr(items)) => {
                        if let Ok(i) = key.parse::<usize>() {
                            if let Some(item) = items.get(i) {
                                next.push(item.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            values = next;
        }
        Ok(values)
    }

    /// `passesFilter(String, List<String>)`: a filter matches the field name
    /// itself or its name with the last `[...]` selector removed.
    pub fn passes_filter(field_name: &str, filters: &[String]) -> Option<String> {
        let base = field_name.split('[').next().unwrap_or(field_name);
        filters
            .iter()
            .find(|filter| {
                let filter_base = filter.split('[').next().unwrap_or(filter);
                filter.as_str() == field_name || filter_base == base
            })
            .cloned()
    }

    /// Resolves the `modelPartsFields` of a config against the model data and
    /// returns the matching values together with their field names.
    pub fn resolve_model_parts(
        config: &ModelConfig,
        root: &Json,
    ) -> Result<Vec<(String, Json)>, String> {
        let mut result = Vec::new();
        for field in &config.model_parts_fields {
            for value in Self::resolve(root, field)? {
                result.push((field.clone(), value));
            }
        }
        Ok(result)
    }

    /// Resolves the model's root path: each entry of `modelRootPath` is an
    /// alternative chain of field references.
    pub fn resolve_root_path(config: &ModelConfig, root: &Json) -> Result<Vec<Json>, String> {
        let mut result = Vec::new();
        for path in &config.model_root_path {
            let mut values = vec![root.clone()];
            for field in path {
                let mut next = Vec::new();
                for value in &values {
                    let reference = Self::parse(field)?;
                    match value {
                        Json::Obj(entries) => {
                            if let Some(item) = object_get(entries, &reference.field) {
                                next.push(item.clone());
                            }
                        }
                        _ => {}
                    }
                }
                values = next;
            }
            result.extend(values);
        }
        Ok(result)
    }
}

/// `ModelPartRenderTrace`: a model part made visible with a tint colour.
/// The Java `ModelPart` object is represented by an opaque id supplied by
/// the game renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPartRenderTrace {
    pub model_part: usize,
    pub color: u32,
}

/// `ModelRenderTrace`: what one prerender pass recorded for a model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelRenderTrace {
    /// Opaque model id from the game renderer.
    pub model: usize,
    /// Texture ids by render layer name.
    pub textures: BTreeMap<String, String>,
    pub use_overlay: bool,
    pub affects_crumbling: bool,
    pub sort_on_upload: bool,
    pub color: u32,
    pub all_visible: bool,
    pub visible_parts: BTreeMap<usize, ModelPartRenderTrace>,
}

impl ModelRenderTrace {
    pub fn new(model: usize, color: u32) -> Self {
        ModelRenderTrace {
            model,
            textures: BTreeMap::new(),
            use_overlay: false,
            affects_crumbling: false,
            sort_on_upload: false,
            color,
            all_visible: false,
            visible_parts: BTreeMap::new(),
        }
    }

    /// `addVisibleModelPart(ModelPart, int)`.
    pub fn add_visible_model_part(&mut self, model_part: usize, color: u32) {
        self.visible_parts.insert(
            model_part,
            ModelPartRenderTrace {
                model_part,
                color,
            },
        );
        self.all_visible = false;
    }

    /// `getModelPartRenderInfo(ModelPart)`.
    pub fn get_model_part_render_info(&self, model_part: usize) -> Option<&ModelPartRenderTrace> {
        self.visible_parts.get(&model_part)
    }

    /// `isEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.visible_parts.is_empty()
    }

    /// `sameVisibility(ModelRenderTrace)`.
    pub fn same_visibility(&self, other: &ModelRenderTrace) -> bool {
        if self.all_visible != other.all_visible {
            return false;
        }
        self.visible_parts
            .keys()
            .eq(other.visible_parts.keys())
            && (self.all_visible
                || self
                    .visible_parts
                    .keys()
                    .all(|key| self.visible_parts[key].color == other.visible_parts[key].color))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::radar_icons::IconFormType;

    #[test]
    fn prerenderer_capabilities_match_the_java_form_types() {
        let model = FormPrerenderer::for_type(IconFormType::Model).unwrap();
        assert!(model.requires_entity_model && model.outlined && !model.flipped);
        let sprite = FormPrerenderer::for_type(IconFormType::Sprite).unwrap();
        assert!(sprite.flipped && !sprite.outlined && !sprite.requires_entity_model);
        let outlined = FormPrerenderer::for_type(IconFormType::OutlinedSprite).unwrap();
        assert!(outlined.outlined && !outlined.flipped);
        let item = FormPrerenderer::for_type(IconFormType::Item).unwrap();
        assert_eq!(item.failure_icon, Some("DOT"));
        assert!(FormPrerenderer::for_type(IconFormType::Dot).is_none());
    }

    #[test]
    fn parses_java_field_references() {
        assert_eq!(
            ModelFieldResolver::parse("head"),
            Ok(FieldReference::new("head"))
        );
        assert_eq!(
            ModelFieldResolver::parse("root[0, 1]"),
            Ok(FieldReference {
                field: "root".into(),
                indices: vec![FieldIndex::Index(0), FieldIndex::Index(1)],
            })
        );
        assert_eq!(
            ModelFieldResolver::parse("parts['left_arm']").unwrap().as_string(),
            "parts['left_arm']"
        );
        assert!(ModelFieldResolver::parse("root[0]").is_ok());
        assert!(ModelFieldResolver::parse("bad]").is_err());
    }

    #[test]
    fn resolves_fields_and_filters() {
        let data = xaerominimap_json::parse(
            r#"{"parts":{"head":{"visible":true},"body":{"visible":false}},"list":[10,20]}"#,
        )
        .unwrap();
        let values = ModelFieldResolver::resolve(&data, "parts['head']['visible']").unwrap();
        assert_eq!(values, vec![Json::Bool(true)]);
        let values = ModelFieldResolver::resolve(&data, "list[1]").unwrap();
        assert_eq!(values, vec![Json::Num(20.0)]);
        assert!(ModelFieldResolver::resolve(&data, "missing").unwrap().is_empty());

        let filters = vec!["parts['head']".to_string(), "list".to_string()];
        assert_eq!(
            ModelFieldResolver::passes_filter("parts['body']", &filters),
            Some("parts['head']".to_string())
        );
        assert_eq!(
            ModelFieldResolver::passes_filter("other", &filters),
            None
        );
    }

    #[test]
    fn model_config_field_selection_and_traces() {
        let config = ModelConfig {
            model_parts_fields: vec!["parts['head']".into(), "parts['body']".into()],
            model_root_path: vec![vec!["parts".into(), "head".into()]],
            ..ModelConfig::default()
        };
        let data = xaerominimap_json::parse(
            r#"{"parts":{"head":{"visible":true},"body":{"visible":false}}}"#,
        )
        .unwrap();
        let parts = ModelFieldResolver::resolve_model_parts(&config, &data).unwrap();
        assert_eq!(parts.len(), 2);
        let roots = ModelFieldResolver::resolve_root_path(&config, &data).unwrap();
        let expected = xaerominimap_json::get(
            xaerominimap_json::get(&data, "parts").unwrap(),
            "head",
        )
        .unwrap()
        .clone();
        assert_eq!(roots, vec![expected]);

        let mut trace = ModelRenderTrace::new(7, 0xffffff);
        assert!(trace.is_empty());
        trace.add_visible_model_part(1, 0xff0000);
        trace.add_visible_model_part(2, 0x00ff00);
        assert!(!trace.is_empty());
        assert_eq!(trace.get_model_part_render_info(1).unwrap().color, 0xff0000);
        let mut other = ModelRenderTrace::new(7, 0xffffff);
        other.add_visible_model_part(1, 0xff0000);
        assert!(!trace.same_visibility(&other));
        other.add_visible_model_part(2, 0x00ff00);
        assert!(trace.same_visibility(&other));
    }
}
