#!/usr/bin/env python3
"""Generate Flatpak cargo-sources.json from rust/Cargo.lock (crates.io)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

CRATES_IO = "https://static.crates.io/crates"
CARGO_HOME = "cargo"
CARGO_CRATES = f"{CARGO_HOME}/vendor"
VENDORED_SOURCES = "vendored-sources"


def parse_lockfile(path: Path) -> list[dict[str, str]]:
    packages: list[dict[str, str]] = []
    current: dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line == "[[package]]":
            if current:
                packages.append(current)
            current = {}
            continue
        if "=" not in line or line.startswith("["):
            continue
        key, _, value = line.partition("=")
        key = key.strip()
        if key not in {"name", "version", "source", "checksum"}:
            continue
        current[key] = value.strip().strip('"')
    if current:
        packages.append(current)
    return packages


def crate_sources(package: dict[str, str]) -> list[dict]:
    name = package["name"]
    version = package["version"]
    checksum = package["checksum"]
    dest = f"{CARGO_CRATES}/{name}-{version}"
    return [
        {
            "type": "archive",
            "archive-type": "tar-gzip",
            "url": f"{CRATES_IO}/{name}/{name}-{version}.crate",
            "sha256": checksum,
            "dest": dest,
        },
        {
            "type": "inline",
            "contents": json.dumps({"package": checksum, "files": {}}),
            "dest": dest,
            "dest-filename": ".cargo-checksum.json",
        },
    ]


def generate(packages: list[dict[str, str]]) -> list[dict]:
    sources: list[dict] = []
    for package in packages:
        source = package.get("source", "")
        if not source:
            continue
        if source.startswith("git+"):
            raise SystemExit(
                f"git crate {package.get('name')} is not supported by this generator"
            )
        if "checksum" not in package:
            raise SystemExit(f"missing checksum for {package.get('name')}")
        sources.extend(crate_sources(package))

    cargo_config = """[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "cargo/vendor"
"""
    sources.append(
        {
            "type": "inline",
            "contents": cargo_config,
            "dest": CARGO_HOME,
            "dest-filename": "config",
        }
    )
    return sources


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--lockfile",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "rust" / "Cargo.lock",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).resolve().parent / "cargo-sources.json",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit 1 if cargo-sources.json does not match the lockfile",
    )
    args = parser.parse_args()

    packages = parse_lockfile(args.lockfile)
    generated = generate(packages)
    serialized = json.dumps(generated, indent=4) + "\n"

    if args.check:
        if not args.output.exists():
            print(f"missing {args.output}", file=sys.stderr)
            return 1
        current = args.output.read_text(encoding="utf-8")
        if current != serialized:
            print(f"{args.output} is out of date; rerun generate-cargo-sources.py", file=sys.stderr)
            return 1
        print(f"{args.output} matches {args.lockfile}")
        return 0

    args.output.write_text(serialized, encoding="utf-8")
    crate_count = sum(1 for p in packages if p.get("checksum"))
    print(f"wrote {args.output} ({crate_count} crates)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
