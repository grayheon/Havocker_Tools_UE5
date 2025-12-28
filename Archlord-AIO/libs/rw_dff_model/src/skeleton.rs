use crate::geometry_full::Vec3;
use crate::util::DecodeError;
use crate::util::DecodeError::UnexpectedEof;

#[derive(Debug, Clone, serde::Serialize)]
pub struct FrameNode {
    pub index: u32,
    pub parent: i32,
    pub flags: i32,
    pub matrix: [f32; 16], // column-major glTF-ready (Z flipped)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HAnimNode {
    pub node_id: i32,
    pub node_index: i32,
    pub flags: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Skeleton {
    pub frames: Vec<FrameNode>,
    pub hanim_nodes: Vec<HAnimNode>,
    pub root_id: u32,
}

pub fn decode_frame_list(payload: &[u8]) -> Result<Vec<FrameNode>, DecodeError> {
    if payload.len() < 4 {
        return Err(UnexpectedEof {
            what: "frame list header",
            need: 4,
            have: payload.len(),
        });
    }
    let count = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
    let stride = 56usize;
    let need = 4 + count * stride;
    if payload.len() < need {
        return Err(UnexpectedEof {
            what: "frame list payload",
            need,
            have: payload.len(),
        });
    }
    let mut frames = Vec::with_capacity(count);
    let mut offs = 4usize;
    for i in 0..count {
        let r = read_vec3(&payload[offs..offs + 12]);
        let u = read_vec3(&payload[offs + 12..offs + 24]);
        let a = read_vec3(&payload[offs + 24..offs + 36]);
        let p = read_vec3(&payload[offs + 36..offs + 48]);
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
        let m = to_gltf_matrix(r, u, a, p);
        frames.push(FrameNode {
            index: i as u32,
            parent,
            flags,
            matrix: m,
        });
        offs += stride;
    }
    Ok(frames)
}

pub fn decode_hanim(payload: &[u8]) -> Result<(u32, Vec<HAnimNode>), DecodeError> {
    if payload.len() < 20 {
        return Err(UnexpectedEof {
            what: "hanim header",
            need: 20,
            have: payload.len(),
        });
    }
    let root_id = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let bone_count =
        u32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]) as usize;
    let mut nodes = Vec::with_capacity(bone_count);
    let mut offs = 20usize;
    for _ in 0..bone_count {
        if offs + 12 > payload.len() {
            return Err(UnexpectedEof {
                what: "hanim nodes",
                need: offs + 12,
                have: payload.len(),
            });
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
        });
        offs += 12;
    }
    Ok((root_id, nodes))
}

fn read_vec3(slice: &[u8]) -> Vec3 {
    Vec3 {
        x: f32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]),
        y: f32::from_le_bytes([slice[4], slice[5], slice[6], slice[7]]),
        z: f32::from_le_bytes([slice[8], slice[9], slice[10], slice[11]]),
    }
}

fn to_gltf_matrix(r: Vec3, u: Vec3, a: Vec3, p: Vec3) -> [f32; 16] {
    // Build column-major matrix with Z-flip (LH -> RH).
    let s = [1.0, 1.0, -1.0, 1.0];
    let mut m = [0f32; 16];
    let cols = [
        [r.x, r.y, r.z, 0.0],
        [u.x, u.y, u.z, 0.0],
        [a.x, a.y, a.z, 0.0],
        [p.x, p.y, p.z, 1.0],
    ];
    for col in 0..4 {
        for row in 0..4 {
            let idx = col * 4 + row;
            m[idx] = s[row] * cols[col][row] * s[col];
        }
    }
    m
}
