# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ftree is a terminal-based file tree TUI application built with Rust and ratatui. It provides vim-like navigation, fuzzy search, Git status display, and file preview with syntax highlighting.

## Build & Run Commands

```bash
# Build
cargo build

# Run in development
cargo run

# Build for release
cargo build --release

# Install locally
cargo install --path .

# Run installed binary
ftree
```

## Architecture

The application follows a simple model-view-controller pattern:

```
main.rs          # Entry point, event loop, terminal setup
├── app.rs       # Application state (App struct) - central state management
├── ui.rs        # Rendering logic using ratatui
├── input.rs     # Keyboard and mouse input handling
├── tree.rs      # File tree data structure (FileTree, TreeNode)
├── search.rs    # Fuzzy search using nucleo-matcher
├── git.rs       # Git status integration via `git status --porcelain`
├── preview.rs   # File preview with syntect syntax highlighting
└── clipboard.rs # Clipboard operations via arboard
```

### Key Data Structures

- **`App`** (`app.rs`): Central application state containing tree, cursor, mode, search state, git status, preview state
- **`FileTree`** (`tree.rs`): Flat vector of `TreeNode`s with parent/children indices, expansion state tracked via HashMap
- **`SearchState`** (`search.rs`): Maintains fuzzy search query and ranked match results
- **`PreviewContent`** (`preview.rs`): Syntax-highlighted file content with lazy-loaded SyntaxSet/ThemeSet

### Modes

The app operates in two modes defined in `app.rs`:
- `Mode::Normal`: Standard file tree navigation
- `Mode::Search`: Fuzzy search input mode

Additional overlays: Help popup (`show_help`), File preview (`show_preview`)

### Event Flow

1. `main.rs` runs the event loop reading crossterm events
2. Events dispatch to `input.rs` handlers based on current mode
3. Handlers mutate `App` state
4. `ui.rs::render()` draws current state to terminal

## Dependencies

- **ratatui** - TUI framework
- **crossterm** - Terminal backend
- **walkdir** - Directory traversal (max depth: 10)
- **nucleo-matcher** - Fuzzy matching
- **syntect** - Syntax highlighting (theme: base16-ocean.dark)
- **arboard** - System clipboard

## Notes

- Uses Rust edition 2024
- Git status uses `git status --porcelain -uall` and propagates status to parent directories
- Preview limits to 1000 lines, checks first 8192 bytes for binary detection
- Mouse capture is disabled during preview to allow text selection
