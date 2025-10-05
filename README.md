# VibeWatch

A fast and extensible file watcher utility built in Rust with glob pattern support.

## Features

- **Cross-platform file watching**: Uses the `notify` crate for efficient file system monitoring
- **Glob pattern support**: Include and exclude files using glob patterns like `*.rs`, `node_modules/**`
- **Extensible architecture**: Clean separation of concerns for easy feature additions
- **Fast performance**: Built in Rust for minimal resource usage
- **Flexible CLI**: Intuitive command-line interface with verbose logging support
- **Comprehensive testing**: 95.77% code coverage with 187 tests covering unit, filesystem, and integration scenarios

## Installation

Make sure you have Rust installed via `mise`:

```bash
# Install dependencies and build
cargo build --release
```

## Usage

### Basic Usage

Watch a directory and log all file changes:

```bash
./target/release/vibewatch /path/to/directory
```

### Include Patterns

Watch only specific file types:

```bash
./target/release/vibewatch /path/to/directory --include "*.rs" --include "*.ts"
```

### Exclude Patterns

Ignore common directories and files:

```bash
./target/release/vibewatch /path/to/directory --exclude "node_modules/**" --exclude ".git/**" --exclude "target/**"
```

### Combined Patterns

Use both include and exclude patterns:

```bash
./target/release/vibewatch . \
  --include "*.rs" \
  --include "*.ts" \
  --include "*.tsx" \
  --exclude "target/**" \
  --exclude "node_modules/**" \
  --verbose
```

### Options

- `<DIRECTORY>`: Directory to watch (can be relative or absolute)
- `-i, --include <PATTERN>`: Include patterns like `*.ts`, `*.tsx`, `*.rs`
- `-e, --exclude <PATTERN>`: Exclude patterns like `node_modules/**`, `.git/**`, `.next/**`
- `-v, --verbose`: Enable verbose output with debug logging
- `-h, --help`: Show help message
- `-V, --version`: Show version information

## Examples

### Watch TypeScript/JavaScript project

```bash
./target/release/vibewatch src \
  --include "*.ts" \
  --include "*.tsx" \
  --include "*.js" \
  --include "*.jsx" \
  --exclude "node_modules/**" \
  --exclude "dist/**"
```

### Watch Rust project

```bash
./target/release/vibewatch . \
  --include "*.rs" \
  --include "Cargo.toml" \
  --exclude "target/**"
```

### Watch documentation changes

```bash
./target/release/vibewatch docs \
  --include "*.md" \
  --include "*.rst" \
  --exclude "_build/**"
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

The current implementation logs file changes to the console. Future versions could include:

- Custom command execution on file changes
- HTTP webhook notifications
- File change aggregation and batching
- Configuration file support
- Plugin system for custom handlers
- Integration with development tools

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
