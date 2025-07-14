# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**clauditor** is a Rust command-line tool that monitors Claude Code usage across multiple sessions, tracking active billing windows and displaying real-time token usage statistics.

## Commands

### Build & Development
```bash
# Build debug version
cargo build

# Build optimized release version
cargo build --release

# Check compilation without building
cargo check

# Run the application
cargo run

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

# Run specific module tests
cargo test parser::tests
cargo test window::tests
cargo test scanner::tests

# Run a specific test
cargo test test_parse_valid_line
```

## Architecture

The application follows a modular architecture with clear separation of concerns:

1. **main.rs** - CLI entry point, orchestrates the monitoring loop with Ctrl+C handling
2. **types.rs** - Core data structures (TokenUsage, UsageEntry, SessionBlock, WindowData)
3. **parser.rs** - JSONL parsing with error resilience
4. **scanner.rs** - Finds session files in ~/.claude/projects/ and ~/.config/claude/projects/
5. **window.rs** - Groups usage into 5-hour billing windows
6. **coordinator.rs** - Integrates scanning and window grouping
7. **display.rs** - Terminal output formatting (follows minimal design, no boxes/emojis)
8. **watcher.rs** - File system monitoring for real-time updates
9. **position_tracker.rs** - Tracks file positions for incremental reading

### Key Design Patterns

- **Incremental File Reading**: Only reads new data from JSONL files to minimize I/O
- **5-Hour Billing Windows**: Groups usage starting from floored hours (e.g., 12:00, 17:00)
- **Error Resilience**: Skips malformed JSONL lines, continues processing
- **Real-time Updates**: Uses `notify` crate for file system monitoring
- **Minimal UI**: No fancy formatting, focuses on clear data presentation

### Performance Targets

- Initial scan: <100ms for 50 sessions
- Memory usage: <50MB
- File watching with minimal overhead

## Testing Approach

- Unit tests embedded in modules using `#[cfg(test)]`
- Test data in `test_data/sample.jsonl` includes various edge cases
- Tests cover parsing errors, window calculations, and display formatting
- Use `tempfile` crate for file system tests