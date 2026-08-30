#!/bin/bash
set -euo pipefail

# Build and optionally install the Gosh Authenticator Flatpak (KDE runtime).

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_ID="com.goshapps.YubicoAuthenticator"
MANIFEST="$PROJECT_DIR/${APP_ID}.yml"
BUILD_DIR="${FLATPAK_BUILD_DIR:-$PROJECT_DIR/.flatpak-builder/app}"
STATE_DIR="${FLATPAK_STATE_DIR:-$PROJECT_DIR/.flatpak-builder/state}"
REPO_DIR="${FLATPAK_REPO_DIR:-$PROJECT_DIR/.flatpak-builder/repo}"
BUNDLE="${FLATPAK_BUNDLE:-$PROJECT_DIR/dist/${APP_ID}.flatpak}"
RUNTIME_VERSION="${FLATPAK_RUNTIME_VERSION:-6.9}"

VERSION_INPUT="${1:-${VERSION:-}}"
if [ -z "$VERSION_INPUT" ] && command -v git >/dev/null 2>&1; then
  TAG="$(git -C "$PROJECT_DIR" describe --tags --abbrev=0 2>/dev/null || true)"
  VERSION_INPUT="${TAG#v}"
fi
VERSION_INPUT="${VERSION_INPUT:-2.0.0}"

if ! command -v flatpak-builder >/dev/null 2>&1; then
  echo "ERROR: flatpak-builder not found. Install flatpak and flatpak-builder." >&2
  exit 1
fi
if ! command -v eu-strip >/dev/null 2>&1; then
  echo "NOTE: eu-strip not found. Install elfutils so Flatpak debuginfo stripping works." >&2
fi

echo "===================================="
echo "Building Gosh Authenticator Flatpak"
echo "Version: $VERSION_INPUT"
echo "===================================="

if ! flatpak remotes --user | grep -q '^flathub'; then
  echo "Adding Flathub remote..."
  flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
fi

echo "Installing KDE $RUNTIME_VERSION SDK and Platform..."
flatpak install --user -y --noninteractive flathub \
  "org.kde.Platform//${RUNTIME_VERSION}" \
  "org.kde.Sdk//${RUNTIME_VERSION}"

mkdir -p "$PROJECT_DIR/dist" "$BUILD_DIR" "$STATE_DIR" "$REPO_DIR"

BUILDER_ARGS=(
  --user
  --force-clean
  --disable-rofiles-fuse
  --install-deps-from=flathub
  --state-dir="$STATE_DIR"
  --repo="$REPO_DIR"
  --default-branch=stable
)

echo "Running flatpak-builder..."
flatpak-builder "${BUILDER_ARGS[@]}" --install "$BUILD_DIR" "$MANIFEST"

BUNDLE_NAME="gosh-authenticator-${VERSION_INPUT}.flatpak"
BUNDLE_PATH="$PROJECT_DIR/dist/$BUNDLE_NAME"
echo "Exporting bundle $BUNDLE_PATH..."
flatpak build-bundle "$REPO_DIR" "$BUNDLE_PATH" "$APP_ID" stable

# Keep a stable filename as well for docs/scripts that expect the app-id bundle.
cp -f "$BUNDLE_PATH" "$BUNDLE"

echo ""
echo "===================================="
echo "Build complete!"
echo "===================================="
echo "Bundle: $BUNDLE_PATH"
echo "Install from the bundle:"
echo "  flatpak install --user $BUNDLE_PATH"
echo "Run:"
echo "  flatpak run $APP_ID"
ls -lh "$BUNDLE_PATH"
