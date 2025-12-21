use crate::util::{DecodeError, LeReader};
use serde::Serialize;

/// A decoded BinMesh plugin payload (classic RW layout).
///
/// # Layout (classic)
/// - flags: u32
/// - num_meshes: u32
/// - total_indices: u32
/// - for each mesh:
///   - num_indices: u32
///   - material_index: i32
///   - indices[num_indices]: u32
///
/// # Notes
/// Some games/engines store variants; we keep parsing strict and add plausibility checks.
#[derive(Debug, Clone, Serialize)]
pub struct BinMesh {
    pub flags: u32,
    pub num_meshes: u32,
    pub total_indices: u32,
    pub meshes: Vec<BinMeshMesh>,
    pub remaining_bytes: usize,
}

/// One mesh entry inside BinMesh.
#[derive(Debug, Clone, Serialize)]
pub struct BinMeshMesh {
    pub num_indices: u32,
    pub material_index: i32,

    /// Basic stats for debugging (do not store full arrays by default).
    pub min_index: Option<u32>,
    pub max_index: Option<u32>,

    /// First N indices for quick inspection.
    pub indices_preview: Vec<u32>,
}

/// Parses a BinMeshPLG payload according to the classic RW layout.
///
/// # Behavior
/// - Strict bounds checks.
/// - Collects only a small preview of indices for debugging.
/// - Returns remaining bytes count to detect layout mismatches.
pub fn decode_binmesh(payload: &[u8], preview_count: usize) -> Result<BinMesh, DecodeError> {
    let mut r = LeReader::new(payload);

    let flags = r.read_u32("binmesh.flags")?;
    let num_meshes = r.read_u32("binmesh.num_meshes")?;
    let total_indices = r.read_u32("binmesh.total_indices")?;

    let mut meshes = Vec::with_capacity(num_meshes as usize);

    for _ in 0..num_meshes {
        let num_indices = r.read_u32("binmesh.mesh.num_indices")?;
        let material_index = r.read_i32("binmesh.mesh.material_index")?;

        let mut min_index: Option<u32> = None;
        let mut max_index: Option<u32> = None;
        let mut indices_preview = Vec::new();

        for i in 0..num_indices {
            let idx = r.read_u32("binmesh.mesh.index")?;

            min_index = Some(min_index.map(|m| m.min(idx)).unwrap_or(idx));
            max_index = Some(max_index.map(|m| m.max(idx)).unwrap_or(idx));

            if (i as usize) < preview_count {
                indices_preview.push(idx);
            }
        }

        meshes.push(BinMeshMesh {
            num_indices,
            material_index,
            min_index,
            max_index,
            indices_preview,
        });
    }

    Ok(BinMesh {
        flags,
        num_meshes,
        total_indices,
        meshes,
        remaining_bytes: r.remaining(),
    })
}
