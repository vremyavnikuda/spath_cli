# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-01-06

### Changed
- Major code refactoring for better maintainability:
- Improved separation of concerns (I/O separated from business logic)

## [0.1.2] - 2025-12-30

### Added
- New `visualize` command to display PATH structure with visual indicators
  - Simple list view with status indicators (✓ exists, ✗ missing, ⚠ issues)
  - Tree view (`--tree`) showing directory hierarchy
  - Color coding: green (valid), red (missing), yellow (issues), cyan (user-specific)
  - Flags: `--user`, `--system`, `--no-color`
  - Automatic detection of issues (non-existent paths, unquoted spaces, duplicates)
  - Summary statistics and legend

### Changed
- Code quality improvements:
  - Replaced hardcoded values with constants across all modules
- Enhanced logging:
  - Added structured logging with `tracing`
  - Log levels: ERROR (registry failures), WARN (vulnerabilities), INFO (operations), DEBUG (details)
  - Controllable via `RUST_LOG` environment variable
- Security improvements:
  - Added ACL protection for backup files
  - Backup files now accessible only to current user
  - Windows API integration for DACL management

### Dependencies
- Added `tracing` and `tracing-subscriber` for structured logging
- Added `atty` for terminal detection
- Added `windows` crate features for security APIs

### Documentation
- Updated with `visualize` command

## [0.1.1] - 2025-12-20

### Added
- New `verify` command to check if critical issues are actually exploitable
- `--system` flag for `scan` command to scan SYSTEM PATH separately
- Function `check_path_exploitable()` to identify truly dangerous unquoted paths
- Function `expand_env_vars()` for proper Windows environment variable expansion
- Detailed exploit file detection (checks for `.exe`, `.com`, `.bat`, `.cmd` files)

### Changed
- Scanner now reads USER PATH from registry by default (not combined SYSTEM+USER)
- Critical issues are now only reported for exploitable paths in system directories (`C:\Program Files`)
- Unquoted paths with spaces in non-system directories are reported as INFO
- Improved security classification: distinguishes between potential risks and real threats
- Updated all documentation (English, Russian, Japanese) with new commands and workflows

### Fixed
- **[#1](https://github.com/vremyavnikuda/spath_cli/issues/1)** Environment variable expansion bug in scanner and fixer
  - **Problem**: Paths like `%SystemRoot%\system32` were incorrectly reported as "Path does not exist"
  - **Root cause**: `trim_matches('%')` removed all `%` characters, causing lookup of wrong variable names (e.g., `SystemRoot%\system32` instead of `SystemRoot`)
  - **Solution**: Implemented proper `expand_env_vars()` function that correctly parses `%VAR%` patterns and expands them recursively
  - **Impact**: Eliminated false positives for all Windows system paths using environment variables
  - **Files affected**: `src/scanner/mod.rs`, `src/fixer/mod.rs`


## [0.1.0] - 2025-12-13

### Added
- `scan` command - Analyze PATH for security issues
- `fix` command - Fix USER PATH issues automatically
- `analyze` command - Analyze both SYSTEM and USER PATH
- `clean` command - Remove duplicate paths and optimize PATH
- `backup` command - Create backup of current PATH
- `list-backups` command - List all available backups
- `restore` command - Restore PATH from backup
- Security vulnerability detection for unquoted paths with spaces
- Automatic backup before any changes
- Dry-run mode for previewing changes
- Delicate mode with confirmation prompts
- Audit reports with detailed information
- Support for both USER and SYSTEM PATH (with appropriate permissions)

### Security
- Detects CRITICAL security vulnerabilities in PATH entries
- Prevents privilege escalation attacks from unquoted paths
