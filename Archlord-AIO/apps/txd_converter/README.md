# 🎨 TXD Converter

Converts `.txd` files into `.dds` and `.png`, depending on the embedded format.

## Features
- Detects DXT1, DXT3, DXT5 → writes `.dds`
- Converts DDS to PNG using texconv
- Detects raw image data → generates PNG directly
- Creates only required output folders (PNG/DDS)
