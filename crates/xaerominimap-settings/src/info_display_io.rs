//! Text IO for info-display configs, ported from
//! `xaero/hud/minimap/info/InfoDisplayIO` of the original mod.
//!
//! The legacy `.txt` format has two kinds of lines:
//!
//! ```text
//! infoDisplayOrder:<id>:<id>:...
//! infoDisplay:<id>:<state|- >:<text colour|- >:<background colour|- >
//! ```
//!
//! `-` means "no value"; the state string is raw and colours are decimal
//! integers. Missing configurations for ids in the order line are skipped
//! on encode (the Java code logs a warning).

use crate::info_display::{InfoDisplayConfigData, InfoDisplayManagerConfigData};

/// Java `InfoDisplayIO.getInfoDisplayLine`.
pub fn info_display_line(id: &str, config: &InfoDisplayConfigData) -> String {
    let state = config.state.as_deref().unwrap_or("-");
    let text = config
        .text_color
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let background = config
        .background_color
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    format!("infoDisplay:{id}:{state}:{text}:{background}")
}

/// Java `InfoDisplayIO.encode`.
pub fn encode(data: &InfoDisplayManagerConfigData) -> String {
    let mut out = String::from("infoDisplayOrder");
    for id in data.order_iter() {
        out.push(':');
        out.push_str(id);
    }
    out.push('\n');
    for id in data.order_iter() {
        if let Some(config) = data.get(id) {
            out.push_str(&info_display_line(id, config));
            out.push('\n');
        }
    }
    out
}

fn optional_int(value: &str) -> Result<Option<i32>, String> {
    if value == "-" {
        Ok(None)
    } else {
        value
            .parse::<i32>()
            .map(Some)
            .map_err(|error| format!("invalid info display colour `{value}`: {error}"))
    }
}

/// Java `InfoDisplayIO.decode`. Java throws `NumberFormatException` for bad
/// colours; the Rust port returns an error instead.
pub fn decode(input: &str) -> Result<InfoDisplayManagerConfigData, String> {
    let mut order = Vec::new();
    let mut configs = std::collections::BTreeMap::new();
    for line in input.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "infoDisplayOrder" => {
                // Java skips duplicates while adding the parts to the order.
                for id in parts.iter().skip(1) {
                    if !order.iter().any(|existing: &String| existing == id) {
                        order.push((*id).to_string());
                    }
                }
            }
            "infoDisplay" => {
                if parts.len() < 5 {
                    return Err(format!("truncated info display line: `{line}`"));
                }
                let state = if parts[2] == "-" {
                    None
                } else {
                    Some(parts[2].to_string())
                };
                let text_color = optional_int(parts[3])?;
                let background_color = optional_int(parts[4])?;
                configs.insert(
                    parts[1].to_string(),
                    InfoDisplayConfigData::new(background_color, text_color, state),
                );
            }
            _ => {}
        }
    }
    Ok(InfoDisplayManagerConfigData::new(order, configs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> InfoDisplayManagerConfigData {
        let mut data = InfoDisplayManagerConfigData::empty();
        data.put(
            "coords",
            InfoDisplayConfigData::new(Some(0x2200_0000), Some(-1), Some("short".into())),
        );
        data.put("biome", InfoDisplayConfigData::new(None, None, None));
        data
    }

    #[test]
    fn encodes_order_and_config_lines() {
        assert_eq!(
            encode(&sample()),
            "infoDisplayOrder:coords:biome\n\
             infoDisplay:coords:short:-1:570425344\n\
             infoDisplay:biome:-:-:-\n"
        );
    }

    #[test]
    fn decodes_and_round_trips() {
        let original = sample();
        let text = encode(&original);
        let decoded = decode(&text).unwrap();
        assert_eq!(decoded.copy_order(), original.copy_order());
        assert_eq!(decoded.get("coords"), original.get("coords"));
        assert_eq!(decoded.get("biome"), original.get("biome"));
    }

    #[test]
    fn decode_skips_duplicate_ids_and_unknown_lines() {
        let decoded = decode(
            "infoDisplayOrder:a:b:a\n\
             unknown:line\n\
             infoDisplay:a:state:-:-\n\
             infoDisplay:b:-:-:-\n",
        )
        .unwrap();
        assert_eq!(decoded.copy_order(), vec!["a", "b"]);
        assert_eq!(decoded.get("a").unwrap().state.as_deref(), Some("state"));
    }

    #[test]
    fn invalid_colours_are_errors() {
        assert!(decode("infoDisplay:a:s:oops:-").is_err());
        assert!(decode("infoDisplay:a:s:1").is_err());
    }
}
