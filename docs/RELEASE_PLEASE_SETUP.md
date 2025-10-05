# Release Please Setup Guide

This document explains how to configure automated releases using Release Please.

## The Problem

GitHub Actions has a security restriction: workflows using `GITHUB_TOKEN` cannot create or approve pull requests. This prevents Release Please from creating release PRs.

**Error message:**
```
Error: GitHub Actions is not permitted to create or approve pull requests.
```

## The Solution

Use a Personal Access Token (PAT) with `repo` scope instead of the default `GITHUB_TOKEN`.

## Setup Instructions

### 1. Create a Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Direct link: https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Configure the token:
   - **Note**: `vibewatch-release-please` (or any descriptive name)
   - **Expiration**: Choose appropriate expiration (or "No expiration" for long-term projects)
   - **Scopes**: Select `repo` (Full control of private repositories)
     - This includes: `repo:status`, `repo_deployment`, `public_repo`, `repo:invite`, `security_events`
4. Click "Generate token"
5. **Copy the token immediately** (you won't be able to see it again)

### 2. Add Token as Repository Secret

1. Go to your repository on GitHub
2. Navigate to Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Configure the secret:
   - **Name**: `RELEASE_PLEASE_TOKEN`
   - **Secret**: Paste the PAT you created in step 1
5. Click "Add secret"

### 3. Verify Configuration

The workflow is already configured to use the token:

```yaml
- name: Run Release Please
  id: release
  uses: googleapis/release-please-action@v4
  with:
    release-type: rust
    # Uses PAT if available, falls back to GITHUB_TOKEN
    token: ${{ secrets.RELEASE_PLEASE_TOKEN || secrets.GITHUB_TOKEN }}
```

### 4. Test the Setup

1. Make a commit with a conventional commit message:
   ```bash
   git commit -m "feat: add new feature"
   git push origin master
   ```

2. Check the Actions tab to verify Release Please creates a PR:
   - Go to: https://github.com/rodrigogs/vibewatch/actions
   - Look for the "Release Please" workflow
   - It should complete successfully and create a release PR

## How It Works

### Workflow Trigger
Release Please runs on every push to the `master` branch:

```yaml
on:
  push:
    branches:
      - master
```

### Release PR Creation
1. Release Please analyzes commits since the last release
2. Based on conventional commit messages, it determines the version bump:
   - `feat:` → minor version bump (0.X.0)
   - `fix:` → patch version bump (0.0.X)
   - `feat!:` or `fix!:` → major version bump (X.0.0)
3. It creates/updates a release PR with:
   - Updated `CHANGELOG.md`
   - Updated version in `Cargo.toml`
   - Updated `Cargo.lock`

### Release Creation
1. When you merge the release PR, Release Please:
   - Creates a GitHub Release with tag (e.g., `v0.2.0`)
   - Includes release notes from `CHANGELOG.md`
2. The CI workflow then:
   - Publishes to crates.io (if `CARGO_TOKEN` is configured)
   - Builds binaries for 6 platforms
   - Attaches binaries to the GitHub Release

## Troubleshooting

### Error: "GitHub Actions is not permitted to create or approve pull requests"

**Cause**: The `RELEASE_PLEASE_TOKEN` secret is not configured.

**Solution**: Follow steps 1-2 above to create and add the PAT.

### Error: "Resource not accessible by integration"

**Cause**: The PAT doesn't have sufficient permissions.

**Solution**: Verify the token has `repo` scope enabled.

### Error: "Bad credentials"

**Cause**: The PAT has expired or been revoked.

**Solution**: Generate a new PAT and update the secret.

### Release PR is not created

**Possible causes:**
1. No conventional commits since last release
   - Solution: Use `feat:`, `fix:`, etc. in commit messages
2. Token not configured correctly
   - Solution: Check secret name is exactly `RELEASE_PLEASE_TOKEN`
3. Workflow permissions insufficient
   - Solution: Verify workflow has `contents: write` and `pull-requests: write` permissions

## Alternative: Workflow Permissions (Organization Setting)

If you have admin access to the organization/repository, you can allow GitHub Actions to create pull requests:

1. Go to Settings → Actions → General
2. Scroll to "Workflow permissions"
3. Select "Read and write permissions"
4. Check "Allow GitHub Actions to create and approve pull requests"
5. Click "Save"

**Note**: This is less secure than using a PAT with limited scope. PAT is the recommended approach.

## Security Considerations

- **PAT Scope**: Only grant `repo` scope, nothing more
- **PAT Expiration**: Set an expiration date and rotate tokens regularly
- **Secret Storage**: Never commit the PAT to the repository
- **Access Control**: Limit who can modify repository secrets
- **Audit**: Regularly review PAT usage in GitHub's audit log

## Further Reading

- [Release Please Documentation](https://github.com/googleapis/release-please)
- [Conventional Commits Specification](https://www.conventionalcommits.org/)
- [GitHub Actions Permissions](https://docs.github.com/en/actions/security-guides/automatic-token-authentication)
- [GitHub Personal Access Tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)
