# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | :white_check_mark: |
| < 0.3   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in spath, please report it responsibly.

### How to Report

1. **Do NOT** open a public GitHub issue for security vulnerabilities
2. Email the maintainer directly at: [hopperplayer0@gmail.com](mailto:hopperplayer0@gmail.com)
3. Include the following information:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Resolution Timeline**: Depends on severity
  - Critical: 24-72 hours
  - High: 1-2 weeks
  - Medium: 2-4 weeks
  - Low: Next release

### Disclosure Policy

- We follow responsible disclosure practices
- Security fixes will be released as soon as possible
- Credit will be given to reporters (unless anonymity is requested)
- Public disclosure after patch is available

## Security Considerations

### What spath Does

spath is a security tool that:
- Scans Windows PATH for vulnerabilities (unquoted paths with spaces)
- Modifies Windows Registry (HKCU and HKLM)
- Creates backup files in `%LOCALAPPDATA%\spath\backups\`

### Permissions

| Operation | Admin Required | Registry Key |
|-----------|----------------|--------------|
| Scan PATH | No | Read only |
| Fix USER PATH | No | HKCU\Environment |
| Analyze | No | Read only |
| Fix SYSTEM PATH | **Yes** | HKLM\...\Environment |
| Clean (with --system) | **Yes** | HKLM\...\Environment |

### Security Features

1. **Automatic Backups**: Before any PATH modification, a backup is created
2. **Dry-Run Mode**: Preview changes with `--dry-run` before applying
3. **Delicate Mode**: Confirmation prompts with `--delicate`
4. **PATH Length Validation**: Prevents exceeding Windows 2047 character limit
5. **No Network Access**: All operations are local

### Known Limitations

1. **Backup files are not encrypted**: Stored as plain JSON in user's AppData
2. **No file locking**: Concurrent modifications may cause issues
3. **Registry permissions**: Relies on Windows ACLs for protection

### Best Practices

When using spath:

1. **Always backup first**: Run `spath backup` before making changes
2. **Use dry-run**: Preview with `--dry-run` before applying
3. **Review changes**: Check what will be modified
4. **Run as standard user**: Only use admin when necessary
5. **Keep backups**: Don't delete backup files immediately

### Threat Model

spath is designed to fix security vulnerabilities, but consider:

| Threat | Mitigation |
|--------|------------|
| Malicious PATH injection | spath detects and can fix unquoted paths |
| Backup file tampering | Store backups in protected location |
| Registry corruption | Automatic backup before changes |
| Privilege escalation | Minimal admin operations, user confirmation |

## Security Checklist for Contributors

When contributing code:

- [ ] No hardcoded credentials or secrets
- [ ] Input validation for all user inputs
- [ ] Error messages don't leak sensitive information
- [ ] Registry operations use minimal required permissions
- [ ] File operations validate paths (no path traversal)
- [ ] Dependencies are from trusted sources
- [ ] No unnecessary network access

## Dependencies

spath uses the following security-relevant dependencies:

| Crate | Purpose | Security Notes |
|-------|---------|----------------|
| `winreg` | Registry access | Well-maintained, Windows-specific |
| `anyhow` | Error handling | No security concerns |
| `clap` | CLI parsing | Input validation built-in |
| `serde_json` | JSON parsing | Safe deserialization |

All dependencies are regularly updated via Dependabot.

## Audit History

| Date | Version | Auditor | Findings |
|------|---------|---------|----------|
| 2024-12 | 0.3.0 | Internal | Initial security review |

## Contact

- **Security Issues**: [hopperplayer0@gmail.com](mailto:hopperplayer0@gmail.com)
- **General Issues**: [GitHub Issues](https://github.com/vremyavnikuda/spath/issues)
- **Repository**: [github.com/vremyavnikuda/spath](https://github.com/vremyavnikuda/spath)
