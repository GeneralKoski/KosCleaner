# Roadmap

KosCleaner is in **alpha**. The pieces below are deliberately deferred — listed so contributors and users know what's on the horizon and what's intentionally out of scope.

## Known limitations

### Windows trash
The XDG and macOS trash cleaners are implemented. Windows is not: `C:\$Recycle.Bin\<SID>\` is per-volume and SID-keyed, which needs the `windows` / `windows-sys` crate to query the user SID and enumerate volumes. The `trash_dir()` platform helper currently returns `None` on Windows, so the trash cleaner produces an empty report there rather than guessing.

### System-level package caches
PROMPT scope and current implementation only cover **user-level** package caches: pip, npm, Homebrew (per-user), yay. APT/DNF/Pacman caches under `/var/cache` need `sudo` and are intentionally out of scope until we design a policy for privilege elevation (likely a separate signed helper binary).

### Code signing / notarization
The release pipeline produces unsigned `.dmg`, `.msi`, and AppImage artifacts. Code signing requires paid certificates (Apple Developer ID, Microsoft Authenticode) and is not wired into CI yet.

### Per-item selection
The UI currently scans and then cleans the entire report. Cherry-picking individual items in the table is straightforward to add (the executor already takes an explicit `paths` list) but not implemented.

### Auto-updater
Disabled by design (no remote endpoints, in keeping with the zero-telemetry policy). Updates happen via the user re-downloading from GitHub Releases.

## Planned

- [ ] Per-item checkboxes in the scan report
- [ ] More browsers: Vivaldi, Edge, Opera, Safari (read-only — Safari uses `~/Library/Safari`)
- [ ] Thumbnail / preview cache cleaners (`~/.cache/thumbnails` on Linux, QuickLook on macOS)
- [ ] Recent-files / jumplist clearing
- [ ] Scheduled scans (opt-in only, no background daemon by default)
- [ ] Audit log viewer inside the app
- [ ] Audit log rotation
- [ ] Localization beyond IT/EN (community PRs welcome)

## Not planned

- Cloud sync, accounts, telemetry. Ever.
- Registry "cleaning" on Windows. It's a known anti-pattern.
- "Privacy" features that actually require sending data anywhere.
