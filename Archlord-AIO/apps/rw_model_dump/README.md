# rw_model_dump

`rw_model_dump` is a specialized tool for extracting a high-level summary of model data from RenderWare files. It focuses on the semantic content of the model, such as geometries and materials.

## Features
- **Model Summarization**: Decodes RenderWare `Geometry` and `Material` chunks into a readable format.
- **Detailed Reporting**: Includes information about vertices, triangles, texture references, and used plugins.
- **Deterministic Output**: Generates JSON suitable for comparative analysis and automated testing.

## How it works
The tool uses the `rw_dff_model` library to build a comprehensive report of a RenderWare file. It parses the chunk tree and then applies decoding logic to extract meaningful data from the individual payloads. The result is a structured JSON object that describes the model's visual and structural properties.

## Dependencies
- **Standalone**: Can be used as a standalone analysis tool for model files.
- **Libraries**: Depends on `rw_dff` for tree parsing and `rw_dff_model` for model-specific decoding.

## Usage
```bash
cargo run -p rw_model_dump -- <path-to-file>
```

### Parameters
- `<path-to-file>`: Path to the RenderWare `.dff` or `.txd` file you want to analyze.

### Example
```bash
cargo run -p rw_model_dump -- h0000a00.dff > h0000a00.model.json
```
This command generates a high-level model report for `h0000a00.dff` and saves the output to `h0000a00.model.json`.
