use crate::util::DecodeError;
use rw_dff::ids::ids;
use rw_dff::tree::RwChunkNode;
use serde::Serialize;
/// A minimal, deterministic description of one plugin/extension chunk.
///
/// # Purpose
/// - Make "opaque" chunks visible and trackable.
/// - Avoid guessing while still providing stable forensic data.
#[derive(Debug, Clone, Serialize)]
pub struct PluginEntry {
    pub id: u32,
    pub name: String,
    pub version: u32,
    pub header_off: u64,
    pub size: u32,

    /// Optional interpretation tag for known small plugins.
    pub kind: Option<String>,

    /// Optional small preview (hex) for tiny chunks.
    pub preview_hex: Option<String>,
}

/// Collects immediate children of an Extension chunk as plugin entries.
///
/// # Behavior
/// - Only looks at direct children of the Extension node.
/// - For very small payloads (<= 32 bytes), emits a hex preview.
pub fn collect_extension_plugins(ext: &RwChunkNode, file_bytes: &[u8]) -> Vec<PluginEntry> {
    if ext.header.id != ids::RW_EXTENSION {
        return Vec::new();
    }

    ext.children
        .iter()
        .map(|c| {
            let mut preview_hex = None;

            let payload_off = c.payload_off as usize;
            let payload_end = c.payload_end as usize;
            if payload_end <= file_bytes.len() && payload_off <= payload_end {
                let len = payload_end - payload_off;
                if len <= 32 {
                    preview_hex = Some(hex_preview(&file_bytes[payload_off..payload_end]));
                }
            }

            let kind = match c.header.id {
                ids::BINMESH_PLG => Some("BinMeshPLG".to_string()),
                ids::SKYLINE_MESH => Some("SkylineMesh".to_string()),
                ids::SKYLINE_NATIVEDATA => Some("SkylineNativeData".to_string()),
                ids::SKYLINE_DUMMY => Some("SkylineDummy".to_string()),
                _ => None,
            };

            PluginEntry {
                id: c.header.id,
                name: ids::chunk_name(c.header.id).to_string(),
                version: c.header.version,
                header_off: c.header_off,
                size: c.header.size,
                kind,
                preview_hex,
            }
        })
        .collect()
}

/// Converts bytes into a deterministic hex preview without allocations beyond the output.
fn hex_preview(buf: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(buf.len() * 2);
    for &b in buf {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xF) as usize] as char);
    }
    out
}

/// Returns the payload bytes for a node (by offsets into the file buffer).
///
/// # Behavior
/// - Performs strict bounds checks.
/// - Returns a slice into `file_bytes`.
pub fn payload_slice<'a>(
    node: &RwChunkNode,
    file_bytes: &'a [u8],
) -> Result<&'a [u8], DecodeError> {
    let off = node.payload_off as usize;
    let end = node.payload_end as usize;
    if end > file_bytes.len() || off > end {
        return Err(DecodeError::UnexpectedEof {
            what: "payload_slice",
            need: end.saturating_sub(off),
            have: file_bytes.len().saturating_sub(off),
        });
    }
    Ok(&file_bytes[off..end])
}
