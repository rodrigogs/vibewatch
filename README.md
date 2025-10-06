# VibeWatch

[![CI](https://github.com/rodrigogs/vibewatch/actions/workflows/ci.yml/badge.svg)](https://github.com/rodrigogs/vibewatch/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/rodrigogs/vibewatch/branch/master/graph/badge.svg)](https://codecov.io/gh/rodrigogs/vibewatch)
[![Crates.io](https://img.shields.io/crates/v/vibewatch.svg)](https://crates.io/crates/vibewatch)
[![Downloads](https://img.shields.io/crates/d/vibewatch.svg)](https://crates.io/crates/vibewatch)
[![License](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.89.0+-orange.svg)](https://www.rust-lang.org)

A fast and extensible file watcher utility built in Rust with glob pattern support and cross-platform compatibility.

## Features

- **Custom command execution**: Run commands on file changes with event-specific triggers (`--on-create`, `--on-modify`, `--on-delete`, `--on-change`)
- **Template substitution**: Use `{file_path}`, `{relative_path}`, `{absolute_path}`, `{event_type}` in commands
- **Cross-platform file watching**: Fully tested on Linux, macOS, and Windows with platform-specific event handling
- **Glob pattern support**: Include and exclude files using glob patterns like `*.rs`, `node_modules/**`
- **Extensible architecture**: Clean separation of concerns for easy feature additions
- **Fast performance**: Built in Rust for minimal resource usage
- **Flexible CLI**: Intuitive command-line interface with verbose logging support
- **Comprehensive testing**: 95.77% code coverage with 187 tests covering unit, filesystem, and integration scenarios

## Installation

### From crates.io (Recommended)

```bash
cargo install vibewatch
```

### From Binary Releases

Download pre-built binaries for Linux, macOS, or Windows from the [latest release](https://github.com/rodrigogs/vibewatch/releases/latest).

### From Source

Make sure you have Rust installed via `mise` or `rustup`:

```bash
# Clone and build
git clone https://github.com/rodrigogs/vibewatch.git
cd vibewatch
cargo build --release
```

## ⚡ Performance

vibewatch v0.2.0 includes significant performance optimizations:

- **40-60% fewer memory allocations** through static strings and optimized path handling
- **15-30% faster event processing** via async channels (tokio)
- **80-95% fewer redundant commands** with intelligent event debouncing
- **Proper shell parsing** with quote support via shell-words crate

### Benchmarks

Run comprehensive benchmarks yourself:

```bash
cargo bench                               # All benchmarks
cargo bench --bench template_substitution # Specific suite
```

The benchmark suite includes:
- **Template substitution** - Single-pass vs multi-pass string operations
- **Path normalization** - Platform-specific optimization validation  
- **Pattern matching** - Glob compilation and matching performance

See commit messages for detailed optimization rationale.

## Usage

### Command Execution on File Changes

The primary use case is executing commands when files change:

```bash
# Run tests on any change
vibewatch . --on-change "npm test"

# Format Rust files when modified
vibewatch src --include "*.rs" --on-modify "rustfmt {file_path}"

# Run linter on TypeScript files
vibewatch . --include "*.{ts,tsx}" --exclude "node_modules/**" --on-modify "npx eslint {file_path} --fix"

# Different commands for different events
vibewatch src \
  --on-create "git add {file_path}" \
  --on-modify "cargo check" \
  --on-delete "echo Removed: {relative_path}"
```

**Available Templates:**
- `{file_path}` - Full path to the changed file
- `{relative_path}` - Path relative to watched directory
- `{absolute_path}` - Absolute path to the changed file
- `{event_type}` - Type of event (create, modify, delete)

### Watch-Only Mode

Watch a directory and log all file changes (no commands):

```bash
vibewatch /path/to/directory
```

### Include Patterns

Watch only specific file types:

```bash
vibewatch /path/to/directory --include "*.rs" --include "*.ts"
```

### Exclude Patterns

Ignore common directories and files:

```bash
vibewatch /path/to/directory --exclude "node_modules/**" --exclude ".git/**" --exclude "target/**"
```

### Combined Patterns

Use both include and exclude patterns:

```bash
vibewatch . \
  --include "*.rs" \
  --include "*.ts" \
  --include "*.tsx" \
  --exclude "target/**" \
  --exclude "node_modules/**" \
  --verbose
```

### Options

**Directory:**
- `<DIRECTORY>`: Directory to watch (can be relative or absolute)

**Command Execution:**
- `--on-create <COMMAND>`: Run command when files are created
- `--on-modify <COMMAND>`: Run command when files are modified
- `--on-delete <COMMAND>`: Run command when files are deleted
- `--on-change <COMMAND>`: Run command on any file change (fallback)

**Filtering:**
- `-i, --include <PATTERN>`: Include patterns like `*.ts`, `*.tsx`, `*.rs`
- `-e, --exclude <PATTERN>`: Exclude patterns like `node_modules/**`, `.git/**`, `.next/**`

**General:**
- `-v, --verbose`: Enable verbose output with debug logging
- `-h, --help`: Show help message
- `-V, --version`: Show version information

## Examples

### Auto-format TypeScript on save

```bash
vibewatch src \
  --include "*.ts" --include "*.tsx" \
  --exclude "node_modules/**" --exclude "dist/**" \
  --on-modify "npx prettier --write {file_path}"
```

### Run Rust tests on file changes

```bash
vibewatch . \
  --include "*.rs" --include "Cargo.toml" \
  --exclude "target/**" \
  --on-change "cargo test"
```

### Rebuild documentation on changes

```bash
vibewatch docs \
  --include "*.md" --include "*.rst" \
  --exclude "_build/**" \
  --on-change "mdbook build"
```

### Watch and restart development server

```bash
vibewatch src \
  --include "*.js" --include "*.json" \
  --exclude "node_modules/**" \
  --on-change "pkill -f 'node server.js' && node server.js &"
```

### Auto-commit on file creation

```bash
vibewatch src \
  --on-create "git add {file_path} && git commit -m 'Add {relative_path}'"
```

## Architecture

The application is structured for extensibility:

- **`main.rs`**: CLI argument parsing and application entry point
- **`watcher.rs`**: Core file watching logic using the `notify` crate
- **`filter.rs`**: Glob pattern matching for include/exclude functionality

## Common Glob Patterns

### Exclude Patterns
- `node_modules/**` - Node.js dependencies
- `.git/**` - Git repository files
- `.next/**` - Next.js build files
- `target/**` - Rust build directory
- `dist/**` - Build output directory
- `*.tmp` - Temporary files
- `*.swp` - Vim swap files

### Include Patterns
- `*.rs` - Rust source files
- `*.ts`, `*.tsx` - TypeScript files
- `*.js`, `*.jsx` - JavaScript files  
- `*.py` - Python files
- `*.go` - Go files
- `*.cpp`, `*.c`, `*.h` - C/C++ files
- `*.md` - Markdown files

## Future Enhancements

The following features are planned for future releases:

- **Configuration file support**: Store watch patterns and commands in `.vibewatch.toml`
- **Ignore file support**: Respect `.gitignore`, `.watchignore` patterns

## Requirements

- Rust 1.70+ (managed via `mise`)
- Unix-like system (macOS, Linux) or Windows

## Development

### Quick Start with Just

The project uses [`just`](https://github.com/casey/just) for task automation:

```bash
# Install just (if not already installed)
cargo install just
# or: brew install just

# List all available tasks
just --list

# Common tasks
just test              # Run all tests
just coverage          # Generate coverage report
just lint              # Run linter
just check             # Run all checks (fmt, lint, test)
just demo              # Run vibewatch on src/ directory
```

### Running Tests

```bash
# Run all tests (187 total: 140 unit + 21 filesystem + 26 integration)
cargo test
# or: just test

# Run tests with output
cargo test -- --nocapture
# or: just test-verbose

# Run specific test suite
cargo test --test it  # Integration tests only
# or: just test-integration test_name
```

### Code Coverage

The project maintains **95.77% code coverage** (996/1040 lines):

```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace --html

# View report
open target/llvm-cov/html/index.html
```

**Coverage by file:**
- `filter.rs`: 100.00% (190/190) ✅
- `main.rs`: 98.05% (252/257) ✅
- `watcher.rs`: 93.42% (554/593) ✅

> **Note:** The remaining 4.23% uncovered lines are in `main()` and `start_watching()` functions that run as subprocesses during integration tests. Coverage tools cannot track across process boundaries. These lines are functionally tested through our comprehensive integration test suite.

### Development Commands

```bash
# Run with debug logging
RUST_LOG=debug cargo run -- /path/to/directory --verbose

# Build for release
cargo build --release

# Run linter
cargo clippy

# Format code
cargo fmt
```

## Contributing

### Commit Message Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/) for automated versioning and changelog generation.

**Format**: `<type>(<optional scope>): <description>`

**Types:**
- `feat:` - New feature (triggers minor version bump)
- `fix:` - Bug fix (triggers patch version bump)
- `docs:` - Documentation changes
- `chore:` - Maintenance tasks
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `feat!:` or `fix!:` - Breaking changes (triggers major version bump)

**Examples:**
```bash
feat: add support for symlink watching
fix: resolve race condition in file detection
docs: update README with new examples
feat!: change CLI argument structure (breaking change)
```

### Release Process

Releases are automated via [Release Please](https://github.com/googleapis/release-please):

1. **Commit using conventional commits** - Each commit to `master` is analyzed
2. **Release PR is created** - Release Please opens a PR with updated version and CHANGELOG
3. **Merge the Release PR** - This triggers:
   - GitHub Release creation
   - Binary builds for Linux, macOS, Windows (x86_64, ARM64)
   - Optional publish to crates.io (if `CARGO_TOKEN` secret is configured)

**Manual release**: Just merge the automatically created "chore: release X.Y.Z" PR.

**Setup Required**: Release Please needs a Personal Access Token to create PRs. See [`docs/RELEASE_PLEASE_SETUP.md`](docs/RELEASE_PLEASE_SETUP.md) for detailed setup instructions.

### CI/CD

All PRs and pushes to `master` run:
- ✅ Tests (187 tests on Linux, macOS, Windows)
- ✅ Formatting check (`cargo fmt --check`)
- ✅ Linting (`cargo clippy`)
- ✅ Coverage report (uploaded to Codecov)

## Documentation

For comprehensive technical documentation:
- **Release Please Setup**: [`docs/RELEASE_PLEASE_SETUP.md`](docs/RELEASE_PLEASE_SETUP.md) - Configure automated releases with PAT
- **Testing Guide**: `docs/TESTING.md` - Test organization, best practices, and quick reference
- **Coverage Analysis**: `docs/COVERAGE.md` - Detailed coverage metrics and industry benchmarks
- **Integration Tests**: `docs/INTEGRATION_TEST.md` - Testing research, rationale, and best practices
- **Justfile Guide**: `docs/JUSTFILE_IMPLEMENTATION.md` - Task runner implementation and benefits
