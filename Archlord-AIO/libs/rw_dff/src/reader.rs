use std::io::{Read, Seek, SeekFrom};
use thiserror::Error;

/// RW chunk header as stored on disk (little-endian).
///
/// # Layout
/// - id: u32
/// - size: u32 (payload size, excluding this 12-byte header)
/// - version: u32
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RwChunkHeader {
    pub id: u32,
    pub size: u32,
    pub version: u32,
}

/// Reader errors for parsing RW streams.
///
/// The goal is to fail loudly and early for malformed files, while still
/// allowing the caller to choose conservative policies for unknown chunks.
#[derive(Debug, Error)]
pub enum RwReadError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unexpected EOF while reading chunk header")]
    UnexpectedEof,

    #[error("chunk payload exceeds file bounds: header at 0x{header_off:X}, payload_end=0x{payload_end:X}, file_len=0x{file_len:X}")]
    OutOfBounds {
        header_off: u64,
        payload_end: u64,
        file_len: u64,
    },

    #[error("invalid size: payload_end overflow")]
    SizeOverflow,

    #[error("embedded stream sanity check failed at 0x{off:X}")]
    EmbeddedSanityFailed { off: u64 },
}

/// Low-level RW stream reader.
///
/// This provides primitive operations to read a RW chunk header and to seek.
/// Higher-level tree parsing lives in `tree.rs`.
pub struct RwChunkReader<R: Read + Seek> {
    inner: R,
    file_len: u64,
}

impl<R: Read + Seek> RwChunkReader<R> {
    /// Creates a new reader and captures the file length.
    ///
    /// # Behavior
    /// - Seeks to end to determine length, then returns to start position 0.
    pub fn new(mut inner: R) -> Result<Self, RwReadError> {
        // Save the current position (should be 0 in typical use).
        let _ = inner.seek(SeekFrom::Start(0))?;
        let file_len = inner.seek(SeekFrom::End(0))?;
        inner.seek(SeekFrom::Start(0))?;
        Ok(Self { inner, file_len })
    }

    /// Returns the total file length in bytes.
    pub fn file_len(&self) -> u64 {
        self.file_len
    }

    /// Returns the current stream position.
    pub fn position(&mut self) -> Result<u64, RwReadError> {
        Ok(self.inner.seek(SeekFrom::Current(0))?)
    }

    /// Seeks to an absolute position.
    pub fn seek_to(&mut self, pos: u64) -> Result<(), RwReadError> {
        self.inner.seek(SeekFrom::Start(pos))?;
        Ok(())
    }

    /// Reads exactly 12 bytes and decodes a RW chunk header (little-endian).
    ///
    /// # Behavior
    /// - If fewer than 12 bytes are available, returns `UnexpectedEof`.
    pub fn read_header(&mut self) -> Result<RwChunkHeader, RwReadError> {
        let mut buf = [0u8; 12];
        let mut read_total = 0usize;
        while read_total < buf.len() {
            let n = self.inner.read(&mut buf[read_total..])?;
            if n == 0 {
                return Err(RwReadError::UnexpectedEof);
            }
            read_total += n;
        }

        let id = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let size = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let version = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        Ok(RwChunkHeader { id, size, version })
    }

    /// Validates that a chunk's payload end does not exceed the file length.
    ///
    /// # Behavior
    /// - Computes `payload_end = header_off + 12 + size`.
    /// - Returns `OutOfBounds` if payload_end > file_len.
    pub fn validate_bounds(&self, header_off: u64, size: u32) -> Result<u64, RwReadError> {
        let payload_end = header_off
            .checked_add(12)
            .and_then(|v| v.checked_add(size as u64))
            .ok_or(RwReadError::SizeOverflow)?;

        if payload_end > self.file_len {
            return Err(RwReadError::OutOfBounds {
                header_off,
                payload_end,
                file_len: self.file_len,
            });
        }
        Ok(payload_end)
    }

    /// Reads a small slice for sanity checks without allocating large buffers.
    ///
    /// # Behavior
    /// - Reads `len` bytes from the current position.
    /// - Caller should seek as needed before calling.
    pub fn read_peek(&mut self, len: usize) -> Result<Vec<u8>, RwReadError> {
        let mut v = vec![0u8; len];
        self.inner.read_exact(&mut v)?;
        Ok(v)
    }
}
