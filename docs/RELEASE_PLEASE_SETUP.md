# Release Please Setup Guide

This document explains the automated release configuration for vibewatch.

## Status: ✅ Fully Configured

Release Please is **already configured and working**. No additional setup required!

## How It Works

### Repository Configuration

The repository has been configured with the following settings to allow automated releases:

**Workflow Permissions** (Settings → Actions → General):
- ✅ Default workflow permissions: **Read and write**
- ✅ Allow GitHub Actions to create and approve pull requests: **Enabled**

This allows the Release Please workflow to:
- Analyze conventional commits
- Create/update release pull requests
- Update CHANGELOG.md and version numbers automatically

### Release Workflow

The `.github/workflows/release.yml` workflow runs on every push to `master` and:

1. **Analyzes commits** using conventional commit format:
   - `feat:` → minor version bump (0.X.0)
   - `fix:` → patch version bump (0.0.X)
   - `feat!:` or `fix!:` → major version bump (X.0.0)

2. **Creates/updates a release PR** with:
   - Updated version in `Cargo.toml`
   - Updated `Cargo.lock`
   - Generated `CHANGELOG.md` entries

3. **When you merge the release PR**:
   - Creates a GitHub Release with tag
   - Publishes to crates.io (if `CARGO_TOKEN` is configured)
   - Builds binaries for 6 platforms
   - Attaches binaries to the release

## Using Release Please

### 1. Make commits with conventional format

```bash
git commit -m "feat: add new feature"
git commit -m "fix: resolve bug"
git commit -m "feat!: breaking change"
git push origin master
```

### 2. Wait for Release Please to create a PR

After pushing, check the Pull Requests tab for a PR titled like "chore: release 0.2.0".

### 3. Review and merge the release PR

The PR will show:
- Version bump
- CHANGELOG updates
- All commits since last release

Merge it when ready to release!

### 4. Release is automatically published

After merging:
- GitHub Release is created
- Binaries are built and attached
- (Optional) Package is published to crates.io

## Conventional Commit Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Commit Types

- `feat:` - New feature (minor bump)
- `fix:` - Bug fix (patch bump)
- `docs:` - Documentation only
- `chore:` - Maintenance/tooling
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `perf:` - Performance improvement
- `ci:` - CI/CD changes

### Breaking Changes

Add `!` after type or add `BREAKING CHANGE:` in footer:

```bash
feat!: redesign CLI interface
```

or

```bash
feat: redesign CLI

BREAKING CHANGE: old CLI flags are no longer supported
```

## Troubleshooting

### Issue: Release PR is not created

**Possible causes:**

1. **No commits since last release**
   - Solution: Make a commit with conventional format

2. **Commits don't follow conventional format**
   - Solution: Use `feat:`, `fix:`, etc. prefixes

3. **Workflow permissions reverted**
   - Solution: Check Settings → Actions → General → Workflow permissions
   - Ensure "Read and write permissions" is selected
   - Ensure "Allow GitHub Actions to create and approve pull requests" is enabled

### Issue: Workflow fails with permission error

**Error**: "GitHub Actions is not permitted to create or approve pull requests"

**Solution**: The workflow permissions may have been changed. To fix:

```bash
gh api -X PUT repos/rodrigogs/vibewatch/actions/permissions/workflow \
  -f default_workflow_permissions=write \
  -F can_approve_pull_request_reviews=true
```

Or manually via GitHub UI:
1. Go to Settings → Actions → General
2. Under "Workflow permissions":
   - Select "Read and write permissions"
   - Check "Allow GitHub Actions to create and approve pull requests"
3. Click "Save"

### Issue: Release is not published to crates.io

**Cause**: `CARGO_TOKEN` secret is not configured.

**Solution**: Add your crates.io API token:

```bash
# Get your token from https://crates.io/me
gh secret set CARGO_TOKEN
# Paste your token when prompted
```

## Security Notes

Using repository workflow permissions (instead of a PAT) is the **recommended approach** because:
- ✅ No token management required
- ✅ No token expiration issues
- ✅ Automatically scoped to repository
- ✅ Follows principle of least privilege
- ✅ Easier to audit and maintain

The workflow has explicit permissions defined:
```yaml
permissions:
  contents: write        # Create releases
  pull-requests: write   # Create release PRs
```

## Alternative: Personal Access Token (Not Recommended)

If you prefer to use a PAT instead of repository permissions:

1. Create a PAT at https://github.com/settings/tokens with `repo` scope
2. Add it as a secret:
   ```bash
   gh secret set RELEASE_PLEASE_TOKEN
   ```
3. Update `.github/workflows/release.yml`:
   ```yaml
   token: ${{ secrets.RELEASE_PLEASE_TOKEN }}
   ```

**Note**: This approach requires token maintenance and rotation. Repository permissions are simpler.

## Further Reading

- [Release Please Documentation](https://github.com/googleapis/release-please)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub Actions Permissions](https://docs.github.com/en/actions/security-guides/automatic-token-authentication)
