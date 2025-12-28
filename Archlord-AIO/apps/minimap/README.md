# minimap

`minimap` is a tool for generating a comprehensive world map from Archlord's terrain data. It assembles individual terrain tiles into a single, unified map image.

## Features
- **Terrain Assembly**: Combines scattered terrain information into a coherent map.
- **Image Generation**: Produces a visual representation of the game world.
- **Tile Merging**: Specifically merges 2x2 tiles (64 small tiles = 1 full map segment) into a complete world map.

## How it works
The tool scans the destination directory for terrain-related data (e.g., `mapXXXXa/b/c/d.*` files) and uses algorithms in `shared_utils` to stitch these pieces together. It handles the coordinate mapping of tiles to ensure the final world map is spatially accurate and exports the result as a PNG or BMP.

## Dependencies
- **Standalone**: Can be run independently, provided the required terrain data is available in the configured destination path.
- **Integrated**: Launched by `core_main` during the automated processing pipeline.
- **Libraries**: Relies on `shared_utils` for terrain processing and map generation logic.

## Usage
```bash
cargo run -p minimap
```
The tool processes terrain data found in the destination directory specified in your `config.ini` to generate the world map.
