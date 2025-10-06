# CI/CD Architecture for vibewatch

## Overview

This document describes the CI/CD architecture for vibewatch, which automates releases while ensuring code quality through branch protection and comprehensive testing.

**Status**: ✅ Fully implemented and operational as of v0.3.0 (October 2025)

## Current Implementation

The project uses a standard Release Please workflow triggered on push to master:

```yaml
on:
  push:
    branches:
      - master
```

This approach aligns with Release Please best practices and provides:
- Fast release cycles (no waiting for duplicate CI runs)
- Simple workflow logic
- Branch protection as the primary safety mechanism
- Reliable cross-platform binary builds

## Complete Developer Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. Developer creates feature branch from dev or master          │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Make changes, commit using Conventional Commits spec        │
│    - feat: new feature (minor bump)                             │
│    - fix: bug fix (patch bump)                                  │
│    - feat!: breaking change (major bump)                        │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Push branch and open PR to master                            │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. CI workflow runs automatically (6 checks)                    │
│    ✓ Test Suite (ubuntu-latest, stable)                        │
│    ✓ Test Suite (macos-latest, stable)                         │
│    ✓ Test Suite (windows-latest, stable)                       │
│    ✓ Rustfmt                                                    │
│    ✓ Clippy                                                     │
│    ✓ Code Coverage                                              │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Branch Protection enforces requirements                      │
│    ❌ Cannot merge if ANY check fails                           │
│    ❌ Cannot merge if branch not up-to-date (strict mode)       │
│    ✓ Can merge when all checks pass                             │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 6. Merge PR (squash-merge recommended for clean history)       │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 7. Push to master triggers Release Please workflow             │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 8. Release Please analyzes commits since last release          │
│    - Parses Conventional Commits                                │
│    - Calculates version bump (major/minor/patch)                │
│    - Generates CHANGELOG.md entries                             │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 9. Release Please creates/updates Release PR                   │
│    - Updates Cargo.toml version                                 │
│    - Updates CHANGELOG.md                                        │
│    - Labels PR with "autorelease: pending"                      │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 10. Review and merge Release PR                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 11. Release Please tags release and creates GitHub Release     │
│     Labels PR with "autorelease: tagged"                        │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 12. Automated post-release tasks (parallel execution)           │
│     ✓ Publish to crates.io (if CARGO_TOKEN configured)         │
│     ✓ Build binaries for 5 platforms:                           │
│       - Linux x86_64 (native build)                             │
│       - Linux ARM64 (cross-compilation via 'cross')             │
│       - macOS Intel (native build)                              │
│       - macOS ARM (native build)                                │
│       - Windows x64 (native build)                              │
│     ✓ Upload binaries to GitHub Release                         │
└─────────────────────────────────────────────────────────────────┘
```

## Branch Protection Configuration

### Current Configuration (Applied)

✅ **Status**: Branch protection is enabled on `master` branch

The following settings are currently active:

| Setting | Value | Description |
|---------|-------|-------------|
| **Required Status Checks** | 6 checks | All CI jobs must pass before merge |
| **Strict Mode** | Enabled | Branch must be up-to-date with base |
| **Enforce Admins** | Disabled | Allows emergency admin bypass |
| **Linear History** | Enabled | Enforces clean git history |
| **Force Pushes** | Disabled | Prevents history rewriting |
| **Deletions** | Disabled | Prevents accidental branch deletion |
| **Conversation Resolution** | Enabled | All PR comments must be resolved |

### Required Status Checks

The following CI checks must pass before merging to master:
- Test Suite (ubuntu-latest, stable)
- Test Suite (macos-latest, stable)
- Test Suite (windows-latest, stable)
- Rustfmt
- Clippy
- Code Coverage

### Viewing Current Configuration

To view the current branch protection settings:

```bash
gh api /repos/rodrigogs/vibewatch/branches/master/protection | jq
```

### Modifying Branch Protection

To update branch protection settings via GitHub CLI:

```bash
# Save your desired configuration to a JSON file
cat > branch-protection.json <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Test Suite (ubuntu-latest, stable)",
      "Test Suite (macos-latest, stable)",
      "Test Suite (windows-latest, stable)",
      "Rustfmt",
      "Clippy",
      "Code Coverage"
    ]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_conversation_resolution": true
}
EOF

# Apply the configuration
gh api -X PUT /repos/rodrigogs/vibewatch/branches/master/protection \
  --input branch-protection.json

# Clean up
rm branch-protection.json
```

Alternatively, use the GitHub web UI: Settings → Branches → Branch protection rules

### Configuration Rationale

| Setting | Value | Rationale |
|---------|-------|-----------|
| `strict` | `true` | **CRITICAL**: Ensures PR branch is up-to-date with base before merge. This guarantees the exact code being merged has been tested. |
| `contexts` | 6 checks | All CI jobs must pass. Matrix jobs are listed individually. |
| `enforce_admins` | `false` | Allows admin bypass for emergencies (solo developer project). |
| `required_pull_request_reviews` | `null` | No required reviews for solo developer (maintains velocity). |
| `required_linear_history` | `true` | Enforces clean git history via squash-merge (aligns with Release Please best practice). |
| `allow_force_pushes` | `false` | Prevents accidental history rewriting on master. |
| `allow_deletions` | `false` | Prevents accidental branch deletion. |
| `required_conversation_resolution` | `true` | Good practice: all PR comments must be resolved before merge. |

### Status Check Names

**IMPORTANT**: Status check names must match EXACTLY what GitHub Actions reports. These are derived from:
- Job name (e.g., `Test Suite`, `Rustfmt`)
- Matrix parameters (e.g., `(ubuntu-latest, stable)`)

To verify status check names:
1. Look at any PR with CI runs
2. Check the "Checks" tab
3. Use the exact names shown there

## Token Configuration

### GITHUB_TOKEN (Current)

The workflow currently uses `GITHUB_TOKEN`:
```yaml
token: ${{ secrets.GITHUB_TOKEN }}
```

**Status**: ✅ Working correctly for current workflow needs

**Limitation**: From GitHub documentation:
> When you use the repository's GITHUB_TOKEN to perform tasks, events triggered by the GITHUB_TOKEN will not create a new workflow run.

**Impact**: If you add workflows that should trigger on `release.created` events, they won't run with `GITHUB_TOKEN`.

### Personal Access Token (Optional Upgrade)

**When to upgrade**: Only if you need workflows that trigger on Release Please's actions (e.g., deploy on release, notify on release).

**Setup**:
1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Generate new token (classic)
3. Scopes: `repo` (Full control of private repositories)
4. Add as repository secret: `RELEASE_PLEASE_TOKEN`
5. Update `release.yml`: `token: ${{ secrets.RELEASE_PLEASE_TOKEN }}`

### CARGO_TOKEN (Configured)

**Status**: ✅ Configured and working

Used for automated publishing to crates.io:
```yaml
run: cargo publish --token ${{ secrets.CARGO_TOKEN }}
```

**Token management**:
- Source: https://crates.io/settings/tokens
- Stored as: GitHub repository secret `CARGO_TOKEN`
- Recommendation: Rotate annually for security

## Binary Build Architecture

### Current Implementation (v0.3.0+)

The release workflow builds binaries for 5 platforms using `taiki-e/upload-rust-binary-action@v1`:

**Build Matrix**:
```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - target: x86_64-unknown-linux-gnu      # Linux x86_64 (native)
        os: ubuntu-latest
      - target: aarch64-unknown-linux-gnu     # Linux ARM64 (cross)
        os: ubuntu-latest
      - target: x86_64-apple-darwin           # macOS Intel (native)
        os: macos-latest
      - target: aarch64-apple-darwin          # macOS ARM (native)
        os: macos-latest
      - target: x86_64-pc-windows-msvc        # Windows x64 (native)
        os: windows-latest
```

**Build Strategy**:
- **Linux x86_64**: Native cargo build on Ubuntu runner (~40s)
- **Linux ARM64**: Cross-compilation using `cross` tool via Docker (~1m16s)
- **macOS Intel**: Native build on macOS runner (~51s)
- **macOS ARM**: Native build on macOS runner (~1m2s)
- **Windows x64**: Native build on Windows runner (~3m52s)

**Key Features**:
- ✅ Parallel execution (all 5 builds run simultaneously)
- ✅ Automatic cross-compilation for ARM64 Linux
- ✅ Proper archive formats (.tar.gz for Unix, .zip for Windows)
- ✅ No manual build/strip/rename steps needed
- ✅ Reliable (no timeout issues like previous manual builds)
- ✅ Battle-tested tool (used by tokio-console, cargo-hack)

**Why taiki-e/upload-rust-binary-action?**
1. Actively maintained (v1.27.0, June 2024)
2. Handles cross-compilation automatically via `cross` tool
3. Simplifies workflow from ~60 to ~40 lines
4. No GitHub Actions timeout issues
5. Proper binary naming and archiving

### Release Assets

Each GitHub release includes 5 binaries:

| Platform | Filename | Size | Format |
|----------|----------|------|--------|
| Linux x86_64 | `vibewatch-x86_64-unknown-linux-gnu.tar.gz` | ~1.3 MB | tar.gz |
| Linux ARM64 | `vibewatch-aarch64-unknown-linux-gnu.tar.gz` | ~1.3 MB | tar.gz |
| macOS Intel | `vibewatch-x86_64-apple-darwin.tar.gz` | ~1.2 MB | tar.gz |
| macOS ARM | `vibewatch-aarch64-apple-darwin.tar.gz` | ~1.2 MB | tar.gz |
| Windows x64 | `vibewatch-x86_64-pc-windows-msvc.zip` | ~1.0 MB | zip |

### Historical Context

**v0.2.0 and earlier**: Used manual cross-compilation with apt packages (musl-tools, gcc-aarch64-linux-gnu). This approach suffered from:
- ❌ Frequent timeouts in GitHub Actions
- ❌ Complex manual build/strip/rename/upload steps
- ❌ 7 targets attempted (including musl variants)

**v0.2.1**: Temporarily removed Linux builds to fix timeouts
- ✅ Released successfully with 3 binaries (macOS x2, Windows)
- ⚠️ Linux users had to use `cargo install vibewatch`

**v0.3.0+**: Current approach with taiki-e action
- ✅ Restored Linux support (x86_64 + ARM64)
- ✅ Reliable builds with no timeouts
- ✅ Simplified workflow maintenance

## Implementation History

### ✅ Phase 1: Standard Release Pattern (Completed)

**Status**: Implemented in v0.3.0

The workflow uses the standard `on: push` trigger pattern:
```yaml
on:
  push:
    branches:
      - master
```

This aligns with Release Please best practices and provides fast release cycles.

### ✅ Phase 2: Cross-Platform Binary Builds (Completed)

**Status**: Implemented in v0.3.0

Migrated from manual cross-compilation to `taiki-e/upload-rust-binary-action`, restoring Linux support with improved reliability.

### ✅ Phase 3: Branch Protection (Completed)

**Method 1: GitHub CLI (Recommended)**

**Status**: Branch protection enabled on `master` branch with all 6 required status checks.

See "Branch Protection Configuration" section above for current settings.

## Token Configuration (Optional Enhancement)

**Method 2: GitHub Web UI**

1. Go to Settings → Branches
2. Click "Add branch protection rule"
3. Branch name pattern: `master`
4. Enable:
   - ✓ Require a pull request before merging
     - ❌ Require approvals (leave at 0 for solo dev)
   - ✓ Require status checks to pass before merging
     - ✓ Require branches to be up to date before merging (STRICT MODE)
     - Add status checks: Test Suite (all 3 OS), Rustfmt, Clippy, Code Coverage
   - ✓ Require conversation resolution before merging
   - ✓ Require linear history
   - ❌ Do not allow bypassing the above settings (keep disabled for admin bypass)
   - ✓ Do not allow force pushes
   - ✓ Do not allow deletions

### Phase 4: Configure Optional Tokens (Priority 4 - Optional)

## Validating a Release

### Quick Validation

To verify the release workflow is working:

```bash
# Check the latest release
gh release view --web

# Or get release details via CLI
gh release view v0.3.0

# Verify all 5 binaries are present
gh release view v0.3.0 --json assets --jq '.assets[].name'
```

Expected output:
```
vibewatch-aarch64-apple-darwin.tar.gz
vibewatch-aarch64-unknown-linux-gnu.tar.gz
vibewatch-x86_64-apple-darwin.tar.gz
vibewatch-x86_64-pc-windows-msvc.zip
vibewatch-x86_64-unknown-linux-gnu.tar.gz
```

### End-to-End Release Test

**Complete workflow** (tested successfully in v0.3.0):
1. Make feature branch: `git checkout -b feature-branch`
2. Commit using Conventional Commits: `git commit -m "feat: new feature"`
3. Open PR: `gh pr create --base master`
4. Wait for CI checks (all 6 must pass)
5. Merge PR: `gh pr merge --squash` (requires admin for branch protection)
6. Release Please creates release PR automatically
7. Review release PR (check CHANGELOG.md and version bump)
8. Merge release PR: `gh pr merge <pr-number> --admin --merge`
9. Workflow automatically:
   - Creates GitHub release with tag
   - Publishes to crates.io
   - Builds and uploads 5 platform binaries
10. Verify: `gh release view <version>`

**Typical timeline**:
- CI checks on PR: ~2-3 minutes
- Release workflow after merge: ~4 minutes total
- Binary builds (parallel): 40s - 4m per platform

## Monitoring and Maintenance

### Regular Checks

1. **Status Check Names**: Verify they match after CI workflow changes
2. **Branch Protection**: Review settings quarterly
3. **Token Expiration**: Monitor PAT expiration dates
4. **Release Notes**: Review Release Please PRs for accuracy
5. **Failed Releases**: Monitor for cargo publish failures

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| PR can't merge despite passing tests | Status check names don't match | Update branch protection with exact names from PR |
| Release Please doesn't create PR | No releasable commits (only chore/docs) | Add feat/fix commits |
| Cargo publish fails | Missing or invalid CARGO_TOKEN | Regenerate token from crates.io |
| Other workflows don't trigger on release | Using GITHUB_TOKEN instead of PAT | Switch to Personal Access Token (see Token Configuration) |
| Binary build fails | Platform-specific build issue | Check workflow logs, may need cross-compilation adjustment |
| Linux ARM64 build timeout | Cross tool not installed | Verified working in v0.3.0, check `taiki-e/install-action` step |

### Emergency Procedures

**Temporarily disable branch protection** (emergencies only):

```bash
# Disable branch protection
gh api -X DELETE /repos/rodrigogs/vibewatch/branches/master/protection

# Make emergency fix

# Re-enable protection (see "Modifying Branch Protection" section above)
```

**Roll back a release**:

```bash
# Delete the GitHub release and tag
gh release delete v<version> --yes
git push origin :refs/tags/v<version>

# Yank from crates.io (makes it unavailable for new downloads)
cargo yank --vers <version>

# If needed, publish a patch release with fix
```

## Security Considerations

1. **Token Management**
   - Rotate CARGO_TOKEN annually
   - Use PAT with minimum required scopes
   - Never commit tokens to repository

2. **Branch Protection**
   - Keep `enforce_admins: false` for emergency access
   - Monitor for unexpected bypass usage
   - Review protection rules with each team member addition

3. **Release Process**
   - Review release PRs before merging
   - Verify CHANGELOG.md accuracy
   - Monitor published releases for security issues

## Future Enhancements

### Potential Improvements

1. **Binary Signing**: Sign release binaries for verification (cosign, GPG)
2. **Performance Benchmarks**: Add benchmark CI checks for regression detection
3. **Release Notes Enhancement**: Customize Release Please templates with more detail
4. **Automated Security Scanning**: Add dependency vulnerability checks
5. **Release Notifications**: Notify on release creation (Discord, Slack)
6. **Download Statistics**: Track binary download metrics
7. **Additional Platforms**: Consider adding more targets (FreeBSD, musl variants)

### Scaling Considerations

When vibewatch grows beyond solo development:
- Add required pull request reviews
- Implement CODEOWNERS for module ownership
- Consider merge queue for high-velocity development
- Add deployment environments if needed (staging, production)
- Implement more granular access controls

## References

- [Release Please Documentation](https://github.com/googleapis/release-please)
- [Release Please Action](https://github.com/googleapis/release-please-action)
- [Conventional Commits Specification](https://www.conventionalcommits.org/)
- [GitHub Branch Protection Documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [GitHub Actions workflow_run Trigger](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#workflow_run)
- [SemVer Specification](https://semver.org/)

---

**Last Updated**: October 6, 2025
**Version**: 2.0 (Updated to reflect v0.3.0 implementation)
**Maintainer**: @rodrigogs

## Recent Changes

### v2.0 (October 6, 2025)
- Updated to reflect current v0.3.0 implementation
- Documented taiki-e/upload-rust-binary-action migration
- Added 5-platform binary build details
- Removed outdated implementation plan phases (all completed)
- Updated branch protection to current configured state
- Added historical context for build evolution
- Removed `.github/branch-protection.json` (now documented inline)
