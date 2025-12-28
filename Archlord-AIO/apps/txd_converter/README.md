# txd_converter

`txd_converter` is a specialized tool for converting RenderWare `.txd` (Texture Dictionary) files into modern image formats like `.png`. This allows for easy viewing and editing of game textures.

## Features
- **TXD Parsing**: Decodes the binary RenderWare texture dictionaries.
- **Format Conversion**: Converts internal texture formats (including DXT1, DXT3, DXT5, and raw data) into standard PNG images.
- **Pipeline Integration**: Processes all textures within a defined directory structure as part of an automated workflow.

## How it works
The tool iterates through the game's texture directories, identifies `.txd` files, and uses the `process_txd_pipeline` from `shared_utils`. It extracts the raw or compressed image data from the RenderWare chunks and converts it to PNG, preserving transparency where applicable.

## Dependencies
- **Standalone**: Can be used as a standalone conversion utility.
- **Integrated**: Commonly invoked by `core_main` to ensure all game textures are available in a modern format for other tools or development.
- **Libraries**: Relies on `shared_utils` for the heavy lifting of texture decoding and conversion.

## Usage
```bash
cargo run -p txd_converter
```
The tool converts all `.txd` files in the destination directory specified in your `config.ini` to `.png`.
