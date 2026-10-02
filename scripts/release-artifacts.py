#!/usr/bin/env python3
"""Require every advertised native/Flatpak artifact before a draft release."""
import argparse
import hashlib
import shutil
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = tomllib.loads((ROOT / "rust/Cargo.toml").read_text())["package"]["version"]


def collect(source: Path, destination: Path, tag: str) -> None:
    if tag != f"v{VERSION}":
        raise ValueError("Release tag must exactly match Cargo.toml")
    patterns = [
        f"gosh-authenticator-{VERSION}-windows-x64.msi",
        *[f"gosh-authenticator-{VERSION}-macos-{arch}.zip" for arch in ("x64", "arm64")],
        *[f"gosh-authenticator-{VERSION}-linux-{arch}.tar.gz" for arch in ("x64", "arm64")],
        *[f"gosh-authenticator_{VERSION.replace('-', '~', 1)}_{arch}.deb" for arch in ("amd64", "arm64")],
        *[f"gosh-authenticator-{VERSION.replace('-', '~', 1)}-*.{arch}.rpm" for arch in ("x86_64", "aarch64")],
        *[f"gosh-authenticator-{arch}.flatpak" for arch in ("x86_64", "aarch64")],
    ]
    selected = []
    for pattern in patterns:
        matches = list(source.rglob(pattern))
        if len(matches) != 1 or matches[0].stat().st_size == 0:
            raise ValueError(f"Expected one nonempty artifact matching {pattern}: {matches}")
        selected.append(matches[0])
    # Only create the release directory once the complete artifact set exists.
    destination.mkdir(parents=True, exist_ok=False)
    hashes = []
    for artifact in selected:
        name = artifact.name
        if artifact.suffix == ".flatpak":
            name = name.replace("gosh-authenticator-", f"gosh-authenticator-{VERSION}-", 1)
        output = destination / name
        shutil.copy2(artifact, output)
        with output.open("rb") as handle:
            checksum = hashlib.file_digest(handle, "sha256").hexdigest()
        hashes.append(f"{checksum}  {name}\n")
    (destination / "SHA256SUMS").write_text("".join(sorted(hashes)))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--tag", required=True)
    args = parser.parse_args()
    collect(args.source, args.destination, args.tag)
