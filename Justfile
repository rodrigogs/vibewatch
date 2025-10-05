# vibewatch - Justfile (task runner)
# Install just: https://github.com/casey/just

# Default recipe - list all available commands
default:
    @just --list

# Run all tests (187 total: 140 unit + 21 filesystem + 26 integration)
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run specific integration test
test-integration TEST:
    cargo test {{TEST}} --test it

# Run with specific threads (useful for debugging flaky tests)
test-serial:
    cargo test -- --test-threads=1 --nocapture

# Generate HTML coverage report and open it
coverage:
    cargo llvm-cov --all-features --workspace --html
    @echo "Opening coverage report..."
    open target/llvm-cov/html/index.html

# Generate coverage in terminal
coverage-text:
    cargo llvm-cov --all-features --workspace

# Generate LCOV format for CI
coverage-lcov:
    cargo llvm-cov --all-features --workspace --lcov --output-path coverage.lcov

# Build release binary
build:
    cargo build --release

# Build and run in debug mode with example directory
run DIR="." ARGS="":
    cargo run -- {{DIR}} {{ARGS}}

# Build and run in release mode
run-release DIR="." ARGS="":
    cargo run --release -- {{DIR}} {{ARGS}}

# Run with debug logging
run-debug DIR="." ARGS="":
    RUST_LOG=debug cargo run -- {{DIR}} --verbose {{ARGS}}

# Run linter
lint:
    cargo clippy -- -D warnings

# Run linter and auto-fix
lint-fix:
    cargo clippy --fix --allow-dirty --allow-staged

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt --check

# Run all checks (fmt, lint, test)
check: fmt-check lint test

# Clean build artifacts
clean:
    cargo clean

# Clean and rebuild
rebuild: clean build

# Watch and run tests on file changes (requires cargo-watch)
watch:
    cargo watch -x test

# Watch and run coverage on file changes
watch-coverage:
    cargo watch -x "llvm-cov --html"

# Install development tools
install-tools:
    cargo install cargo-llvm-cov
    cargo install cargo-watch

# Run the watcher on the src directory with verbose output
demo:
    cargo run -- src --include "*.rs" --verbose --on-change "echo Changed: {relative_path}"

# Run all benchmarks
bench:
    cargo bench

# Run specific benchmark
bench-one NAME:
    cargo bench --bench {{NAME}}

# Run benchmarks and save as baseline
bench-baseline NAME="main":
    cargo bench -- --save-baseline {{NAME}}

# Compare benchmarks against baseline
bench-compare NAME="main":
    cargo bench -- --baseline {{NAME}}

# Run quick benchmark test (doesn't do full measurement)
bench-test:
    cargo bench -- --test

# Show project statistics
stats:
    @echo "=== Project Statistics ==="
    @echo "Source files:"
    @find src -name "*.rs" | wc -l
    @echo "Test files:"
    @find tests -name "*.rs" | wc -l
    @echo "Benchmark files:"
    @find benches -name "*.rs" | wc -l
    @echo "Lines of code (src):"
    @find src -name "*.rs" -exec wc -l {} + | tail -1
    @echo "Lines of code (tests):"
    @find tests -name "*.rs" -exec wc -l {} + | tail -1
    @echo "Total documentation:"
    @find docs -name "*.md" -exec wc -l {} + | tail -1
