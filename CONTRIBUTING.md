# Contributing to KosCleaner

Thanks for your interest in contributing! KosCleaner is in early alpha — the codebase, design, and roadmap are still moving fast.

## Ground rules

- **Safety is non-negotiable.** Any code that touches the filesystem MUST go through `safety.rs`. PRs that bypass it will be rejected.
- **No telemetry.** No network calls unless explicitly requested by the user.
- **Whitelist over blacklist.** Cleaners only act on paths from a known, reviewed list.

## Getting started

1. Fork the repository and clone your fork.
2. Install prerequisites:
   - Rust (stable, edition 2021)
   - Node.js 20+
   - [Tauri 2.0 system dependencies](https://v2.tauri.app/start/prerequisites/) for your OS
3. Install JS deps and run the app in dev mode:
   ```bash
   npm install
   cargo tauri dev
   ```

## Running tests

```bash
# Rust unit/integration tests
cargo test --manifest-path src-tauri/Cargo.toml

# Frontend tests
npm test
```

## Pull request workflow

1. Create a feature branch from `main` (e.g. `feat/firefox-cleaner`).
2. Keep PRs focused — one feature or fix per PR.
3. Follow [Conventional Commits](https://www.conventionalcommits.org/) for commit messages: `feat:`, `fix:`, `chore:`, `docs:`, `test:`, `ci:`, `refactor:`.
4. Make sure `cargo test` and `cargo clippy` pass before opening a PR.
5. Describe in the PR what changed, why, and how you tested it.

## Coding style

- **Rust:** `cargo fmt` and `cargo clippy -- -D warnings`. No `unsafe` unless strictly necessary, with a comment justifying it.
- **TypeScript/Svelte:** project-configured Prettier + ESLint.
- Code, identifiers, and commit messages in English. Documentation may be bilingual (EN/IT).

## Adding a new cleaner

A cleaner must:

- Live under `src-tauri/src/cleaners/` as its own module.
- Use `PlatformPaths` to resolve paths — never hardcode them.
- Validate every path through `safety::is_path_safe_to_delete` before any deletion.
- Have a dry-run scanner that returns a file list and total size *without* mutating anything.
- Be covered by unit tests with fixture directories.

## Reporting bugs and security issues

- **Bugs / feature requests:** open a GitHub issue.
- **Security issues:** please do *not* open a public issue. Email the maintainers privately first.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0.
