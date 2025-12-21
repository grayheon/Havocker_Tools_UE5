/// Common RenderWare chunk IDs and Archlord/engine-specific IDs.
///
/// This module is intentionally small and stable: it maps IDs to readable names
/// for diagnostics and JSON output. It does not implement parsing logic.
pub mod ids {
    // Core RW
    pub const RW_STRUCT: u32 = 0x01;
    pub const RW_STRING: u32 = 0x02;
    pub const RW_EXTENSION: u32 = 0x03;

    pub const RW_TEXTURE: u32 = 0x06;
    pub const RW_MATERIAL: u32 = 0x07;
    pub const RW_MATERIALLIST: u32 = 0x08;

    pub const RW_FRAMELIST: u32 = 0x0E;
    pub const RW_GEOMETRY: u32 = 0x0F;
    pub const RW_CLUMP: u32 = 0x10;
    pub const RW_ATOMIC: u32 = 0x14;

    pub const RW_TEXTNATIVE: u32 = 0x15;
    pub const RW_TEXTDICTIONARY: u32 = 0x16;

    pub const RW_GEOMETRYLIST: u32 = 0x1A;

    // Archlord / Skyline / engine specific
    pub const SKYLINE_ATOMIC: u32 = 0x0112;
    pub const SKYLINE_NATIVEDATA: u32 = 0x011E;
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
            RW_TEXTURE => "Texture",
            RW_MATERIAL => "Material",
            RW_MATERIALLIST => "MaterialList",
            RW_FRAMELIST => "FrameList",
            RW_GEOMETRY => "Geometry",
            RW_CLUMP => "Clump",
            RW_ATOMIC => "Atomic",
            RW_TEXTNATIVE => "TextureNative",
            RW_TEXTDICTIONARY => "TextureDictionary",
            RW_GEOMETRYLIST => "GeometryList",
            SKYLINE_NATIVEDATA => "SkylineNativeData",
            SKYLINE_MESH => "SkylineMesh",
            SKYLINE_ATOMIC => "SkylineAtomic",
            BINMESH_PLG => "BinMeshPLG",
            SKYLINE_DUMMY => "SkylineDummy",
            _ => "Unknown",
        }
    }
}
