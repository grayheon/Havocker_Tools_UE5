# rw_dump

`rw_dump` is a diagnostic tool for inspecting the internal structure of RenderWare files (`.dff`, `.txd`). it provides a raw view of the RenderWare chunk tree.

## Features
- **Chunk Tree Visualization**: Parses the nested structure of RenderWare chunks.
- **JSON Output**: Generates a deterministic JSON representation of the file structure.
- **No Interpretation**: Focuses on raw data and hierarchy without applying game-specific semantics.

## How it works
The tool takes a path to a RenderWare file as input, uses the `rw_dff` library to parse the binary chunk hierarchy, and serializes the resulting tree to JSON, which is then printed to stdout. It is primarily used for debugging and generating "golden files" for regression testing.

## Dependencies
- **Standalone**: Operates as a simple command-line utility.
- **Libraries**: Relies on `rw_dff` for the core parsing logic and `serde_json` for serialization.

## Usage
```bash
cargo run -p rw_dump -- <path-to-file>
```

### Parameters
- `<path-to-file>`: Path to the RenderWare `.dff` or `.txd` file you want to dump.

### Example
```bash
cargo run -p rw_dump -- h0000a00.dff > h0000a00.tree.json
```
This command parses the chunk tree of `h0000a00.dff` and saves the resulting JSON structure to `h0000a00.tree.json`.
