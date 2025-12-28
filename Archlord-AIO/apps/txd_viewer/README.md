# txd_viewer

`txd_viewer` is a graphical debug tool for visualizing the contents of RenderWare `.txd` (Texture Dictionary) files. It allows developers to inspect texture data, offsets, and shifts in real-time.

## Features
- **Visual Inspection**: Load and view textures directly from binary `.txd` files.
- **Dynamic Adjustments**: Real-time sliders for adjusting data offsets, X/Y displacements, and column shifts to help identify the correct alignment of raw texture data.
- **Zooming**: Inspect textures in detail using a zoom feature.
- **Format Support**: Handles different texture dimensions and pixel data.

## How it works
The tool is built using the `eframe` (egui) framework. It provides a user interface where you can pick a `.txd` file, and then manually adjust various parameters (Offset, X-Shift, Y-Shift, etc.) to see how they affect the reconstruction of the image from the raw byte stream. This is particularly useful for reverse-engineering unknown texture formats or debugging alignment issues.

## Dependencies
- **Standalone**: Operates as a standalone GUI application.
- **Libraries**: Uses `eframe` for the UI and `image` for image processing and resizing.

## Usage
```bash
cargo run -p txd_viewer
```
