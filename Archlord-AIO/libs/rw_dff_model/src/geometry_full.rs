use crate::util::{DecodeError, LeReader};
use serde::Serialize;

/// Minimal vector types used in decoded geometry.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// One triangle from the classic RpGeometry triangle array.
///
/// # Layout on disk (8 bytes):
/// - u16 i0
/// - u16 i1
/// - u16 i2
/// - u16 material_index
#[derive(Debug, Clone, Copy, Serialize)]
pub struct RwTriangle {
    pub i0: u16,
    pub i1: u16,
    pub i2: u16,
    pub material_index: u16,
}

/// Decoded PI-geometry data from a Geometry Struct payload.
///
/// # Scope
/// - Extracts UV0 (if present), triangle list, first morph target positions, and normals.
/// - Ignores additional UV sets beyond UV0 for now (we can extend later).
/// - Ignores prelit colors for now (we can add later).
#[derive(Debug, Clone, Serialize)]
pub struct GeometryData {
    pub flags: u32,
    pub num_triangles: u32,
    pub num_vertices: u32,
    pub num_morph_targets: u32,
    pub num_uv_sets: u8,
    pub has_prelit: bool,
    pub has_normals: bool,

    pub uvs0: Vec<Vec2>,
    pub triangles: Vec<RwTriangle>,
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,

    pub remaining_bytes: usize,
}

/// Decodes a classic PI Geometry Struct payload into usable arrays.
///
/// # Behavior
/// - Strictly follows classic layout: UVs, prelit, triangles, morph targets.
/// - Only reads UV0 even if more UV sets exist (keeps it deterministic).
/// - Reads the first morph target's positions/normals (if present).
/// - Returns `remaining_bytes` to detect unknown trailing data.
pub fn decode_geometry_struct_full(payload: &[u8]) -> Result<GeometryData, DecodeError> {
    let mut r = LeReader::new(payload);

    let flags = r.read_u32("geometry.flags")?;
    let num_triangles = r.read_u32("geometry.num_triangles")?;
    let num_vertices = r.read_u32("geometry.num_vertices")?;
    let num_morph_targets = r.read_u32("geometry.num_morph_targets")?;

    let num_uv_sets = ((flags >> 16) & 0xFF) as u8;
    let has_prelit = (flags & 0x0000_0008) != 0;
    let has_normals = (flags & 0x0000_0010) != 0;

    // --- UVs ---
    // Classic layout stores all UV sets consecutively. We only take UV0.
    let mut uvs0 = Vec::new();
    if num_uv_sets > 0 {
        uvs0.reserve(num_vertices as usize);
        for _ in 0..num_vertices {
            let u = r.read_f32("geometry.uv0.u")?;
            let v = r.read_f32("geometry.uv0.v")?;
            uvs0.push(Vec2 { x: u, y: v });
        }

        // Skip remaining UV sets (if any) without decoding.
        if num_uv_sets > 1 {
            let skip_bytes = (num_vertices as usize)
                .saturating_mul((num_uv_sets as usize).saturating_sub(1))
                .saturating_mul(8);
            r.read_bytes("geometry.uvN.skip", skip_bytes)?;
        }
    } else {
        // No UVs: keep empty vec (caller decides fallback).
    }

    // --- Prelit ---
    if has_prelit {
        let skip = (num_vertices as usize).saturating_mul(4);
        r.read_bytes("geometry.prelit.skip", skip)?;
    }

    // --- Triangles ---
    let mut triangles = Vec::with_capacity(num_triangles as usize);
    for _ in 0..num_triangles {
        let i0 = r.read_u16("geometry.tri.i0")?;
        let i1 = r.read_u16("geometry.tri.i1")?;
        let i2 = r.read_u16("geometry.tri.i2")?;
        let mat = r.read_u16("geometry.tri.mat")?;
        triangles.push(RwTriangle {
            i0,
            i1,
            i2,
            material_index: mat,
        });
    }

    // --- Morph targets ---
    // We decode only the first morph target's positions/normals into arrays.
    let mut positions = Vec::new();
    let mut normals = Vec::new();

    for mt in 0..num_morph_targets {
        // sphere
        r.read_bytes("geometry.morph.sphere", 16)?;
        let has_vertices = r.read_u32("geometry.morph.has_vertices")?;
        let has_normals_mt = r.read_u32("geometry.morph.has_normals")?;

        let want_arrays = mt == 0;

        if has_vertices != 0 {
            if want_arrays {
                positions.reserve(num_vertices as usize);
                for _ in 0..num_vertices {
                    let x = r.read_f32("geometry.pos.x")?;
                    let y = r.read_f32("geometry.pos.y")?;
                    let z = r.read_f32("geometry.pos.z")?;
                    positions.push(Vec3 { x, y, z });
                }
            } else {
                let skip = (num_vertices as usize).saturating_mul(12);
                r.read_bytes("geometry.morph.vertices.skip", skip)?;
            }
        }

        if has_normals_mt != 0 {
            if want_arrays {
                normals.reserve(num_vertices as usize);
                for _ in 0..num_vertices {
                    let x = r.read_f32("geometry.nrm.x")?;
                    let y = r.read_f32("geometry.nrm.y")?;
                    let z = r.read_f32("geometry.nrm.z")?;
                    normals.push(Vec3 { x, y, z });
                }
            } else {
                let skip = (num_vertices as usize).saturating_mul(12);
                r.read_bytes("geometry.morph.normals.skip", skip)?;
            }
        }
    }

    // If normals are expected by flags but missing in morph target: keep empty and let the caller decide.
    let _ = has_normals;

    Ok(GeometryData {
        flags,
        num_triangles,
        num_vertices,
        num_morph_targets,
        num_uv_sets,
        has_prelit,
        has_normals,
        uvs0,
        triangles,
        positions,
        normals,
        remaining_bytes: r.remaining(),
    })
}
