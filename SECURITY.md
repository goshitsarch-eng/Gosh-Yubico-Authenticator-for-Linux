# Security and dependency review

Report a suspected issue through the repository's private vulnerability reporting
facility when available. Do not include real OTP secrets, passwords, codes or
personal account labels in public issues or logs.

The app stores OATH secrets on the key, not in a local credential database. It
uses the OS PC/SC stack, Yubico's 16-byte PBKDF2-HMAC-SHA1 authentication key,
secure random challenges and constant-time proof verification. Rust secret/key
buffers use best-effort zeroization. WebView/process memory cannot be guaranteed
to be wiped; masked form values are transient and never deliberately written to
disk. Clipboard clearing compares current contents with the last copied code.
Clipboard timing has its own thread so PC/SC and network waits do not delay it.

Settings are bounded, validated and atomically replaced; malformed existing
files are preserved. QR files are bounded by file size, dimensions, total pixels
and decoding allocation. APDU continuation and TLV parsing fail on malformed or
oversized responses. Duplicate credentials require explicit deletion before
replacement. Favicon requests are disabled by default and use only an explicitly
entered, validated DNS hostname, HTTPS and bounded image decoding/cache paths.
The UI uses embedded content and loads no remote application page.

## Audit observations

`cargo audit --file rust/Cargo.lock` found RUSTSEC-2026-0285 in Rustls 0.23.43.
The lockfile now uses patched 0.23.45. rqrr was updated to 0.11.0, which removes
lru 0.12.5 and its two reported unsoundness warnings. QR regression tests decode
actual generated images after this update.

Four warnings remain, without ignored advisory IDs or suppressed audit output:

| Dependency | Advisory | Context / remaining action |
|---|---|---|
| glib 0.18.5 | RUSTSEC-2024-0429, VariantStrIter unsoundness | Linux GTK 3 bindings required by stable Dioxus/Wry; this app does not directly use VariantStrIter. Track upstream migration and review transitive reachability before a stable release. |
| rand 0.7.3 | RUSTSEC-2026-0097, custom logger unsoundness | phf_generator build dependency through Wry's Windows HTML parser; application randomness uses getrandom. Upstream dependency update remains needed. |
| fxhash 0.2.1 | RUSTSEC-2025-0057, unmaintained | Transitive dependency; track supported upstream replacement. |
| proc-macro-error 1.0.4 | RUSTSEC-2024-0370, unmaintained | Transitive macro dependency; track supported upstream replacement. |

Successful audit exit status does not mean these warnings are resolved. Native
WebView and smart-card libraries are maintained by each OS/runtime. No new
first-party unsafe Rust was added. TLS verification and the cloud session proxy
remain enabled throughout dependency acquisition.
