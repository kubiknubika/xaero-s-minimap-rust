//! Sync codecs for large config values (`MinimapValueSyncCodecs`).
//!
//! | Java class | Rust function |
//! | --- | --- |
//! | `.../value/type/sync/serialization/MinimapValueSyncCodecs.INFO_DISPLAY_CONFIG` | [`info_display_to_nbt`] / [`info_display_from_nbt`] |
//!
//! The Java codec stores the ordered display ids under `o` (a `ListTag` of
//! strings) and the per-id settings under `c` (a `CompoundTag`). Each config
//! entry writes `s` (state), `b` (background colour) and `t` (text colour)
//! only when present; an empty compound decodes to all-`null` values.
//! Entity radar categories use the same shape through the mod's JSON codec,
//! which lives in `xaerominimap-settings`.

use xaerominimap_settings::{
    InfoDisplayConfigData, InfoDisplayManagerConfigData,
};

use crate::nbt::NbtValue;

/// Java `INFO_DISPLAY_CONFIG` serializer (`lambda$static$2`/`$4`).
pub fn info_display_to_nbt(data: &InfoDisplayManagerConfigData) -> NbtValue {
    let order: Vec<NbtValue> = data
        .order_iter()
        .map(|id| NbtValue::String(id.to_string()))
        .collect();
    let mut configs = Vec::new();
    for id in data.order_iter() {
        let Some(config) = data.get(id) else {
            continue;
        };
        let mut entry = Vec::new();
        if let Some(state) = config.state.as_deref() {
            entry.push(("s".to_string(), NbtValue::String(state.to_string())));
        }
        if let Some(background) = config.background_color {
            entry.push(("b".to_string(), NbtValue::Int(background)));
        }
        if let Some(text) = config.text_color {
            entry.push(("t".to_string(), NbtValue::Int(text)));
        }
        if !entry.is_empty() {
            configs.push((id.to_string(), NbtValue::Compound(entry)));
        }
    }
    NbtValue::Compound(vec![
        ("o".to_string(), NbtValue::List(order)),
        ("c".to_string(), NbtValue::Compound(configs)),
    ])
}

fn read_config(compound: &[(String, NbtValue)]) -> InfoDisplayConfigData {
    if compound.is_empty() {
        return InfoDisplayConfigData::new(None, None, None);
    }
    let get_string = |key: &str| {
        compound
            .iter()
            .find(|(name, _)| name == key)
            .and_then(|(_, value)| value.as_str())
            .unwrap_or("")
            .to_string()
    };
    let get_int = |key: &str| {
        compound
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_int().unwrap_or(0))
    };
    let state = compound
        .iter()
        .find(|(name, _)| name == "s")
        .map(|_| get_string("s"));
    InfoDisplayConfigData::new(get_int("b"), get_int("t"), state)
}

/// Java `INFO_DISPLAY_CONFIG` deserializer (`lambda$static$5`/`$6`).
pub fn info_display_from_nbt(value: &NbtValue) -> Result<InfoDisplayManagerConfigData, String> {
    let NbtValue::Compound(entries) = value else {
        return Err("info display config must be a compound tag".into());
    };
    let mut order = Vec::new();
    if let Some(values) = entries
        .iter()
        .find(|(name, _)| name == "o")
        .and_then(|(_, value)| value.as_list())
    {
        for value in values {
            order.push(
                value
                    .as_str()
                    .ok_or_else(|| "info display order entry is not a string".to_string())?
                    .to_string(),
            );
        }
    }
    let config_entries = entries
        .iter()
        .find(|(name, _)| name == "c")
        .and_then(|(_, value)| match value {
            NbtValue::Compound(entries) => Some(entries.as_slice()),
            _ => None,
        })
        .unwrap_or(&[]);
    let mut configs = std::collections::BTreeMap::new();
    for id in &order {
        let compound = config_entries
            .iter()
            .find(|(name, _)| name == id)
            .and_then(|(_, value)| match value {
                NbtValue::Compound(entries) => Some(entries.as_slice()),
                _ => None,
            })
            .unwrap_or(&[]);
        configs.insert(id.clone(), read_config(compound));
    }
    Ok(InfoDisplayManagerConfigData::new(order, configs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{PacketReader, PacketWriter};

    #[test]
    fn info_display_config_round_trips_through_nbt() {
        let mut data = InfoDisplayManagerConfigData::empty();
        data.put(
            "coords",
            InfoDisplayConfigData::new(Some(0x2200_0000), Some(-1), Some("short".into())),
        );
        data.put("biome", InfoDisplayConfigData::new(None, None, None));
        data.put(
            "time",
            InfoDisplayConfigData::new(Some(7), None, Some("long".into())),
        );

        let value = info_display_to_nbt(&data);
        let mut writer = PacketWriter::new();
        crate::nbt::write_root(&value, &mut writer);
        let bytes = writer.into_bytes();
        let mut reader = PacketReader::new(&bytes);
        let decoded_value = crate::nbt::read_root(&mut reader).unwrap();
        assert!(reader.is_empty());
        let decoded = info_display_from_nbt(&decoded_value).unwrap();

        assert_eq!(decoded.copy_order(), vec!["coords", "biome", "time"]);
        assert_eq!(decoded.get("coords"), data.get("coords"));
        assert_eq!(decoded.get("biome"), data.get("biome"));
        assert_eq!(decoded.get("time"), data.get("time"));
    }

    #[test]
    fn missing_optional_fields_decode_as_none() {
        let value = NbtValue::Compound(vec![
            (
                "o".into(),
                NbtValue::List(vec![NbtValue::String("a".into())]),
            ),
            (
                "c".into(),
                NbtValue::Compound(vec![(
                    "a".into(),
                    NbtValue::Compound(vec![("t".into(), NbtValue::Int(9))]),
                )]),
            ),
        ]);
        let decoded = info_display_from_nbt(&value).unwrap();
        let config = decoded.get("a").unwrap();
        assert_eq!(config.background_color, None);
        assert_eq!(config.text_color, Some(9));
        assert_eq!(config.state, None);
    }

    #[test]
    fn empty_and_malformed_values_are_handled() {
        let empty = info_display_from_nbt(&info_display_to_nbt(
            &InfoDisplayManagerConfigData::empty(),
        ))
        .unwrap();
        assert!(empty.is_empty());
        assert!(info_display_from_nbt(&NbtValue::Int(1)).is_err());
    }
}
