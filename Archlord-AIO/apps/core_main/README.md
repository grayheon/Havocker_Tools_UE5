# 🧠 Core Main

This is the central orchestration module that executes all processing steps in a defined sequence, partially in parallel.

## Features
- Loads configuration file
- Executes in order:
  - Extraction
  - TXD conversion
  - Minimap generation
  - Object structure checks
  - DFF scanner
- Handles and reports errors consistently
- Controls parallel execution and waits for completion

