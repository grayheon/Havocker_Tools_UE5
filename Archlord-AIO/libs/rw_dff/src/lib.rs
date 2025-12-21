//! RenderWare DFF/TXD chunk-tree parser.
//!
//! # Scope
//! - Parses RW chunk headers into a tree.
//! - Handles Archlord embedded stream chunks (SkylineAtomic).
//! - Does not interpret Struct payloads (that belong to the model layer).

pub mod ids;
pub mod json;
pub mod policy;
pub mod reader;
pub mod tree;

use crate::policy::ContainerPolicy;
use crate::reader::{RwChunkReader, RwReadError};
use crate::tree::RwChunkNode;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

/// Parses a DFF/TXD file and returns the chunk tree root.
///
/// # Behavior
/// - Uses `ContainerPolicy::archlord_default()`.
/// - Reads the root chunk from file start (offset 0).
pub fn parse_file(path: &Path) -> Result<RwChunkNode, RwReadError> {
    let file = File::open(path)?;
    parse_reader(file, &ContainerPolicy::archlord_default())
}

/// Parses from any `Read+Seek` with an explicit container policy.
///
/// # Behavior
/// - Starts at position 0.
pub fn parse_reader<R: Read + Seek>(
    mut reader: R,
    policy: &ContainerPolicy,
) -> Result<RwChunkNode, RwReadError> {
    use std::io::SeekFrom;

    reader.seek(SeekFrom::Start(0))?;
    let mut rdr = RwChunkReader::new(reader)?;
    rdr.seek_to(0)?;
    tree::parse_root(&mut rdr, policy)
}
