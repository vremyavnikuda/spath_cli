# spath

![spath banner](img/1765746905745-019b1eb7-0a53-7269-9766-481e69cf3b4e.png)

**Languages:** [Русский](docs/README.ru.md) | [日本語](docs/README.ja.md)

Windows PATH security scanner and optimizer.

## Problem

Windows PATH entries with spaces but without quotes create security vulnerabilities that can be exploited for privilege escalation attacks.

## Solution

spath detects and fixes these vulnerabilities automatically.

## Installation

```bash
cargo build --release
```

Binary location: `target/release/spath.exe`

## Commands

### Scan

Analyze PATH for security issues.

```bash
spath scan
spath scan --verbose
spath scan --audit
```

### Fix

Fix USER PATH issues (no admin required).

```bash
spath fix --dry-run
spath fix
spath fix --delicate
```

### Analyze

Analyze both SYSTEM and USER PATH.

```bash
spath analyze
```

### Clean

Remove duplicate paths and optimize PATH.

```bash
spath clean --dry-run
spath clean
spath clean --system
spath clean --delicate
```

### Backup Management

```bash
spath backup
spath list-backups
spath restore <backup-file>
spath restore <backup-file> --delicate
```

## Issue Types

**CRITICAL**: Unquoted paths with spaces - security vulnerability

**WARNING**: Non-existent paths or relative paths

**INFO**: Informational messages

## Workflow

1. Scan: `spath scan --audit`
2. Analyze: `spath analyze`
3. Backup: `spath backup`
4. Fix USER PATH: `spath fix`
5. Remove duplicates: `spath clean`
6. If needed, restore: `spath restore <backup-file>`

## Requirements

- Windows 10 or later
- Rust 1.70+ (for building from source)

## Options

- `--dry-run` or `-d` - Preview changes without applying
- `--delicate` - Ask for confirmation before applying changes
- `--system` or `-s` - Include SYSTEM PATH operations (requires admin)
- `--verbose` or `-v` - Show detailed information
- `--audit` or `-a` - Show detailed audit report

## Notes

- USER PATH changes do not require administrator rights
- SYSTEM PATH changes require administrator rights
- Automatic backup before any changes
- Restart applications to apply PATH changes
- Use `--delicate` for extra safety with confirmation prompts

## License

MIT License - see the [LICENSE](LICENSE) file for details
