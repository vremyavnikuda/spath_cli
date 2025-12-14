# Contributing to spath

Thank you for your interest in contributing to spath! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Windows 10/11** - This is a Windows-specific tool
- **Git** - For version control

### Setup

```bash
# Clone the repository
git clone https://github.com/vremyavnikuda/spath.git
cd spath

# Build the project
cargo build

# Run tests
cargo test

# Run the tool
cargo run -- scan
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

Follow the code style guidelines below and ensure your changes:
- Compile without errors: `cargo check`
- Pass all tests: `cargo test`
- Have no clippy warnings: `cargo clippy`
- Are properly formatted: `cargo fmt`

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test scanner_tests

# Run with verbose output
cargo test -- --nocapture
```

### 4. Submit a Pull Request

- Write a clear PR description
- Reference any related issues
- Ensure CI checks pass

## Project Structure

```
spath/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── registry.rs       # Windows Registry operations with file locking
│   ├── formatter.rs      # Console output formatting
│   ├── scanner/          # PATH scanning
│   ├── analyzer/         # SYSTEM/USER PATH analysis
│   ├── fixer/            # PATH fixing and backup
│   └── migrator/         # PATH migration and cleanup
├── tests/                # Test files
└── docs/                 # Localized documentation
```

## Code Style

### Formatting

We use `rustfmt` with the following settings (see `rustfmt.toml`):
- Max line width: 100 characters
- 4 spaces indentation (no tabs)
- Auto reorder imports

Run before committing:
```bash
cargo fmt
```

### Linting

We use `clippy` for linting:
```bash
cargo clippy
```

All warnings should be resolved before submitting a PR.

### Error Handling

- Use `anyhow::Result` for fallible operations
- Add context with `.context()` for better error messages
- Handle errors gracefully, especially for admin-required operations

```rust
// Good
let path = RegistryHelper::read_user_path()
    .context("Failed to read user PATH")?;

// Avoid
let path = RegistryHelper::read_user_path().unwrap();
```

### Documentation

- Add doc comments for public functions
- Use `///` for documentation comments
- Include examples where helpful

```rust
/// Reads the USER PATH from Windows Registry.
///
/// # Returns
/// A vector of path entries parsed from the PATH string.
///
/// # Errors
/// Returns an error if the registry key cannot be accessed.
pub fn read_user_path() -> Result<Vec<String>> {
    // ...
}
```

## Testing Guidelines

### Writing Tests

- Place unit tests in the same file or in `tests/` directory
- Use descriptive test names
- Test both success and error cases

```rust
#[test]
fn test_parse_path_string_with_empty_entries() {
    let path = "C:\\Windows;;C:\\System32";
    let parsed = RegistryHelper::parse_path_string(path);
    assert_eq!(parsed.len(), 2);
}
```

### Test Categories

- `scanner_tests.rs` - PATH scanning and issue detection
- `analyzer_tests.rs` - Path categorization
- `fixer_tests.rs` - Backup and restore operations
- `migrator_tests.rs` - Migration planning
- `registry_tests.rs` - Registry operations and file locking
- `integration_tests.rs` - End-to-end workflows
- `utils_tests.rs` - Utility functions

## Adding New Features

### 1. Plan the Feature

- Open an issue to discuss the feature
- Get feedback before implementing

### 2. Implement

- Follow existing patterns in the codebase
- Keep modules focused (single responsibility)
- Use the `RegistryHelper` for registry operations

### 3. Add Tests

- Write tests for new functionality
- Ensure existing tests still pass

### 4. Update Documentation

- Update README.md if needed
- Add doc comments to new public APIs
- Update localized docs if applicable (docs/README.ru.md, docs/README.ja.md)

## Reporting Issues

### Bug Reports

Include:
- Windows version
- spath version
- Steps to reproduce
- Expected vs actual behavior
- Error messages (if any)

### Feature Requests

Include:
- Use case description
- Proposed solution
- Alternatives considered

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Help others learn and grow

## Questions?

- Open an issue for questions
- Check existing issues and documentation first

Thank you for contributing! 🎉
