# dff2gltf

`dff2gltf` is a converter that transforms RenderWare `.dff` model files into the modern glTF 2.0 format. This enables the use of Archlord assets in modern 3D engines and tools like Blender.

## Features
- **glTF 2.0 Export**: Produces standard `.gltf` and `.bin` files.
- **Unified Mesh Processing**: Uses a unified mesh representation to ensure consistent geometry export.
- **Material Support**: Creates glTF primitives corresponding to RenderWare material splits.
- **Texture Coordination**: Emits multiple UV sets (texcoord_0..4) and maps them to the correct materials.
- **Automatic Organization**: Places the exported files and referenced textures into a designated output directory.

## How it works
The tool first builds a unified mesh report from the input `.dff` file using `rw_dff_model`. It then hands this report over to the `gltf_writer` library, which constructs the glTF JSON structure and the accompanying binary buffer. It also attempts to locate and link the necessary textures.

## Dependencies
- **Standalone**: Can be used as a standalone command-line converter.
- **Libraries**: Depends on `rw_dff`, `rw_dff_model`, and `gltf_writer`.

## Usage
```bash
cargo run -p dff2gltf -- <input.dff> [output_directory]
```

### Parameters
- `<input.dff>`: Path to the RenderWare `.dff` model file you want to convert.
- `[output_directory]` (Optional): The directory where the resulting `.gltf` and `.bin` files will be saved. If omitted, the files will be placed in the same directory as the input file.

### Example
```bash
cargo run -p dff2gltf -- models/h0000a00.dff ./export
```
This command converts `h0000a00.dff` and saves `h0000a00.gltf` and `h0000a00.bin` into the `./export` folder. It will also attempt to locate and copy the necessary textures into the same folder.
