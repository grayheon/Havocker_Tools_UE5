use crate::scan::GeometryReportEntry;
use serde::Serialize;

/// Top-level report for a single file.
///
/// # Purpose
/// - Deterministic golden-file output
/// - Quick sanity checks for geometry layouts
#[derive(Debug, Serialize)]
pub struct DffModelReport {
    pub file: String,
    pub geometries: Vec<GeometryReportEntry>,
}
