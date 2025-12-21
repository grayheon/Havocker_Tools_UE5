use crate::util::{DecodeError, LeReader};
use serde::Serialize;

/// Decoded header fields of a RenderWare RpGeometry struct payload.
///
/// # Notes
/// This decoder targets classic "PI geometry" layouts used in many DFF files.
/// It does not attempt to decode platform-native geometry (rpGEOMETRYNATIVE).
#[derive(Debug, Clone, Serialize)]
pub struct GeometryStructSummary {
    pub flags: u32,
    pub num_triangles: u32,
    pub num_vertices: u32,
    pub num_morph_targets: u32,

    /// Number of texcoord sets as encoded in `flags`.
    pub num_uv_sets: u8,

    /// Whether prelit colors are present.
    pub has_prelit: bool,

    /// Whether normals are present.
    pub has_normals: bool,

    /// Byte lengths of major arrays (if present).
    pub uv_bytes: usize,
    pub prelit_bytes: usize,
    pub triangle_bytes: usize,
    pub morph_bytes: usize,

    /// Remaining undecoded bytes (should be 0 for known layouts).
    pub remaining_bytes: usize,
}

/// Decodes a Geometry Struct payload and returns a validated summary.
///
/// # Behavior
/// - Reads the 16-byte geometry header (flags, triCount, vertCount, morphCount).
/// - Derives uv-set count from flags (high byte at bits 16..23).
/// - Computes expected array sizes for UVs, prelit, triangles, morph target blocks.
/// - Advances the reader accordingly and reports remaining bytes.
///
/// # Safety/Correctness
/// - Does strict bounds checks.
/// - Does not guess unknown layouts; remaining bytes signal "needs investigation".
pub fn decode_geometry_struct(buf: &[u8]) -> Result<GeometryStructSummary, DecodeError> {
    let mut r = LeReader::new(buf);

    let flags = r.read_u32("geometry.flags")?;
    let num_triangles = r.read_u32("geometry.num_triangles")?;
    let num_vertices = r.read_u32("geometry.num_vertices")?;
    let num_morph_targets = r.read_u32("geometry.num_morph_targets")?;

    let num_uv_sets = ((flags >> 16) & 0xFF) as u8;
    let has_prelit = (flags & 0x0000_0008) != 0; // rpGEOMETRYPRELIT (classic bit)
    let has_normals = (flags & 0x0000_0010) != 0; // rpGEOMETRYNORMALS (classic bit)

    // Arrays follow in this order for classic PI geometry:
    // - texture coordinates for each set: num_vertices * sizeof(RwTexCoords=8)
    // - prelit colors (if present): num_vertices * 4 (RGBA)
    // - triangles: num_triangles * 8 (u16 idx0, u16 idx1, u16 idx2, u16 matIndex)
    // - morph targets: repeated blocks with sphere + flags + vertices + normals
    let uv_bytes = (num_vertices as usize)
        .saturating_mul(num_uv_sets as usize)
        .saturating_mul(8);

    let prelit_bytes = if has_prelit {
        (num_vertices as usize).saturating_mul(4)
    } else {
        0
    };

    let triangle_bytes = (num_triangles as usize).saturating_mul(8);

    // Morph target block layout (classic):
    // - sphere (RwSphere): 16 bytes (x,y,z,r)
    // - hasVertices (u32), hasNormals (u32)
    // - vertices array: num_vertices * 12 if hasVertices != 0
    // - normals array: num_vertices * 12 if hasNormals != 0
    // We cannot know hasVertices/hasNormals without reading them per target.
    // So we decode each target deterministically.
    r.read_bytes("geometry.uvs", uv_bytes)?;
    r.read_bytes("geometry.prelit", prelit_bytes)?;
    r.read_bytes("geometry.triangles", triangle_bytes)?;

    let mut morph_bytes = 0usize;
    for mt in 0..num_morph_targets {
        let _sphere = r.read_bytes("geometry.morph.sphere", 16)?;
        let has_vertices = r.read_u32("geometry.morph.has_vertices")?;
        let has_normals = r.read_u32("geometry.morph.has_normals")?;

        morph_bytes += 16 + 4 + 4;

        if has_vertices != 0 {
            let vb = (num_vertices as usize).saturating_mul(12);
            r.read_bytes("geometry.morph.vertices", vb)?;
            morph_bytes += vb;
        }
        if has_normals != 0 {
            let nb = (num_vertices as usize).saturating_mul(12);
            r.read_bytes("geometry.morph.normals", nb)?;
            morph_bytes += nb;
        }

        // mt is not used now but the loop is intentionally explicit.
        let _ = mt;
    }

    Ok(GeometryStructSummary {
        flags,
        num_triangles,
        num_vertices,
        num_morph_targets,
        num_uv_sets,
        has_prelit,
        has_normals,
        uv_bytes,
        prelit_bytes,
        triangle_bytes,
        morph_bytes,
        remaining_bytes: r.remaining(),
    })
}
