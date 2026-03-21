#!/bin/bash
set -e

# Script to build the Gosh Authenticator DEB package (pure Rust)

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
NAME="gosh-authenticator"

# Priority: arg > env > git tag > fallback
VERSION_INPUT="${1:-${VERSION:-}}"
if [ -z "$VERSION_INPUT" ] && command -v git >/dev/null 2>&1; then
  TAG="$(git -C "$PROJECT_DIR" describe --tags --abbrev=0 2>/dev/null || true)"
  VERSION_INPUT="${TAG#v}"
fi
VERSION_INPUT="${VERSION_INPUT:-2.0.0}"

ARCH_INPUT="${2:-${DEB_ARCH:-}}"
if [ -z "$ARCH_INPUT" ]; then
  if command -v dpkg >/dev/null 2>&1; then
    ARCH_INPUT="$(dpkg --print-architecture)"
  else
    echo "ERROR: dpkg not found. Install dpkg." >&2
    exit 1
  fi
fi

if ! command -v dpkg-deb >/dev/null 2>&1; then
  echo "ERROR: dpkg-deb not found. Install with: sudo apt-get install dpkg-dev" >&2
  exit 1
fi

# Build the Rust binary
echo "Building Rust binary..."
cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"

BINARY="$PROJECT_DIR/target/release/gosh-authenticator"
if [ ! -f "$BINARY" ]; then
  echo "ERROR: Binary not found at: $BINARY" >&2
  exit 1
fi

OUT_NAME="${NAME}_${VERSION_INPUT}_${ARCH_INPUT}.deb"
OUT_PATH="$PROJECT_DIR/dist/$OUT_NAME"

echo "===================================="
echo "Building Gosh Authenticator DEB"
echo "Version: $VERSION_INPUT"
echo "Arch:    $ARCH_INPUT"
echo "===================================="

STAGE_DIR="$(mktemp -d)"
PKG_DIR="$STAGE_DIR/${NAME}_${VERSION_INPUT}_${ARCH_INPUT}"

mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/scalable/apps"

# Install binary
install -m 0755 "$BINARY" "$PKG_DIR/usr/bin/gosh-authenticator"

# Desktop file
cat > "$PKG_DIR/usr/share/applications/com.github.gosh.gosh_authenticator.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Gosh Authenticator
Comment=YubiKey OATH credential manager
Exec=gosh-authenticator
Icon=com.github.gosh.gosh_authenticator
Terminal=false
Categories=Utility;Security;
Keywords=YubiKey;TOTP;HOTP;OTP;Authenticator;2FA;
EOF

# Icon
install -m 0644 "$PROJECT_DIR/icon.svg" \
  "$PKG_DIR/usr/share/icons/hicolor/scalable/apps/com.github.gosh.gosh_authenticator.svg"

# Control
cat > "$PKG_DIR/DEBIAN/control" << EOF
Package: ${NAME}
Version: ${VERSION_INPUT}
Section: utils
Priority: optional
Architecture: ${ARCH_INPUT}
Maintainer: Builder <builder@localhost>
Depends: libpcsclite1
Description: Desktop app for managing OATH credentials on YubiKey
 Gosh Authenticator is a pure Rust cross-platform desktop application
 for managing OATH (TOTP/HOTP) credentials on YubiKey devices.
 Built with the Iced GUI framework.
EOF
chmod 0644 "$PKG_DIR/DEBIAN/control"

# postinst/postrm
cat > "$PKG_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f /usr/share/icons/hicolor >/dev/null 2>&1 || true
fi
exit 0
EOF
chmod 0755 "$PKG_DIR/DEBIAN/postinst"

cat > "$PKG_DIR/DEBIAN/postrm" << 'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f /usr/share/icons/hicolor >/dev/null 2>&1 || true
fi
exit 0
EOF
chmod 0755 "$PKG_DIR/DEBIAN/postrm"

mkdir -p "$PROJECT_DIR/dist"
dpkg-deb --build "$PKG_DIR" "$OUT_PATH" >/dev/null

rm -rf "$STAGE_DIR"

echo ""
echo "===================================="
echo "Build complete!"
echo "===================================="
echo "DEB package location:"
ls -lh "$OUT_PATH"
