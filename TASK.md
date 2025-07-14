# TASK.md: Multi-Session Claude Code Usage Tracker

## Executive Summary

Build a focused command-line tool that tracks token usage across ALL active Claude Code sessions simultaneously, addressing the critical limitation where `ccusage blocks --live` only displays one active session at a time.

## Problem Context

### How Claude Code Stores Usage Data

Claude Code stores usage data in JSONL (JSON Lines) files with the following structure:

```
~/.claude/projects/{project-name}/{session-uuid}.jsonl
```

Example paths:
```
~/.claude/projects/-Users-phaedrus-Development-ccusage/a8e2b963-cd62-4c5f-9e6e-f9cf4dcb23b5.jsonl
~/.claude/projects/-Users-phaedrus-Development-adminifi-web--feature-a-160/e5d33a11-e9cb-4ac7-90da-1c9f31d749a8.jsonl
```

Note: Claude Code recently moved from `~/.claude` to `~/.config/claude` as the default location. Tools must check both paths.

### JSONL Entry Structure

Each line in a JSONL file is a separate JSON object. The critical entries for usage tracking have this structure:

```json
{
  "timestamp": "2025-07-12T16:03:28.593Z",
  "message": {
    "id": "msg_01QB3q4aPG1gsE54YVH185S9",
    "type": "message",
    "role": "assistant",
    "model": "claude-opus-4-20250514",
    "usage": {
      "input_tokens": 10,
      "output_tokens": 7,
      "cache_creation_input_tokens": 5174,
      "cache_read_input_tokens": 13568
    }
  },
  "costUSD": 0.0125,  // Optional - may not be present
  "requestId": "req_011CR3QAZByoJd2TpJFRxWLf",
  "version": "1.0.51"
}
```

### Session Block Concept

Claude bills usage in 5-hour blocks. A "session block" is:
- A 5-hour window starting from the first activity
- "Active" if the last activity was within 5 hours
- Can contain gaps if no activity for extended periods
- Multiple blocks can be active simultaneously across different projects

### The Multi-Session Problem

Currently, `ccusage blocks --live` has this critical flaw:

```typescript
// Find active block
return sortedBlocks.find(block => block.isActive) ?? null;
```

This returns only the FIRST active block, ignoring all other concurrent sessions. Users running multiple Claude Code instances simultaneously only see partial usage data.

## Detailed Requirements

### Core Functionality

1. **Multi-Session Detection**
   - Monitor ALL projects directories: `~/.claude/projects/*` and `~/.config/claude/projects/*`
   - Track all JSONL files modified within the session duration (default 5 hours)
   - Identify which files belong to currently active sessions
   - Handle sessions that span multiple JSONL files within the same project

2. **Real-Time Monitoring**
   - Watch for new entries appended to existing JSONL files
   - Detect new session files created during monitoring
   - Update metrics immediately when new usage data arrives
   - Efficient incremental reading (only new lines since last check)

3. **Usage Aggregation**
   - Sum tokens across all active sessions:
     - `input_tokens`
     - `output_tokens`
     - `cache_creation_input_tokens`
     - `cache_read_input_tokens`
   - Calculate combined burn rate (tokens per minute)
   - Project total usage if current rate continues
   - Track costs (using costUSD when available, or calculate from pricing data)

4. **Project Attribution**
   - Extract project names from file paths
   - Show per-project token usage
   - Identify which projects are consuming the most resources
   - Handle malformed project paths gracefully

### Display Requirements

#### Layout Structure

```
┌────────────────────────────────────────────────────────────────┐
│          CLAUDE CODE - MULTI-SESSION USAGE MONITOR             │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│ 🔥 ACTIVE SESSIONS: 3                                          │
│                                                                │
│ ⚡ COMBINED USAGE                                              │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│ Total Tokens:     45,678 / 500,000 (9.1%)                     │
│ Burn Rate:        1,234 tokens/min (HIGH)                      │
│ Projected:        234,567 tokens by session end                │
│ Total Cost:       $12.45                                       │
│                                                                │
│ 📊 PER-PROJECT BREAKDOWN                                       │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│ ccusage                        12,345 tokens  27%  $3.21       │
│   └─ Started: 2:45 PM, 1h 23m elapsed, HIGH burn rate         │
│ adminifi-web--feature-a-160    23,456 tokens  51%  $6.78       │
│   └─ Started: 3:15 PM, 53m elapsed, MODERATE burn rate        │
│ adminifi-consumer-portal        9,877 tokens  22%  $2.46       │
│   └─ Started: 3:30 PM, 38m elapsed, NORMAL burn rate          │
│                                                                │
│ ⚠️  WARNINGS                                                    │
│ • Combined usage approaching 50% of daily limit                │
│ • Project 'adminifi-web' has HIGH burn rate (>1000 tok/min)   │
│                                                                │
├────────────────────────────────────────────────────────────────┤
│       ↻ Refreshing every 5s  •  Press Ctrl+C to stop          │
└────────────────────────────────────────────────────────────────┘
```

#### Display Features

1. **Header Section**
   - Clear title indicating multi-session monitoring
   - Count of active sessions

2. **Combined Usage Section**
   - Total tokens across all sessions with progress bar
   - Combined burn rate with severity indicator (NORMAL/MODERATE/HIGH)
   - Projected total usage if current rate continues
   - Total cost accumulation

3. **Per-Project Breakdown**
   - Project name (cleaned from file path)
   - Token count and percentage of total
   - Cost for that project
   - Session details: start time, elapsed time, burn rate
   - Visual indicators for concerning usage patterns

4. **Warnings Section**
   - Token limit warnings (when approaching or exceeding limits)
   - High burn rate alerts
   - Cost threshold warnings
   - New session detection notices

### Technical Implementation Details

#### File Watching Strategy

```typescript
interface SessionFile {
  path: string;
  project: string;
  sessionId: string;
  lastModified: number;
  lastReadPosition: number;
  entries: UsageEntry[];
}

class MultiSessionMonitor {
  private sessions: Map<string, SessionFile> = new Map();
  private lastScanTime: number = Date.now() - (5 * 60 * 60 * 1000);
  
  async scanForActiveSessions(): Promise<void> {
    // Find all JSONL files modified since lastScanTime
    // Group by project
    // Track file positions for incremental reading
  }
  
  async readNewEntries(session: SessionFile): Promise<UsageEntry[]> {
    // Read only from lastReadPosition to EOF
    // Parse JSONL lines
    // Update lastReadPosition
  }
}
```

#### Efficient Incremental Reading

1. Track file size/position for each monitored file
2. Only read new bytes appended since last check
3. Handle partial line reads (store incomplete lines for next read)
4. Parse complete JSONL lines into usage entries
5. Deduplicate entries using requestId or messageId

#### Data Aggregation

```typescript
interface AggregatedUsage {
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCacheCreationTokens: number;
  totalCacheReadTokens: number;
  totalCost: number;
  projectBreakdown: Map<string, ProjectUsage>;
  oldestActiveSession: Date;
  newestActivity: Date;
}

interface ProjectUsage {
  projectName: string;
  sessions: SessionInfo[];
  inputTokens: number;
  outputTokens: number;
  cacheTokens: number;
  cost: number;
  startTime: Date;
  lastActivity: Date;
  burnRate: number; // tokens per minute
}
```

#### Terminal Rendering

1. Use ANSI escape codes for:
   - Cursor positioning (avoid full screen clears)
   - Color coding (red for warnings, yellow for caution, green for normal)
   - Progress bars with Unicode box characters

2. Double-buffering to prevent flicker:
   ```typescript
   startBuffering();
   renderDisplay(aggregatedData);
   flush();
   ```

3. Responsive layout:
   - Detect terminal width
   - Compact mode for narrow terminals (<80 chars)
   - Truncate project names if needed

### Error Handling

1. **Missing Directories**
   - Check both `~/.claude` and `~/.config/claude`
   - Create informative error if neither exists

2. **Malformed JSONL**
   - Skip invalid lines silently
   - Log parsing errors to debug file if verbose mode

3. **File Access Issues**
   - Handle locked files gracefully
   - Retry failed reads with exponential backoff

4. **Performance Degradation**
   - Limit number of monitored files (e.g., last 100 sessions)
   - Implement memory caps for stored entries
   - Clear old inactive sessions from memory

### Configuration Options

```bash
# Command-line arguments
--refresh-interval <seconds>    # Update frequency (default: 5)
--token-limit <number>          # Daily token limit for warnings
--session-duration <hours>      # Override 5-hour default
--projects <list>               # Monitor only specific projects
--exclude <list>               # Exclude specific projects
--cost-threshold <amount>       # Warn when cost exceeds threshold
--compact                       # Force compact display mode
--json                          # Output JSON instead of TUI
--debug                         # Enable debug logging
```

### Cost Calculation

When `costUSD` is not present in entries:

1. Use model-specific pricing:
   ```
   claude-sonnet-4-20250514: $0.003/1K input, $0.015/1K output
   claude-opus-4-20250514: $0.015/1K input, $0.075/1K output
   ```

2. Cache token pricing is typically:
   - Cache creation: 25% of input token cost
   - Cache read: 10% of input token cost

3. Fall back to LiteLLM pricing database if available

### Performance Targets

- Initial scan: <100ms for 1000 sessions
- Incremental updates: <10ms per active session
- Memory usage: <50MB for typical usage
- CPU usage: <5% during monitoring

### Testing Scenarios

1. **Single Active Session**
   - Verify it shows correctly
   - Ensure burn rate calculation is accurate

2. **Multiple Sessions Same Project**
   - Multiple JSONL files in same project directory
   - Correct aggregation of tokens

3. **Multiple Projects**
   - 5+ concurrent sessions across different projects
   - Proper project name extraction and display

4. **Session Transitions**
   - Session becoming inactive (>5 hours old)
   - New session starting during monitoring
   - Session reactivating after gap

5. **Edge Cases**
   - Corrupted JSONL files
   - Extremely long project names
   - Rapid token usage (>10K tokens/minute)
   - Clock skew in timestamps

### Future Enhancements

1. **Historical Analysis**
   - Show usage trends over time
   - Compare current session to historical averages

2. **Alerts and Notifications**
   - Desktop notifications for limit warnings
   - Webhook integration for team alerts

3. **Export Capabilities**
   - CSV export of session data
   - Integration with monitoring tools

4. **Session Management**
   - Ability to "kill" expensive sessions
   - Set per-project token limits

This tool fills a critical gap in Claude Code usage monitoring, providing the comprehensive multi-session visibility that teams need to manage their token consumption effectively.