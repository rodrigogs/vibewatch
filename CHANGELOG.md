# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0](https://github.com/rodrigogs/vibewatch/compare/v0.3.0...v0.4.0) (2025-10-06)


### Features

* add --quiet flag and show command output to users ([47b71c5](https://github.com/rodrigogs/vibewatch/commit/47b71c5d81c818fd085732d20badec26f4cc62ea))
* implement structured, objective logging with timestamps and exit codes ([c7c251c](https://github.com/rodrigogs/vibewatch/commit/c7c251cc58d7bc91ba17e643f405fe28e90cfca9))

## [0.3.0](https://github.com/rodrigogs/vibewatch/compare/v0.2.1...v0.3.0) (2025-10-06)


### Features

* **ci:** use taiki-e/upload-rust-binary-action for reliable cross-platform builds ([5ac53a8](https://github.com/rodrigogs/vibewatch/commit/5ac53a892b13a1f358f1264c75516432c1b90fd0))

## [0.2.1](https://github.com/rodrigogs/vibewatch/compare/v0.2.0...v0.2.1) (2025-10-06)


### Bug Fixes

* **ci:** remove Linux cross-compilation targets to prevent build failures ([64d20ef](https://github.com/rodrigogs/vibewatch/commit/64d20efda61c21b95560d12eef004c3fefcc6153))

## [0.2.0](https://github.com/rodrigogs/vibewatch/compare/v0.1.0...v0.2.0) (2025-10-06)


### Features

* add event debouncing with configurable delay ([d819bcf](https://github.com/rodrigogs/vibewatch/commit/d819bcfe16a9201bfe1ebe2f6e759a26a616bdf3))
* add shell-words for proper command parsing ([e018867](https://github.com/rodrigogs/vibewatch/commit/e018867c754ced9ee60db253d6e1fb914d3c47ed))


### Bug Fixes

* **ci:** increase test timeouts for CI environment ([2bb2429](https://github.com/rodrigogs/vibewatch/commit/2bb2429143369736825287db2a32af33a905a3ce))
* **ci:** increase test timeouts to 6 seconds for slower CI environment ([abd791c](https://github.com/rodrigogs/vibewatch/commit/abd791ccd2b77cdb3a35175eb9b526a9fd742f09))
* **ci:** make release workflow depend on CI success ([3c83cc4](https://github.com/rodrigogs/vibewatch/commit/3c83cc4bc2c1ea48e69796d8bcc402a99ddf93dc))
* **ci:** replace fixed timeouts with polling for marker file detection ([ce9be66](https://github.com/rodrigogs/vibewatch/commit/ce9be665ea9693123af56adbaea037bc07ec9097))
* resolve clippy warnings and formatting in benchmarks ([ccc3a85](https://github.com/rodrigogs/vibewatch/commit/ccc3a85c49f0d686a478799bc29a117c415bed7f))
* **tests:** add cross-platform touch command for Windows compatibility ([b724aae](https://github.com/rodrigogs/vibewatch/commit/b724aae2884f1e64bbfb2838fb6691db6283b3b8))
* **tests:** remove needless borrow in test command arg ([a1a3310](https://github.com/rodrigogs/vibewatch/commit/a1a3310b547dd9ff53e3a83c9a0a53b62c54218e))
* **watcher:** handle Linux inotify Access(Close(Write)) events for cross-platform compatibility ([e9cf160](https://github.com/rodrigogs/vibewatch/commit/e9cf160ed7f0a101078957fb88eaec9b50c71881)), closes [#3](https://github.com/rodrigogs/vibewatch/issues/3)
* **watcher:** normalize Access(Close(Write)) events for cross-platform consistency ([dc59308](https://github.com/rodrigogs/vibewatch/commit/dc59308a9438da55c264361c002e157831b460fc))


### Performance Improvements

* add comprehensive benchmark suite with Criterion ([4bf880b](https://github.com/rodrigogs/vibewatch/commit/4bf880b4675d4602fc08a6c7969f584b1ba3b04d))
* convert to async channels with tokio::sync::mpsc ([2cd44e0](https://github.com/rodrigogs/vibewatch/commit/2cd44e0d86e22a919d616ea608f7661af801bf70))
* optimize path normalization to avoid unnecessary string operations ([b62c5c5](https://github.com/rodrigogs/vibewatch/commit/b62c5c54e5dbad0d24fcd5e4c8c6517c0384de89))
* optimize template substitution with single-pass algorithm ([06850f1](https://github.com/rodrigogs/vibewatch/commit/06850f11569d1fcc56f33e5d753a51079d661823))
* use static strings for event types to reduce allocations ([ee20cb3](https://github.com/rodrigogs/vibewatch/commit/ee20cb3fbe0c8a5e140d0eedbb27b76052646f29))

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
