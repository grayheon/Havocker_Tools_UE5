use crate::util::DecodeError;
use crate::util::DecodeError::UnexpectedEof;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SkinData {
    pub bone_count: u32,
    pub used_bone_indices: Vec<u32>,
    pub max_weights: u8,
    pub indices: Vec<u8>,
    pub weights: Vec<f32>,
    pub inverse_bind_matrices: Vec<[f32; 16]>,
}

/// Decode a RenderWare SkinPLG payload given the vertex count.
/// Layout: counts (4 bytes) -> usedBoneIndices -> vertex indices (u8) -> vertex weights (f32) -> IBMs.
pub fn decode_skin_plg(payload: &[u8], vertex_count: usize) -> Result<SkinData, DecodeError> {
    if payload.len() < 4 {
        return Err(UnexpectedEof {
            what: "skin header",
            need: 4,
            have: payload.len(),
        });
    }
    let bone_count = payload[0] as u32;
    let used_bone_count = payload[1] as usize;
    let max_weights = payload[2];

    let used_start = 4usize;
    let used_end = used_start + used_bone_count;
    if used_end > payload.len() {
        return Err(UnexpectedEof {
            what: "used bone indices",
            need: used_end,
            have: payload.len(),
        });
    }
    let used_bone_indices = payload[used_start..used_end]
        .iter()
        .map(|b| *b as u32)
        .collect::<Vec<_>>();

    // align to 4 bytes
    let used_aligned = (used_end + 3) & !3;
    if used_aligned > payload.len() {
        return Err(UnexpectedEof {
            what: "used bone indices padding",
            need: used_aligned,
            have: payload.len(),
        });
    }

    let idx_len = vertex_count
        .checked_mul(max_weights as usize)
        .ok_or(UnexpectedEof {
            what: "skin indices overflow",
            need: usize::MAX,
            have: 0,
        })?;
    let idx_start = used_aligned;
    let idx_end = idx_start + idx_len;
    if idx_end > payload.len() {
        return Err(UnexpectedEof {
            what: "skin vertex indices",
            need: idx_end,
            have: payload.len(),
        });
    }
    let indices = payload[idx_start..idx_end].to_vec();

    let weights_len = idx_len.checked_mul(4).ok_or(UnexpectedEof {
        what: "skin weights overflow",
        need: usize::MAX,
        have: 0,
    })?;
    let weights_start = idx_end;
    let weights_end = weights_start + weights_len;
    if weights_end > payload.len() {
        return Err(UnexpectedEof {
            what: "skin vertex weights",
            need: weights_end,
            have: payload.len(),
        });
    }
    let mut weights = Vec::with_capacity(idx_len);
    for chunk in payload[weights_start..weights_end].chunks_exact(4) {
        weights.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }

    // Remaining bytes are inverse bind matrices: prefer 4x4 (64 bytes each), fall back to 3x4 (48 bytes).
    let remaining = payload.len() - weights_end;
    let (stride, count) = if remaining >= bone_count as usize * 64 {
        (64usize, bone_count as usize)
    } else if remaining >= bone_count as usize * 48 {
        (48usize, bone_count as usize)
    } else {
        (0, 0)
    };

    let mut inverse_bind_matrices = Vec::new();
    if stride > 0 {
        let mats_start = weights_end;
        let mats_end = mats_start + stride * count;
        if mats_end <= payload.len() {
            for chunk in payload[mats_start..mats_end].chunks_exact(stride) {
                let mut m = [0f32; 16];
                // If 3x4, fill last row as [0 0 0 1]
                let floats: Vec<f32> = chunk
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                if stride == 64 {
                    m.copy_from_slice(&floats[..16]);
                } else {
                    m[..12].copy_from_slice(&floats[..12]);
                    m[15] = 1.0;
                }
                inverse_bind_matrices.push(flip_matrix_z(m));
            }
        }
    }

    Ok(SkinData {
        bone_count,
        used_bone_indices,
        max_weights,
        indices,
        weights,
        inverse_bind_matrices,
    })
}

/// Convert a LH matrix to RH by flipping Z (S*M*S with S=diag(1,1,-1,1)).
fn flip_matrix_z(m: [f32; 16]) -> [f32; 16] {
    // column-major: index = col*4 + row
    let s = [1.0, 1.0, -1.0, 1.0];
    let mut out = [0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let idx = col * 4 + row;
            out[idx] = s[row] * m[idx] * s[col];
        }
    }
    out
}
