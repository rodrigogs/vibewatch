# Copilot Instructions for vibewatch

## Project Overview
vibewatch is a Rust binary crate (no lib.rs) that watches directories for file changes and executes commands. Think watchexec/entr but with explicit event-specific commands and template substitution.

## Architecture (3 Core Modules)

**`src/main.rs`** - CLI entry point using clap with derive macros. Separate `create_watcher_from_args()` function for testability (called by main, tested via unit tests). Uses edition 2024, async tokio runtime.

**`src/watcher.rs`** - Core logic with `FileWatcher` struct. Uses `notify` crate's `RecommendedWatcher` with channel-based event loop. Template system substitutes `{file_path}`, `{relative_path}`, `{absolute_path}`, `{event_type}` in commands. Commands execute via `tokio::process::Command` (async, fire-and-forget).

**`src/filter.rs`** - Pattern matching via `glob` crate. Exclude patterns take precedence over include patterns. Empty include list = watch everything (except excludes).

## Critical Testing Architecture

### Single Integration File Pattern (Performance)
- **All integration tests** in `tests/it.rs` (597 lines, 26 tests)
- **Why**: Multiple test files = 3x slower compile, 5x larger artifacts ([matklad's research](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html))
- **Exception**: `tests/filesystem_utils_tests.rs` (21 tests) exists separately for filesystem-specific operations
- Unit tests are inline with source (`#[cfg(test)]` modules in each src file)

### Test Infrastructure
```rust
// tests/common/mod.rs defines standard timeouts:
WATCHER_STARTUP_TIME: 1500ms  // Watcher initialization
EVENT_DETECTION_TIME: 1500ms  // File event processing
COMMAND_EXECUTION_TIME: 500ms // Command completion
```

**Integration tests use subprocess execution** via `assert_cmd::Command::cargo_bin("vibewatch")` - this is correct and intentional. Coverage tools can't track across process boundaries (documented limitation).

### Test Artifacts
- Use `/tmp/` for test output files, **never project root**
- Example: `touch /tmp/vibewatch-test-output.txt` not `touch output.txt`
- `assert_fs::TempDir` auto-cleans test directories

## Task Runner - Justfile (Recommended)

**Use `just` for common tasks** - simpler and more discoverable than raw cargo commands:

```bash
just                    # List all available tasks
just test              # Run all 187 tests
just coverage          # Generate and open HTML coverage report
just test-serial       # Run tests one at a time (debug flaky tests)
just lint              # Run clippy with strict warnings
just check             # Run fmt-check + lint + test (pre-commit)
just demo              # Run vibewatch on src/ directory
just stats             # Show project statistics
```

**Direct cargo commands** (when needed):
```bash
cargo test                              # Run all tests
cargo llvm-cov --html                   # Generate coverage
cargo test test_name --test it          # Run specific integration test
cargo test -- --nocapture --test-threads=1  # Debug with output
```

**Why `just` over Makefile**: Cleaner syntax, better error messages, cross-platform, no PHONY targets needed. See `Justfile` for all 20+ available tasks.

## Coverage Philosophy (Non-Negotiable)

**95.77% coverage is EXCELLENT and accepted.** Remaining 4.23% is in:
- `main()` function (lines 82-85, 97, 101) - subprocess limitation
- `start_watching()` event loop (lines 139-159) - subprocess limitation

**Do NOT refactor architecture to chase 100% coverage.** These lines are functionally tested via integration tests. Subprocess execution is the correct approach per Rust testing best practices.

## Parameterized Testing Pattern

Use `rstest` for table-driven tests:
```rust
#[rstest]
#[case("*.rs", "main.rs", true)]
#[case("*.rs", "main.js", false)]
fn test_pattern(#[case] pattern: &str, #[case] path: &str, #[case] expected: bool) {
    // Test implementation
}
```

## CLI Patterns (clap with derive)

- Group args with `help_heading` (FILTERING_HELP, COMMANDS_HELP, GENERAL_HELP)
- Extensive `after_help` with examples and template documentation
- Testability: Split logic into `create_watcher_from_args()` function
- Unit tests use `Args::parse_from(&["vibewatch", ...])` for CLI testing

## Command Execution Pattern

Event-specific commands (`--on-create`, `--on-modify`, `--on-delete`) with `--on-change` as fallback. `CommandConfig::get_command_for_event()` implements fallback logic.

Template substitution in `TemplateContext::substitute_template()` - simple string replace, no fancy templating engine.

## Error Handling Convention

Use `anyhow::Result` everywhere, `anyhow::Context` for wrapping errors with context:
```rust
PatternFilter::new(include, exclude)
    .context("Failed to compile include patterns")?;
```

## Documentation Structure

- `README.md` - User-facing, usage examples, architecture overview
- `docs/TESTING.md` - Test organization, techniques, CI/CD patterns
- `docs/COVERAGE.md` - Coverage analysis, uncovered lines explanation, benchmarks
- `docs/INTEGRATION_TEST.md` - Research notes, rationale for single test file

**Keep docs synchronized** - coverage percentages, test counts, line numbers must match reality.

## Development Environment

- Rust managed via `mise` (see `.mise.toml`)
- Rust 1.89.0 (edition 2024 support)
- `RUST_BACKTRACE=1` set in mise config

## Release Process & Conventional Commits

**This project uses automated releases via Release Please.**

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<optional scope>): <description>

[optional body]

[optional footer(s)]
```

**Types that trigger releases:**
- `feat:` - New feature → **minor** version bump (0.X.0)
- `fix:` - Bug fix → **patch** version bump (0.0.X)
- `feat!:`, `fix!:`, `refactor!:` - Breaking change → **major** version bump (X.0.0)

**Other types** (no version bump):
- `docs:` - Documentation only
- `chore:` - Maintenance tasks
- `test:` - Test additions/changes
- `refactor:` - Code refactoring
- `style:` - Code formatting

**Examples:**
```bash
feat: add symlink watching support
fix: resolve race condition in event detection
docs: update README with new examples
feat!: change CLI argument structure (BREAKING CHANGE)
```

### Release Workflow

1. **Commit to `master`** using conventional commits
2. **Release Please creates PR** - Automatically updates `CHANGELOG.md` and `Cargo.toml` version
3. **Merge the Release PR** - Triggers:
   - GitHub Release creation with release notes
   - Binary builds (Linux x86_64/ARM64, macOS x86_64/ARM64, Windows x86_64)
   - Optional publish to crates.io (requires `CARGO_TOKEN` secret)

**Never manually edit** `CHANGELOG.md` or version in `Cargo.toml` - Release Please manages these.

### CI/CD Pipeline

**On every PR and push to `master`:**
- ✅ Tests (187 tests) on Linux, macOS, Windows
- ✅ Rustfmt check (`cargo fmt --check`)
- ✅ Clippy with `-D warnings`
- ✅ Coverage generation (uploaded to Codecov)

**Workflow files:**
- `.github/workflows/ci.yml` - Continuous integration checks
- `.github/workflows/release.yml` - Automated releases and binary publishing

## When Adding Features

1. Write unit tests inline in same file (`#[cfg(test)]` module)
2. Add integration test to `tests/it.rs` (never create new integration test file)
3. Use `tests/common/mod.rs` helpers for file operations and timeouts
4. Update coverage docs if uncovered lines change
5. Update README examples if user-facing behavior changes
6. **Use conventional commit messages** for proper versioning

## Common Pitfalls to Avoid

- ❌ Creating multiple integration test files (performance killer)
- ❌ Using project root for test artifacts (use `/tmp/`)
- ❌ Refactoring main() to chase 100% coverage (subprocess testing is correct)
- ❌ Hardcoding timeouts in tests (use `common::*_TIME` constants)
- ❌ Generic error messages (use `.context()` for specificity)
- ❌ Synchronous command execution (use `tokio::process::Command`)
- ❌ Manually editing CHANGELOG.md or version numbers (let Release Please handle it)
- ❌ Non-conventional commit messages (breaks automated releases)
