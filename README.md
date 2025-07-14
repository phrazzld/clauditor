# clauditor

Real-time Claude Code billing window tracker. Monitor token usage across all your projects with accurate 5-hour billing windows.

## Features

- **Real-time monitoring** of Claude Code usage across multiple projects
- **Accurate billing windows** with chronological processing 
- **Live token counts** with burn rate calculations
- **Project breakdown** showing usage by individual projects
- **Color-coded indicators** for time remaining and burn rates
- **File system monitoring** for instant updates when you use Claude Code
- **Terminal-optimized display** with dynamic alignment

## Installation

### From GitHub

```bash
git clone https://github.com/phrazzld/clauditor
cd clauditor
cargo build --release
./target/release/clauditor
```

### Using cargo install

```bash
cargo install --path .
```

## Usage

Simply run `clauditor` to start monitoring:

```bash
clauditor
```

The tool will continuously monitor your Claude Code sessions and display the active billing window. Press `Ctrl+C` to stop.

## Example Output

```
Active billing window
────────────────────────────────────────────────────────────────────────────────

Started 6:00 PM, ends in 3h 21m
Total: 140,214,685 tokens (1,504,718 tokens/min)

scry                                                     28%  40,264,718 tokens
clauditor                                                27%  38,869,826 tokens
chrondle                                                 23%  33,061,672 tokens
vanity                                                   11%  16,585,652 tokens
adminifi/web/feature/a/120                                 8%  11,432,817 tokens
```

## How It Works

clauditor monitors your Claude Code session files located in:
- `~/.claude/projects/*/sessions/`
- `~/.config/claude/projects/*/sessions/`

It implements Claude Code's actual billing model:
- **5-hour billing windows** starting from the floored hour of first activity
- **Single account-wide window** - all projects contribute to the same billing period
- **Chronological processing** ensures entries belong to their correct windows
- **Real-time updates** as you interact with Claude Code

## Performance

- **Initial scan**: <100ms for 50 sessions
- **Memory usage**: <50MB
- **File watching**: Minimal overhead with instant updates
- **Incremental parsing**: Only reads new data from session files

For best performance, use release builds with `cargo build --release`.

## Architecture

- **Modular design** with clear separation of concerns
- **Error resilience** - skips malformed entries and continues processing
- **Incremental file reading** - tracks file positions to minimize I/O
- **Cross-platform** file system monitoring

## Contributing

Built with Rust for performance and reliability. See `CLAUDE.md` for development guidelines.