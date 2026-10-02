#!/bin/bash
set -euo pipefail

# Build and optionally install the Gosh Authenticator Flatpak.

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_ID="com.goshapps.YubicoAuthenticator"
MANIFEST="$PROJECT_DIR/${APP_ID}.yml"
BUILD_DIR="${FLATPAK_BUILD_DIR:-$PROJECT_DIR/.flatpak-builder/app}"
STATE_DIR="${FLATPAK_STATE_DIR:-$PROJECT_DIR/.flatpak-builder/state}"
REPO_DIR="${FLATPAK_REPO_DIR:-$PROJECT_DIR/.flatpak-builder/repo}"
BUNDLE="${FLATPAK_BUNDLE:-$PROJECT_DIR/dist/${APP_ID}.flatpak}"
RUNTIME_VERSION="${FLATPAK_RUNTIME_VERSION:-50}"
VERSION_DEFAULT="$(python3 -c 'import sys, tomllib; print(tomllib.load(open(sys.argv[1], "rb"))["package"]["version"])' "$PROJECT_DIR/rust/Cargo.toml")"
VERSION_INPUT="${1:-${VERSION:-$VERSION_DEFAULT}}"
if [ "$VERSION_INPUT" != "$VERSION_DEFAULT" ]; then
  echo "ERROR: bundle version must match Cargo.toml ($VERSION_DEFAULT)" >&2
  exit 1
fi

if ! command -v eu-strip >/dev/null 2>&1; then
  echo "NOTE: eu-strip not found. Install elfutils so Flatpak debuginfo stripping works." >&2
fi
if ! command -v rsvg-convert >/dev/null 2>&1; then
  echo "NOTE: rsvg-convert not found. Install librsvg2-bin if you need to regenerate PNG icons." >&2
fi

if [ ! -f "$PROJECT_DIR/flatpak/cargo-sources.json" ]; then
  echo "ERROR: missing flatpak/cargo-sources.json" >&2
  echo "Generate it with: python3 flatpak/generate-cargo-sources.py" >&2
  exit 1
fi

echo "===================================="
echo "Building Gosh Authenticator Flatpak"
echo "Version: $VERSION_INPUT"
echo "===================================="

python3 "$PROJECT_DIR/flatpak/generate-cargo-sources.py" --check

if ! flatpak remotes --user --columns=name | grep -qx flathub; then
  echo "Adding Flathub remote..."
  flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
fi

echo "Installing GNOME $RUNTIME_VERSION SDK and Platform..."
flatpak install --user -y --noninteractive flathub \
  "org.gnome.Platform//${RUNTIME_VERSION}" \
  "org.gnome.Sdk//${RUNTIME_VERSION}"
# The manifest supplies its own checksum-pinned Rust archive for each CPU.

mkdir -p "$PROJECT_DIR/dist" "$BUILD_DIR" "$STATE_DIR" "$REPO_DIR"

BUILDER_ARGS=(
  --user
  --force-clean
  --delete-build-dirs
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
