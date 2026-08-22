#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root/rust"

command -v cargo-about >/dev/null 2>&1 || {
  printf 'cargo-about is required; install with: cargo install cargo-about --locked --features cli\n' >&2
  exit 1
}

cargo about generate -o ../THIRD_PARTY_LICENSES.html about.hbs
printf 'RUST_THIRD_PARTY_LICENSES_GENERATED output=%s\n' "$root/THIRD_PARTY_LICENSES.html"
