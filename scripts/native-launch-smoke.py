#!/usr/bin/env python3
"""Check native process startup/shutdown; this is not full native UI QA."""
import argparse
import subprocess
import tempfile
import time
import tomllib
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("binary", type=Path)
args = parser.parse_args()
binary = args.binary.resolve(strict=True)
version=tomllib.loads((Path(__file__).resolve().parents[1]/'rust/Cargo.toml').read_text())['package']['version']
assert version in subprocess.check_output([str(binary), "--version"], text=True)
with tempfile.TemporaryDirectory(prefix="Gosh smoke 配置 ") as directory:
    path = Path(directory) / "settings.json"
    with (Path(directory) / "app.log").open("w+") as log:
        process = subprocess.Popen([str(binary), "--settings", str(path)], stdout=log, stderr=log)
        try:
            time.sleep(5)
            if process.poll() is not None:
                log.seek(0)
                raise RuntimeError(f"Native startup failed: {log.read()}")
            print("Native desktop process stayed alive for 5 seconds. Full interaction/hardware QA remains required.")
        finally:
            process.terminate()
            process.wait(timeout=15)
