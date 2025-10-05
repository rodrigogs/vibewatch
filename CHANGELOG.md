# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/rodrigogs/vibewatch/compare/v0.1.0...v0.2.0) (2025-10-05)


### Features

* add event debouncing with configurable delay ([d819bcf](https://github.com/rodrigogs/vibewatch/commit/d819bcfe16a9201bfe1ebe2f6e759a26a616bdf3))

## 0.1.0 (2025-10-05)


### Features

* add GitHub Actions CI/CD with automated releases ([71e9bd3](https://github.com/rodrigogs/vibewatch/commit/71e9bd3258cf6ef525178ccf0a7f92c715c952af))


### Bug Fixes

* enable repository workflow permissions for Release Please ([1cb4d9c](https://github.com/rodrigogs/vibewatch/commit/1cb4d9ce7fa77343e887a39cba02600653fef33c))
* normalize path separators to forward slashes for cross-platform compatibility ([02ac3e5](https://github.com/rodrigogs/vibewatch/commit/02ac3e5dfbcbb3c0ed763894e592e56d7dc9bf56))
* resolve clippy warnings and formatting issues ([31abf77](https://github.com/rodrigogs/vibewatch/commit/31abf77f2d806a38222260b066558ed8c14f9f81))

## [Unreleased]

### Added
- File watcher utility with glob pattern support
- Event-specific commands (create, modify, delete)
- Template substitution system for file paths
- Comprehensive test suite (187 tests, 95.77% coverage)
- Justfile task runner with 24 recipes
- GitHub Actions CI/CD pipeline
- Automated releases with Release Please

### Documentation
- README with usage examples and architecture
- Copilot instructions for AI agents
- Testing documentation
- Coverage analysis
- Justfile implementation guide
