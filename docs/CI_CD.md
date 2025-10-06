# CI/CD Architecture for vibewatch

## Overview

This document describes the complete CI/CD architecture for vibewatch, designed to automate releases while ensuring code quality through branch protection and comprehensive testing.

## Architecture Decision: workflow_run vs on: push

### Current Implementation (workflow_run)
The current `release.yml` uses the `workflow_run` trigger:

```yaml
on:
  workflow_run:
    workflows: ["CI"]
    types: [completed]
    branches: [master]
```

### Recommended Change: Revert to Standard Pattern

After deep analysis of Release Please best practices and GitHub's branch protection capabilities, **the recommended approach is to revert to the standard `on: push` trigger**:

```yaml
on:
  push:
    branches:
      - master
```

### Rationale

1. **Branch Protection is the Primary Safety Mechanism**
   - With strict branch protection enabled, PRs cannot be merged unless all CI checks pass
   - The PR branch must be up-to-date with master before merge (strict mode)
   - This ensures that the code being merged has been tested in its final form

2. **Alignment with Best Practices**
   - The `workflow_run` pattern is NOT mentioned in Release Please documentation
   - Google (Release Please maintainers) recommends the standard `on: push` pattern
   - Mainstream Rust projects follow this approach

3. **Performance Benefits**
   - No waiting for CI to re-run on master after merge
   - Faster release cycle
   - Simpler workflow logic

4. **Edge Cases are Handled**
   - "What if tests pass on PR but fail on master?" → Strict mode prevents this
   - "What if admin bypasses and pushes directly?" → Don't enable admin bypass
   - "What if there's an environment-specific issue?" → Extremely rare, monitor via alerts

### When workflow_run Makes Sense

The `workflow_run` pattern would be appropriate if:
- You have multiple teams with varying permissions
- You need defense-in-depth against admin bypass
- You have complex merge strategies beyond squash-merge
- You have environment-specific concerns not caught by PR testing

**For vibewatch (single developer, clean merge strategy, comprehensive CI), the standard pattern is optimal.**

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
│ 12. Automated post-release tasks                                │
│     ✓ Publish to crates.io (if CARGO_TOKEN configured)         │
│     ✓ Build binaries for 7 platforms                            │
│     ✓ Upload binaries to GitHub Release                         │
└─────────────────────────────────────────────────────────────────┘
```

## Branch Protection Configuration

### Required Configuration

```json
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
```

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

### GITHUB_TOKEN vs Personal Access Token (PAT)

#### Current Configuration
```yaml
token: ${{ secrets.GITHUB_TOKEN }}
```

#### Issue with GITHUB_TOKEN
From GitHub documentation:
> When you use the repository's GITHUB_TOKEN to perform tasks, events triggered by the GITHUB_TOKEN will not create a new workflow run.

**Impact**: Workflows triggered by `release.created` events won't run if Release Please uses `GITHUB_TOKEN`.

#### Recommended: Personal Access Token (PAT)

Create a PAT with `repo` scope and configure it as `RELEASE_PLEASE_TOKEN`:

```yaml
token: ${{ secrets.RELEASE_PLEASE_TOKEN }}
```

**Benefits**:
- Other workflows can trigger on Release Please's actions
- More predictable workflow chaining
- Better for complex automation

**Note**: For vibewatch's current simple workflow, `GITHUB_TOKEN` works fine. Upgrade to PAT if you add workflows that should trigger on release creation.

### CARGO_TOKEN for crates.io

Required for automated publishing to crates.io:

1. Get token from https://crates.io/settings/tokens
2. Add as GitHub secret: `CARGO_TOKEN`
3. Used in `publish-crate` job:
   ```yaml
   run: cargo publish --token ${{ secrets.CARGO_TOKEN }}
   ```

## Implementation Plan

### Phase 1: Fix Integration Test Timeouts (BLOCKING - Priority 1)

**Problem**: 6 integration tests fail in CI due to insufficient timeouts.

**Files to modify**: `tests/common/mod.rs`

```rust
// Change from:
pub const WATCHER_STARTUP_TIME: u64 = 1500;
pub const EVENT_DETECTION_TIME: u64 = 1500;

// To:
pub const WATCHER_STARTUP_TIME: u64 = 3000;
pub const EVENT_DETECTION_TIME: u64 = 3000;
```

**Why this is blocking**: Cannot enable branch protection with failing tests (creates catch-22).

**Steps**:
1. Fix timeouts locally
2. Run `just test` to verify all 187 tests pass
3. Push to dev branch
4. Verify CI passes on dev
5. Merge to master (while protection is still disabled)

### Phase 2: Revert to Standard Release Pattern (Priority 2)

**File to modify**: `.github/workflows/release.yml`

Change from:
```yaml
on:
  workflow_run:
    workflows: ["CI"]
    types: [completed]
    branches: [master]

jobs:
  release-please:
    if: ${{ github.event.workflow_run.conclusion == 'success' }}
```

To:
```yaml
on:
  push:
    branches:
      - master

jobs:
  release-please:
    runs-on: ubuntu-latest
```

**Why**: Aligns with Release Please best practices, simpler, faster, and branch protection provides the safety mechanism.

### Phase 3: Enable Branch Protection (Priority 3)

**Method 1: GitHub CLI (Recommended)**

Save protection config to `branch-protection.json`:
```json
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
```

Apply configuration:
```bash
gh api -X PUT /repos/rodrigogs/vibewatch/branches/master/protection \
  --input branch-protection.json
```

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

#### Personal Access Token (Optional)
1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Generate new token (classic)
3. Scopes: `repo` (Full control of private repositories)
4. Add as repository secret: `RELEASE_PLEASE_TOKEN`
5. Update `.github/workflows/release.yml`:
   ```yaml
   token: ${{ secrets.RELEASE_PLEASE_TOKEN }}
   ```

#### crates.io Token (Required for Publishing)
1. Go to https://crates.io/settings/tokens
2. Create new token with publish permission
3. Add as repository secret: `CARGO_TOKEN`
4. Already configured in `release.yml`

### Phase 5: Validate Complete Flow (Priority 5)

**Test Scenario 1: PR with failing tests**
```bash
# Create branch with intentional test failure
git checkout -b test-branch-protection
# Add failing test
git commit -m "test: intentional failure"
git push origin test-branch-protection
# Open PR
gh pr create --base master --head test-branch-protection
# Verify: PR cannot be merged (checks failing)
```

**Test Scenario 2: PR with passing tests**
```bash
# Create branch with valid changes
git checkout -b test-release-flow
# Make changes following conventional commits
git commit -m "feat: add new feature for testing"
git push origin test-release-flow
# Open PR
gh pr create --base master --head test-release-flow
# Wait for CI to pass
# Verify: PR can be merged
# Merge PR
gh pr merge --squash
# Verify: Release Please creates release PR
```

**Test Scenario 3: Complete release**
1. Verify release PR created with correct version bump
2. Review CHANGELOG.md changes
3. Merge release PR
4. Verify: GitHub release created with tag
5. Verify: Binaries built and uploaded
6. Verify: crates.io publish succeeds (if token configured)

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
| Other workflows don't trigger on release | Using GITHUB_TOKEN instead of PAT | Switch to Personal Access Token |
| Tests timeout in CI | Insufficient timeout constants | Increase WATCHER_STARTUP_TIME / EVENT_DETECTION_TIME |

### Rollback Plan

If branch protection causes issues:

```bash
# Disable branch protection temporarily
gh api -X DELETE /repos/rodrigogs/vibewatch/branches/master/protection

# Fix issues

# Re-enable protection
gh api -X PUT /repos/rodrigogs/vibewatch/branches/master/protection \
  --input branch-protection.json
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

1. **Required Reviews**: Add when team grows beyond solo developer
2. **CODEOWNERS**: Define code ownership for different modules
3. **Deploy Preview**: Add preview deployments for PRs
4. **Performance Benchmarks**: Add benchmark CI checks
5. **Automated Rollback**: Add workflow to rollback failed releases
6. **Release Notes Enhancement**: Customize Release Please templates
7. **Binary Signing**: Sign release binaries for verification

### Scaling Considerations

As vibewatch grows:
- Add required reviewers for critical paths
- Implement CODEOWNERS for module ownership
- Consider merge queue for high-velocity merging
- Add deployment environments (staging, production)
- Implement blue-green deployment strategy

## References

- [Release Please Documentation](https://github.com/googleapis/release-please)
- [Release Please Action](https://github.com/googleapis/release-please-action)
- [Conventional Commits Specification](https://www.conventionalcommits.org/)
- [GitHub Branch Protection Documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [GitHub Actions workflow_run Trigger](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#workflow_run)
- [SemVer Specification](https://semver.org/)

---

**Last Updated**: October 5, 2025
**Version**: 1.0
**Maintainer**: @rodrigogs
