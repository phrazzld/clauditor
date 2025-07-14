# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**clauditor** is a Rust command-line tool that monitors Claude Code usage across multiple projects. It tracks the single, account-wide 5-hour billing window and displays real-time token usage statistics.

## Commands

### Build & Development
```bash
# Build debug version
cargo build

# Build optimized release version
cargo build --release

# Check compilation without building
cargo check

# Run the application (one-shot mode)
cargo run

# Run in watch mode
cargo run -- --watch

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Architecture

The application follows a modular architecture with clear separation of concerns:

1.  **`main.rs`**: CLI entry point. Parses arguments to run in one-shot or watch mode.
2.  **`types.rs`**: Core data structures (`UsageEntry`, `SessionBlock`, etc.).
3.  **`parser.rs`**: Handles parsing of JSONL session files.
4.  **`scanner.rs`**: Finds and reads session files from the local filesystem.
5.  **`window.rs`**: Implements the logic for grouping usage into a single, 5-hour billing window.
6.  **`coordinator.rs`**: Integrates the scanner and windowing logic to produce a final `SessionBlock`.
7.  **`display.rs`**: Formats the `SessionBlock` for terminal output.
8.  **`watcher.rs`**: (Used in `--watch` mode) Monitors the filesystem for real-time updates.
9.  **`position_tracker.rs`**: (Used in `--watch` mode) Tracks file positions for efficient, incremental reading.

### Key Design Patterns

- **Single Account-Wide Window**: Correctly models Claude's billing by creating a single 5-hour window for all activity.
- **Incremental File Reading**: In watch mode, only reads new data from JSONL files to minimize I/O.
- **Error Resilience**: Skips malformed JSONL lines without crashing.
- **CLI Modes**: Provides both a one-shot view and a continuous watch mode.
