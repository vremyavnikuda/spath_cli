# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
