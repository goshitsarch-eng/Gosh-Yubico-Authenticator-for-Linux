# Contributing

Use [BUILDING.md](BUILDING.md) to prepare a native development environment. Rust
1.98.0 and Cargo.lock are pinned. Run formatting, Clippy, core/full tests,
packaging metadata checks and the appropriate desktop smoke before submitting.
Update generated Flatpak sources and third-party notices after dependency edits.

Keep OATH parsing/crypto in `core`, worker orchestration in `services`, explicit
commands/events/state in `app`, persistence in `settings`, and native OS APIs in
`platform`. Dioxus components render state and dispatch commands. Avoid blocking
I/O, detached uncontrolled work, secret logging or silent filesystem errors in
UI callbacks. Preserve legacy settings fields and unknown extensions.

Use a separate branch and a reviewable PR. Include the concrete behavior change,
validation results and remaining platform/hardware limits. Never count fixtures
as real-device QA or a Linux build as Windows/macOS verification. Update
MIGRATION_AUDIT.md, QA.md and PLATFORM_SUPPORT.md when closing a migration gate.
Release tags must match Cargo.toml; the workflow requires the complete artifact
set and creates a draft release for review.
