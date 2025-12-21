use crate::geometry_full::decode_geometry_struct_full;
use crate::scan::scan_geometry_structs;
use crate::unified::{build_helper_mesh, build_unified_mesh, HelperMesh, UnifiedMesh};
use rw_dff::ids::ids;
use rw_dff::parse_file;
use rw_dff::tree::RwChunkNode;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Unified report for one file.
#[derive(Debug, Serialize)]
pub struct UnifiedReport {
    pub file: String,
    pub meshes: Vec<UnifiedMesh>,
    pub helpers: Vec<HelperMesh>,
}

/// Builds a UnifiedReport by re-decoding Geometry Struct payloads into full arrays.
///
/// # Behavior
/// - Uses the chunk tree to locate Geometry->Struct payloads.
/// - Uses the existing model scan to reuse material decoding results.
/// - Helper geometries are separated into `helpers`.
pub fn build_unified_report(path: &Path) -> Result<UnifiedReport, UnifiedError> {
    let root = parse_file(path)?;
    let file_bytes = fs::read(path)?;

    // Reuse the existing scan (has material_list and helper flag).
    let model_entries = scan_geometry_structs(path, &root)?;

    // Map geometry_header_off -> GeometryNode to access struct payload offsets
    let mut meshes = Vec::new();
    let mut helpers = Vec::new();

    for entry in &model_entries {
        let geo_node = find_node_by_header_off(&root, entry.geometry_header_off)
            .ok_or(UnifiedError::MissingGeometryNode { off: entry.geometry_header_off })?;

        // Find Struct child
        let struct_node = geo_node.children.iter().find(|c| c.header.id == ids::RW_STRUCT)
            .ok_or(UnifiedError::MissingStruct { off: entry.geometry_header_off })?;

        let payload = crate::plugins::payload_slice(struct_node, &file_bytes)
            .map_err(UnifiedError::Decode)?;

        let geo_full = decode_geometry_struct_full(payload)
            .map_err(UnifiedError::Decode)?;

        if entry.is_helper_geometry {
            helpers.push(build_helper_mesh(entry.geometry_header_off, &geo_full));
        } else {
            meshes.push(build_unified_mesh(entry.geometry_header_off, &geo_full, entry.material_list.as_ref()));
        }
    }

    Ok(UnifiedReport {
        file: path.display().to_string(),
        meshes,
        helpers,
    })
}

/// Finds a node in the tree by its header offset.
///
/// # Behavior
/// - Performs a full traversal.
/// - Returns the first match.
fn find_node_by_header_off(node: &RwChunkNode, off: u64) -> Option<&RwChunkNode> {
    if node.header_off == off {
        return Some(node);
    }
    for c in &node.children {
        if let Some(found) = find_node_by_header_off(c, off) {
            return Some(found);
        }
    }
    None
}

/// Errors produced while building a unified report.
#[derive(Debug, thiserror::Error)]
pub enum UnifiedError {
    #[error("rw tree parse error: {0}")]
    Tree(#[from] rw_dff::reader::RwReadError),

    /// 👇 DAS FEHLTE
    #[error("scan/model error: {0}")]
    Scan(#[from] crate::scan::ScanError),

    #[error("missing Geometry node at 0x{off:X}")]
    MissingGeometryNode { off: u64 },

    #[error("missing Struct child for Geometry at 0x{off:X}")]
    MissingStruct { off: u64 },

    #[error("decode error: {0}")]
    Decode(#[from] crate::util::DecodeError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

