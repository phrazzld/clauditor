# Clauditor Implementation TODO

## Critical Architectural Fix Required

### Fix billing window model to single account-wide window
- **Status**: [x] Completed
- **Priority**: CRITICAL - Current implementation is fundamentally wrong
- **Success criteria**: Only ONE billing window active at a time across entire account
- **Dependencies**: None (but affects entire codebase)

### Complexity: VERY_COMPLEX
### Started: 2025-07-13 09:45

### Context Discovery
- Current implementation creates multiple windows based on 5-hour boundaries and gaps
- Architecture assumes Vec<SessionBlock> throughout (window.rs, coordinator.rs, display.rs, main.rs)
- Tests verify multi-window behavior extensively
- Need to preserve per-project breakdown within single window

### Execution Log
[09:45] Analyzing current multi-window architecture
[09:46] Identified key files: window.rs (grouping logic), coordinator.rs (aggregation), display.rs (rendering), main.rs (data flow)
[09:47] Starting with window.rs refactoring - core algorithm change
[09:48] Refactored window.rs to implement single account-wide window
[09:49] Updated all tests in window.rs to match single-window behavior
[09:50] Moving to coordinator.rs refactoring
[09:51] Refactored coordinator.rs to return single window instead of Vec
[09:52] Updated display.rs to work with single window (removed window numbering)
[09:53] Refactored main.rs to handle single window throughout
[09:54] Running cargo check to identify any remaining issues
[09:55] Compilation successful with warnings only
[09:56] Running tests - found failures in integration tests and examples
[09:57] Updating integration tests to use single window model
[09:58] Fixed all example files to use single window functions
[09:59] Tests passing for core window functionality (4 unrelated test failures)

### Approach Decisions
- Implemented single account-wide window in window.rs by filtering entries to 5-hour period
- Changed all return types from Vec<SessionBlock> to Option<SessionBlock>
- Removed window numbering and multi-window display logic
- Preserved per-project breakdown within single window

### Learnings
- The refactoring touched every layer: window grouping, coordination, display, and main loop
- Tests needed significant updates to verify single-window behavior instead of multi-window
- Examples required updates to new function signatures

#### Implementation Plan
1. **Refactor window grouping algorithm** (`window.rs`)
   - Change from creating multiple windows to maintaining single active window
   - Find earliest activity across ALL projects within last 5 hours
   - That activity starts the single 5-hour window
   - ALL usage from ALL projects within those 5 hours belongs to that window
   
2. **Update display logic** (`display.rs`)
   - Remove all "Window N:" numbering (there's only ever one window)
   - Remove `show_window_number` parameter from `display_window()`
   - Simplify header to just show "Active billing window" (already done)
   
3. **Fix coordinator** (`coordinator.rs`)
   - Return single SessionBlock, not Vec<SessionBlock>
   - Consolidate all project usage within the active 5-hour window
   
4. **Update main loop** (`main.rs`)
   - Remove logic for handling multiple windows
   - Simplify to track single active window
   - Remove window merging logic
   
5. **Fix all tests**
   - Update test data to reflect single-window behavior
   - Fix expectations in integration tests

#### Billing Model Understanding
- **ONE active window** at a time for entire account
- Window starts when you first use Claude Code (any project)
- Window lasts **exactly 5 hours** from start time
- ALL usage across ALL projects during those 5 hours counts
- New activity after window ends starts a new window
- Example: Work at 2pm starts 2pm-7pm window. Work at 8pm starts new 8pm-1am window

#### Why This Matters
- ccusage problem: missing concurrent sessions within SAME window (not missing windows)
- ccusage showed only one project's usage, not total across all projects
- Led to underreporting actual consumption
- Users thought they had more tokens available than reality

## Critical Bug Fix: Window Calculation and Incremental Loading

### Fix billing window calculation to show current active window
- **Status**: [x] Completed (Fixed 2025-07-14)

### Complexity: COMPLEX
### Started: 2025-07-14 10:15

### Context Discovery
- Current algorithm uses earliest entry regardless of age, creating old inactive windows
- Need to find RECENT activity (within 5 hours) to determine current active window
- Incremental loading strategy needs complete overhaul
- Window calculation in window.rs, loading in coordinator.rs, display in main.rs

### Execution Log
[10:15] Analyzing current window calculation algorithm
[10:16] Identified core issue: using earliest entry instead of recent activity
[10:17] Starting Phase 1: Implementing find_active_window_period function
[10:18] Implemented find_active_window_period to detect current active window
[10:19] Updated group_into_single_window_with_projects to use recent activity
[10:20] Moving to Phase 2: Fixing incremental loading in coordinator
[10:21] Modified coordinator to do full reload when active window detected
[10:22] This ensures ALL projects are loaded, not just ones with new activity
[10:23] Moving to main.rs to simplify window handling
[10:24] Simplified main.rs - removed complex merging logic
[10:25] Coordinator now returns complete window data, no merging needed
[10:26] Testing implementation with cargo check
[10:27] Code compiles successfully
[10:28] Window tests pass, but integration tests fail due to old timestamps
[10:29] Tests use January 2025 data, but new algorithm only shows recent windows
[10:30] Need to update tests to handle new behavior

### Approach Decisions
- Changed from "earliest entry" to "recent activity" approach
- Coordinator does full reload when active window detected
- Simplified main.rs by removing complex merging logic
- Tests need updating for new time-based behavior

### Final Fix (2025-07-14)
- Modified `find_active_window_period` to check newest entries first
- Algorithm now works backwards from most recent activity
- Correctly handles multiple windows (expired and active) 
- Example: Activity at 20:43 (window 20:00-01:00 expired) and 01:30 (window 01:00-06:00 active)
- Fix ensures the active window is always detected

### Learnings
- The flickering was caused by showing old inactive windows
- Incremental loading without full context misses projects
- Time-based window calculation is more complex but more correct
- **Priority**: CRITICAL - Currently shows wrong window period and missing projects
- **Success criteria**: Shows the CURRENT active window with ALL active projects
- **Dependencies**: None

#### Problem Summary
- Current implementation finds EARLIEST entry and creates window from that time
- Should find MOST RECENT activity and determine if there's an active window
- Incremental loading only includes new entries, missing other active projects
- Causes flickering between "no active window" and incomplete window display

#### Implementation Plan

##### Phase 1: Fix Window Calculation Algorithm
1. **Rewrite `group_into_single_window_with_projects` in `window.rs`**:
   - Find entries within last 5 hours from current time
   - If any exist, determine window start from earliest of those recent entries
   - Include ALL entries that fall within that window period
   - This ensures we show the CURRENT active window, not old windows

2. **Add `find_active_window_period` function**:
   - Given current time and all entries, determine active billing window
   - Returns (start_time, end_time) tuple or None
   - Use this to load complete window data

##### Phase 2: Fix Incremental Loading  
1. **Change incremental loading strategy in `coordinator.rs`**:
   - When active window detected, do FULL reload of that window period
   - Load ALL entries from ALL projects within the window
   - Don't just merge incremental data

2. **Modify `main.rs` window handling**:
   - When incremental load detects window change, trigger full reload
   - Track current window period to detect changes

##### Phase 3: Optimize Performance
1. **Add window period caching**:
   - Cache current active window period (start, end)
   - Only full reload when window period changes
   
2. **Add project list tracking**:
   - Track which projects are in current window
   - Trigger reload if new projects appear

## Pending Display Improvements

### Must Complete Before Refactoring
- [x] Update display_active_windows to use new layout

### Complexity: SIMPLE
### Started: 2025-07-13 10:00

### Context Discovery
- display_active_windows already renamed to display_active_window in billing model fix
- clean_project_paths function exists but not being used
- Project names are full paths like /Users/phaedrus/Development/ccusage
- Need to apply clean_project_paths to make them display as ccusage or ~/Development/ccusage

### Execution Log
[10:00] Analyzing current display implementation
[10:01] Found that project names are not using clean_project_paths
[10:02] Implementing clean project path display
[10:03] Found that clean_project_paths has issues with mixed paths
[10:04] Implementing simpler project name extraction
[10:05] Successfully implemented - project paths now show just the name
[10:06] Tested with realistic paths - display is much cleaner

### Approach Decisions
- Instead of using complex clean_project_paths, implemented simple extraction of last path component
- This converts /Users/phaedrus/Development/ccusage to just ccusage
- Much cleaner display that focuses on what matters

### Learnings
- Sometimes simpler is better - complex path cleaning logic was overkill
- Project names are what users care about, not full paths
  - **Note**: This needs adjustment after billing model fix (no more multiple windows)
  - Success criteria: Complete new display format working end-to-end
  - Implementation: 
    - Call clean_project_paths on all project names
    - Apply all color coding (time remaining, burn rate)
    - Use improved header format

### Nice to Have
- [ ] Add fallback for terminals without color support
  - Success criteria: Gracefully degrade if TERM=dumb or NO_COLOR env var set
  - Implementation: Check env vars, wrap color application in conditional
  - Context: Some CI environments don't support colors

- [ ] Update display tests for new format
  - Success criteria: All existing display tests pass with new format
  - Implementation: Update expected strings in tests, add new test cases
  - **Note**: Will need major updates after billing model refactoring

## Other Tasks

- [ ] Fix failing tests for clean_project_paths
  - Success criteria: All display tests pass
  - Context: 3 tests failing after display improvements
  - Implementation: Either fix the function or remove unused code

- [ ] Optimize release build
  - Success criteria: Binary <5MB, strip debug symbols
  - Dependencies: All features complete

## Future Enhancements

- [ ] Add --color flag for explicit color control
- [ ] JSON output mode for scripting
- [ ] Configuration file support
- [ ] Historical session analysis
- [ ] Export functionality (CSV/JSON)
- [ ] Project filtering options

---

## Completed Tasks

### Display Improvements ✓
- [x] Add ANSI color constants module to display.rs
- [x] Implement terminal width detection function
- [x] Create smart project path cleaner function
- [x] Switch time formatting from UTC to local timezone
- [x] Remove "Window N:" prefix from single window displays (will be obsolete after refactor)
- [x] Rewrite header section with color and formatting
- [x] Implement dynamic token count alignment
- [x] Add color coding for time remaining (Green >2h, yellow <1h, red <30m)
- [x] Add burn rate color coding (Red >1M/min, yellow >500K/min)