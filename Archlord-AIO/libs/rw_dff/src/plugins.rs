use crate::reader::RwReadError;

pub fn decode_hanim(payload: &[u8]) -> Result<HAnimInfo, RwReadError> {
    if payload.len() < 20 {
        return Err(RwReadError::UnexpectedEof);
    }

    // RenderWare HAnim header (see RpHAnimHierarchy)
    let root_id = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let hierarchy_id = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let bone_count = u32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
    let max_key_frame_size =
        u32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]);
    let flags = u32::from_le_bytes([payload[16], payload[17], payload[18], payload[19]]);

    // Node table: one entry per bone (id, index, flags/type)
    let mut nodes = Vec::new();
    let mut offs = 20usize;
    let node_stride = 12usize;
    for _ in 0..bone_count {
        if offs + node_stride > payload.len() {
            return Err(RwReadError::UnexpectedEof);
        }
        let nid = i32::from_le_bytes([
            payload[offs],
            payload[offs + 1],
            payload[offs + 2],
            payload[offs + 3],
        ]);
        let nindex = i32::from_le_bytes([
            payload[offs + 4],
            payload[offs + 5],
            payload[offs + 6],
            payload[offs + 7],
        ]);
        let nflags = i32::from_le_bytes([
            payload[offs + 8],
            payload[offs + 9],
            payload[offs + 10],
            payload[offs + 11],
        ]);

        nodes.push(HAnimNode {
            node_id: nid,
            node_index: nindex,
            flags: nflags,
            type_label: hanim_node_type_label(nflags).to_string(),
        });
        offs += node_stride;
    }

    Ok(HAnimInfo {
        root_id,
        hierarchy_id,
        bone_count,
        flags,
        max_key_frame_size,
        nodes_preview: nodes.iter().take(16).cloned().collect(),
        nodes,
    })
}

/// Decode Skin plugin with a best-effort layout: packed counts + lists + matrices.
/// We stay conservative (no heuristics into geometry), but expose lengths and previews.
pub fn decode_skin(payload: &[u8]) -> Result<SkinInfo, RwReadError> {
    if payload.len() < 4 {
        return Err(RwReadError::UnexpectedEof);
    }

    let bone_count = payload[0] as u32;
    let used_bone_count = payload[1] as u32;
    let max_weights = payload[2] as u32;
    let header_byte3 = payload[3];

    let header_words: Vec<u32> = payload
        .chunks_exact(4)
        .take(16)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();

    let used_end = 4usize + used_bone_count as usize;
    if used_end > payload.len() {
        return Err(RwReadError::UnexpectedEof);
    }
    let used_bone_indices = payload[4..used_end]
        .iter()
        .map(|b| *b as u32)
        .collect::<Vec<_>>();

    // align to 4-byte boundary
    let used_aligned = (used_end + 3) & !3;
    if used_aligned > payload.len() {
        return Err(RwReadError::UnexpectedEof);
    }

    // Remaining layout: [vertex-bone-indices][vertex-weights(f32)][inverse-bind-matrices]
    // Try canonical RW Skin layout: indices are u8, weights are f32, matrices are 3x4 or 4x4.
    let remaining_bytes = payload.len().saturating_sub(used_aligned);
    let mut matrix_stride = 0usize;
    let mut matrix_format = None;
    let mut vertex_section_len = 0usize;
    let mut weight_section_len = 0usize;
    let mut bone_vertex_indices: &[u8] = &[];
    let mut vertex_weights_bytes: &[u8] = &[];
    let mut inferred_vertex_count: Option<usize> = None;

    for (stride, label) in &[(64usize, "4x4"), (48usize, "3x4")] {
        let matrix_bytes = bone_count as usize * *stride;
        if remaining_bytes < matrix_bytes {
            continue;
        }
        let vertex_bytes = remaining_bytes - matrix_bytes;
        let per_vertex = max_weights as usize * (1 + 4); // 1 byte index + 4-byte float weight
        if per_vertex == 0 || vertex_bytes < per_vertex {
            continue;
        }
        // Allow small padding before matrices (some files align to 16)
        let remainder = vertex_bytes % per_vertex;
        if remainder > 16 {
            continue;
        }
        let vtx_count = (vertex_bytes - remainder) / per_vertex;
        let idx_len = vtx_count * max_weights as usize;
        let w_len = vtx_count * max_weights as usize * 4;
        let idx_start = used_aligned;
        let w_start = idx_start + idx_len;
        let matrices_start = w_start + w_len + remainder;
        if matrices_start + matrix_bytes != payload.len() || matrices_start > payload.len() {
            continue;
        }
        matrix_stride = *stride;
        matrix_format = Some(*label);
        vertex_section_len = idx_len;
        weight_section_len = w_len;
        bone_vertex_indices = &payload[idx_start..w_start];
        vertex_weights_bytes = &payload[w_start..(w_start + w_len)];
        inferred_vertex_count = Some(vtx_count);
        break;
    }

    // Fallback: split remaining bytes roughly in half (previous heuristic)
    if matrix_stride == 0 {
        let (_vertex_count, v_section_len, stride, fmt) =
            infer_skin_layout(remaining_bytes, bone_count, max_weights);
        matrix_stride = stride;
        matrix_format = fmt;
        vertex_section_len = v_section_len;
        let matrix_bytes = bone_count as usize * matrix_stride;
        let vert_data_end = payload
            .len()
            .saturating_sub(matrix_bytes)
            .min(payload.len());
        if used_aligned > vert_data_end {
            return Err(RwReadError::UnexpectedEof);
        }
        let vert_slice = &payload[used_aligned..vert_data_end];
        let (idx, w) = if vertex_section_len > 0 && vert_slice.len() >= vertex_section_len {
            vert_slice.split_at(vertex_section_len)
        } else {
            let mid = vert_slice.len() / 2;
            vert_slice.split_at(mid)
        };
        bone_vertex_indices = idx;
        vertex_weights_bytes = w;
        inferred_vertex_count = if max_weights > 0 {
            Some(idx.len() / max_weights as usize)
        } else {
            None
        };
    }

    let inv_bind_matrices_preview = if matrix_stride > 0 {
        let matrix_bytes = bone_count as usize * matrix_stride;
        if payload.len() >= matrix_bytes {
            let mats_start = payload.len() - matrix_bytes;
            let mats_slice = &payload[mats_start..];
            decode_matrix_preview(mats_slice, matrix_stride, 2)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let vertex_weights_preview_f32: Vec<f32> = vertex_weights_bytes
        .chunks_exact(4)
        .take(16)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();

    let preview_hex = payload
        .iter()
        .take(64)
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    Ok(SkinInfo {
        bone_count,
        used_bone_count,
        max_weights,
        header_byte3,
        used_bone_indices,
        bone_vertex_indices_len: bone_vertex_indices.len(),
        vertex_weights_len: vertex_weights_bytes.len(),
        bone_vertex_indices_preview: bone_vertex_indices.iter().take(64).copied().collect(),
        vertex_weights_preview: vertex_weights_bytes.iter().take(64).copied().collect(),
        vertex_weights_preview_f32,
        vertex_section_len,
        weight_section_len,
        vertex_count_guess: inferred_vertex_count,
        matrix_format: matrix_format.map(str::to_string),
        inverse_bind_matrices_preview: inv_bind_matrices_preview,
        header_words,
        preview_hex,
    })
}

fn hanim_node_type_label(flags: i32) -> &'static str {
    match flags {
        0 => "Deformable",
        1 => "Nub",
        2 => "Unknown(2)",
        3 => "Rigid",
        _ => "Unknown",
    }
}

fn decode_matrix_preview(bytes: &[u8], stride: usize, limit: usize) -> Vec<Vec<f32>> {
    if stride == 0 {
        return Vec::new();
    }
    let mut mats = Vec::new();
    let mut offs = 0usize;
    for _ in 0..limit {
        if offs + stride > bytes.len() {
            break;
        }
        let mut vals = Vec::new();
        for chunk in bytes[offs..offs + stride].chunks_exact(4) {
            let v = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            vals.push(v);
        }
        mats.push(vals);
        offs += stride;
    }
    mats
}

fn infer_skin_layout(
    remaining_bytes: usize,
    bone_count: u32,
    max_weights: u32,
) -> (usize, usize, usize, Option<&'static str>) {
    // Try to infer vertex_section_len and matrix stride.
    // We prefer 3x4 (48 bytes) matrices, then 4x4 (64 bytes).
    for (stride, label) in &[(48usize, "3x4"), (64usize, "4x4")] {
        let matrix_bytes = bone_count as usize * *stride;
        if remaining_bytes < matrix_bytes {
            continue;
        }
        let vert_bytes = remaining_bytes - matrix_bytes;
        if max_weights > 0 && vert_bytes % (max_weights as usize * 2) == 0 {
            // Split indices/weights equally
            let vertex_section_len = vert_bytes / 2;
            let vertex_count = vertex_section_len / max_weights as usize;
            return (vertex_count, vertex_section_len, *stride, Some(*label));
        }
    }
    // Fallback: no inference
    (0, 0, 0, None)
}

/// RenderWare FrameList Struct decoder (preview only: first 16 frames).
pub fn decode_frame_list(payload: &[u8]) -> Result<FrameListInfo, RwReadError> {
    if payload.len() < 4 {
        return Err(RwReadError::UnexpectedEof);
    }
    let frame_count = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let stride = 56usize; // 9 floats (R,U,A) + 3 floats pos + parent i32 + flags i32
    let mut frames = Vec::new();
    let mut offs = 4usize;
    for i in 0..frame_count.min(16) {
        if offs + stride > payload.len() {
            break;
        }
        let right = read_vec3(&payload[offs..offs + 12]);
        let up = read_vec3(&payload[offs + 12..offs + 24]);
        let at = read_vec3(&payload[offs + 24..offs + 36]);
        let pos = read_vec3(&payload[offs + 36..offs + 48]);
        let parent = i32::from_le_bytes([
            payload[offs + 48],
            payload[offs + 49],
            payload[offs + 50],
            payload[offs + 51],
        ]);
        let flags = i32::from_le_bytes([
            payload[offs + 52],
            payload[offs + 53],
            payload[offs + 54],
            payload[offs + 55],
        ]);
        frames.push(FrameInfo {
            index: i as u32,
            parent,
            flags,
            right,
            up,
            at,
            position: pos,
        });
        offs += stride;
    }
    Ok(FrameListInfo {
        frame_count,
        frames,
    })
}

/// Decode Geometry Struct for counts/flags and optional first bounding sphere.
pub fn decode_geometry(payload: &[u8]) -> Result<GeometryInfo, RwReadError> {
    if payload.len() < 16 {
        return Err(RwReadError::UnexpectedEof);
    }
    let flags = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let triangles = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let vertices = u32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
    let morph_targets = u32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]);

    // Surf props and first morph target bounding sphere (best-effort)
    let mut bsphere = None;
    // Surf props: 3 floats (ambient/diffuse/specular)
    let offs = 16usize + 12; // skip surf props
    if morph_targets > 0 && offs + 24 <= payload.len() {
        let center = read_vec3(&payload[offs..offs + 12]);
        let radius = f32::from_le_bytes([
            payload[offs + 12],
            payload[offs + 13],
            payload[offs + 14],
            payload[offs + 15],
        ]);
        let has_vertices = u32::from_le_bytes([
            payload[offs + 16],
            payload[offs + 17],
            payload[offs + 18],
            payload[offs + 19],
        ]);
        let has_normals = u32::from_le_bytes([
            payload[offs + 20],
            payload[offs + 21],
            payload[offs + 22],
            payload[offs + 23],
        ]);
        bsphere = Some(BoundingSphere {
            center,
            radius,
            has_vertices: has_vertices != 0,
            has_normals: has_normals != 0,
        });
    }

    Ok(GeometryInfo {
        flags,
        triangles,
        vertices,
        morph_targets,
        has_normals: flags & 0x10 != 0,
        has_prelit: flags & 0x08 != 0,
        has_texcoords: flags & 0x04 != 0,
        bounding_sphere: bsphere,
    })
}

fn read_vec3(slice: &[u8]) -> [f32; 3] {
    [
        f32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]),
        f32::from_le_bytes([slice[4], slice[5], slice[6], slice[7]]),
        f32::from_le_bytes([slice[8], slice[9], slice[10], slice[11]]),
    ]
}

/// MaterialList Struct decoder (count + unknown marker)
pub fn decode_material_list(payload: &[u8]) -> Result<MaterialListInfo, RwReadError> {
    if payload.len() < 8 {
        return Err(RwReadError::UnexpectedEof);
    }
    let material_count = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let unknown = i32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
    Ok(MaterialListInfo {
        material_count,
        unknown,
    })
}

/// Material Struct decoder (flags/color/unused/texture count/surface props)
pub fn decode_material(payload: &[u8]) -> Result<MaterialInfo, RwReadError> {
    if payload.len() < 28 {
        return Err(RwReadError::UnexpectedEof);
    }
    let flags = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let color = [payload[4], payload[5], payload[6], payload[7]];
    let unknown0 = i32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
    let texture_count = i32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]);
    let surface_props = [
        f32::from_le_bytes([payload[16], payload[17], payload[18], payload[19]]),
        f32::from_le_bytes([payload[20], payload[21], payload[22], payload[23]]),
        f32::from_le_bytes([payload[24], payload[25], payload[26], payload[27]]),
    ];
    Ok(MaterialInfo {
        flags,
        color,
        unknown0,
        texture_count,
        surface_props,
    })
}

/// BinMesh PLG decoder (type + mesh splits + preview of indices).
pub fn decode_binmesh(payload: &[u8]) -> Result<BinMeshInfo, RwReadError> {
    if payload.len() < 12 {
        return Err(RwReadError::UnexpectedEof);
    }
    let type_code = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let mesh_count = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let total_indices = u32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);

    let table_off = 12usize;
    let table_len = mesh_count as usize * 8;
    if table_off + table_len > payload.len() {
        return Err(RwReadError::UnexpectedEof);
    }

    let mut meshes = Vec::new();
    let mut indices_offset = table_off + table_len;
    let mut first_indices: Vec<u32> = Vec::new();

    for i in 0..mesh_count as usize {
        let base = table_off + i * 8;
        let index_count = u32::from_le_bytes([
            payload[base],
            payload[base + 1],
            payload[base + 2],
            payload[base + 3],
        ]);
        let material_index = u32::from_le_bytes([
            payload[base + 4],
            payload[base + 5],
            payload[base + 6],
            payload[base + 7],
        ]);

        let need_bytes = index_count as usize * 4;
        if indices_offset + need_bytes > payload.len() {
            return Err(RwReadError::UnexpectedEof);
        }
        if meshes.is_empty() {
            // Preview first mesh indices (up to 32)
            for chunk in payload[indices_offset..indices_offset + need_bytes]
                .chunks_exact(4)
                .take(32)
            {
                first_indices.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
        }

        meshes.push(BinMeshSplit {
            index_count,
            material_index,
        });
        indices_offset += need_bytes;
    }

    Ok(BinMeshInfo {
        type_code,
        mesh_count,
        total_indices,
        meshes,
        indices_preview: first_indices,
    })
}

#[derive(Debug, Clone)]
pub struct HAnimInfo {
    pub root_id: u32,
    pub hierarchy_id: u32,
    pub bone_count: u32,
    pub flags: u32,
    pub max_key_frame_size: u32,
    pub nodes_preview: Vec<HAnimNode>,
    pub nodes: Vec<HAnimNode>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HAnimNode {
    pub node_id: i32,
    pub node_index: i32,
    pub flags: i32,
    pub type_label: String,
}

#[derive(Debug, Clone)]
pub struct SkinInfo {
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct FrameInfo {
    pub index: u32,
    pub parent: i32,
    pub flags: i32,
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub at: [f32; 3],
    pub position: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct FrameListInfo {
    pub frame_count: u32,
    pub frames: Vec<FrameInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BoundingSphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub has_vertices: bool,
    pub has_normals: bool,
}

#[derive(Debug, Clone)]
pub struct GeometryInfo {
    pub flags: u32,
    pub triangles: u32,
    pub vertices: u32,
    pub morph_targets: u32,
    pub has_normals: bool,
    pub has_prelit: bool,
    pub has_texcoords: bool,
    pub bounding_sphere: Option<BoundingSphere>,
}

#[derive(Debug, Clone)]
pub struct MaterialListInfo {
    pub material_count: i32,
    pub unknown: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MaterialInfo {
    pub flags: i32,
    pub color: [u8; 4],
    pub unknown0: i32,
    pub texture_count: i32,
    pub surface_props: [f32; 3],
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BinMeshSplit {
    pub index_count: u32,
    pub material_index: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BinMeshInfo {
    pub type_code: u32,
    pub mesh_count: u32,
    pub total_indices: u32,
    pub meshes: Vec<BinMeshSplit>,
    pub indices_preview: Vec<u32>,
}
