from __future__ import annotations

import pathlib

ROOT = pathlib.Path(__file__).parents[1]
MANIFEST = ROOT / "com.goshapps.YubicoAuthenticator.yml"


def test_manifest_uses_the_kde_runtime() -> None:
    text = MANIFEST.read_text()
    assert "runtime: org.kde.Platform" in text
    assert "sdk: org.kde.Sdk" in text
    assert "--socket=pcsc" in text
    # The sandbox has no network permission on purpose.
    assert "--share=network" not in text


def test_manifest_installs_license_material() -> None:
    text = MANIFEST.read_text()
    top_level_cleanup = text.split("cleanup:", 1)[1].split("modules:", 1)[0]
    assert "/share/licenses" not in top_level_cleanup
    assert "install -Dm644 ../COPYING /app/share/licenses/com.goshapps.YubicoAuthenticator/pcsc-lite-COPYING" in text
    assert "install -Dm644 LICENSE /app/share/licenses/com.goshapps.YubicoAuthenticator/zxing-cpp-LICENSE" in text
    assert "install -Dm644 LICENSE /app/share/licenses/com.goshapps.YubicoAuthenticator/LICENSE" in text


def test_first_party_gpl_license_is_complete() -> None:
    license_text = (ROOT / "LICENSE").read_text()
    assert len(license_text.splitlines()) > 600
    assert "END OF TERMS AND CONDITIONS" in license_text


def test_product_trademark_notice_is_present() -> None:
    readme = (ROOT / "README.md").read_text()
    assert "not affiliated" in readme.lower()
    assert "Yubico" in readme
    assert "YubiKey" in readme


def test_no_gtk_or_rust_leftovers() -> None:
    assert not (ROOT / "rust").exists()
    assert not (ROOT / "flatpak" / "cargo-sources.json").exists()
    for path in [
        ROOT / "README.md",
        ROOT / "RELEASE.md",
        ROOT / "RPM-BUILD.md",
        ROOT / "gosh-authenticator.spec",
        ROOT / "build-rpm.sh",
        ROOT / "build-deb.sh",
        ROOT / "build-flatpak.sh",
        MANIFEST,
        ROOT / "data" / "metainfo" / "com.goshapps.YubicoAuthenticator.metainfo.xml",
        ROOT / "data" / "applications" / "com.goshapps.YubicoAuthenticator.desktop",
    ]:
        text = path.read_text().lower()
        assert "gtk" not in text, path
        assert "adwaita" not in text, path
        assert "cargo" not in text, path


def test_cmake_project_builds_the_qt_app() -> None:
    cmake = (ROOT / "CMakeLists.txt").read_text()
    assert "find_package(Qt6" in cmake
    assert "qt_add_qml_module" in cmake
    assert "libpcsclite" in cmake
