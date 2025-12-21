//! RW DFF "model" decoding layer.
//!
//! # Scope
//! - Decodes known Struct payloads into readable summaries.
//! - Does not apply Archlord engine semantics (splits, UV derivation, etc.).
//!
//! This crate depends on `rw_dff` for the chunk tree and uses offsets to read
//! payload bytes from the original file.

pub mod geometry;
pub mod json;
pub mod scan;
pub mod util;
mod texture;
mod plugins;
mod material;
mod binmesh;
mod geometry_full;
mod unified;
pub mod unified_scan;

use crate::json::DffModelReport;
use crate::scan::scan_geometry_structs;
use rw_dff::parse_file;
use std::path::Path;

/// Builds a model report for one DFF/TXD file.
///
/// # Behavior
/// - Parses a chunk tree via `rw_dff`.
/// - Scans all Geometry->Struct payloads and decodes summaries.
/// - Returns a deterministic report suitable for golden-files.
pub fn build_report(path: &Path) -> Result<DffModelReport, BuildError> {
    let root = parse_file(path)?;
    let geometries = scan_geometry_structs(path, &root)?;
    Ok(DffModelReport {
        file: path.display().to_string(),
        geometries,
    })
}

/// Errors produced while building the report.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("rw tree parse error: {0}")]
    Tree(#[from] rw_dff::reader::RwReadError),

    #[error("scan/decode error: {0}")]
    Scan(#[from] crate::scan::ScanError),
}
