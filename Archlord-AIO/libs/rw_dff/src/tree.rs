use crate::policy::ContainerPolicy;
use crate::reader::{RwChunkHeader, RwChunkReader, RwReadError};
use std::io::{Read, Seek};

/// Node in the RW chunk tree.
///
/// # Fields
/// - `header_off`: absolute file offset of the 12-byte header.
/// - `payload_off`: absolute file offset where the payload begins (header_off + 12).
/// - `payload_end`: absolute file offset where the payload ends (payload_off + header.size).
#[derive(Debug, Clone)]
pub struct RwChunkNode {
    pub header: RwChunkHeader,
    pub header_off: u64,
    pub payload_off: u64,
    pub payload_end: u64,
    pub children: Vec<RwChunkNode>,
}

impl RwChunkNode {
    /// Returns the total size of this chunk on disk (header + payload).
    pub fn total_size(&self) -> u64 {
        self.payload_end - self.header_off
    }
}

/// Parses a RW stream into a tree starting at the current position.
///
/// # Behavior
/// - Reads a single root chunk header at current offset.
/// - Validates bounds.
/// - If the chunk is a container, parses children within the payload range.
/// - If the chunk is an embedded stream container, parses child chunks as a RW stream
///   starting at payload_off, continuing until payload_end.
/// - Unknown chunks become leaf nodes.
pub fn parse_root<R: Read + Seek>(
    rdr: &mut RwChunkReader<R>,
    policy: &ContainerPolicy,
) -> Result<RwChunkNode, RwReadError> {
    let header_off = rdr.position()?;
    let header = rdr.read_header()?;
    let payload_off = header_off + 12;
    let payload_end = rdr.validate_bounds(header_off, header.size)?;

    let mut node = RwChunkNode {
        header,
        header_off,
        payload_off,
        payload_end,
        children: Vec::new(),
    };

    // Parse children depending on container style
    if policy.is_embedded_stream(header.id) {
        node.children = parse_stream_range(rdr, policy, payload_off, payload_end)?;
    } else if policy.is_container(header.id) {
        node.children = parse_children_in_payload(rdr, policy, payload_off, payload_end)?;
    } else if policy.looks_like_embedded_stream(rdr, payload_off, payload_end)? {
        node.children = parse_stream_range(rdr, policy, payload_off, payload_end)?;
    }

    // Finally, seek to end of this chunk so the caller can continue.
    rdr.seek_to(payload_end)?;
    Ok(node)
}

/// Parses child chunks inside a container payload as sequential chunks.
///
/// # Behavior
/// - Seeks to payload_off and reads chunks until payload_end.
/// - Each child chunk is parsed recursively by `parse_root`.
fn parse_children_in_payload<R: Read + Seek>(
    rdr: &mut RwChunkReader<R>,
    policy: &ContainerPolicy,
    payload_off: u64,
    payload_end: u64,
) -> Result<Vec<RwChunkNode>, RwReadError> {
    let mut children = Vec::new();
    rdr.seek_to(payload_off)?;

    while rdr.position()? < payload_end {
        let pos = rdr.position()?;
        if payload_end - pos < 12 {
            // trailing bytes (padding) are uncommon; treat as error to stay strict
            return Err(RwReadError::UnexpectedEof);
        }
        let child = parse_root(rdr, policy)?;
        children.push(child);
    }

    Ok(children)
}

/// Parses an embedded RW stream within a payload range.
///
/// # Behavior
/// - Treats payload bytes as a standalone RW stream: sequence of chunk headers.
/// - Stops exactly at payload_end.
fn parse_stream_range<R: Read + Seek>(
    rdr: &mut RwChunkReader<R>,
    policy: &ContainerPolicy,
    payload_off: u64,
    payload_end: u64,
) -> Result<Vec<RwChunkNode>, RwReadError> {
    let mut children = Vec::new();
    rdr.seek_to(payload_off)?;

    while rdr.position()? < payload_end {
        let pos = rdr.position()?;
        if payload_end - pos < 12 {
            return Err(RwReadError::EmbeddedSanityFailed { off: pos });
        }
        let child = parse_root(rdr, policy)?;
        children.push(child);
    }

    Ok(children)
}
