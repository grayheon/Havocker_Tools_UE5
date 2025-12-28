/// A simple binary buffer builder for glTF `.bin` files.
///
/// # Behavior
/// - Appends typed data to an internal Vec<u8>.
/// - Supports 4-byte alignment (recommended by glTF).
/// - Returns byte ranges for creating BufferViews/Accessors.
///
/// # Notes
/// We keep this minimal and deterministic to avoid dependency bloat.
pub struct BinBuilder {
    data: Vec<u8>,
}

impl BinBuilder {
    /// Creates an empty BIN builder.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Returns the current byte length.
    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }

    /// Returns the underlying bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// Pads with zero bytes until 4-byte aligned.
    pub fn align4(&mut self) {
        while !self.data.len().is_multiple_of(4) {
            self.data.push(0);
        }
    }

    /// Appends f32 slice in little-endian and returns (offset, length).
    pub fn push_f32(&mut self, values: &[f32]) -> (u64, u64) {
        self.align4();
        let off = self.len();
        for &v in values {
            self.data.extend_from_slice(&v.to_le_bytes());
        }
        let len = (values.len() * 4) as u64;
        (off, len)
    }

    /// Appends u32 slice in little-endian and returns (offset, length).
    pub fn push_u32(&mut self, values: &[u32]) -> (u64, u64) {
        self.align4();
        let off = self.len();
        for &v in values {
            self.data.extend_from_slice(&v.to_le_bytes());
        }
        let len = (values.len() * 4) as u64;
        (off, len)
    }

    /// Appends u16 slice in little-endian and returns (offset, length).
    pub fn push_u16(&mut self, values: &[u16]) -> (u64, u64) {
        self.align4();
        let off = self.len();
        for &v in values {
            self.data.extend_from_slice(&v.to_le_bytes());
        }
        let len = (values.len() * 2) as u64;
        (off, len)
    }
}
