//! Byte buffer helpers corresponding to Minecraft's `FriendlyByteBuf` methods
//! used by the minimap packets (`writeInt`, `writeBoolean`, `writeNbt`, …).

/// Binary writer for packet payloads.
#[derive(Debug, Clone, Default)]
pub struct PacketWriter {
    bytes: Vec<u8>,
}

impl PacketWriter {
    pub fn new() -> Self {
        PacketWriter::default()
    }

    pub fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(if value { 1 } else { 0 });
    }

    pub fn write_i16(&mut self, value: i16) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_f64(&mut self, value: f64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    /// `FriendlyByteBuf.writeUtf`: u16 byte length followed by UTF-8 bytes.
    pub fn write_utf(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_i16(bytes.len() as i16);
        self.bytes.extend_from_slice(bytes);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// Binary reader for packet payloads.
#[derive(Debug, Clone)]
pub struct PacketReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> PacketReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        PacketReader { bytes, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| "packet length overflow".to_string())?;
        if end > self.bytes.len() {
            return Err("packet ended unexpectedly".into());
        }
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    pub fn read_u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    pub fn read_bool(&mut self) -> Result<bool, String> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_i16(&mut self) -> Result<i16, String> {
        let bytes: [u8; 2] = self.take(2)?.try_into().unwrap();
        Ok(i16::from_be_bytes(bytes))
    }

    pub fn read_i32(&mut self) -> Result<i32, String> {
        let bytes: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(i32::from_be_bytes(bytes))
    }

    pub fn read_i64(&mut self) -> Result<i64, String> {
        let bytes: [u8; 8] = self.take(8)?.try_into().unwrap();
        Ok(i64::from_be_bytes(bytes))
    }

    pub fn read_f64(&mut self) -> Result<f64, String> {
        let bytes: [u8; 8] = self.take(8)?.try_into().unwrap();
        Ok(f64::from_be_bytes(bytes))
    }

    pub fn read_utf(&mut self) -> Result<String, String> {
        let len = self.read_i16()?;
        if len < 0 {
            return Err("negative UTF string length".into());
        }
        let bytes = self.take(len as usize)?;
        String::from_utf8(bytes.to_vec()).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_round_trip() {
        let mut writer = PacketWriter::new();
        writer.write_bool(true);
        writer.write_i32(-7);
        writer.write_i64(9);
        writer.write_f64(0.5);
        writer.write_utf("dimension");
        let mut reader = PacketReader::new(writer.bytes());
        assert!(reader.read_bool().unwrap());
        assert_eq!(reader.read_i32().unwrap(), -7);
        assert_eq!(reader.read_i64().unwrap(), 9);
        assert_eq!(reader.read_f64().unwrap(), 0.5);
        assert_eq!(reader.read_utf().unwrap(), "dimension");
        assert!(reader.is_empty());
    }

    #[test]
    fn truncated_packets_are_reported() {
        let mut reader = PacketReader::new(&[0x12]);
        assert!(reader.read_i32().is_err());
        assert!(reader.read_u8().is_ok());
        assert!(reader.read_u8().is_err());
    }
}
