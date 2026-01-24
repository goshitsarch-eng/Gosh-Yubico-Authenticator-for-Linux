#!/bin/bash
set -e

# Script to build the Gosh Authenticator RPM package

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
NAME="gosh-authenticator"
VERSION_DEFAULT="1.2.0"
VERSION="${1:-${VERSION:-$VERSION_DEFAULT}}"

# Where rpmbuild writes BUILD/RPMS/SOURCES/etc.
# Set RPMBUILD_TOPDIR to keep artifacts inside the repo (useful for CI).
TOPDIR="${RPMBUILD_TOPDIR:-$HOME/rpmbuild}"

echo "===================================="
echo "Building Gosh Authenticator RPM"
echo "===================================="

# Check prerequisites
echo "Checking prerequisites..."
if ! command -v rpmbuild &> /dev/null; then
    echo "ERROR: rpmbuild not found. Install with: sudo dnf install rpm-build rpmdevtools"
    exit 1
fi

if ! command -v flutter &> /dev/null; then
    echo "ERROR: flutter not found in PATH"
    echo "Please install Flutter SDK and add it to your PATH"
    echo "Download from: https://docs.flutter.dev/get-started/install/linux"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found. Install Rust from https://rustup.rs/"
    exit 1
fi

# Setup RPM build environment
echo "Setting up RPM build environment..."
mkdir -p "$TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Create source tarball
echo "Creating source tarball..."
TEMP_DIR=$(mktemp -d)
cp -r "$PROJECT_DIR" "$TEMP_DIR/${NAME}-${VERSION}"
cd "$TEMP_DIR"
tar --exclude='.git' \
    --exclude='flutter_app/.dart_tool' \
    --exclude='flutter_app/build' \
    --exclude='flutter_app/linux/flutter/ephemeral' \
    --exclude='flutter_app/windows/flutter/ephemeral' \
    --exclude='flutter_app/macos/Flutter/ephemeral' \
    --exclude='rust/target' \
    --exclude='*.swp' \
    --exclude='*.kate-swp' \
    -czf "$TOPDIR/SOURCES/${NAME}-${VERSION}.tar.gz" \
    "${NAME}-${VERSION}"
rm -rf "$TEMP_DIR"

# Copy spec file
echo "Copying spec file..."
sed -E "s/^Version:[[:space:]]+.*/Version:        ${VERSION}/" \
  "$PROJECT_DIR/${NAME}.spec" > "$TOPDIR/SPECS/${NAME}.spec"

# Build RPM
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
  echo "Available RPM output directories:"
  ls -1 "$TOPDIR/RPMS" || true
fi
echo ""
echo "To install:"
echo "  sudo dnf install $TOPDIR/RPMS/$RPM_ARCH/${NAME}-${VERSION}-*.rpm"
echo ""
echo "Or to test install:"
echo "  sudo rpm -ivh $TOPDIR/RPMS/$RPM_ARCH/${NAME}-${VERSION}-*.rpm"
