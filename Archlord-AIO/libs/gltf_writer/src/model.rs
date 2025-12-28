use serde::Serialize;

/// glTF top-level document (minimal subset).
///
/// # Purpose
/// This struct models a minimal glTF 2.0 JSON file that is sufficient for
/// static meshes with multiple UV sets and material splits.
///
/// # Notes
/// - We emit external `.bin` buffers (no base64).
/// - We keep the schema minimal and add fields as needed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gltf {
    pub asset: Asset,
    pub scene: usize,
    pub scenes: Vec<Scene>,
    pub nodes: Vec<Node>,
    pub meshes: Vec<Mesh>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skins: Vec<Skin>,

    pub buffers: Vec<Buffer>,
    pub buffer_views: Vec<BufferView>,
    pub accessors: Vec<Accessor>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<Material>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub textures: Vec<Texture>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<Image>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub samplers: Vec<Sampler>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub nodes: Vec<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mesh {
    pub primitives: Vec<Primitive>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Primitive {
    pub attributes: Attributes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indices: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>, // 4 = TRIANGLES
}

#[derive(Debug, Serialize)]
pub struct Attributes {
    #[serde(rename = "POSITION")]
    pub position: usize,
    #[serde(rename = "NORMAL")]
    pub normal: usize,
    #[serde(skip_serializing_if = "Option::is_none", rename = "JOINTS_0")]
    pub joints0: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "WEIGHTS_0")]
    pub weights0: Option<usize>,
    #[serde(flatten)]
    pub texcoords: std::collections::BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Buffer {
    pub byte_length: u64,
    pub uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    pub buffer: usize,
    pub byte_offset: u64,
    pub byte_length: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_stride: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>, // 34962 ARRAY_BUFFER, 34963 ELEMENT_ARRAY_BUFFER
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessor {
    pub buffer_view: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<u64>,
    pub component_type: u32,
    pub count: u64,
    pub r#type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Vec<f32>>,
}

/// Minimal PBR material with optional baseColor texture.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Material {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "pbrMetallicRoughness")]
    pub pbr: Pbr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_sided: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pbr {
    #[serde(skip_serializing_if = "Option::is_none", rename = "baseColorTexture")]
    pub base_color_texture: Option<TextureInfo>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureInfo {
    pub index: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Texture {
    pub source: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampler: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sampler {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mag_filter: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_filter: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_s: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_t: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub joints: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inverse_bind_matrices: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton: Option<usize>,
}
