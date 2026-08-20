/*
 * GitHub CodeQL default setup still includes the c-cpp language from when
 * this repository shipped a C FFI layer for Flutter. The app is now a
 * native Rust / GTK 4 binary and has no C sources for the extractor to
 * index, which makes Analyze (c-cpp) fail with:
 *
 *   Extraction failed: No source files found.
 *
 * This no-op translation unit is only here so that job can extract. Remove
 * it after unchecking C/C++ in Settings → Advanced Security → CodeQL.
 */
int gosh_authenticator_codeql_placeholder(void) { return 0; }
