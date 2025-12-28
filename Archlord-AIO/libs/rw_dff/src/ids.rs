/// Common RenderWare chunk IDs and Archlord/engine-specific IDs.
///
/// This module is intentionally small and stable: it maps IDs to readable names
/// for diagnostics and JSON output. It does not implement parsing logic.
// Core RW
pub const RW_STRUCT: u32 = 0x01;
pub const RW_STRING: u32 = 0x02;
pub const RW_EXTENSION: u32 = 0x03;

pub const RW_CAMERA: u32 = 0x05;
pub const RW_TEXTURE: u32 = 0x06;
pub const RW_MATERIAL: u32 = 0x07;
pub const RW_MATERIALLIST: u32 = 0x08;
pub const RW_ATOMICSECTION: u32 = 0x09;
pub const RW_PLANESECTION: u32 = 0x0A;
pub const RW_WORLD: u32 = 0x0B;
pub const RW_SPLINE: u32 = 0x0C;
pub const RW_MATRIX: u32 = 0x0D;

pub const RW_FRAMELIST: u32 = 0x0E;
pub const RW_GEOMETRY: u32 = 0x0F;
pub const RW_CLUMP: u32 = 0x10;
pub const RW_LIGHT: u32 = 0x12;
pub const RW_UNICODESTRING: u32 = 0x13;
pub const RW_ATOMIC: u32 = 0x14;

pub const RW_TEXTNATIVE: u32 = 0x15;
pub const RW_TEXTDICTIONARY: u32 = 0x16;
pub const RW_ANIM_DB: u32 = 0x17;
pub const RW_IMAGE: u32 = 0x18;
pub const RW_SKIN_ANIM: u32 = 0x19;

pub const RW_GEOMETRYLIST: u32 = 0x1A;
pub const RW_HANIM: u32 = 0x11E; // RenderWare HAnim PLG (decimal 286)
pub const RW_SKIN_PLG: u32 = 0x116;

// Archlord / Skyline / engine specific
pub const SKYLINE_ATOMIC: u32 = 0x0112;
pub const SKYLINE_MESH: u32 = 0x011F;
pub const BINMESH_PLG: u32 = 0x050E;
pub const SKYLINE_DUMMY: u32 = 0xFFFF_F001;

/// Returns a human-readable chunk name.
///
/// This is used purely for diagnostics and output formatting.
pub fn chunk_name(id: u32) -> &'static str {
    match id {
        RW_STRUCT => "Struct",
        RW_STRING => "String",
        RW_EXTENSION => "Extension",
        RW_CAMERA => "Camera",
        RW_TEXTURE => "Texture",
        RW_MATERIAL => "Material",
        RW_MATERIALLIST => "MaterialList",
        RW_ATOMICSECTION => "AtomicSection",
        RW_PLANESECTION => "PlaneSection",
        RW_WORLD => "World",
        RW_SPLINE => "Spline",
        RW_MATRIX => "Matrix",
        RW_FRAMELIST => "FrameList",
        RW_GEOMETRY => "Geometry",
        RW_CLUMP => "Clump",
        RW_LIGHT => "Light",
        RW_UNICODESTRING => "UnicodeString",
        RW_ATOMIC => "Atomic",
        RW_TEXTNATIVE => "TextureNative",
        RW_TEXTDICTIONARY => "TextureDictionary",
        RW_ANIM_DB => "AnimationDatabase",
        RW_IMAGE => "Image",
        RW_SKIN_ANIM => "SkinAnimation",
        RW_GEOMETRYLIST => "GeometryList",
        RW_HANIM => "HAnim",
        RW_SKIN_PLG => "SkinPLG",
        SKYLINE_MESH => "SkylineMesh",
        SKYLINE_ATOMIC => "SkylineAtomic",
        BINMESH_PLG => "BinMeshPLG",
        SKYLINE_DUMMY => "SkylineDummy",
        _ => "Unknown",
    }
}
