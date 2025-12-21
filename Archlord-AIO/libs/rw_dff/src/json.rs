use crate::ids::ids;
use crate::tree::RwChunkNode;
use serde::Serialize;

/// A stable JSON representation for golden-file regression tests.
///
/// # Design
/// - Keep it simple and deterministic.
/// - Avoid including non-essential derived fields that could change.
#[derive(Debug, Serialize)]
pub struct ChunkJson {
    pub id: u32,
    pub name: String,
    pub version: u32,
    pub header_off: u64,
    pub size: u32,
    pub children: Vec<ChunkJson>,
}

/// Converts a node into a stable JSON structure.
///
/// # Behavior
/// - Recursively maps children.
/// - Name is derived from id mapping (useful in diffs).
pub fn to_json(node: &RwChunkNode) -> ChunkJson {
    ChunkJson {
        id: node.header.id,
        name: ids::chunk_name(node.header.id).to_string(),
        version: node.header.version,
        header_off: node.header_off,
        size: node.header.size,
        children: node.children.iter().map(to_json).collect(),
    }
}
