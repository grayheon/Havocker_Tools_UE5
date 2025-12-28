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

    /// All indices (list or strip depending on flags).
    pub indices: Vec<u32>,
}

pub fn peek_binmesh_header(payload: &[u8]) -> Option<(u32, u32)> {
    // English comment: BinMeshPLG header is at least 8 bytes (flags + numMeshes).
    if payload.len() < 8 {
        return None;
    }
    let flags = u32::from_le_bytes(payload[0..4].try_into().ok()?);
    let num_meshes = u32::from_le_bytes(payload[4..8].try_into().ok()?);
    Some((flags, num_meshes))
}

/// Parses a BinMeshPLG payload according to the classic RW layout.
///
/// # Behavior
/// - Strict bounds checks.
/// - Collects only a small preview of indices for debugging.
/// - Returns remaining bytes count to detect layout mismatches.
pub fn decode_binmesh(payload: &[u8], _preview_count: usize) -> Result<BinMesh, DecodeError> {
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
        let mut indices = Vec::with_capacity(num_indices as usize);

        for _i in 0..num_indices {
            let idx = r.read_u32("binmesh.mesh.index")?;

            min_index = Some(min_index.map(|m| m.min(idx)).unwrap_or(idx));
            max_index = Some(max_index.map(|m| m.max(idx)).unwrap_or(idx));

            indices.push(idx);
        }

        meshes.push(BinMeshMesh {
            num_indices,
            material_index,
            min_index,
            max_index,
            indices,
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

/// A decoded BinMeshPLG block.
///
/// # Purpose
/// RenderWare's BinMeshPLG describes how a geometry is split into material-pure parts.
/// Many game pipelines treat this as the authoritative draw-call layout.
///
/// # Notes
/// - `flags & 1 != 0` typically indicates TRI_STRIP topology.
/// - Indices may be 16-bit or 32-bit depending on exporter/pipeline.
#[derive(Debug, Clone, Serialize)]
pub struct BinMeshPlg {
    pub flags: u32,
    pub index_size: usize,
    pub meshes: Vec<BinMeshPart>,
}

/// A single material-pure mesh part inside BinMeshPLG.
///
/// # Purpose
/// Represents one drawcall in the original pipeline (material slot + index stream).
#[derive(Debug, Clone, Serialize)]
pub struct BinMeshPart {
    pub material_index: u32,
    pub indices: Vec<u32>, // raw indices (strip or list depending on flags)
}

/// Decodes the BinMeshPLG payload.
///
/// # Behavior
/// - Reads `flags` and `num_meshes`.
/// - Reads mesh headers (num_indices, material_index).
/// - Infers index width (u16 vs u32) by remaining payload length.
/// - Returns raw indices per part (strip or list depending on flags).
pub fn decode_binmesh_plg(payload: &[u8]) -> Result<BinMeshPlg, DecodeError> {
    let mut r = LeReader::new(payload);

    let flags = r.read_u32("binmesh.flags")?;
    let num_meshes = r.read_u32("binmesh.num_meshes")? as usize;
    // The classic layout includes total_indices; several Archlord files follow that.
    // Consume it to keep the stream aligned with mesh headers.
    let _total_indices = r.read_u32("binmesh.total_indices").ok();

    let mut headers = Vec::with_capacity(num_meshes);
    for _ in 0..num_meshes {
        let num_indices = r.read_u32("binmesh.mesh.num_indices")? as usize;
        let mat_index = r.read_u32("binmesh.mesh.material_index")?;
        headers.push((num_indices, mat_index));
    }

    let total_indices: usize = headers.iter().map(|(n, _)| *n).sum();
    let remaining = payload.len().saturating_sub(r.pos());

    // English comment: Prefer the index width that exactly matches the remaining bytes.
    let index_size = if total_indices > 0 && remaining == total_indices * 2 {
        2
    } else if total_indices > 0 && remaining == total_indices * 4 {
        4
    } else {
        2 // historical default
    };

    let mut meshes = Vec::with_capacity(num_meshes);
    for (num_indices, mat_index) in headers {
        let mut indices = Vec::with_capacity(num_indices);
        if index_size == 2 {
            for _ in 0..num_indices {
                indices.push(r.read_u16("binmesh.idx16")? as u32);
            }
        } else {
            for _ in 0..num_indices {
                indices.push(r.read_u32("binmesh.idx32")?);
            }
        }
        meshes.push(BinMeshPart {
            material_index: mat_index,
            indices,
        });
    }

    Ok(BinMeshPlg {
        flags,
        index_size,
        meshes,
    })
}

/// Expands a triangle strip into a triangle list.
///
/// # Behavior
/// - Alternates winding each step.
/// - Skips degenerate triangles (also used as strip separators).
pub fn tristrip_to_trilist(strip: &[u32]) -> Vec<u32> {
    let mut out = Vec::new();
    if strip.len() < 3 {
        return out;
    }

    for i in 0..(strip.len() - 2) {
        let a = strip[i];
        let b = strip[i + 1];
        let c = strip[i + 2];

        // English comment: Degenerate triangles are skipped.
        if a == b || b == c || a == c {
            continue;
        }

        // English comment: Triangle strip parity flips winding.
        // glTF expects CCW. If original RW was CW, we flip it.
        if (i & 1) == 0 {
            out.extend_from_slice(&[a, c, b]);
        } else {
            out.extend_from_slice(&[b, c, a]);
        }
    }

    out
}
