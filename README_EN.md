# Codex Switch

<p>
  <strong>English</strong> |
  <a href="README.md">中文</a>
</p>

Codex Switch is a Codex provider switcher focused on provider switching plus automatic local history bucket synchronization.

## Downloads

Download builds from [Releases](https://github.com/gstranded/codex-switch/releases).

Current Windows x64 assets:

- `Codex-Switch-0.1.0-Windows-x64-Setup.exe`: standard installer, recommended for most users.
- `Codex-Switch-0.1.0-Windows-x64.msi`: MSI installer for users or deployment flows that prefer MSI.
- `Codex-Switch-0.1.0-Windows-x64-Portable.zip`: portable build. Extract and run `codex-switch.exe`.
- `SHA256SUMS.txt`: checksums for release assets.

This is a preview build and is not code-signed yet. Windows may show an unknown publisher or SmartScreen warning.

## What Changed

- Renamed the app/fork to Codex Switch.
- Supports provider/config management and one-click switching.
- Added automatic Codex history synchronization after Codex provider switching.
- Rewrites local Codex `.jsonl` session metadata and `state_5.sqlite` thread provider buckets to the active provider.
- Creates backups before rewriting history data.

## History Sync Behavior

After a Codex switch succeeds, Codex Switch:

1. Reads the active `model_provider` from the live Codex `config.toml`.
2. Finds known provider buckets from live config, saved provider configs, JSONL session metadata, SQLite thread rows, and built-in legacy provider ids.
3. Rewrites matching local history bucket ids to the active provider id.
4. Skips backup creation when there is nothing to change.

Backups are stored under:

```text
~/.cc-switch/backups/codex-auto-history-sync-v1/<timestamp>/
```

The app intentionally keeps the legacy `~/.cc-switch` storage path for now, so existing provider data can still be reused during this fork stage.

## Limitation

This makes old conversations visible under the active provider bucket. It does not guarantee every old conversation can be resumed successfully across providers, because Codex may store provider-specific or encrypted content in session data.

## Local Development

```powershell
pnpm install
pnpm typecheck
pnpm tauri dev
```

For full backend checks on Windows, install Visual Studio Build Tools with the C++ workload and Windows SDK, then run:

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
```

## Verification Status

- GitHub Actions CI passed frontend typecheck, formatting, unit tests, backend `cargo fmt`, `cargo clippy`, and `cargo test`.
- The Windows x64 release workflow successfully built and uploaded release assets.
- Full local Windows backend build/check was not completed on this machine because the Windows C++/SDK linker environment is missing.

## Project Note

Codex Switch keeps the legacy data-compatibility paths needed to avoid losing existing provider configuration and session history during migration.
