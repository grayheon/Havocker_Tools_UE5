use crate::ids::ids;
use crate::reader::{RwChunkReader, RwReadError};
use std::collections::HashSet;
use std::io::{Read, Seek};

/// Policy to decide whether a chunk contains nested RW chunks.
///
/// # Design
/// - Keep the policy configurable to avoid hardcoding engine specifics everywhere.
/// - Support special "embedded stream" chunks (e.g., SkylineAtomic) where the payload
///   begins with a valid RW chunk header sequence.
#[derive(Debug, Clone)]
pub struct ContainerPolicy {
    /// IDs that are treated as regular containers (children are inside the payload).
    pub containers: HashSet<u32>,
    /// IDs whose payload is treated as an embedded RW stream.
    pub embedded_streams: HashSet<u32>,
    /// If true: attempt embedded-stream detection heuristically for unknown IDs.
    pub allow_heuristic_embedded: bool,
}

impl ContainerPolicy {
    /// Returns the default policy for Archlord DFF inspection.
    ///
    /// # Behavior
    /// - Enables all standard RW containers needed for DFF/TXD trees.
    /// - Marks SkylineAtomic as an embedded stream container.
    pub fn archlord_default() -> Self {
        let mut containers = HashSet::new();
        // Standard RW containers
        containers.insert(ids::RW_CLUMP);
        containers.insert(ids::RW_FRAMELIST);
        containers.insert(ids::RW_GEOMETRYLIST);
        containers.insert(ids::RW_GEOMETRY);
        containers.insert(ids::RW_ATOMIC);
        containers.insert(ids::RW_MATERIALLIST);
        containers.insert(ids::RW_MATERIAL);
        containers.insert(ids::RW_TEXTURE);
        containers.insert(ids::RW_EXTENSION);
        containers.insert(ids::RW_TEXTDICTIONARY);
        containers.insert(ids::RW_TEXTNATIVE);

        let mut embedded_streams = HashSet::new();
        embedded_streams.insert(ids::SKYLINE_ATOMIC);

        Self {
            containers,
            embedded_streams,
            allow_heuristic_embedded: false,
        }
    }

    /// Checks whether `id` is a normal container.
    pub fn is_container(&self, id: u32) -> bool {
        self.containers.contains(&id)
    }

    /// Checks whether `id` is an embedded stream container.
    pub fn is_embedded_stream(&self, id: u32) -> bool {
        self.embedded_streams.contains(&id)
    }

    /// Heuristically checks if the payload at `payload_off` looks like a RW stream.
    ///
    /// # Behavior
    /// - Peeks 12 bytes and checks for a plausible RW header.
    /// - This is intentionally conservative to avoid false positives.
    pub fn looks_like_embedded_stream<R: Read + Seek>(
        &self,
        rdr: &mut RwChunkReader<R>,
        payload_off: u64,
        payload_end: u64,
    ) -> Result<bool, RwReadError> {
        if !self.allow_heuristic_embedded {
            return Ok(false);
        }

        // Need at least 12 bytes for a header.
        if payload_end <= payload_off + 12 {
            return Ok(false);
        }

        rdr.seek_to(payload_off)?;
        let peek = rdr.read_peek(12)?;
        let id = u32::from_le_bytes([peek[0], peek[1], peek[2], peek[3]]);
        let size = u32::from_le_bytes([peek[4], peek[5], peek[6], peek[7]]);
        // version is available, but we keep the check minimal.

        // Very conservative sanity:
        // - id must not be zero
        // - size must not exceed payload bounds
        if id == 0 {
            return Ok(false);
        }

        let inner_end = payload_off
            .checked_add(12)
            .and_then(|v| v.checked_add(size as u64))
            .ok_or(RwReadError::SizeOverflow)?;

        Ok(inner_end <= payload_end)
    }
}
