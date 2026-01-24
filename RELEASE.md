# Release Process

This document describes how to create a new release of Gosh Authenticator.

## Automated Release Workflow

The project uses GitHub Actions to automatically build and release packages when you tag a new version.

### What Gets Built

For each release, the workflow builds:
- **RPM packages** for Fedora/RHEL (x64 and ARM64)
- **Portable tarballs** (x64 and ARM64)
- **SHA256 checksums** for all packages

### Creating a Release

1. **Ensure all changes are committed and pushed to main:**
   ```bash
   git status
   git push origin main
   ```

2. **Tag the release:**
   ```bash
   # Format: v<major>.<minor>.<patch>
   git tag v1.0.1
   ```

3. **Push the tag to trigger the workflow:**
   ```bash
   git push origin v1.0.1
   ```

4. **Monitor the build:**
   - Go to https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux/actions
   - Watch the "Release Build" workflow
   - Build takes approximately 30-45 minutes

5. **Release is published automatically:**
   - Once complete, the release appears at: https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux/releases
   - All packages are attached to the release
   - Release notes are auto-generated

### Workflow Details

**Jobs:**
1. `build-packages` - Builds packages for x64 and ARM64 in parallel
2. `create-release` - Downloads artifacts and creates GitHub release

**Architectures:**
- x64 (native build on Ubuntu 22.04)
- ARM64 (cross-compiled via Docker/QEMU)

**Artifacts:**
- `gosh-authenticator-<version>-1.fc43.x86_64.rpm`
- `gosh-authenticator-<version>-1.fc43.aarch64.rpm`
- `gosh-authenticator-<version>-linux-x64.tar.gz`
- `gosh-authenticator-<version>-linux-arm64.tar.gz`
- `SHA256SUMS`

### Manual Build (if needed)

If you need to build packages manually without triggering a release:

```bash
# Build RPM
./build-rpm.sh

# Create tarball
cd flutter_app
flutter build linux --release
cd build/linux/x64/release
tar -czf gosh-authenticator-$(git describe --tags --abbrev=0 | sed 's/v//').tar.gz bundle/
```

### Version Numbering

Follow semantic versioning:
- **Major (X.0.0):** Breaking changes
- **Minor (x.Y.0):** New features, backward compatible
- **Patch (x.y.Z):** Bug fixes, backward compatible

Examples:
- `v1.0.0` - Initial stable release
- `v1.1.0` - Added new feature
- `v1.1.1` - Fixed bug
