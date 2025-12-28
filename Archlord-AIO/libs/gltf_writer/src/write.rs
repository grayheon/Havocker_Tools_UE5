use crate::bin::BinBuilder;
use crate::model::*;
use rw_dff_model::unified_scan::UnifiedReport;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// A vertex identity key for remapping/splitting.
///
/// # Purpose
/// glTF indices require that all attributes share the same index stream.
/// RenderWare/game pipelines often reuse a position with different UVs/normals.
/// This key ensures we split vertices deterministically.
///
/// # Notes
/// Uses `to_bits()` to hash floats losslessly (no FP epsilon games).
#[derive(Clone, Eq, PartialEq)]
struct SplitKey {
    px: u32,
    py: u32,
    pz: u32,
    nx: u32,
    ny: u32,
    nz: u32,
    uv: Vec<[u32; 2]>,
    joints: [u16; 4],
    weights: [u32; 4],
}

impl Hash for SplitKey {
    fn hash<H: Hasher>(&self, s: &mut H) {
        self.px.hash(s);
        self.py.hash(s);
        self.pz.hash(s);
        self.nx.hash(s);
        self.ny.hash(s);
        self.nz.hash(s);
        for uv in &self.uv {
            uv[0].hash(s);
            uv[1].hash(s);
        }
        self.joints.hash(s);
        for w in &self.weights {
            w.hash(s);
        }
    }
}
/// Builds a primitive-local vertex buffer and remapped indices from a shared mesh.
///
/// # Behavior
/// - Walks the provided index list (triangle list or strip, doesn't matter).
/// - Splits vertices on (pos, normal, uv0..uv4).
/// - Returns (packed_vertices, remapped_indices).
///
/// # Notes
/// This mirrors the proven OBJ exporter approach and prevents "torn" meshes.
fn build_primitive_buffers(
    vertices: &[rw_dff_model::unified::Vertex],
    indices: &[u32],
) -> (Vec<rw_dff_model::unified::Vertex>, Vec<u32>) {
    use std::collections::HashMap;

    let mut remap: HashMap<SplitKey, u32> = HashMap::new();
    let mut out_vertices: Vec<rw_dff_model::unified::Vertex> = Vec::new();
    let mut out_indices: Vec<u32> = Vec::with_capacity(indices.len());

    for &orig_idx in indices {
        let v = match vertices.get(orig_idx as usize) {
            Some(v) => v,
            None => continue,
        };

        let key = SplitKey {
            px: v.pos.x.to_bits(),
            py: v.pos.y.to_bits(),
            pz: v.pos.z.to_bits(),
            nx: v.nrm.x.to_bits(),
            ny: v.nrm.y.to_bits(),
            nz: v.nrm.z.to_bits(),
            uv: v
                .uv
                .iter()
                .map(|u| [u.x.to_bits(), u.y.to_bits()])
                .collect(),
            joints: v.joints,
            weights: [
                v.weights[0].to_bits(),
                v.weights[1].to_bits(),
                v.weights[2].to_bits(),
                v.weights[3].to_bits(),
            ],
        };

        let new_idx = if let Some(&ni) = remap.get(&key) {
            ni
        } else {
            let ni = out_vertices.len() as u32;
            out_vertices.push(v.clone());
            remap.insert(key, ni);
            ni
        };

        out_indices.push(new_idx);
    }

    (out_vertices, out_indices)
}

/// Converts a UnifiedReport into a glTF JSON document + BIN buffer.
///
/// # Behavior
/// - Creates one glTF scene containing one node per unified mesh.
/// - Each unified mesh becomes one glTF mesh.
/// - Each material split becomes a glTF primitive with its own index buffer.
/// - Emits position, normal, texcoord_0..4.
/// - Copies referenced textures into `out_dir` if found.
///
/// # Notes
/// - Indices are emitted as UNSIGNED_INT (u32) for simplicity and safety.
/// - Materials are deduplicated globally by texture name (Option<String>).
pub fn from_unified(
    stem: &str,
    bin_name: &str,
    report: &UnifiedReport,
    out_dir: &Path,
) -> Result<(Gltf, Vec<u8>), WriteError> {
    let mut bin = BinBuilder::new();

    let mut buffer_views: Vec<BufferView> = Vec::new();
    let mut accessors: Vec<Accessor> = Vec::new();
    let mut all_primitives: Vec<Primitive> = Vec::new();

    // Global material/texture caches (dedupe)
    let mut samplers: Vec<Sampler> = Vec::new();

    // One default sampler (repeat). Keep deterministic.
    let sampler_index = {
        samplers.push(Sampler {
            mag_filter: None,
            min_filter: None,
            wrap_s: Some(10497), // REPEAT
            wrap_t: Some(10497), // REPEAT
        });
        0usize
    };

    // We search textures relative to the DFF location and a few common subdirectories.
    // We don't have the input path here, so we use the report.file directory as the primary root.
    let file_dir = PathBuf::from(&report.file)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let search_roots = texture_search_roots(&file_dir);

    let mut caches = MaterialCaches {
        material_cache: HashMap::new(),
        image_cache: HashMap::new(),
        texture_cache: HashMap::new(),
        images: Vec::new(),
        textures: Vec::new(),
        materials: Vec::new(),
    };
    let mut missing_textures: BTreeSet<String> = BTreeSet::new();

    // Build one glTF mesh per unified mesh
    for (_mesh_idx, um) in report.meshes.iter().enumerate() {
        let clean_vertices = sanitize_vertices(&um.vertices);
        // --- Build primitives (one per submesh/material) ---
        let mut primitives: Vec<Primitive> = Vec::new();

        for sm in &um.submeshes {
            // 1) Split vertices for this specific primitive.
            // glTF requires that all attributes of a primitive use the same index count.
            // Splitting ensures that if a vertex is shared by different UVs/Normals, it gets its own copy.
            let (prim_vertices, prim_indices) =
                build_primitive_buffers(&clean_vertices, &sm.indices);
            if prim_vertices.is_empty() {
                continue;
            }

            // 2) Prepare attribute arrays for this primitive
            let v_count = prim_vertices.len();
            let mut pos_f32 = Vec::with_capacity(v_count * 3);
            let mut nrm_f32 = Vec::with_capacity(v_count * 3);

            let uv_count = prim_vertices[0].uv.len();
            let mut uv_f32: Vec<Vec<f32>> = (0..uv_count)
                .map(|_| Vec::with_capacity(v_count * 2))
                .collect();
            let mut joints_u16: Vec<u16> = Vec::with_capacity(v_count * 4);
            let mut weights_f32: Vec<f32> = Vec::with_capacity(v_count * 4);

            let (min_pos, max_pos) = compute_min_max_pos(&prim_vertices);

            for v in &prim_vertices {
                // RenderWare (Archlord) is Left-Handed, glTF is Right-Handed.
                // Invert Z to convert.
                pos_f32.extend_from_slice(&[v.pos.x, v.pos.y, -v.pos.z]);
                nrm_f32.extend_from_slice(&[v.nrm.x, v.nrm.y, -v.nrm.z]);
                for (i, uv) in v.uv.iter().enumerate() {
                    // RenderWare UVs bereits im erwarteten Raum; kein Flip von v
                    uv_f32[i].extend_from_slice(&[uv.x, uv.y]);
                }
                joints_u16.extend_from_slice(&v.joints);
                weights_f32.extend_from_slice(&v.weights);
            }

            // 3) Push attribute data into BIN and create accessors
            let (pos_off, pos_len) = bin.push_f32(&pos_f32);
            let pos_bv = push_buffer_view(
                &mut buffer_views,
                0,
                pos_off,
                pos_len,
                Some(12),
                Some(34962),
            );
            let pos_acc = push_accessor(
                &mut accessors,
                pos_bv,
                5126, // FLOAT
                v_count as u64,
                "VEC3",
                Some(min_pos.to_vec()),
                Some(max_pos.to_vec()),
            );

            let (nrm_off, nrm_len) = bin.push_f32(&nrm_f32);
            let nrm_bv = push_buffer_view(
                &mut buffer_views,
                0,
                nrm_off,
                nrm_len,
                Some(12),
                Some(34962),
            );
            let nrm_acc = push_accessor(
                &mut accessors,
                nrm_bv,
                5126,
                v_count as u64,
                "VEC3",
                None,
                None,
            );

            let mut texcoords = std::collections::BTreeMap::new();
            for (i, uv) in uv_f32.iter().enumerate() {
                let (uv_off, uv_len) = bin.push_f32(uv);
                let uv_bv =
                    push_buffer_view(&mut buffer_views, 0, uv_off, uv_len, Some(8), Some(34962));
                let acc = push_accessor(
                    &mut accessors,
                    uv_bv,
                    5126,
                    v_count as u64,
                    "VEC2",
                    None,
                    None,
                );
                texcoords.insert(format!("TEXCOORD_{i}"), acc);
            }

            // Joints/Weights
            let joints_acc = if !joints_u16.is_empty() {
                let (off, len) = bin.push_u16(&joints_u16);
                let bv = push_buffer_view(&mut buffer_views, 0, off, len, Some(8), Some(34962));
                Some(push_accessor(
                    &mut accessors,
                    bv,
                    5123, // UNSIGNED_SHORT
                    v_count as u64,
                    "VEC4",
                    None,
                    None,
                ))
            } else {
                None
            };

            let weights_acc = if !weights_f32.is_empty() {
                let (off, len) = bin.push_f32(&weights_f32);
                let bv = push_buffer_view(&mut buffer_views, 0, off, len, Some(16), Some(34962));
                Some(push_accessor(
                    &mut accessors,
                    bv,
                    5126, // FLOAT
                    v_count as u64,
                    "VEC4",
                    None,
                    None,
                ))
            } else {
                None
            };

            // 4) Push index data
            let (idx_off, idx_len) = bin.push_u32(&prim_indices);
            let idx_bv =
                push_buffer_view(&mut buffer_views, 0, idx_off, idx_len, None, Some(34963));
            let idx_acc = push_accessor(
                &mut accessors,
                idx_bv,
                5125, // UNSIGNED_INT
                prim_indices.len() as u64,
                "SCALAR",
                None,
                None,
            );

            // 5) Material handling
            let tex_name = um
                .materials
                .get(sm.material_index as usize)
                .and_then(|m| m.base_texture_name.clone());

            let mat_index = get_or_create_material(
                &tex_name,
                &search_roots,
                out_dir,
                sampler_index,
                &mut missing_textures,
                &mut caches,
            )?;

            primitives.push(Primitive {
                attributes: Attributes {
                    position: pos_acc,
                    normal: nrm_acc,
                    joints0: joints_acc,
                    weights0: weights_acc,
                    texcoords,
                },
                indices: Some(idx_acc),
                material: Some(mat_index),
                mode: Some(if sm.is_strip { 5 } else { 4 }), // TRIANGLE_STRIP or TRIANGLES
            });
        }

        all_primitives.extend(primitives);
    }

    // Single glTF mesh aggregating all primitives (closer to client export layout)
    let mut meshes: Vec<Mesh> = Vec::new();
    meshes.push(Mesh {
        primitives: all_primitives,
        name: Some(format!("{stem}_mesh")),
    });

    // Build skeleton nodes if present
    let mut nodes: Vec<Node> = Vec::new();
    let mut skins: Vec<Skin> = Vec::new();
    let mut scene_nodes: Vec<usize> = Vec::new();

    let skin_mesh_opt = report.meshes.iter().find(|m| m.skin.is_some());

    if let (Some(skel), Some(mesh_with_skin)) = (report.skeleton.as_ref(), skin_mesh_opt) {
        let frame_count = skel.frames.len();
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); frame_count];
        for f in &skel.frames {
            if f.parent >= 0 {
                let p = f.parent as usize;
                if p < frame_count {
                    children[p].push(f.index as usize);
                }
            }
        }

        for (i, f) in skel.frames.iter().enumerate() {
            let mat = if is_identity_matrix(&f.matrix) {
                None
            } else {
                Some(f.matrix.to_vec())
            };
            nodes.push(Node {
                mesh: None,
                name: Some(format!("frame_{i}")),
                children: if children[i].is_empty() {
                    None
                } else {
                    Some(children[i].clone())
                },
                matrix: mat,
                skin: None,
            });
        }

        let mesh_node_index = nodes.len();

        nodes.push(Node {
            mesh: Some(0),
            name: Some(format!("{stem}_node")),
            children: None,
            matrix: None,
            skin: Some(skins.len()),
        });

        // Scene roots: all frames with parent <0 (or 0 if none), plus mesh node if no frames
        if nodes.is_empty() {
            scene_nodes.push(mesh_node_index);
        } else {
            for (i, f) in skel.frames.iter().enumerate() {
                if f.parent < 0 {
                    scene_nodes.push(i);
                }
            }
            scene_nodes.push(mesh_node_index);
        }

        // Skin: joints are frame indices, IBMs padded to length
        let skin_info = mesh_with_skin.skin.as_ref().unwrap();
        let joint_count = frame_count.max(skin_info.inverse_bind_matrices.len());
        let ibm_mats = build_inverse_bind_matrices(skel, skin_info, joint_count);
        let mut ibm_f32: Vec<f32> = Vec::with_capacity(ibm_mats.len() * 16);
        for m in &ibm_mats {
            ibm_f32.extend_from_slice(m);
        }
        let (ibm_off, ibm_len) = bin.push_f32(&ibm_f32);
        let ibm_bv = push_buffer_view(&mut buffer_views, 0, ibm_off, ibm_len, None, None);
        let ibm_acc = push_accessor(
            &mut accessors,
            ibm_bv,
            5126,
            joint_count as u64,
            "MAT4",
            None,
            None,
        );

        let joints: Vec<usize> = (0..joint_count).collect();
        skins.push(Skin {
            joints,
            inverse_bind_matrices: Some(ibm_acc),
            skeleton: Some(0),
        });
    } else {
        // Fallback: no skin/skeleton, single mesh node
        nodes.push(Node {
            mesh: Some(0),
            name: Some(format!("{stem}_node")),
            children: None,
            matrix: None,
            skin: None,
        });
        scene_nodes.push(0);
    }

    // Build glTF
    let gltf = Gltf {
        asset: Asset {
            version: "2.0".to_string(),
            generator: Some("Archlord-AIO dff2gltf".to_string()),
        },
        scene: 0,
        scenes: vec![Scene { nodes: scene_nodes }],
        nodes,
        meshes,
        skins,
        buffers: vec![Buffer {
            byte_length: bin.len(),
            uri: bin_name.to_string(),
        }],
        buffer_views,
        accessors,
        materials: caches.materials,
        textures: caches.textures,
        images: caches.images,
        samplers,
    };

    // Ensure empty vectors are handled (serde skip rules depend on your struct attrs).
    // If you didn't mark samplers/images/textures/materials as skip-if-empty, keep them anyway.

    if !missing_textures.is_empty() {
        let missing_path = out_dir.join(format!("{stem}_missing_textures.txt"));
        if let Ok(mut file) = fs::File::create(&missing_path) {
            use std::io::Write;
            for name in missing_textures {
                let _ = writeln!(file, "{name}");
            }
        }
    }

    Ok((gltf, bin.bytes().to_vec()))
}

/// Writer errors.
///
/// # Design
/// - Keep errors explicit and informative.
/// - Avoid hiding filesystem failures when copying textures.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("texture resolution failed for '{0}'")]
    TextureResolveFailed(String),
}

fn push_buffer_view(
    buffer_views: &mut Vec<BufferView>,
    buffer_index: usize,
    byte_offset: u64,
    byte_length: u64,
    byte_stride: Option<u64>,
    target: Option<u32>,
) -> usize {
    let idx = buffer_views.len();
    buffer_views.push(BufferView {
        buffer: buffer_index,
        byte_offset,
        byte_length,
        byte_stride,
        target,
    });
    idx
}

fn push_accessor(
    accessors: &mut Vec<Accessor>,
    buffer_view: usize,
    component_type: u32,
    count: u64,
    ty: &str,
    min: Option<Vec<f32>>,
    max: Option<Vec<f32>>,
) -> usize {
    let idx = accessors.len();
    accessors.push(Accessor {
        buffer_view,
        byte_offset: None,
        component_type,
        count,
        r#type: ty.to_string(),
        min,
        max,
    });
    idx
}

/// Computes position min/max for glTF accessors.
///
/// # Behavior
/// - Deterministic even if vertices are empty (returns zeros).
fn compute_min_max_pos(vertices: &[rw_dff_model::unified::Vertex]) -> ([f32; 3], [f32; 3]) {
    if vertices.is_empty() {
        return ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    }

    let mut min_v = [vertices[0].pos.x, vertices[0].pos.y, -vertices[0].pos.z];
    let mut max_v = min_v;

    for v in vertices.iter().skip(1) {
        let p = [v.pos.x, v.pos.y, -v.pos.z];
        for i in 0..3 {
            if p[i] < min_v[i] {
                min_v[i] = p[i];
            }
            if p[i] > max_v[i] {
                max_v[i] = p[i];
            }
        }
    }

    (min_v, max_v)
}

/// Sanitizes joints/weights to avoid NaNs, negatives, duplicates, or zero-weight joints.
fn sanitize_vertices(
    vertices: &[rw_dff_model::unified::Vertex],
) -> Vec<rw_dff_model::unified::Vertex> {
    vertices
        .iter()
        .map(|v| {
            let mut out = v.clone();
            let mut pairs: Vec<(u16, f32)> = Vec::new();

            for i in 0..4 {
                let mut w = out.weights[i];
                if !w.is_finite() || w < 0.0 {
                    w = 0.0;
                }
                let j = out.joints[i];
                if w == 0.0 {
                    continue;
                }
                if let Some(existing) = pairs.iter_mut().find(|(joint, _)| *joint == j) {
                    existing.1 += w;
                } else {
                    pairs.push((j, w));
                }
            }

            pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let mut joints = [0u16; 4];
            let mut weights = [0f32; 4];
            for (slot, (j, w)) in pairs.into_iter().take(4).enumerate() {
                joints[slot] = j;
                weights[slot] = w;
            }

            let sum: f32 = weights.iter().sum();
            if sum > 0.0 && sum.is_finite() {
                for w in &mut weights {
                    *w /= sum;
                }
            }

            let eps = 1e-6;
            for i in 0..4 {
                if !weights[i].is_finite() || weights[i].abs() < eps {
                    weights[i] = 0.0;
                    joints[i] = 0;
                }
            }

            let sum_after: f32 = weights.iter().sum();
            if sum_after > 0.0 && sum_after.is_finite() {
                for w in &mut weights {
                    *w /= sum_after;
                }
                // Force exact normalization: add residual to the largest weight.
                let norm_sum: f32 = weights.iter().sum();
                if norm_sum.is_finite() && norm_sum != 1.0 {
                    let (max_idx, _) = weights
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap_or((0, &weights[0]));
                    weights[max_idx] += 1.0 - norm_sum;
                }
            } else {
                joints = [0u16; 4];
                weights = [1.0, 0.0, 0.0, 0.0];
            }

            out.joints = joints;
            out.weights = weights;
            out
        })
        .collect()
}

/// Builds safe inverse bind matrices; falls back to skeleton world matrices if provided data is invalid.
fn build_inverse_bind_matrices(
    skeleton: &rw_dff_model::skeleton::Skeleton,
    skin: &rw_dff_model::unified::MeshSkin,
    joint_count: usize,
) -> Vec<[f32; 16]> {
    let world = compute_world_matrices(&skeleton.frames);
    let mut out = Vec::with_capacity(joint_count);

    for i in 0..joint_count {
        let world_inv = world
            .get(i)
            .and_then(|m| invert_4x4(*m))
            .filter(is_matrix_finite);
        let candidate = skin
            .inverse_bind_matrices
            .get(i)
            .cloned()
            .filter(is_matrix_finite);
        let chosen = world_inv.or(candidate).unwrap_or([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]);
        out.push(sanitize_ibm(chosen));
    }

    out
}

fn compute_world_matrices(frames: &[rw_dff_model::skeleton::FrameNode]) -> Vec<[f32; 16]> {
    let mut cache: Vec<Option<[f32; 16]>> = vec![None; frames.len()];
    for idx in 0..frames.len() {
        resolve_world_matrix(idx, frames, &mut cache);
    }
    cache
        .into_iter()
        .map(|m| m.unwrap_or([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]))
        .collect()
}

fn resolve_world_matrix(
    idx: usize,
    frames: &[rw_dff_model::skeleton::FrameNode],
    cache: &mut [Option<[f32; 16]>],
) -> [f32; 16] {
    if let Some(m) = cache[idx] {
        return m;
    }
    let local = frames[idx].matrix;
    let world = if frames[idx].parent >= 0 {
        let p = frames[idx].parent as usize;
        let parent_world = resolve_world_matrix(p, frames, cache);
        mat_mul(parent_world, local)
    } else {
        local
    };
    cache[idx] = Some(world);
    world
}

fn mat_mul(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut out = [0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut v = 0.0;
            for k in 0..4 {
                v += a[k * 4 + row] * b[col * 4 + k];
            }
            out[col * 4 + row] = v;
        }
    }
    out
}

fn invert_4x4(m: [f32; 16]) -> Option<[f32; 16]> {
    let mut inv = [0f32; 16];

    inv[0] = m[5] * m[10] * m[15]
        - m[5] * m[11] * m[14]
        - m[9] * m[6] * m[15]
        + m[9] * m[7] * m[14]
        + m[13] * m[6] * m[11]
        - m[13] * m[7] * m[10];
    inv[4] = -m[4] * m[10] * m[15]
        + m[4] * m[11] * m[14]
        + m[8] * m[6] * m[15]
        - m[8] * m[7] * m[14]
        - m[12] * m[6] * m[11]
        + m[12] * m[7] * m[10];
    inv[8] = m[4] * m[9] * m[15]
        - m[4] * m[11] * m[13]
        - m[8] * m[5] * m[15]
        + m[8] * m[7] * m[13]
        + m[12] * m[5] * m[11]
        - m[12] * m[7] * m[9];
    inv[12] = -m[4] * m[9] * m[14]
        + m[4] * m[10] * m[13]
        + m[8] * m[5] * m[14]
        - m[8] * m[6] * m[13]
        - m[12] * m[5] * m[10]
        + m[12] * m[6] * m[9];

    inv[1] = -m[1] * m[10] * m[15]
        + m[1] * m[11] * m[14]
        + m[9] * m[2] * m[15]
        - m[9] * m[3] * m[14]
        - m[13] * m[2] * m[11]
        + m[13] * m[3] * m[10];
    inv[5] = m[0] * m[10] * m[15]
        - m[0] * m[11] * m[14]
        - m[8] * m[2] * m[15]
        + m[8] * m[3] * m[14]
        + m[12] * m[2] * m[11]
        - m[12] * m[3] * m[10];
    inv[9] = -m[0] * m[9] * m[15]
        + m[0] * m[11] * m[13]
        + m[8] * m[1] * m[15]
        - m[8] * m[3] * m[13]
        - m[12] * m[1] * m[11]
        + m[12] * m[3] * m[9];
    inv[13] = m[0] * m[9] * m[14]
        - m[0] * m[10] * m[13]
        - m[8] * m[1] * m[14]
        + m[8] * m[2] * m[13]
        + m[12] * m[1] * m[10]
        - m[12] * m[2] * m[9];

    inv[2] = m[1] * m[6] * m[15]
        - m[1] * m[7] * m[14]
        - m[5] * m[2] * m[15]
        + m[5] * m[3] * m[14]
        + m[13] * m[2] * m[7]
        - m[13] * m[3] * m[6];
    inv[6] = -m[0] * m[6] * m[15]
        + m[0] * m[7] * m[14]
        + m[4] * m[2] * m[15]
        - m[4] * m[3] * m[14]
        - m[12] * m[2] * m[7]
        + m[12] * m[3] * m[6];
    inv[10] = m[0] * m[5] * m[15]
        - m[0] * m[7] * m[13]
        - m[4] * m[1] * m[15]
        + m[4] * m[3] * m[13]
        + m[12] * m[1] * m[7]
        - m[12] * m[3] * m[5];
    inv[14] = -m[0] * m[5] * m[14]
        + m[0] * m[6] * m[13]
        + m[4] * m[1] * m[14]
        - m[4] * m[2] * m[13]
        - m[12] * m[1] * m[6]
        + m[12] * m[2] * m[5];

    inv[3] = -m[1] * m[6] * m[11]
        + m[1] * m[7] * m[10]
        + m[5] * m[2] * m[11]
        - m[5] * m[3] * m[10]
        - m[9] * m[2] * m[7]
        + m[9] * m[3] * m[6];
    inv[7] = m[0] * m[6] * m[11]
        - m[0] * m[7] * m[10]
        - m[4] * m[2] * m[11]
        + m[4] * m[3] * m[10]
        + m[8] * m[2] * m[7]
        - m[8] * m[3] * m[6];
    inv[11] = -m[0] * m[5] * m[11]
        + m[0] * m[7] * m[9]
        + m[4] * m[1] * m[11]
        - m[4] * m[3] * m[9]
        - m[8] * m[1] * m[7]
        + m[8] * m[3] * m[5];
    inv[15] = m[0] * m[5] * m[10]
        - m[0] * m[6] * m[9]
        - m[4] * m[1] * m[10]
        + m[4] * m[2] * m[9]
        + m[8] * m[1] * m[6]
        - m[8] * m[2] * m[5];

    let mut det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];
    if !det.is_finite() || det.abs() < 1e-8 {
        return None;
    }
    det = 1.0 / det;

    for v in &mut inv {
        *v *= det;
    }

    if inv.iter().all(|v| v.is_finite()) {
        Some(inv)
    } else {
        None
    }
}

fn sanitize_ibm(mut m: [f32; 16]) -> [f32; 16] {
    // Enforce affine bottom row and clamp tiny residuals.
    m[3] = 0.0;
    m[7] = 0.0;
    m[11] = 0.0;
    m[15] = 1.0;
    m
}

fn is_identity_matrix(m: &[f32; 16]) -> bool {
    const EPS: f32 = 1e-6;
    let id = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    m.iter()
        .zip(id.iter())
        .all(|(a, b)| (*a - *b).abs() <= EPS)
}

fn is_matrix_finite(m: &[f32; 16]) -> bool {
    m.iter().all(|v| v.is_finite())
}

/// Returns a list of default texture search roots relative to the extracted client.
///
/// # Behavior
/// - Uses the directory of the source DFF as the primary root.
/// - Adds common Archlord-style texture folders.
fn default_texture_roots(file_dir: &Path) -> Vec<PathBuf> {
    vec![
        file_dir.to_path_buf(),
        // English variants
        file_dir.join("texture"),
        // Common subfolders
        file_dir.join("texture").join("object"),
        file_dir.join("texture").join("world"),
        file_dir.join("texture").join("character"),
        file_dir.join("texture").join("npc"),
        file_dir.join("texture").join("item"),
        file_dir.join("texture").join("skill"),
    ]
}

/// Gets or creates a glTF material for a given texture name.
///
/// # Behavior
/// - Deduplicates materials by `Option<String>` key (None => one shared "no texture" material).
/// - If a texture is found, it creates image+texture entries (deduped by URI).
/// - Copies the texture file into out_dir and uses the copied filename as image URI.
fn get_or_create_material(
    tex_name: &Option<String>,
    search_roots: &[PathBuf],
    out_dir: &Path,
    sampler_index: usize,
    missing: &mut BTreeSet<String>,
    caches: &mut MaterialCaches,
) -> Result<usize, WriteError> {
    if let Some(&idx) = caches.material_cache.get(tex_name) {
        return Ok(idx);
    }

    let base_color_texture = tex_name.as_ref().and_then(|name| {
        resolve_texture(name, search_roots, out_dir, sampler_index, caches)
            .transpose()
            .ok()
            .flatten()
            .or_else(|| {
                missing.insert(format!("{name}.png"));
                None
            })
    });

    let mat_index = caches.materials.len();
    caches.materials.push(Material {
        name: tex_name.clone(),
        pbr: Pbr {
            base_color_texture,
            metallic_factor: 0.0,
            roughness_factor: 1.0,
        },
        double_sided: Some(true),
    });

    caches.material_cache.insert(tex_name.clone(), mat_index);
    Ok(mat_index)
}

fn resolve_texture(
    name: &str,
    search_roots: &[PathBuf],
    out_dir: &Path,
    sampler_index: usize,
    caches: &mut MaterialCaches,
) -> Option<Result<TextureInfo, WriteError>> {
    find_texture_file(name, search_roots)
        .map(|src_path| {
            let file_name = src_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("{name}.png"));

            // Copy the texture into the glTF output folder (per-asset sandbox)
            fs::create_dir_all(out_dir)?;
            let dst_path = out_dir.join(&file_name);
            if !dst_path.exists() {
                fs::copy(&src_path, &dst_path)?;
            }

            Ok(texture_info_for(file_name, sampler_index, caches))
        })
        .or_else(|| None)
}

fn texture_info_for(
    file_name: String,
    sampler_index: usize,
    caches: &mut MaterialCaches,
) -> TextureInfo {
    let image_index = *caches
        .image_cache
        .entry(file_name.clone())
        .or_insert_with(|| {
            let idx = caches.images.len();
            caches.images.push(Image {
                uri: file_name.clone(),
            });
            idx
        });

    let tex_index = *caches.texture_cache.entry(file_name).or_insert_with(|| {
        let idx = caches.textures.len();
        caches.textures.push(Texture {
            source: image_index,
            sampler: Some(sampler_index),
        });
        idx
    });

    TextureInfo { index: tex_index }
}

struct MaterialCaches {
    material_cache: HashMap<Option<String>, usize>,
    image_cache: HashMap<String, usize>,
    texture_cache: HashMap<String, usize>,
    images: Vec<Image>,
    textures: Vec<Texture>,
    materials: Vec<Material>,
}

/// Builds a texture search list that understands the extracted destination layout.
///
/// # Behavior
/// - Starts with the default roots (next to the DFF/ECL).
/// - Adds sibling `/texture` folders (and their known subfolders/DDS) relative to the asset folder.
/// - Deduplicates entries for deterministic lookups.
fn texture_search_roots(file_dir: &Path) -> Vec<PathBuf> {
    let mut roots = default_texture_roots(file_dir);

    // Common sibling layout: <dest>/{object|npc|...} and <dest>/texture/<category>/{DDS,PNG}
    if let Some(parent) = file_dir.parent() {
        let tex_root = parent.join("texture");
        let mut extras = vec![
            tex_root.clone(),
            tex_root.join("DDS"),
            tex_root.join("dds"),
            tex_root.join("PNG"),
            tex_root.join("png"),
        ];

        // Known categories first
        let mut categories: Vec<String> = vec![
            "character".into(),
            "effect".into(),
            "etc".into(),
            "item".into(),
            "minimap".into(),
            "notpacked".into(),
            "object".into(),
            "skill".into(),
            "ui".into(),
            "world".into(),
            "worldmap".into(),
        ];

        // Add all first-level subdirectories dynamically if the texture root exists
        if tex_root.exists() {
            if let Ok(entries) = fs::read_dir(&tex_root) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            categories.push(name.to_string());
                        }
                    }
                }
            }
        }

        for sub in categories {
            let subdir = tex_root.join(sub);
            extras.push(subdir.clone());
            extras.push(subdir.join("DDS"));
            extras.push(subdir.join("dds"));
            extras.push(subdir.join("PNG"));
            extras.push(subdir.join("png"));
        }
        roots.extend(extras);
    }

    // Also try local DDS/PNG folders next to the model
    roots.push(file_dir.join("DDS"));
    roots.push(file_dir.join("dds"));
    roots.push(file_dir.join("PNG"));
    roots.push(file_dir.join("png"));

    roots.sort();
    roots.dedup();
    roots
}

/// Finds a texture file by base name in known roots.
///
/// # Behavior
/// - Checks common extensions.
/// - Returns the first match in deterministic order.
fn find_texture_file(base_name: &str, roots: &[PathBuf]) -> Option<PathBuf> {
    let ext_s = ["png", "jpg", "jpeg", "tga", "bmp"];

    for root in roots {
        for ext in ext_s {
            let p = root.join(format!("{base_name}.{ext}"));
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}
