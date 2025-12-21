use crate::util::DecodeError;
use rw_dff::ids::ids;
use rw_dff::tree::RwChunkNode;
use serde::Serialize;

use crate::plugins::{collect_extension_plugins, PluginEntry};

#[derive(Debug, Clone, Serialize)]
pub struct TextureInfo {
    pub name: Option<String>,
    pub mask: Option<String>,
    pub plugins: Vec<PluginEntry>,
}
/// Decoded RW String (0x02) payload.
///
/// # Behavior
/// - RW strings are typically null-terminated ASCII in many games.
/// - We decode as lossy UTF-8 for robustness and trim trailing NULs.
/// - Raw bytes are not preserved here; if needed, store them separately.
#[derive(Debug, Clone, Serialize)]
pub struct RwString {
    pub value: String,
}

/// Decodes a RW String chunk payload.
///
/// # Notes
/// RenderWare commonly stores C-strings including the trailing NUL.
/// Some files may omit NUL; we handle both.
pub fn decode_rw_string(payload: &[u8]) -> Result<RwString, DecodeError> {
    // Trim trailing NULs but keep internal NULs (rare).
    let trimmed = payload
        .iter()
        .rposition(|&b| b != 0)
        .map(|i| &payload[..=i])
        .unwrap_or(&[]);

    let value = String::from_utf8_lossy(trimmed).to_string();
    Ok(RwString { value })
}


/// Decodes a Texture chunk (0x06) inside a Material.
///
/// # Expected structure (common RW)
/// - Struct (0x01)
/// - String (0x02) name
/// - String (0x02) mask
/// - Extension (0x03)
///
/// # Behavior
/// - Decodes name/mask strings if present.
/// - Collects extension plugins.
pub fn decode_texture_node(
    tex: &RwChunkNode,
    file_bytes: &[u8],
) -> Result<TextureInfo, DecodeError> {
    if tex.header.id != ids::RW_TEXTURE {
        return Err(DecodeError::InvalidValue {
            what: "texture_node.id",
            value: tex.header.id as u64,
        });
    }

    let mut name = None;
    let mut mask = None;
    let mut plugins = Vec::new();

    // Find the first two String children (name, mask)
    let mut strings = tex.children.iter().filter(|c| c.header.id == ids::RW_STRING);

    if let Some(s) = strings.next() {
        name = Some(read_string_payload(s, file_bytes)?);
    }
    if let Some(s) = strings.next() {
        mask = Some(read_string_payload(s, file_bytes)?);
    }

    if let Some(ext) = tex.children.iter().find(|c| c.header.id == ids::RW_EXTENSION) {
        plugins = collect_extension_plugins(ext, file_bytes);
    }

    Ok(TextureInfo { name, mask, plugins })
}

fn read_string_payload(node: &RwChunkNode, file_bytes: &[u8]) -> Result<String, DecodeError> {
    let off = node.payload_off as usize;
    let end = node.payload_end as usize;
    if end > file_bytes.len() || off > end {
        return Err(DecodeError::UnexpectedEof {
            what: "rw_string.payload",
            need: end.saturating_sub(off),
            have: file_bytes.len().saturating_sub(off),
        });
    }
    let s = decode_rw_string(&file_bytes[off..end])?;
    Ok(s.value)
}
