# Release procedure

Version 2.0 remains a prerelease until QA.md, PLATFORM_SUPPORT.md and the migration
ledger have concrete native/hardware evidence. Do not publish a stable release
based only on compilation or fixtures.

1. Update the Cargo version/lockfile, AppStream, changelog and documentation.
2. Regenerate Flatpak sources and third-party licenses. Run all local checks.
3. Push a review branch. Inspect each native/Flatpak CI job and its actual artifacts.
4. On native machines, test MSI install/upgrade/uninstall, both macOS bundles,
   Linux packages and sandboxed Flatpak. Record real key and every-control QA.
5. Add optional Windows signing and macOS Developer ID signing/notarization for
   trusted public distribution; validate the signed outputs separately.
6. Tag the reviewed commit with exactly `v<Cargo version>`. The release workflow
   reruns CI, requires every advertised artifact and produces SHA256SUMS.
7. Inspect the resulting draft release, verify checksums and the contained license
   material, then publish after the outstanding QA gates are closed.

The workflow distinguishes prerelease tags and never silently omits failed
architectures. `workflow_dispatch` validates without publishing an untagged build.
No release tag or public release is created automatically during development.
