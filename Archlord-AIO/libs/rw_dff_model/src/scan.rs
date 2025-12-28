use crate::binmesh::{BinMesh, BinMeshPlg};
use crate::geometry::{GeometryStructSummary, decode_geometry_struct};
use crate::material::MaterialListInfo;
use crate::plugins::PluginEntry;
use crate::skin::SkinData;
use rw_dff::ids;
use rw_dff::tree::RwChunkNode;
use std::fs;
use std::path::Path;

/// A report entry for one Geometry chunk found in a file.
#[derive(Debug, serde::Serialize)]
pub struct GeometryReportEntry {
    pub geometry_header_off: u64,
    pub struct_header_off: u64,
    pub struct_size: u32,
    pub summary: GeometryStructSummary,

    pub material_list: Option<MaterialListInfo>,
    pub extension_plugins: Vec<PluginEntry>,

    pub is_helper_geometry: bool,

    pub binmesh: Option<BinMesh>,
    pub binmesh_warnings: Vec<String>,

    pub binmesh_preview: Option<BinMesh>,
    pub binmesh_plg: Option<BinMeshPlg>,

    pub skin: Option<SkinData>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("I/O error: {0}")]
    Io(std::io::Error),

    #[error("missing Struct child for Geometry at 0x{off:X}")]
    MissingGeometryStruct { off: u64 },

    #[error(
        "out of bounds read at 0x{off:X}: payload_off=0x{payload_off:X}, payload_end=0x{payload_end:X}, file_len=0x{file_len:X}"
    )]
    OutOfBounds {
        off: u64,
        payload_off: u64,
        payload_end: u64,
        file_len: u64,
    },

    /// 👇 DAS FEHLTE
    #[error("decode error: {0}")]
    Decode(#[from] crate::util::DecodeError),
}

/// Scans a parsed chunk tree, finds all Geometry->Struct nodes,
/// reads their payload bytes from disk, and decodes summaries.
///
/// # Behavior
/// - Traverses the whole tree (including embedded streams already expanded by rw_dff).
/// - For each Geometry chunk, finds the first child Struct chunk and decodes it.
/// - Returns a deterministic list sorted by file offset.
///
/// # Notes
/// This function reads raw bytes from disk based on offsets; therefore it requires
/// the original file path.
pub fn scan_geometry_structs(
    path: &Path,
    root: &RwChunkNode,
) -> Result<Vec<GeometryReportEntry>, ScanError> {
    let data = fs::read(path).map_err(ScanError::Io)?;

    let mut out = Vec::new();
    collect_geometry(root, &data, &mut out)?;

    out.sort_by_key(|e| e.geometry_header_off);
    Ok(out)
}

fn collect_geometry(
    node: &RwChunkNode,
    data: &[u8],
    out: &mut Vec<GeometryReportEntry>,
) -> Result<(), ScanError> {
    if node.header.id == ids::RW_GEOMETRY {
        // Find first child Struct
        let struct_node = node
            .children
            .iter()
            .find(|c| c.header.id == ids::RW_STRUCT)
            .ok_or(ScanError::MissingGeometryStruct {
                off: node.header_off,
            })?;

        let payload_off = struct_node.payload_off as usize;
        let payload_end = struct_node.payload_end as usize;

        if payload_end > data.len() || payload_off > payload_end {
            return Err(ScanError::OutOfBounds {
                off: struct_node.header_off,
                payload_off: struct_node.payload_off,
                payload_end: struct_node.payload_end,
                file_len: data.len() as u64,
            });
        }

        let buf = &data[payload_off..payload_end];
        let summary = decode_geometry_struct(buf).map_err(ScanError::Decode)?;

        // MaterialList (optional)
        let material_list = node
            .children
            .iter()
            .find(|c| c.header.id == ids::RW_MATERIALLIST)
            .map(|ml| crate::material::decode_material_list_node(ml, data))
            .transpose()
            .ok()
            .flatten();

        // Geometry Extension plugins (optional)
        let extension_plugins = node
            .children
            .iter()
            .find(|c| c.header.id == ids::RW_EXTENSION)
            .map(|ext| crate::plugins::collect_extension_plugins(ext, data))
            .unwrap_or_default();

        let material_count = material_list
            .as_ref()
            .map(|m| m.material_count)
            .unwrap_or(0);
        let has_any_texture = material_list
            .as_ref()
            .map(|ml| ml.materials.iter().any(|m| m.texture.is_some()))
            .unwrap_or(false);

        let is_helper_geometry = summary.num_vertices == 3
            && summary.num_triangles == 1
            && summary.num_uv_sets == 0
            && !summary.has_normals
            && material_count == 1
            && !has_any_texture;

        let mut binmesh_preview: Option<BinMesh> = None;
        let mut binmesh_plg: Option<BinMeshPlg> = None;
        let mut skin: Option<SkinData> = None;

        let mut binmesh = None;
        let mut binmesh_warnings = Vec::new();

        if let Some(bm) = node
            .children
            .iter()
            .find(|c| c.header.id == ids::RW_EXTENSION)
            .and_then(|ext| {
                ext.children
                    .iter()
                    .find(|c| c.header.id == ids::BINMESH_PLG)
            })
        {
            let payload = crate::plugins::payload_slice(bm, data)?;

            match crate::binmesh::decode_binmesh(payload, 16) {
                Ok(decoded) => {
                    if let Some(ml) = &material_list {
                        for (i, m) in decoded.meshes.iter().enumerate() {
                            if m.material_index < 0
                                || (m.material_index as u32) >= ml.material_count
                            {
                                binmesh_warnings.push(format!(
                                    "mesh[{i}]: material_index={} out of range (material_count={})",
                                    m.material_index, ml.material_count
                                ));
                            }
                        }
                    }

                    for (i, m) in decoded.meshes.iter().enumerate() {
                        if matches!(m.max_index, Some(max_i) if max_i >= summary.num_vertices) {
                            binmesh_warnings.push(format!(
                                "mesh[{i}]: max_index={} >= num_vertices={} (engine-expanded vertices or non-classic layout)",
                                m.max_index.unwrap(), summary.num_vertices
                            ));
                        }
                    }

                    if decoded.remaining_bytes != 0 {
                        binmesh_warnings.push(format!(
                            "binmesh: remaining_bytes={} (non-classic layout or padding)",
                            decoded.remaining_bytes
                        ));
                    }

                    binmesh = Some(decoded);
                }
                Err(e) => {
                    binmesh_warnings.push(format!("binmesh decode failed: {e}"));
                }
            }

            match crate::binmesh::decode_binmesh(payload, 16) {
                Ok(decoded) => {
                    binmesh_preview = Some(decoded);
                }
                Err(e) => {
                    binmesh_warnings.push(format!("binmesh preview decode failed: {e}"));
                }
            }

            match crate::binmesh::decode_binmesh_plg(payload) {
                Ok(decoded) => {
                    binmesh_plg = Some(decoded);
                }
                Err(e) => {
                    binmesh_warnings.push(format!("binmesh_plg decode failed: {e}"));
                }
            }

        }

        // Some files may place SkinPLG directly under Geometry->Extension without BinMesh
        if skin.is_none() {
            if let Some(ext) = node
                .children
                .iter()
                .find(|c| c.header.id == ids::RW_EXTENSION)
            {
                if let Some(skin_node) = ext
                    .children
                    .iter()
                    .find(|c| c.header.id == ids::RW_SKIN_PLG)
                {
                    if let Ok(bytes) = crate::plugins::payload_slice(skin_node, data) {
                        match crate::skin::decode_skin_plg(bytes, summary.num_vertices as usize) {
                            Ok(decoded) => skin = Some(decoded),
                            Err(e) => binmesh_warnings.push(format!("skin decode failed: {e}")),
                        }
                    }
                }
            }
        }
        out.push(GeometryReportEntry {
            geometry_header_off: node.header_off,
            struct_header_off: struct_node.header_off,
            struct_size: struct_node.header.size,
            summary,
            material_list,
            extension_plugins,
            is_helper_geometry,
            binmesh,
            binmesh_warnings,
            binmesh_preview,
            binmesh_plg,
            skin,
        });
    }

    for c in &node.children {
        collect_geometry(c, data, out)?;
    }
    Ok(())
}
