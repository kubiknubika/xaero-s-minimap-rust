//! Minimal NBT subset used by the minimap packets
//! (`xaero/hud/packet` and `xaero/common/server/level`).
//!
//! The Java mod serializes all structured packets as a `CompoundTag` with
//! boolean (`TAG_Byte`), double, string, int-array and long-array entries.
//! `FriendlyByteBuf.writeNbt` writes a named root tag with an empty name, so
//! that is what [`write_root`] and [`read_root`] implement.

use crate::codec::{PacketReader, PacketWriter};

/// NBT tag type ids used here (the remaining ids are kept for completeness).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NbtType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl NbtType {
    pub fn from_id(id: u8) -> Option<NbtType> {
        match id {
            0 => Some(NbtType::End),
            1 => Some(NbtType::Byte),
            2 => Some(NbtType::Short),
            3 => Some(NbtType::Int),
            4 => Some(NbtType::Long),
            5 => Some(NbtType::Float),
            6 => Some(NbtType::Double),
            7 => Some(NbtType::ByteArray),
            8 => Some(NbtType::String),
            9 => Some(NbtType::List),
            10 => Some(NbtType::Compound),
            11 => Some(NbtType::IntArray),
            12 => Some(NbtType::LongArray),
            _ => None,
        }
    }
}

/// A value that can appear in the packets' NBT compounds.
#[derive(Debug, Clone, PartialEq)]
pub enum NbtValue {
    /// `CompoundTag.putBoolean` stores a `TAG_Byte` of 0 or 1.
    Byte(i8),
    Int(i32),
    Double(f64),
    String(String),
    /// `ListTag`: the element type is derived from the first entry on write.
    List(Vec<NbtValue>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    Compound(Vec<(String, NbtValue)>),
}

impl NbtValue {
    pub fn tag_type(&self) -> NbtType {
        match self {
            NbtValue::Byte(_) => NbtType::Byte,
            NbtValue::Int(_) => NbtType::Int,
            NbtValue::Double(_) => NbtType::Double,
            NbtValue::String(_) => NbtType::String,
            NbtValue::List(_) => NbtType::List,
            NbtValue::IntArray(_) => NbtType::IntArray,
            NbtValue::LongArray(_) => NbtType::LongArray,
            NbtValue::Compound(_) => NbtType::Compound,
        }
    }

    /// NBT list element type: derived from the first entry, `TAG_End` when
    /// the list is empty (as Minecraft writes it).
    pub fn list_element_type(&self) -> NbtType {
        match self {
            NbtValue::List(values) => values
                .first()
                .map(|value| value.tag_type())
                .unwrap_or(NbtType::End),
            _ => NbtType::End,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            NbtValue::Byte(b) => Some(*b != 0),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            NbtValue::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[NbtValue]> {
        match self {
            NbtValue::List(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_double(&self) -> Option<f64> {
        match self {
            NbtValue::Double(d) => Some(*d),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            NbtValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int_array(&self) -> Option<&[i32]> {
        match self {
            NbtValue::IntArray(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_long_array(&self) -> Option<&[i64]> {
        match self {
            NbtValue::LongArray(values) => Some(values),
            _ => None,
        }
    }
}

/// Java `CompoundTag.getBoolean(key).orElse(false)`.
pub fn get_bool(entries: &[(String, NbtValue)], key: &str) -> bool {
    entries
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, value)| value.as_bool())
        .unwrap_or(false)
}

pub fn get_double(entries: &[(String, NbtValue)], key: &str) -> f64 {
    entries
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, value)| value.as_double())
        .unwrap_or(0.0)
}

pub fn get_string<'a>(entries: &'a [(String, NbtValue)], key: &str) -> Option<&'a str> {
    entries
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, value)| value.as_str())
}

pub fn get_uuid<'a>(entries: &'a [(String, NbtValue)], key: &str) -> Option<crate::Uuid> {
    let value = entries.iter().find(|(name, _)| name == key).map(|(_, v)| v)?;
    if let Some(values) = value.as_int_array() {
        return crate::Uuid::from_int_array(values);
    }
    if let Some(values) = value.as_long_array() {
        return crate::Uuid::from_long_array(values);
    }
    None
}

pub fn put_bool(entries: &mut Vec<(String, NbtValue)>, key: &str, value: bool) {
    entries.push((key.to_string(), NbtValue::Byte(if value { 1 } else { 0 })));
}

pub fn put_double(entries: &mut Vec<(String, NbtValue)>, key: &str, value: f64) {
    entries.push((key.to_string(), NbtValue::Double(value)));
}

pub fn put_string(entries: &mut Vec<(String, NbtValue)>, key: &str, value: &str) {
    entries.push((key.to_string(), NbtValue::String(value.to_string())));
}

/// Writes a root tag, i.e. type byte, empty name and payload.
pub fn write_root(value: &NbtValue, out: &mut PacketWriter) {
    out.write_u8(value.tag_type() as u8);
    out.write_utf("");
    write_payload(value, out);
}

pub fn read_root(input: &mut PacketReader<'_>) -> Result<NbtValue, String> {
    let tag_type = NbtType::from_id(input.read_u8()?)
        .ok_or_else(|| "invalid NBT tag id".to_string())?;
    let name = input.read_utf()?;
    if !name.is_empty() {
        return Err(format!("expected an unnamed root tag, got `{name}`"));
    }
    read_payload(tag_type, input)
}

fn write_payload(value: &NbtValue, out: &mut PacketWriter) {
    match value {
        NbtValue::Byte(byte) => out.write_u8(*byte as u8),
        NbtValue::Int(int) => out.write_i32(*int),
        NbtValue::Double(double) => out.write_f64(*double),
        NbtValue::String(text) => out.write_utf(text),
        NbtValue::List(values) => {
            out.write_u8(value.list_element_type() as u8);
            out.write_i32(values.len() as i32);
            for entry in values {
                write_payload(entry, out);
            }
        }
        NbtValue::IntArray(values) => {
            out.write_i32(values.len() as i32);
            for value in values {
                out.write_i32(*value);
            }
        }
        NbtValue::LongArray(values) => {
            out.write_i32(values.len() as i32);
            for value in values {
                out.write_i64(*value);
            }
        }
        NbtValue::Compound(entries) => {
            for (name, entry) in entries {
                out.write_u8(entry.tag_type() as u8);
                out.write_utf(name);
                write_payload(entry, out);
            }
            out.write_u8(NbtType::End as u8);
        }
    }
}

fn read_payload(tag_type: NbtType, input: &mut PacketReader<'_>) -> Result<NbtValue, String> {
    match tag_type {
        NbtType::Byte => Ok(NbtValue::Byte(input.read_u8()? as i8)),
        NbtType::Int => Ok(NbtValue::Int(input.read_i32()?)),
        NbtType::Double => Ok(NbtValue::Double(input.read_f64()?)),
        NbtType::String => Ok(NbtValue::String(input.read_utf()?)),
        NbtType::List => {
            let element_type = NbtType::from_id(input.read_u8()?)
                .ok_or_else(|| "invalid NBT list element tag id".to_string())?;
            let len = input.read_i32()?;
            if len < 0 {
                return Err("negative NBT list length".into());
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                values.push(read_payload(element_type, input)?);
            }
            Ok(NbtValue::List(values))
        }
        NbtType::IntArray => {
            let len = input.read_i32()?;
            if len < 0 {
                return Err("negative NBT int array length".into());
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                values.push(input.read_i32()?);
            }
            Ok(NbtValue::IntArray(values))
        }
        NbtType::LongArray => {
            let len = input.read_i32()?;
            if len < 0 {
                return Err("negative NBT long array length".into());
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                values.push(input.read_i64()?);
            }
            Ok(NbtValue::LongArray(values))
        }
        NbtType::Compound => {
            let mut entries = Vec::new();
            loop {
                let tag = NbtType::from_id(input.read_u8()?)
                    .ok_or_else(|| "invalid NBT tag id".to_string())?;
                if tag == NbtType::End {
                    break;
                }
                let name = input.read_utf()?;
                let value = read_payload(tag, input)?;
                entries.push((name, value));
            }
            Ok(NbtValue::Compound(entries))
        }
        other => Err(format!("unsupported NBT tag type {other:?} in minimap packets")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nbt_round_trip_keeps_all_packet_types() {
        let mut entries = Vec::new();
        put_bool(&mut entries, "r", true);
        put_double(&mut entries, "x", 12.5);
        put_string(&mut entries, "d", "minecraft:overworld");
        entries.push((
            "i".into(),
            NbtValue::IntArray(vec![1, 2, 3, 4]),
        ));
        entries.push(("l".into(), NbtValue::LongArray(vec![5, 6])));
        entries.push(("i2".into(), NbtValue::Int(42)));
        entries.push((
            "o".into(),
            NbtValue::List(vec![
                NbtValue::String("one".into()),
                NbtValue::String("two".into()),
            ]),
        ));
        let value = NbtValue::Compound(entries);

        let mut writer = PacketWriter::new();
        write_root(&value, &mut writer);
        let bytes = writer.into_bytes();
        let mut reader = PacketReader::new(&bytes);
        let decoded = read_root(&mut reader).unwrap();
        assert_eq!(decoded, value);
        assert!(reader.is_empty());
    }

    #[test]
    fn root_tag_is_unnamed_and_uses_the_nbt_ids() {
        let mut writer = PacketWriter::new();
        write_root(&NbtValue::Compound(vec![(
            "cm".into(),
            NbtValue::Byte(1),
        )]), &mut writer);
        let bytes = writer.into_bytes();
        assert_eq!(
            bytes,
            vec![
                10, // TAG_Compound
                0, 0, // empty root name
                1, 0, 2, b'c', b'm', // TAG_Byte, name length 2, "cm"
                1, // true
                0, // end of compound
            ]
        );
    }
}
