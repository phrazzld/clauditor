# Clauditor Quality Gates TODO

This document outlines the design and implementation of a quality assurance strategy for `clauditor`, using a combination of local Git hooks and automated GitHub Actions.

## Phase 1: Local Quality Gates (Git Hooks)

Implement a `pre-commit` hook to ensure that all code committed to the repository meets a baseline level of quality. This prevents common issues from ever entering the codebase.

- [x] **Setup `pre-commit` Framework**
- [x] **Add Core Checks to `pre-commit` Hook**

## Phase 2: Automated CI/CD Pipeline (GitHub Actions)

Create a GitHub Actions workflow (`.github/workflows/ci.yml`) that runs on every push and pull request to the `master` branch.

- [x] **Create Base CI Workflow**
- [x] **Implement Core CI Checks**
- [x] **Add Advanced Quality Gates**
    - [x] Code Coverage
    - [ ] Code Complexity
    - [ ] Line Count Limits

## Phase 3: Automated Release Workflow

Create a separate GitHub Actions workflow for releases (`.github/workflows/release.yml`) that automates the process of creating and publishing new versions.

- [x] **Create Release Workflow**
- [x] **Automate Release Artifacts**

## Pending Tasks
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
- [x] **(Critical)** Add fallback for terminals without color support.
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