# Changelog

All notable changes to KosCleaner are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial alpha release.
- Cross-platform path safety module (`safety.rs`) with OS-specific blocklists for Linux, macOS, and Windows.
- Symlink-escape detection (`ensure_within_root`) so cleaners can't be redirected outside their declared roots.
- Cross-platform path resolution (`platform/`) following XDG Base Directory on Linux, Apple File System Programming Guide on macOS, and Known Folders on Windows.
- Read-only scanner with per-cleaner reports (items, total size, errors).
- Executor with mandatory confirmation, IPC re-validation, and audit log.
- JSONL audit log written to platform-appropriate locations.
- Cleaners:
  - System temporary files
  - Firefox cache (all profiles)
  - Chrome / Chromium / Brave caches (all profiles, including Service Worker)
  - pip, npm, Homebrew, yay package caches
  - Trash (Linux XDG + macOS)
- SvelteKit + Tauri 2.0 desktop UI with sidebar navigation, scan/confirm/execute flow.
- Light/dark theme with system-preference detection and persistence.
- Bilingual UI (English / Italian) with locale auto-detection.
- GitHub Actions:
  - CI: cargo fmt + clippy + tests + frontend type-check + tests on Linux, macOS, Windows.
  - Release: tag-triggered cross-platform build (AppImage/.deb on Linux, universal .dmg on macOS, .msi on Windows) via tauri-action.

### Known limitations

See [ROADMAP.md](ROADMAP.md). In summary: Windows trash, system-level package caches, code signing, per-item selection, and auto-updater are intentionally not yet implemented.
