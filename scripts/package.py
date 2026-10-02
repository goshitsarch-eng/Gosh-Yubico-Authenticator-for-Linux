#!/usr/bin/env python3
"""Package an already-built native binary. Does not cross-compile or publish."""
from __future__ import annotations

import argparse
import hashlib
import platform
import plistlib
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP_ID = "com.goshapps.YubicoAuthenticator"
APP_NAME = "Gosh Yubico Authenticator"
VERSION = tomllib.loads((ROOT / "rust/Cargo.toml").read_text())["package"]["version"]


def validate_binary(binary: Path, target: str, arch: str) -> None:
    """Prevent a real artifact from being labelled for the wrong OS/CPU."""
    with binary.open("rb") as handle:
        header = handle.read(64)
        if target == "linux" and header[:6] == b"\x7fELF\x02\x01":
            machine = int.from_bytes(header[18:20], "little")
            expected = {"x64": 62, "arm64": 183}[arch]
        elif target == "macos" and header[:4] == b"\xcf\xfa\xed\xfe":
            machine = int.from_bytes(header[4:8], "little")
            expected = {"x64": 0x01000007, "arm64": 0x0100000C}[arch]
        elif target == "windows" and header[:2] == b"MZ" and len(header) == 64:
            offset = int.from_bytes(header[60:64], "little")
            if offset < 64 or offset > binary.stat().st_size - 6:
                raise ValueError("Invalid PE executable header")
            handle.seek(offset)
            pe = handle.read(6)
            if pe[:4] != b"PE\0\0":
                raise ValueError("Invalid PE executable signature")
            machine = int.from_bytes(pe[4:6], "little")
            expected = {"x64": 0x8664, "arm64": 0xAA64}[arch]
        else:
            raise ValueError(f"Expected a native 64-bit {target} executable")
    if machine != expected:
        raise ValueError(f"Executable CPU {machine:#x} does not match {arch}")


def licenses(destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    for name in ("LICENSE", "THIRD_PARTY_LICENSES.html"):
        shutil.copy2(ROOT / name, destination / name)


def windows_source(binary: Path, directory: Path, arch: str) -> Path:
    """WiX 6 source: stable upgrade identity, embedded files, native uninstall."""
    ns = "http://wixtoolset.org/schemas/v4/wxs"
    ET.register_namespace("", ns)
    def child(parent, tag, **attrs):
        return ET.SubElement(parent, f"{{{ns}}}{tag}", attrs)
    wix = ET.Element(f"{{{ns}}}Wix")
    package = child(wix, "Package", Name=APP_NAME, Manufacturer="Goshitsarch",
                    Version=VERSION.split("-")[0], Language="1033", Scope="perMachine",
                    UpgradeCode="BF45B942-6B8C-4D6A-BDD1-22FE8B4B0499")
    child(package, "MajorUpgrade", DowngradeErrorMessage="A newer version of Gosh is already installed.", AllowSameVersionUpgrades="yes")
    child(package, "MediaTemplate", EmbedCab="yes")
    child(package, "Icon", Id="GoshIcon", SourceFile=str(directory / "gosh.ico"))
    child(package, "Property", Id="ARPPRODUCTICON", Value="GoshIcon")
    programs = child(package, "StandardDirectory", Id="ProgramFiles64Folder" if arch == "x64" else "ProgramFiles6432Folder")
    install = child(programs, "Directory", Id="INSTALLFOLDER", Name=APP_NAME)
    component = child(install, "Component", Id="MainExecutable", Guid="*")
    child(component, "File", Id="GoshExe", Source=str(binary), KeyPath="yes")
    for index, name in enumerate(("LICENSE", "THIRD_PARTY_LICENSES.html")):
        item = child(install, "Component", Id=f"Notice{index}", Guid="*")
        child(item, "File", Source=str(ROOT / name), KeyPath="yes")
    menu = child(package, "StandardDirectory", Id="ProgramMenuFolder")
    menu = child(menu, "Directory", Id="GoshMenu", Name=APP_NAME)
    shortcut = child(menu, "Component", Id="StartMenuShortcut", Guid="*")
    child(shortcut, "Shortcut", Id="GoshShortcut", Name=APP_NAME, Target="[#GoshExe]", WorkingDirectory="INSTALLFOLDER", Icon="GoshIcon")
    child(shortcut, "RemoveFolder", Id="RemoveGoshMenu", On="uninstall")
    child(shortcut, "RegistryValue", Root="HKCU", Key=f"Software\\{APP_ID}", Name="installed", Type="integer", Value="1", KeyPath="yes")
    feature = child(package, "Feature", Id="Main", Title=APP_NAME, Level="1")
    for name in ("MainExecutable", "Notice0", "Notice1", "StartMenuShortcut"):
        child(feature, "ComponentRef", Id=name)
    source = directory / "gosh.wxs"
    ET.ElementTree(wix).write(source, encoding="utf-8", xml_declaration=True)
    return source


def package_windows(binary: Path, stage: Path, out: Path, arch: str) -> Path:
    from PIL import Image
    image = Image.open(ROOT / f"data/icons/hicolor/128x128/apps/{APP_ID}.png")
    image.save(stage / "gosh.ico", sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128)])
    source = windows_source(binary, stage, arch)
    artifact = out / f"gosh-authenticator-{VERSION}-windows-{arch}.msi"
    subprocess.run(["wix", "build", "-arch", arch, "-o", str(artifact), str(source)], check=True)
    return artifact


def package_macos(binary: Path, stage: Path, out: Path, arch: str) -> Path:
    from PIL import Image
    app = stage / f"{APP_NAME}.app"
    contents = app / "Contents"
    (contents / "MacOS").mkdir(parents=True)
    (contents / "Resources").mkdir()
    shutil.copy2(binary, contents / "MacOS/gosh-authenticator")
    licenses(contents / "Resources/licenses")
    iconset = stage / "gosh.iconset"
    iconset.mkdir()
    image = Image.open(ROOT / f"data/icons/hicolor/128x128/apps/{APP_ID}.png")
    for size in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            suffix = "@2x" if scale == 2 else ""
            image.resize((size * scale, size * scale), Image.Resampling.LANCZOS).save(iconset / f"icon_{size}x{size}{suffix}.png")
    subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(contents / "Resources/gosh.icns")], check=True)
    with (contents / "Info.plist").open("wb") as handle:
        plistlib.dump({"CFBundleIdentifier": APP_ID, "CFBundleName": APP_NAME,
                      "CFBundleDisplayName": APP_NAME, "CFBundleExecutable": "gosh-authenticator",
                      "CFBundlePackageType": "APPL", "CFBundleIconFile": "gosh.icns",
                      "CFBundleShortVersionString": VERSION.split("-")[0], "CFBundleVersion": VERSION.split("-")[0],
                      "LSMinimumSystemVersion": "11.0", "NSHighResolutionCapable": True,
                      "NSHumanReadableCopyright": "© Goshitsarch. GPL-3.0-or-later.",
                      "NSRequiresAquaSystemAppearance": False}, handle)
    # Ad-hoc signing allows local execution; distribution signing/notarization is separate.
    subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(app)], check=True)
    subprocess.run(["codesign", "--verify", "--deep", "--strict", str(app)], check=True)
    artifact = out / f"gosh-authenticator-{VERSION}-macos-{arch}.zip"
    subprocess.run(["ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", str(app), str(artifact)], check=True)
    return artifact


def package_linux(binary: Path, stage: Path, out: Path, arch: str) -> Path:
    name = f"gosh-authenticator-{VERSION}-linux-{arch}"
    directory = stage / name
    directory.mkdir()
    shutil.copy2(binary, directory / "gosh-authenticator")
    shutil.copytree(ROOT / "data", directory / "data")
    licenses(directory / "licenses")
    shutil.copy2(ROOT / "BUILDING.md", directory / "BUILDING.md")
    (directory / "INSTALL.txt").write_text(
        "Run ./gosh-authenticator. This dynamically linked Linux build requires "
        "WebKitGTK 4.1, GTK 3, libxdo and pcsc-lite; start the host pcscd service. "
        "Use Flatpak for an isolated runtime. See BUILDING.md for package names.\n")
    artifact = out / f"{name}.tar.gz"
    with tarfile.open(artifact, "w:gz") as archive:
        archive.add(directory, arcname=name)
    return artifact


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", choices=("linux", "windows", "macos"), required=True)
    parser.add_argument("--arch", choices=("x64", "arm64"), required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--out", type=Path, default=ROOT / "dist")
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    if not binary.is_file() or binary.stat().st_size == 0:
        parser.error("Expected a nonempty native binary")
    host = {"Linux": "linux", "Darwin": "macos", "Windows": "windows"}[platform.system()]
    if host != args.platform:
        parser.error("Run native packaging on the target OS")
    validate_binary(binary, args.platform, args.arch)
    args.out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="gosh package ") as temporary:
        artifact = globals()[f"package_{args.platform}"](binary, Path(temporary), args.out.resolve(), args.arch)
    if artifact.stat().st_size == 0:
        raise RuntimeError("The packaging tool produced an empty artifact")
    with artifact.open("rb") as handle:
        checksum = hashlib.file_digest(handle, "sha256").hexdigest()
    artifact.with_name(artifact.name + ".sha256").write_text(f"{checksum}  {artifact.name}\n")
    print(artifact)


if __name__ == "__main__":
    main()
