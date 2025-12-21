use rw_dff::ids::ids;
use rw_dff::tree::RwChunkNode;
use serde::Serialize;

use crate::plugins::{collect_extension_plugins, PluginEntry};
use crate::texture::{decode_texture_node, TextureInfo};
use crate::util::DecodeError;

/// Decoded information about one Material (0x07).
#[derive(Debug, Clone, Serialize)]
pub struct MaterialInfo {
    pub texture: Option<TextureInfo>,
    pub plugins: Vec<PluginEntry>,
}

/// Decoded information about a MaterialList (0x08).
#[derive(Debug, Clone, Serialize)]
pub struct MaterialListInfo {
    pub material_count: u32,
    pub materials: Vec<MaterialInfo>,
}

/// Decodes a MaterialList node.
///
/// # Expected structure (common RW)
/// - Struct (0x01): materialCount (u32) + materialIndices[materialCount] (i32)
/// - Material (0x07) repeated materialCount times
pub fn decode_material_list_node(
    ml: &RwChunkNode,
    file_bytes: &[u8],
) -> Result<MaterialListInfo, DecodeError> {
    if ml.header.id != ids::RW_MATERIALLIST {
        return Err(DecodeError::InvalidValue {
            what: "material_list_node.id",
            value: ml.header.id as u64,
        });
    }

    let struct_node = ml
        .children
        .iter()
        .find(|c| c.header.id == ids::RW_STRUCT)
        .ok_or(DecodeError::UnexpectedEof {
            what: "material_list.struct",
            need: 1,
            have: 0,
        })?;

    let (material_count, _indices) = decode_material_list_struct(struct_node, file_bytes)?;

    let mut materials = Vec::new();
    for m in ml.children.iter().filter(|c| c.header.id == ids::RW_MATERIAL) {
        materials.push(decode_material_node(m, file_bytes)?);
    }

    Ok(MaterialListInfo {
        material_count,
        materials,
    })
}

/// Decodes one Material node.
///
/// # Expected structure (common RW)
/// - Struct (0x01)
/// - Texture (0x06) optional
/// - Extension (0x03)
pub fn decode_material_node(
    mat: &RwChunkNode,
    file_bytes: &[u8],
) -> Result<MaterialInfo, DecodeError> {
    if mat.header.id != ids::RW_MATERIAL {
        return Err(DecodeError::InvalidValue {
            what: "material_node.id",
            value: mat.header.id as u64,
        });
    }

    let texture = mat
        .children
        .iter()
        .find(|c| c.header.id == ids::RW_TEXTURE)
        .map(|t| decode_texture_node(t, file_bytes))
        .transpose()?;

    let plugins = mat
        .children
        .iter()
        .find(|c| c.header.id == ids::RW_EXTENSION)
        .map(|ext| collect_extension_plugins(ext, file_bytes))
        .unwrap_or_default();

    Ok(MaterialInfo { texture, plugins })
}

fn decode_material_list_struct(
    node: &RwChunkNode,
    file_bytes: &[u8],
) -> Result<(u32, Vec<i32>), DecodeError> {
    let off = node.payload_off as usize;
    let end = node.payload_end as usize;
    if end > file_bytes.len() || off > end {
        return Err(DecodeError::UnexpectedEof {
            what: "material_list_struct.payload",
            need: end.saturating_sub(off),
            have: file_bytes.len().saturating_sub(off),
        });
    }

    let mut r = crate::util::LeReader::new(&file_bytes[off..end]);
    let count = r.read_u32("material_list.count")?;

    // Material indices are i32; many files use -1 for "no material".
    let mut indices = Vec::with_capacity(count as usize);
    for _ in 0..count {
        indices.push(r.read_i32("material_list.index")?);
    }

    Ok((count, indices))
}
