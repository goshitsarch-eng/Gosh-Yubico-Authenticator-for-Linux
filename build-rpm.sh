#!/bin/bash
set -e

# Script to build the Gosh Authenticator RPM package

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
NAME="gosh-authenticator"
VERSION="1.0.0"

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
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Create source tarball
echo "Creating source tarball..."
TEMP_DIR=$(mktemp -d)
cp -r "$PROJECT_DIR" "$TEMP_DIR/${NAME}-${VERSION}"
cd "$TEMP_DIR"
tar --exclude='.git' \
    --exclude='flutter_app/build' \
    --exclude='rust/target' \
    --exclude='*.swp' \
    --exclude='*.kate-swp' \
    -czf ~/rpmbuild/SOURCES/${NAME}-${VERSION}.tar.gz \
    "${NAME}-${VERSION}"
rm -rf "$TEMP_DIR"

# Copy spec file
echo "Copying spec file..."
cp "$PROJECT_DIR/${NAME}.spec" ~/rpmbuild/SPECS/

# Build RPM
echo "Building RPM package..."
cd ~/rpmbuild/SPECS
rpmbuild -bb ${NAME}.spec

echo ""
echo "===================================="
echo "Build complete!"
echo "===================================="
echo "RPM package location:"
ls -lh ~/rpmbuild/RPMS/x86_64/${NAME}-*.rpm
echo ""
echo "To install:"
echo "  sudo dnf install ~/rpmbuild/RPMS/x86_64/${NAME}-${VERSION}-*.rpm"
echo ""
echo "Or to test install:"
echo "  sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/${NAME}-${VERSION}-*.rpm"
