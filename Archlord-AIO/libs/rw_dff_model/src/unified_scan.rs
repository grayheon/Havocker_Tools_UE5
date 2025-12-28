use crate::geometry_full::decode_geometry_struct_full;
use crate::scan::scan_geometry_structs;
use crate::skeleton::Skeleton;
use crate::unified::{HelperMesh, Submesh, UnifiedMesh, build_helper_mesh, build_unified_mesh};
use rw_dff::ids;
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
    pub skeleton: Option<Skeleton>,
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

    // Optional skeleton/HAnim data
    let skeleton = decode_skeleton(&root, &file_bytes).ok();

    // Map geometry_header_off -> GeometryNode to access struct payload offsets
    let mut meshes = Vec::new();
    let mut helpers = Vec::new();

    for entry in &model_entries {
        let geo_node = find_node_by_header_off(&root, entry.geometry_header_off).ok_or(
            UnifiedError::MissingGeometryNode {
                off: entry.geometry_header_off,
            },
        )?;

        // Find Struct child
        let struct_node = geo_node
            .children
            .iter()
            .find(|c| c.header.id == ids::RW_STRUCT)
            .ok_or(UnifiedError::MissingStruct {
                off: entry.geometry_header_off,
            })?;

        let payload = crate::plugins::payload_slice(struct_node, &file_bytes)
            .map_err(UnifiedError::Decode)?;

        let mat_count = entry
            .material_list
            .as_ref()
            .map(|ml| ml.material_count as u16);

        let geo_full =
            decode_geometry_struct_full(payload, mat_count).map_err(UnifiedError::Decode)?;

        if entry.is_helper_geometry {
            helpers.push(build_helper_mesh(entry.geometry_header_off, &geo_full));
        } else {
            // Prefer the classic BinMesh (full indices + correct material indices).
            // Fallback to BinMeshPLG, then to the triangle array.
            let submeshes: Vec<Submesh> = if let Some(bm) = entry.binmesh.as_ref() {
                let is_strip = (bm.flags & 1) != 0;

                bm.meshes
                    .iter()
                    .map(|part| Submesh {
                        material_index: part.material_index as u32,
                        indices: part.indices.clone(),
                        is_strip,
                    })
                    .collect()
            } else if let Some(bm) = entry.binmesh_plg.as_ref() {
                let is_strip = (bm.flags & 1) != 0;

                bm.meshes
                    .iter()
                    .map(|part| Submesh {
                        material_index: part.material_index,
                        indices: part.indices.clone(),
                        is_strip,
                    })
                    .collect()
            } else {
                // Fallback: group RW triangle array by material_index
                use std::collections::BTreeMap;

                let mut buckets: BTreeMap<u16, Vec<u32>> = BTreeMap::new();
                for t in &geo_full.triangles {
                    // Swap winding order for glTF (CCW)
                    buckets
                        .entry(t.material_index)
                        .or_default()
                        .extend_from_slice(&[t.i0 as u32, t.i2 as u32, t.i1 as u32]);
                }

                buckets
                    .into_iter()
                    .map(|(mat, idx)| Submesh {
                        material_index: mat as u32,
                        indices: idx,
                        is_strip: false,
                    })
                    .collect()
            };

            meshes.push(build_unified_mesh(
                entry.geometry_header_off,
                &geo_full,
                entry.material_list.as_ref(),
                submeshes,
                entry.skin.as_ref(),
            ));
        }
    }

    Ok(UnifiedReport {
        file: path.display().to_string(),
        meshes,
        helpers,
        skeleton,
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

fn decode_skeleton(root: &RwChunkNode, bytes: &[u8]) -> Result<Skeleton, UnifiedError> {
    // FrameList Struct
    let frame_struct = find_first_chunk(root, ids::RW_FRAMELIST, ids::RW_STRUCT);
    let frames = if let Some(node) = frame_struct {
        let payload = crate::plugins::payload_slice(node, bytes).map_err(UnifiedError::Decode)?;
        crate::skeleton::decode_frame_list(payload).map_err(UnifiedError::Decode)?
    } else {
        Vec::new()
    };

    // HAnim PLG
    let hanim_node = find_first_by_id(root, ids::RW_HANIM);
    let (root_id, hanim_nodes) = if let Some(node) = hanim_node {
        let payload = crate::plugins::payload_slice(node, bytes).map_err(UnifiedError::Decode)?;
        crate::skeleton::decode_hanim(payload).map_err(UnifiedError::Decode)?
    } else {
        (0, Vec::new())
    };

    if frames.is_empty() && hanim_nodes.is_empty() {
        return Err(UnifiedError::Decode(
            crate::util::DecodeError::UnexpectedEof {
                what: "skeleton not found",
                need: 0,
                have: 0,
            },
        ));
    }

    Ok(Skeleton {
        frames,
        hanim_nodes,
        root_id,
    })
}

fn find_first_chunk<'a>(
    node: &'a RwChunkNode,
    parent_id: u32,
    child_id: u32,
) -> Option<&'a RwChunkNode> {
    if node.header.id == parent_id {
        return node.children.iter().find(|c| c.header.id == child_id);
    }
    for c in &node.children {
        if let Some(n) = find_first_chunk(c, parent_id, child_id) {
            return Some(n);
        }
    }
    None
}

fn find_first_by_id<'a>(node: &'a RwChunkNode, id: u32) -> Option<&'a RwChunkNode> {
    if node.header.id == id {
        return Some(node);
    }
    for c in &node.children {
        if let Some(n) = find_first_by_id(c, id) {
            return Some(n);
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
