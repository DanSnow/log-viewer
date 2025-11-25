# Release Process

This project uses [cargo-dist](https://github.com/axodotdev/cargo-dist) to automate releases.

## How It Works

cargo-dist is configured to automatically build and release binaries when you push a git tag that looks like a version number.

## Supported Platforms

The following platforms are automatically built and released:

- **Linux**: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
- **macOS**: `x86_64-apple-darwin` (Intel), `aarch64-apple-darwin` (Apple Silicon)
- **Windows**: `x86_64-pc-windows-msvc`

## Creating a Release

### 1. Update the Version

Edit `Cargo.toml` and bump the version:

```toml
[package]
name = "log-viewer"
version = "0.2.0"  # Update this
```

### 2. Commit the Changes

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
```

### 3. Create and Push a Git Tag

```bash
# Create the tag
git tag v0.2.0

# Push the tag to GitHub
git push origin v0.2.0
```

### 4. Wait for GitHub Actions

The release workflow will automatically:

1. ✅ Build binaries for all supported platforms
2. ✅ Generate installer scripts (shell script for Unix, PowerShell for Windows)
3. ✅ Create checksums for all artifacts
4. ✅ Create a GitHub Release with all artifacts attached
5. ✅ Generate release notes from your git history

You can monitor the progress at: https://github.com/DanSnow/log-viewer/actions

### 5. Edit Release Notes (Optional)

Once the release is created, you can edit the release notes on GitHub to add:

- Highlights of new features
- Breaking changes
- Bug fixes
- Acknowledgments

## Installation Methods for Users

After a release is published, users can install using:

### Pre-built Binaries (Recommended)

**Linux & macOS:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/DanSnow/log-viewer/releases/latest/download/log-viewer-installer.sh | sh
```

**Windows (PowerShell):**
```powershell
powershell -c "irm https://github.com/DanSnow/log-viewer/releases/latest/download/log-viewer-installer.ps1 | iex"
```

### Direct Download

Download binaries directly from: https://github.com/DanSnow/log-viewer/releases

## Testing a Release Locally

You can test the release process locally before creating a tag:

```bash
# Generate release artifacts (doesn't create a release)
dist build

# Check what would be released
dist plan

# Test building for a specific target
dist build --target x86_64-unknown-linux-gnu
```

## Configuration Files

- **`dist-workspace.toml`**: Main cargo-dist configuration
  - Defines supported platforms
  - Configures installers
  - Sets CI backend (GitHub Actions)

- **`.github/workflows/release.yml`**: Auto-generated GitHub Actions workflow
  - **Do not edit manually!** Regenerate with `dist generate`

## Troubleshooting

### Build Failed

Check the GitHub Actions logs at: https://github.com/DanSnow/log-viewer/actions

Common issues:
- Compilation errors (fix in code, push a new commit, create a new tag)
- Missing dependencies on build runners
- Cross-compilation issues

### Want to Add More Platforms?

Edit `dist-workspace.toml`:

```toml
targets = [
  "aarch64-apple-darwin",
  "x86_64-apple-darwin",
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "x86_64-pc-windows-msvc",
  # Add more targets here
]
```

Then regenerate the workflow:

```bash
dist generate
git add .github/workflows/release.yml dist-workspace.toml
git commit -m "chore: update cargo-dist configuration"
```

### Need to Update cargo-dist?

```bash
# Update to latest version
cargo install cargo-dist

# Regenerate configuration
dist generate

# Commit the changes
git add .github/workflows/release.yml dist-workspace.toml
git commit -m "chore: update cargo-dist"
```

## Version Naming

cargo-dist supports various tag formats:

- `v0.1.0` - Standard semantic version (recommended)
- `0.1.0` - Without 'v' prefix
- `v0.1.0-beta.1` - Pre-release (will be marked as pre-release on GitHub)
- `log-viewer/0.1.0` - With package name prefix (for monorepos)

## Documentation

For more information about cargo-dist, see:
- [cargo-dist documentation](https://axodotdev.github.io/cargo-dist/)
- [GitHub repository](https://github.com/axodotdev/cargo-dist)
