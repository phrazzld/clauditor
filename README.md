# clauditor

**Monitor your Claude Code token usage in real-time.**

`clauditor` is a command-line tool that gives you an accurate, up-to-the-minute view of your token consumption across all projects, ensuring you never get surprised by a billing window closing unexpectedly.

## Features

- **Real-Time Monitoring**: Instantly see your token usage as it happens.
- **Accurate Billing Windows**: Tracks the single, account-wide 5-hour window exactly as Claude bills it.
- **Live Token Counts**: View total tokens and burn rate (tokens/minute).
- **Project Breakdown**: See which projects are consuming the most tokens.
- **Color-Coded Urgency**: Time remaining and burn rates are colored to show urgency at a glance.

## Installation

```bash
# Clone the repository
git clone https://github.com/phrazzld/clauditor
cd clauditor

# Build and install
cargo install --path .
```

## Usage

### One-Time Check

Run `clauditor` without any flags to get a snapshot of your current billing window:

```bash
clauditor
```

### Live Monitoring

Use the `--watch` flag to monitor usage continuously. The display will update in real-time as you use Claude Code.

```bash
clauditor --watch
```
Press `Ctrl+C` to exit watch mode.

## How It Works

`clauditor` monitors session files in `~/.claude/projects/` and `~/.config/claude/projects/`. It implements Claude's billing model: a single, 5-hour window for your entire account, starting from the first recent activity. This provides a single source of truth for your token consumption.
