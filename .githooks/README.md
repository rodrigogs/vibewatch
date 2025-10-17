# Git Hooks for vibewatch

This directory contains Git hooks that enforce code quality standards before commits and pushes.

## Available Hooks

### `pre-commit`
Runs before every commit to ensure code quality:
- ✅ **Format check** (`cargo fmt --check`) - Ensures code is properly formatted
- ✅ **Linting** (`cargo clippy -D warnings`) - Catches common mistakes and bad patterns
- ✅ **Tests** (`cargo test`) - Ensures all tests pass

### `pre-push`
Runs before pushing to remote to prevent CI failures:
- ✅ **Full test suite** (`cargo test`) - Comprehensive test coverage
- ✅ **Clippy all targets** - Linting on all code including benchmarks

## Installation

The hooks are automatically installed when you run:

```bash
just setup-hooks
```

Or manually:

```bash
# Configure git to use .githooks directory
git config core.hooksPath .githooks

# Make hooks executable
chmod +x .githooks/pre-commit
chmod +x .githooks/pre-push
```

## Verifying Installation

Check that hooks are configured:

```bash
git config core.hooksPath
# Should output: .githooks
```

## Bypassing Hooks (Emergency Only)

If you absolutely need to bypass hooks (not recommended):

```bash
# Skip pre-commit hook
git commit --no-verify -m "message"

# Skip pre-push hook
git push --no-verify
```

## Disabling Hooks

To temporarily disable hooks:

```bash
# Restore default git hooks directory
git config --unset core.hooksPath
```

To re-enable:

```bash
git config core.hooksPath .githooks
```

## Why Git Hooks?

**Benefits:**
- ✅ **Catch issues early** - Before they reach CI/CD pipeline
- ✅ **Faster feedback** - Instant local validation vs waiting for CI
- ✅ **Prevent bad commits** - Can't commit code that doesn't pass checks
- ✅ **Save CI resources** - Fewer failed CI runs
- ✅ **Zero dependencies** - Native Git feature, no npm/yarn required

**vs JavaScript Tools (husky):**
- Git hooks are native, language-agnostic
- No node_modules bloat for Rust projects
- Works with any Git workflow
- Simpler setup and maintenance

## Troubleshooting

### Hook not running
```bash
# Check if hooks are executable
ls -la .githooks/

# If not, make them executable
chmod +x .githooks/*
```

### Wrong git hooks directory
```bash
# Verify configuration
git config core.hooksPath
# Should output: .githooks

# If not set, run
git config core.hooksPath .githooks
```

### Hook fails but code seems fine
```bash
# Run checks manually to debug
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
