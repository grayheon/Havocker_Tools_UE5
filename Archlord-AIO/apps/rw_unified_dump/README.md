# rw_unified_dump

`rw_unified_dump` is an export-oriented tool that generates a unified mesh representation from RenderWare `.dff` files. It prepares model data in a way that is optimized for conversion to modern formats like glTF.

## Features
- **Unified Mesh Representation**: Merges and organizes disparate RenderWare geometry data into a consistent structure.
- **Material-Based Splitting**: Correctly handles submeshes by splitting them according to their material assignments (using `BinMeshPLG` or fallback logic).
- **Helper Separation**: Distinguishes between actual render meshes and helper geometries (e.g., collision boxes, dummy nodes).
- **Ready for Export**: Provides all necessary data (positions, normals, UVs, indices) in a flat, easy-to-process JSON format.

## How it works
The tool utilizes the `unified_scan` module of `rw_dff_model`. it parses the `.dff` file, identifies all geometries, and reconstructs them into `UnifiedMesh` objects. This includes re-indexing vertex data and handling RenderWare's specific ways of storing triangle strips or lists.

## Dependencies
- **Standalone**: Can be used to inspect how a model will be structured during export.
- **Integrated**: The underlying logic is used by `dff2gltf`.
- **Libraries**: Depends on `rw_dff` and `rw_dff_model`.

## Usage
```bash
cargo run -p rw_unified_dump -- <path-to-dff>
```

### Parameters
- `<path-to-dff>`: Path to the RenderWare `.dff` file to be dumped.

### Example
```bash
cargo run -p rw_unified_dump -- h0000a00.dff > h0000a00.unified.json
```
This command generates a unified JSON representation of `h0000a00.dff` and redirects the output to a file named `h0000a00.unified.json`.
