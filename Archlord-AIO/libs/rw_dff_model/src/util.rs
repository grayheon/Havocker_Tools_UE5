use thiserror::Error;

/// Errors returned by slice-based binary decoding.
#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("unexpected EOF while reading {what}, need {need} bytes, have {have}")]
    UnexpectedEof { what: &'static str, need: usize, have: usize },

    #[error("invalid value for {what}: {value}")]
    InvalidValue { what: &'static str, value: u64 },
}

/// A simple, fast little-endian reader over a byte slice.
///
/// # Design
/// - No allocations
/// - Deterministic
/// - Strict bounds checking
pub struct LeReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> LeReader<'a> {
    /// Creates a new reader over a byte slice.
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Returns the current cursor position.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns how many bytes remain.
    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    /// Returns the total length of the buffer.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Reads exactly `n` bytes.
    pub fn read_bytes(&mut self, what: &'static str, n: usize) -> Result<&'a [u8], DecodeError> {
        if self.remaining() < n {
            return Err(DecodeError::UnexpectedEof {
                what,
                need: n,
                have: self.remaining(),
            });
        }
        let start = self.pos;
        self.pos += n;
        Ok(&self.buf[start..start + n])
    }

    /// Reads a u32 little-endian.
    pub fn read_u32(&mut self, what: &'static str) -> Result<u32, DecodeError> {
        let b = self.read_bytes(what, 4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Reads an i32 little-endian.
    pub fn read_i32(&mut self, what: &'static str) -> Result<i32, DecodeError> {
        let b = self.read_bytes(what, 4)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Reads a f32 little-endian.
    pub fn read_f32(&mut self, what: &'static str) -> Result<f32, DecodeError> {
        let b = self.read_bytes(what, 4)?;
        Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Reads an u16 little-endian.
    ///
    /// # Behavior
    /// - Strict bounds check.
    pub fn read_u16(&mut self, what: &'static str) -> Result<u16, DecodeError> {
        let b = self.read_bytes(what, 2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
}
