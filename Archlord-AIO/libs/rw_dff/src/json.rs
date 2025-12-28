use crate::ids;
use crate::plugins;
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
    pub payload_off: u64,
    pub payload_end: u64,
    pub payload_preview_hex: String,
    pub children_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hanim: Option<HAnimJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<SkinJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_list: Option<FrameListJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeometryJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_list: Option<MaterialListJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<MaterialJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binmesh: Option<BinMeshJson>,
    pub children: Vec<ChunkJson>,
}

/// Converts a node into a stable JSON structure.
///
/// # Behavior
/// - Recursively maps children.
/// - Includes a small payload preview (first 64 bytes) as hex for forensics.
/// - Name is derived from id mapping (useful in diffs).
pub fn to_json(node: &RwChunkNode, file_bytes: &[u8]) -> ChunkJson {
    to_json_with_parent(node, file_bytes, None)
}

fn to_json_with_parent(node: &RwChunkNode, file_bytes: &[u8], parent_id: Option<u32>) -> ChunkJson {
    let preview = payload_preview_hex(node, file_bytes);
    let hanim = decode_hanim(node, file_bytes);
    let skin = decode_skin(node, file_bytes);
    let frame_list = decode_frame_list(node, parent_id, file_bytes);
    let geometry = decode_geometry(node, parent_id, file_bytes);
    let material_list = decode_material_list(node, parent_id, file_bytes);
    let material = decode_material(node, parent_id, file_bytes);
    let binmesh = decode_binmesh(node, file_bytes);
    let children: Vec<ChunkJson> = node
        .children
        .iter()
        .map(|c| to_json_with_parent(c, file_bytes, Some(node.header.id)))
        .collect();

    ChunkJson {
        id: node.header.id,
        name: ids::chunk_name(node.header.id).to_string(),
        version: node.header.version,
        header_off: node.header_off,
        size: node.header.size,
        payload_off: node.payload_off,
        payload_end: node.payload_end,
        payload_preview_hex: preview,
        children_count: children.len(),
        hanim,
        skin,
        frame_list,
        geometry,
        material_list,
        material,
        binmesh,
        children,
    }
}

fn payload_preview_hex(node: &RwChunkNode, file_bytes: &[u8]) -> String {
    let start = node.payload_off as usize;
    let end = node
        .payload_end
        .min((start + 64) as u64)
        .min(file_bytes.len() as u64) as usize;
    if start >= end || start >= file_bytes.len() {
        return String::new();
    }
    file_bytes[start..end]
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_hanim(node: &RwChunkNode, bytes: &[u8]) -> Option<HAnimJson> {
    if node.header.id != ids::RW_HANIM {
        return None;
    }
    let payload = slice_payload(node, bytes)?;
    plugins::decode_hanim(payload).ok().map(|h| HAnimJson {
        root_id: h.root_id,
        hierarchy_id: h.hierarchy_id,
        bone_count: h.bone_count,
        flags: h.flags,
        max_key_frame_size: h.max_key_frame_size,
        nodes: h.nodes,
    })
}

fn decode_skin(node: &RwChunkNode, bytes: &[u8]) -> Option<SkinJson> {
    if node.header.id != ids::RW_SKIN_PLG {
        return None;
    }
    let payload = slice_payload(node, bytes)?;
    plugins::decode_skin(payload).ok().map(|s| SkinJson {
        bone_count: s.bone_count,
        used_bone_count: s.used_bone_count,
        max_weights: s.max_weights,
        header_byte3: s.header_byte3,
        used_bone_indices: s.used_bone_indices,
        bone_vertex_indices_len: s.bone_vertex_indices_len,
        vertex_weights_len: s.vertex_weights_len,
        bone_vertex_indices_preview: s.bone_vertex_indices_preview,
        vertex_weights_preview: s.vertex_weights_preview,
        vertex_weights_preview_f32: s.vertex_weights_preview_f32,
        vertex_section_len: s.vertex_section_len,
        weight_section_len: s.weight_section_len,
        vertex_count_guess: s.vertex_count_guess,
        matrix_format: s.matrix_format,
        inverse_bind_matrices_preview: s.inverse_bind_matrices_preview,
        header_words: s.header_words,
        preview_hex: s.preview_hex,
    })
}

fn decode_frame_list(
    node: &RwChunkNode,
    parent_id: Option<u32>,
    bytes: &[u8],
) -> Option<FrameListJson> {
    if node.header.id != ids::RW_STRUCT || parent_id != Some(ids::RW_FRAMELIST) {
        return None;
    }
    let payload = slice_payload(node, bytes)?;
    plugins::decode_frame_list(payload)
        .ok()
        .map(|f| FrameListJson {
            frame_count: f.frame_count,
            frames: f.frames,
        })
}

fn decode_geometry(
    node: &RwChunkNode,
    parent_id: Option<u32>,
    bytes: &[u8],
) -> Option<GeometryJson> {
    if node.header.id != ids::RW_STRUCT || parent_id != Some(ids::RW_GEOMETRY) {
        return None;
    }
    let payload = slice_payload(node, bytes)?;
    plugins::decode_geometry(payload)
        .ok()
        .map(|g| GeometryJson {
            flags: g.flags,
            triangles: g.triangles,
            vertices: g.vertices,
            morph_targets: g.morph_targets,
            has_normals: g.has_normals,
            has_prelit: g.has_prelit,
            has_texcoords: g.has_texcoords,
            bounding_sphere: g.bounding_sphere,
        })
}

fn decode_material_list(
    node: &RwChunkNode,
    parent_id: Option<u32>,
    bytes: &[u8],
) -> Option<MaterialListJson> {
    if node.header.id != ids::RW_STRUCT || parent_id != Some(ids::RW_MATERIALLIST) {
        return None;
    }
    let payload = slice_payload(node, bytes)?;
    plugins::decode_material_list(payload)
        .ok()
        .map(|m| MaterialListJson {
            material_count: m.material_count,
            unknown: m.unknown,
        })
}

fn decode_material(
    node: &RwChunkNode,
    parent_id: Option<u32>,
    bytes: &[u8],
) -> Option<MaterialJson> {
    if node.header.id != ids::RW_STRUCT || parent_id != Some(ids::RW_MATERIAL) {
        return None;
    }
    let payload = slice_payload(node, bytes)?;
    plugins::decode_material(payload)
        .ok()
        .map(|m| MaterialJson {
            flags: m.flags,
            color: m.color,
            unknown0: m.unknown0,
            texture_count: m.texture_count,
            surface_props: m.surface_props,
        })
}

fn decode_binmesh(node: &RwChunkNode, bytes: &[u8]) -> Option<BinMeshJson> {
    if node.header.id != ids::BINMESH_PLG {
        return None;
    }
    let payload = slice_payload(node, bytes)?;
    plugins::decode_binmesh(payload).ok().map(|b| BinMeshJson {
        type_code: b.type_code,
        mesh_count: b.mesh_count,
        total_indices: b.total_indices,
        meshes: b.meshes,
        indices_preview: b.indices_preview,
    })
}

fn slice_payload<'a>(node: &RwChunkNode, bytes: &'a [u8]) -> Option<&'a [u8]> {
    let start = node.payload_off as usize;
    let end = node.payload_end.min(bytes.len() as u64) as usize;
    if start >= end || start >= bytes.len() {
        return None;
    }
    Some(&bytes[start..end])
}

#[derive(Debug, Serialize)]
pub struct HAnimJson {
    pub root_id: u32,
    pub hierarchy_id: u32,
    pub bone_count: u32,
    pub flags: u32,
    pub max_key_frame_size: u32,
    pub nodes: Vec<plugins::HAnimNode>,
}

#[derive(Debug, Serialize)]
pub struct SkinJson {
    pub bone_count: u32,
    pub used_bone_count: u32,
    pub max_weights: u32,
    pub header_byte3: u8,
    pub used_bone_indices: Vec<u32>,
    pub bone_vertex_indices_len: usize,
    pub vertex_weights_len: usize,
    pub bone_vertex_indices_preview: Vec<u8>,
    pub vertex_weights_preview: Vec<u8>,
    pub vertex_weights_preview_f32: Vec<f32>,
    pub vertex_section_len: usize,
    pub weight_section_len: usize,
    pub vertex_count_guess: Option<usize>,
    pub matrix_format: Option<String>,
    pub inverse_bind_matrices_preview: Vec<Vec<f32>>,
    pub header_words: Vec<u32>,
    pub preview_hex: String,
}

#[derive(Debug, Serialize)]
pub struct FrameListJson {
    pub frame_count: u32,
    pub frames: Vec<plugins::FrameInfo>,
}

#[derive(Debug, Serialize)]
pub struct GeometryJson {
    pub flags: u32,
    pub triangles: u32,
    pub vertices: u32,
    pub morph_targets: u32,
    pub has_normals: bool,
    pub has_prelit: bool,
    pub has_texcoords: bool,
    pub bounding_sphere: Option<plugins::BoundingSphere>,
}

#[derive(Debug, Serialize)]
pub struct MaterialListJson {
    pub material_count: i32,
    pub unknown: i32,
}

#[derive(Debug, Serialize)]
pub struct MaterialJson {
    pub flags: i32,
    pub color: [u8; 4],
    pub unknown0: i32,
    pub texture_count: i32,
    pub surface_props: [f32; 3],
}

#[derive(Debug, Serialize)]
pub struct BinMeshJson {
    pub type_code: u32,
    pub mesh_count: u32,
    pub total_indices: u32,
    pub meshes: Vec<plugins::BinMeshSplit>,
    pub indices_preview: Vec<u32>,
}
