# Codex Switch

Codex Switch is a CC Switch fork focused on Codex provider switching plus automatic local history bucket synchronization.

This project keeps the CC Switch provider-management base, then adds one important behavior for Codex: after a Codex provider switch succeeds, local Codex history rows/files are synchronized to the newly active `model_provider`, so conversations stay visible after switching providers.

## What Changed

- Renamed the app/fork to Codex Switch.
- Kept the existing CC Switch provider/config management flow.
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

The app intentionally keeps the existing `~/.cc-switch` storage path for now, so existing CC Switch provider data can still be reused during this fork stage.

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

- `cargo fmt --all -- --check` passed during the fork session with Rust 1.95.
- Frontend TypeScript check passed via `node_modules/.bin/tsc.cmd --noEmit` during the fork session.
- Full local backend build/check was not completed on this machine because the Windows C++/SDK linker environment is missing.

## Upstream

This is a fork of CC Switch:

https://github.com/farion1231/cc-switch

CC Switch remains the upstream base for the general multi-tool provider manager.
