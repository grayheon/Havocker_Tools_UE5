# obj_checker

`obj_checker` is a validation tool that ensures the consistency and integrity of object templates and their associated models (DFF) and textures (TID) in the Archlord data.

## Features
- **Object Template Processing**: Analyzes and validates the structure of `objecttemplate.ini`.
- **TID Validation**: Cross-checks Texture ID (TID) entries against the object templates.
- **DFF Consistency Check**: Extracts and verifies `.dff` model files to ensure they match the template definitions.
- **Reference Integrity**: Detects missing or mismatched links between templates, models, and textures.

## How it works
The tool loads the processed game data and performs a series of checks using validation logic from `shared_utils`. It iterates through the object templates, follows the references to model and texture files, and verifies their existence and basic properties. This helps identifying data corruption or missing assets that would cause issues in the game client.

## Dependencies
- **Standalone**: Can be executed independently if the processed data is available at the destination path.
- **Integrated**: Automatically triggered by `core_main` during the main processing flow.
- **Libraries**: Heavily relies on `shared_utils` for template parsing and validation routines.

## Usage
```bash
cargo run -p obj_checker
```
The tool validates object and model consistency within the destination directory specified in your `config.ini`.
