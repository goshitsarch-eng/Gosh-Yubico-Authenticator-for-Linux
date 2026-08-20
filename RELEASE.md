# Release Process

This document describes how to create a new release of Gosh Authenticator.

## Automated Release Workflow

The project uses GitHub Actions to automatically build and release packages when you tag a new version.

### What Gets Built

For each release, the workflow builds:
- **Flatpak bundle** (`com.github.gosh.gosh_yubikey_manager`)
- **RPM packages** for Fedora/RHEL (x64 and ARM64)
- **DEB packages** for Debian/Ubuntu (amd64 and arm64)
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
   git tag v1.2.1
   ```

3. **Push the tag to trigger the workflow:**
   ```bash
   git push origin v1.2.1
   ```

4. **Monitor the build:**
   - Go to https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux/actions
   - Watch the "Release Build" workflow

5. **Release is published automatically.**

### Manual Build

```bash
cd rust
cargo build --release
cd ..
./build-flatpak.sh
./build-rpm.sh
./build-deb.sh
```

### Version Numbering

Follow semantic versioning:
- **Major (X.0.0):** Breaking changes
- **Minor (x.Y.0):** New features, backward compatible
- **Patch (x.y.Z):** Bug fixes, backward compatible
