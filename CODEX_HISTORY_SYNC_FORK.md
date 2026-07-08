# Codex Switch History Sync Fork

Codex Switch adds automatic Codex history synchronization after switching Codex providers.

## Behavior

- Codex providers can still be added and switched through the normal provider flow.
- After a Codex provider switch succeeds, the app reads the active `model_provider` from the live Codex `config.toml`.
- It collects known local Codex history provider buckets from:
  - live `config.toml`
  - saved Codex provider configs
  - Codex `.jsonl` session metadata
  - Codex `state_5.sqlite` thread rows
  - built-in and legacy Codex provider ids such as `openai`, `custom`, `deepseek`, `openrouter`, `rightcode`, etc.
- It rewrites matching local history bucket ids to the newly active provider id, so the Codex history list remains visible after switching providers.

## Backups

Before rewriting session files or state DB rows, the fork creates backups under:

```text
~/.cc-switch/backups/codex-auto-history-sync-v1/<timestamp>/
```

The backup generation includes a `meta.json` file recording the Codex config directory it came from.

## Important limitation

This sync makes historical conversations visible under the active provider bucket. It does not guarantee that every old conversation can be resumed successfully across providers, because Codex may store provider-specific or encrypted content in the session data.

## Main changed files

- `src-tauri/src/codex_history_migration.rs`
- `src-tauri/src/services/provider/mod.rs`

## Verification notes

- Verified during this fork session:
  - `cargo fmt --all -- --check` passed with Rust 1.95.
  - Frontend TypeScript check passed via `node_modules/.bin/tsc.cmd --noEmit`.
- GitHub Actions CI passed frontend typecheck, formatting, unit tests, backend `cargo fmt`, `cargo clippy`, and `cargo test`.
- Full local Windows `cargo check` could not complete on this machine because the Windows C++/SDK linker environment is missing:
  - MSVC path fails at missing `link.exe` / Windows SDK libraries.
  - GNU self-contained fallback reaches Rust dependency compilation but fails in the temporary `dlltool`/assembler path.
- To run the full backend check locally, install Visual Studio Build Tools with the C++ workload and Windows SDK, then run:

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
```
