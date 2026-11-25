//! State codecs from `xaero/hud/minimap/info/codec/InfoDisplayCommonStateCodecs`
//! of the original mod.
//!
//! `BOOLEAN` parses only the exact string `true` as true (everything else is
//! false) and has a maximum length of 5; `INTEGER` uses `Integer.parseInt`
//! and a maximum length of 11 (`-2147483648`).

use crate::builtin_info_displays::InfoDisplayCodec;

/// Parsed info-display state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoDisplayState {
    Boolean(bool),
    Integer(i32),
}

impl InfoDisplayState {
    /// Java `InfoDisplayCommonStateCodecs` decoder for a state string.
    pub fn parse(codec: InfoDisplayCodec, text: &str) -> Result<Self, String> {
        match codec {
            InfoDisplayCodec::Boolean => Ok(InfoDisplayState::Boolean(text == "true")),
            InfoDisplayCodec::Integer => text
                .parse::<i32>()
                .map(InfoDisplayState::Integer)
                .map_err(|error| format!("invalid info display integer `{text}`: {error}")),
        }
    }

    /// Java `ConfigValueIOCodec` encoder (`String.valueOf`).
    pub fn format(self) -> String {
        match self {
            InfoDisplayState::Boolean(value) => value.to_string(),
            InfoDisplayState::Integer(value) => value.to_string(),
        }
    }

    /// Maximum state length declared by the codec.
    pub fn max_length(codec: InfoDisplayCodec) -> usize {
        match codec {
            InfoDisplayCodec::Boolean => 5,
            InfoDisplayCodec::Integer => 11,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_codec_accepts_only_true() {
        assert_eq!(
            InfoDisplayState::parse(InfoDisplayCodec::Boolean, "true").unwrap(),
            InfoDisplayState::Boolean(true)
        );
        for text in ["false", "TRUE", "", "1"] {
            assert_eq!(
                InfoDisplayState::parse(InfoDisplayCodec::Boolean, text).unwrap(),
                InfoDisplayState::Boolean(false)
            );
        }
        assert_eq!(InfoDisplayState::Boolean(true).format(), "true");
        assert_eq!(InfoDisplayState::max_length(InfoDisplayCodec::Boolean), 5);
    }

    #[test]
    fn integer_codec_parses_and_formats() {
        assert_eq!(
            InfoDisplayState::parse(InfoDisplayCodec::Integer, "-2147483648").unwrap(),
            InfoDisplayState::Integer(i32::MIN)
        );
        assert!(InfoDisplayState::parse(InfoDisplayCodec::Integer, "oops").is_err());
        assert_eq!(InfoDisplayState::Integer(7).format(), "7");
        assert_eq!(InfoDisplayState::max_length(InfoDisplayCodec::Integer), 11);
    }
}
