# zrogorge
Rust-based NeoVim-like Text Editor

## Current project structure

1. Root Files
- `Cargo.toml`: Defines dependencies, project metadata, and features.
- `src/`: Holds all source code for the application.
2. `src/` Subdirectories
- `main.rs`:
  - The application entry point.
  - Handles the initialization logic and calls appropriate modules based on CLI arguments.
- `splash.rs`:
  - Contains logic to render the splash screen and load ASCII art from the `assets/ascii_art/` directory.
- `cli.rs`:
  - Handles parsing command-line arguments and determines the application mode (splash screen, directory view, or file editing).
`editor.rs`:
  - Implements text buffer, file I/O, and core editor functionality.
- `ui/`:
  - Houses all terminal UI components.
  - `directory_tree.rs`:
    - Displays directory structure for browsing.
  - `editor_ui.rs`:
    - Manages the text editor pane layout and rendering.
  - `status_bar.rs`:
    - Displays contextual information (e.g., file name, mode).
  - `command_line.rs`:
    - Handles command input and execution (e.g., :e, :w).
- `utils/`:
  - Contains utility modules for reusable functionality.
  - `file_operations.rs`:
    - Helper functions for file and directory management.
  - `ascii_art.rs`:
    - Handles reading and rendering ASCII art.
  - `config.rs`:
    - Parses configuration files for themes, key mappings, and startup options.
3. `assets/`
  - `ascii_art/`:
    - Store ASCII art used for the splash screen and other customizations.
  - `config/`:
    - Contains default and user-specific configuration files and themes.
4. `tests/`
  - Write integration tests for the various features of the application.

