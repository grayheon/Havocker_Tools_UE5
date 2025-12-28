# dff_scanner

`dff_scanner` is a tool for validating texture references within RenderWare `.dff` model files. It ensures that all textures referenced by a model are actually available in the target environment.

## Features
- **Texture Extraction**: Recursively parses the RenderWare chunk tree to find `RW_TEXTURE` chunks.
- **Validation**: Compares extracted texture names against a set of known available `.png` textures.
- **Reporting**: Generates a `<model_name>_textures.txt` file for each processed `.dff`, listing valid and invalid/missing texture references.

## How it works
The tool uses the `rw_dff` library to parse the binary structure of `.dff` files. It specifically looks for texture name strings within the RenderWare hierarchy. By cross-referencing these names with actual image files on disk, it can identify broken references that would lead to missing textures in-game.

## Dependencies
- **Standalone**: Can be run independently, but relies on a `config.ini` to define the destination path where the `.dff` files and textures are located.
- **Integrated**: Typically called by `core_main` as part of the full processing pipeline.
- **Libraries**: Depends on `rw_dff` for parsing and `rw_dff_model` for decoding texture information.

## Usage
```bash
cargo run -p dff_scanner
```
The tool scans the directory specified in your `config.ini` for `.dff` files and validates their texture references against the textures found in the same environment.
