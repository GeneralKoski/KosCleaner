# KosCleaner

[![CI](https://github.com/GeneralKoski/KosCleaner/actions/workflows/ci.yml/badge.svg)](https://github.com/GeneralKoski/KosCleaner/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL_3.0-blue.svg)](LICENSE)

A safe, transparent, open-source system cleaner for Linux, Windows, and macOS — a local alternative to CCleaner with no telemetry, no cloud, and no data sent to third parties.

> **Status:** alpha — under active development. Not yet ready for general use.

## Philosophy

1. **Safety first.** Every destructive operation runs as a dry-run by default.
2. **Transparency.** You see exactly which files will be deleted *before* confirming.
3. **Zero telemetry.** No network calls unless explicitly requested by the user.
4. **Whitelist, never blacklist.** Only paths from a known list are ever cleaned.
5. **No root/admin** unless strictly required, and only with explicit confirmation.
6. **Local audit log** of every operation performed.

## Available cleaners

| Category | Cleaner | Linux | macOS | Windows |
|----------|---------|:-----:|:-----:|:-------:|
| System   | Temporary files | ✓ | ✓ | ✓ |
| Browsers | Firefox cache | ✓ | ✓ | ✓ |
| Browsers | Chrome cache | ✓ | ✓ | ✓ |
| Browsers | Chromium cache | ✓ | ✓ | ✓ |
| Browsers | Brave cache | ✓ | ✓ | ✓ |
| Packages | pip cache | ✓ | ✓ | ✓ |
| Packages | npm cache | ✓ | ✓ | ✓ |
| Packages | Homebrew cache | — | ✓ | — |
| Packages | yay cache (AUR) | ✓ | — | — |
| Trash    | Empty trash | ✓ | ✓ | — *(planned)* |

All cleaners are user-level: no `sudo` / admin elevation is requested.

## Tech stack

- **Backend:** Rust (edition 2021), zero runtime dependencies beyond Tauri
- **Desktop framework:** Tauri 2.0
- **Frontend:** SvelteKit (Svelte 5 runes) + TypeScript + Tailwind v4
- **Testing:** `cargo test` (45+ unit tests) + Vitest
- **License:** GPL-3.0

## Features

- Dual theme (light / dark) with system-preference detection
- Bilingual UI: English and Italian (auto-detected from system locale)
- JSONL audit log of every execution, stored locally:
  - Linux: `~/.local/share/koscleaner/audit.log`
  - macOS: `~/Library/Logs/KosCleaner/audit.log`
  - Windows: `%LOCALAPPDATA%\KosCleaner\audit.log`
- Mandatory confirmation modal before any deletion
- Deletes the symlink, never its target

## Platforms

- Linux (primary development target) — AppImage and `.deb`
- macOS — universal `.dmg` (Apple Silicon + Intel)
- Windows — `.msi`

## Building from source

Prerequisites: Rust (stable), Node.js 20+, and the [Tauri 2.0 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```bash
git clone https://github.com/GeneralKoski/KosCleaner.git
cd KosCleaner
npm install
npm run tauri dev
```

To produce release artifacts locally:

```bash
npm run tauri build
```

The Vite dev server runs on port **5174** (Tauri loads it from there).

## Project structure

```
src-tauri/src/
├── safety.rs          Path validation: blocklist + symlink-escape detection
├── platform/          OS-specific path resolution (XDG / Apple / Known Folders)
├── cleaners/          Per-target cleaners (system, browser, chromium, packages, trash)
├── scanner.rs         Read-only walk that produces a ScanReport
├── executor.rs        The only place that calls fs::remove_file
└── audit.rs           Append-only JSONL audit log
```

Every path that reaches `executor.rs` has been re-validated through `safety.rs` — the IPC boundary between frontend and backend is treated as untrusted.

## Inspirations

- [BleachBit](https://github.com/bleachbit/bleachbit) — main reference for cleaner definitions (CleanerML).
- [Stacer](https://github.com/oguzhaninan/Stacer) — UX/UI inspiration.
- Sweeper (KDE) — simplicity.

KosCleaner is written from scratch in Rust. CleanerML definitions, when reused, are credited to the original BleachBit project (both projects are GPL-3.0).

## Roadmap

See [ROADMAP.md](ROADMAP.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

GPL-3.0. See [LICENSE](LICENSE).
