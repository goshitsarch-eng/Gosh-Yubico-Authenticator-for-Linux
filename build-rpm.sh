#!/bin/bash
set -euo pipefail

# Build the Gosh Authenticator RPM package from the native GTK 4 sources.

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
NAME="gosh-authenticator"
VERSION_DEFAULT="1.2.0"
VERSION="${1:-${VERSION:-$VERSION_DEFAULT}}"
TOPDIR="${RPMBUILD_TOPDIR:-$HOME/rpmbuild}"

echo "===================================="
echo "Building Gosh Authenticator RPM"
echo "===================================="

if ! command -v rpmbuild >/dev/null 2>&1; then
  echo "ERROR: rpmbuild not found. Install with: sudo dnf install rpm-build rpmdevtools" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: cargo not found. Install Rust from https://rustup.rs/" >&2
  exit 1
fi

echo "Setting up RPM build environment..."
mkdir -p "$TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

echo "Creating source tarball..."
TEMP_DIR=$(mktemp -d)
cp -r "$PROJECT_DIR" "$TEMP_DIR/${NAME}-${VERSION}"
cd "$TEMP_DIR"
tar --exclude='.git' \
    --exclude='rust/target' \
    --exclude='dist' \
    --exclude='*.swp' \
    --exclude='*.kate-swp' \
    -czf "$TOPDIR/SOURCES/${NAME}-${VERSION}.tar.gz" \
    "${NAME}-${VERSION}"
rm -rf "$TEMP_DIR"

echo "Copying spec file..."
sed -E "s/^Version:[[:space:]]+.*/Version:        ${VERSION}/" \
  "$PROJECT_DIR/${NAME}.spec" > "$TOPDIR/SPECS/${NAME}.spec"

echo "Building RPM package..."
rpmbuild -bb --nodeps \
  --define "_topdir $TOPDIR" \
  "$TOPDIR/SPECS/${NAME}.spec"

echo ""
echo "===================================="
echo "Build complete!"
echo "===================================="
echo "RPM package location:"

RPM_ARCH="$(rpm --eval '%{_arch}' 2>/dev/null || uname -m)"
if compgen -G "$TOPDIR/RPMS/$RPM_ARCH/${NAME}-*.rpm" > /dev/null; then
  ls -lh "$TOPDIR/RPMS/$RPM_ARCH/${NAME}-"*.rpm
else
  echo "No RPMs found in: $TOPDIR/RPMS/$RPM_ARCH/"
  ls -1 "$TOPDIR/RPMS" || true
fi
