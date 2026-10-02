"""Packaging guard tests use header fixtures, never pretend distributables."""
import importlib.util
from pathlib import Path

import pytest

ROOT = Path(__file__).parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


@pytest.mark.parametrize("target,arch,machine", [
    ("linux", "x64", 62), ("linux", "arm64", 183),
    ("macos", "x64", 0x01000007), ("macos", "arm64", 0x0100000C),
    ("windows", "x64", 0x8664), ("windows", "arm64", 0xAA64),
])
def test_binary_architecture_cannot_be_mislabelled(tmp_path, target, arch, machine):
    header = bytearray(64)
    if target == "linux":
        header[:6] = b"\x7fELF\x02\x01"
        header[18:20] = machine.to_bytes(2, "little")
    elif target == "macos":
        header[:4] = b"\xcf\xfa\xed\xfe"
        header[4:8] = machine.to_bytes(4, "little")
    else:
        header[:2] = b"MZ"
        header[60:64] = (64).to_bytes(4, "little")
        header.extend(b"PE\0\0" + machine.to_bytes(2, "little"))
    binary = tmp_path / "header fixture"
    binary.write_bytes(header)
    package = module("package")
    package.validate_binary(binary, target, arch)
    with pytest.raises(ValueError, match="does not match"):
        package.validate_binary(binary, target, "arm64" if arch == "x64" else "x64")


def test_invalid_binary_and_out_of_bounds_pe_are_rejected(tmp_path):
    binary = tmp_path / "invalid"
    binary.write_bytes(b"a text file")
    package = module("package")
    with pytest.raises(ValueError, match="native"):
        package.validate_binary(binary, "linux", "x64")
    header = bytearray(64)
    header[:2] = b"MZ"
    header[60:64] = (2**32 - 1).to_bytes(4, "little")
    binary.write_bytes(header)
    with pytest.raises(ValueError, match="PE"):
        package.validate_binary(binary, "windows", "x64")


def test_incomplete_release_or_wrong_tag_creates_no_output(tmp_path):
    release = module("release-artifacts")
    source = tmp_path / "artifacts"
    source.mkdir()
    destination = tmp_path / "release"
    with pytest.raises(ValueError, match="exactly match"):
        release.collect(source, destination, "v0.0.0")
    assert not destination.exists()
    with pytest.raises(ValueError, match="nonempty artifact"):
        release.collect(source, destination, f"v{release.VERSION}")
    assert not destination.exists()
