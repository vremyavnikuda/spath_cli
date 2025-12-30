# spath

![spath banner](img/1765746905745-019b1eb7-0a53-7269-9766-481e69cf3b4e.png)

**Languages:** [Русский](docs/README.ru.md) | [日本語](docs/README.ja.md)

Windows PATH security scanner and optimizer.

## Problem

Windows PATH entries with spaces but without quotes create security vulnerabilities that can be exploited for privilege escalation attacks.

## Solution

spath detects and fixes these vulnerabilities automatically.

## Installation

### Download (easiest)

Download from [GitHub Releases](https://github.com/vremyavnikuda/spath_cli/releases):

- `spath-setup.exe` — Installer (adds to PATH automatically)
- `spath.exe` — Standalone executable
- `spath-windows-x64.zip` — Archive with docs

### From crates.io

```bash
cargo install spath-cli
```

### From source

```bash
cargo build --release
```

Binary location: `target/release/spath.exe`

## Commands

### Scan

Analyze PATH for security issues.

```bash
spath scan                    # Scan USER PATH only
spath scan --verbose          # Show detailed information
spath scan --audit            # Show audit statistics
spath scan --system           # Scan SYSTEM PATH (requires admin to fix)
```

### Verify

Check if critical issues are actually exploitable by verifying the presence of malicious files.

```bash
spath verify                  # Verify USER PATH security
spath verify --system         # Verify SYSTEM PATH security
```

This command checks if unquoted paths with spaces can actually be exploited by looking for malicious files like `C:\Program.exe` that could hijack legitimate programs.

### Fix

Fix USER PATH issues (no admin required).

```bash
spath fix --dry-run           # Preview changes without applying
spath fix                     # Apply fixes to USER PATH
spath fix --delicate          # Ask for confirmation before changes
```

### Analyze

Analyze both SYSTEM and USER PATH.

```bash
spath analyze
```

### Clean

Remove duplicate paths and optimize PATH.

```bash
spath clean --dry-run         # Preview cleanup
spath clean                   # Clean USER PATH
spath clean --system          # Clean SYSTEM PATH (requires admin)
spath clean --delicate        # Ask for confirmation
```

### Visualize

Display PATH structure with visual indicators.

```bash
spath visualize               # Simple list view with status indicators
spath visualize --tree        # Tree view showing directory hierarchy
spath visualize --user        # Show only USER PATH
spath visualize --system      # Show only SYSTEM PATH
spath visualize --no-color    # Disable color output
```

Visual indicators:
- ✓ (green) - Path exists and is valid
- ✗ (red) - Path does not exist
- ⚠ (yellow) - Path has issues (unquoted spaces, duplicates)
- (cyan) - User-specific paths

### Backup Management

```bash
spath backup                  # Create backup of current PATH
spath list-backups            # List all available backups
spath restore <backup-file>   # Restore from backup
spath restore <backup-file> --delicate  # Restore with confirmation
```

## Issue Types

**CRITICAL**: Unquoted paths with spaces in system directories (e.g., `C:\Program Files`) - potential security vulnerability that could be exploited

**WARNING**: Non-existent paths, relative paths, or unquoted paths with spaces that don't exist

**INFO**: Informational messages about properly quoted paths or minor issues

## Security Verification

The `verify` command distinguishes between:
- **Potential risks**: Vulnerable paths but no exploit files detected (safe for now)
- **Real threats**: Malicious files found that could exploit the vulnerability (immediate action required)

Example: If `C:\Program Files\App\bin` is in PATH without quotes, the tool checks for:
- `C:\Program.exe`
- `C:\Program.com`
- `C:\Program.bat`
- `C:\Program.cmd`

## Workflow

### Basic Workflow
1. Scan: `spath scan --audit`
2. Verify: `spath verify` (check for real threats)
3. Backup: `spath backup`
4. Fix USER PATH: `spath fix`
5. Remove duplicates: `spath clean`
6. If needed, restore: `spath restore <backup-file>`

### Advanced Workflow (with SYSTEM PATH)
1. Scan SYSTEM: `spath scan --system`
2. Verify SYSTEM: `spath verify --system` (check for exploits)
3. If safe, consider fixing SYSTEM PATH (requires admin rights)

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

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.
