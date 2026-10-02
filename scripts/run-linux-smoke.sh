#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
binary=${1:-$root/rust/target/debug/gosh-authenticator}
test -x "$binary"
qa_dir=$(mktemp -d)
trap 'rm -rf "$qa_dir"' EXIT
export GOSH_SMOKE_BINARY="$binary" GOSH_SMOKE_ROOT="$root" GOSH_SMOKE_DIR="$qa_dir"
xvfb-run -a -s '-screen 0 1280x1024x24' dbus-run-session -- bash <<'SH'
set -euo pipefail
export GTK_A11Y=atspi GDK_BACKEND=x11
"$GOSH_SMOKE_BINARY" --settings "$GOSH_SMOKE_DIR/settings.json" > "$GOSH_SMOKE_DIR/app.log" 2>&1 &
app_pid=$!
trap 'kill "$app_pid" 2>/dev/null || true; wait "$app_pid" 2>/dev/null || true; cat "$GOSH_SMOKE_DIR/app.log"' EXIT
python3 "$GOSH_SMOKE_ROOT/scripts/desktop-smoke.py"
kill -0 "$app_pid"
SH
