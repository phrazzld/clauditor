# clauditor

Track active Claude Code billing windows across multiple sessions.

## Installation

### From source

```bash
git clone https://github.com/yourusername/clauditor
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

Started 2:00 PM, ends in 2h 15m
  Total: 45,678 tokens (152 tokens/min)
  
  ccusage                    12,345 tokens
  adminifi-web              23,456 tokens  
  adminifi-consumer          9,877 tokens
```

## Performance

- Initial scan: <100ms for 50 sessions
- Memory usage: <50MB
- Real-time updates via file watching

For best performance, use release builds.