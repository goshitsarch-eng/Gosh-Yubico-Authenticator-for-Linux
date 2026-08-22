#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root/rust"

command -v cargo-about >/dev/null 2>&1 || {
  printf 'cargo-about is required; install with: cargo install cargo-about --locked --features cli\n' >&2
  exit 1
}

cargo about generate -o ../THIRD_PARTY_LICENSES.html about.hbs
python3 - "$root/THIRD_PARTY_LICENSES.html" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
normalized = path.read_bytes().replace(b"\r\n", b"\n").replace(b"\r", b"\n")
path.write_bytes(normalized)
PY
printf 'RUST_THIRD_PARTY_LICENSES_GENERATED output=%s\n' "$root/THIRD_PARTY_LICENSES.html"
