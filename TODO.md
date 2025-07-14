# Clauditor Implementation TODO

## Pending Tasks

- [ ] Add fallback for terminals without color support (check `TERM=dumb` or `NO_COLOR`).
- [ ] Update display tests for new format.
- [ ] Fix failing tests for `clean_project_paths`.
- [ ] Optimize release build (binary size, strip debug symbols).

## Future Enhancements

- [ ] Add `--color` flag for explicit color control.
- [ ] Add `--json` output mode for scripting.
- [ ] Add configuration file support (`~/.config/clauditor/config.toml`).
- [ ] Implement historical session analysis.
- [ ] Add export functionality (CSV/JSON).
- [ ] Add project filtering options (`--projects`, `--exclude`).

---

## Completed Tasks

- [x] **(Critical)** Fixed vertical alignment of percentages in the display.
- [x] **(Critical)** Refactored billing model to a single account-wide window.
- [x] **(Critical)** Fixed window calculation to be based on recent activity.
- [x] Implemented `--watch` flag for continuous monitoring vs. one-shot mode.
- [x] Added ANSI color constants module.
- [x] Implemented terminal width detection.
- [x] Created smart project path cleaner function.
- [x] Switched time formatting from UTC to local timezone.
- [x] Rewrote header section with color and formatting.
- [x] Implemented dynamic token count alignment.
- [x] Added color coding for time remaining and burn rate.
- [x] Implemented JSONL parser with error handling.
- [x] Implemented session scanner for `~/.claude` and `~/.config/claude`.
- [x] Implemented file watcher for real-time updates.
- [x] Implemented incremental file position tracking.
- [x] Implemented session grouping into 5-hour windows.
- [x] Implemented coordinator to integrate scanning and windowing.
- [x] Implemented Ctrl+C handler for graceful shutdown.
- [x] Added comprehensive unit and integration tests.
- [x] Created test data with edge cases.
- [x] Set up modular Rust project with dependencies.
- [x] Added `CLAUDE.md` and `README.md`.