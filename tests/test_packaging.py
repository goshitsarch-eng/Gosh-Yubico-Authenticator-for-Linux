from __future__ import annotations

import pathlib

ROOT = pathlib.Path(__file__).parents[1]
MANIFEST = ROOT / "com.goshapps.YubicoAuthenticator.yml"


def test_manifest_installs_first_and_third_party_license_material() -> None:
    text = MANIFEST.read_text()
    top_level_cleanup = text.split("cleanup:", 1)[1].split("build-options:", 1)[0]
    assert "/share/licenses" not in top_level_cleanup
    assert "install -Dm644 ../COPYING /app/share/licenses/com.goshapps.YubicoAuthenticator/pcsc-lite-COPYING" in text
    assert "install -Dm644 THIRD_PARTY_LICENSES.html /app/share/licenses/com.goshapps.YubicoAuthenticator/THIRD_PARTY_LICENSES.html" in text
    assert "path: THIRD_PARTY_LICENSES.html" in text


def test_manifest_uses_a_pinned_rust_toolchain_archive() -> None:
    text = MANIFEST.read_text()
    assert "sdk-extensions:" not in text
    assert "rust-1.98.0-x86_64-unknown-linux-gnu.tar.xz" in text
    assert "ed8ee2df70909c88cbaf87a6cfa3920dac00b537de12a6abe6906641e0f5952f" in text
    assert "./toolchain/install.sh --prefix=/run/build/gosh-authenticator/rust-toolchain" in text
    assert "append-path: /run/build/gosh-authenticator/rust-toolchain/bin" in text


def test_first_party_gpl_and_generated_rust_notices_are_complete() -> None:
    license_text = (ROOT / "LICENSE").read_text()
    notices = (ROOT / "THIRD_PARTY_LICENSES.html").read_text()
    assert len(license_text.splitlines()) > 600
    assert "END OF TERMS AND CONDITIONS" in license_text
    assert "serde" in notices.lower()
    assert "tokio" in notices.lower()
    assert "MIT License" in notices or "MIT" in notices
    assert "Apache License" in notices or "Apache-2.0" in notices


def test_product_trademark_notice_is_present() -> None:
    readme = (ROOT / "README.md").read_text()
    assert "not affiliated" in readme.lower()
    assert "Yubico" in readme
    assert "YubiKey" in readme


def test_rust_notice_has_a_reproducible_generator() -> None:
    script = (ROOT / "scripts/generate-third-party-licenses.sh").read_text()
    assert "cargo-about --locked --features cli" in script
    assert "cargo about generate -o ../THIRD_PARTY_LICENSES.html about.hbs" in script
    assert 'replace(b"\\r\\n", b"\\n")' in script
