# KosCleaner

A safe, transparent, open-source system cleaner for Linux, Windows, and macOS — a local alternative to CCleaner with no telemetry, no cloud, and no data sent to third parties.

> **Status:** alpha — under active development. Not yet ready for general use.

## Philosophy

1. **Safety first.** Every destructive operation runs as a dry-run by default.
2. **Transparency.** You see exactly which files will be deleted *before* confirming.
3. **Zero telemetry.** No network calls unless explicitly requested by the user.
4. **Whitelist, never blacklist.** Only paths from a known list are ever cleaned.
5. **No root/admin** unless strictly required, and only with explicit confirmation.
6. **Local audit log** of every operation performed.

## Tech stack

- **Backend:** Rust (edition 2021)
- **Desktop framework:** Tauri 2.0
- **Frontend:** SvelteKit + TypeScript + TailwindCSS
- **Testing:** `cargo test` + Vitest
- **License:** GPL-3.0

## Platforms

- Linux (primary development target)
- Windows
- macOS

## Building from source

Prerequisites: Rust (stable), Node.js 20+, and the [Tauri 2.0 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```bash
git clone https://github.com/<your-org>/koscleaner.git
cd koscleaner
npm install
cargo tauri dev
```

## Inspirations

- [BleachBit](https://github.com/bleachbit/bleachbit) — main reference for cleaner definitions (CleanerML).
- [Stacer](https://github.com/oguzhaninan/Stacer) — UX/UI inspiration.
- Sweeper (KDE) — simplicity.

KosCleaner is written from scratch in Rust. CleanerML definitions, when reused, are credited to the original BleachBit project (both projects are GPL-3.0).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

GPL-3.0. See [LICENSE](LICENSE).
