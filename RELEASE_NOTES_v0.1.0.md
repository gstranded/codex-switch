# Codex Switch v0.1.0 preview

Fork of CC Switch focused on Codex provider switching with automatic local history bucket sync.

## Added

- Auto-sync Codex session history after switching Codex providers.
- Rewrite local JSONL `session_meta.payload.model_provider` values to the active provider.
- Rewrite Codex `state_5.sqlite` `threads.model_provider` rows to the active provider.
- Create backups before rewriting under `~/.cc-switch/backups/codex-auto-history-sync-v1/<timestamp>/`.
- Added a Rust test covering sync from `openai`, `deepseek`, and `custom` buckets into an active `openrouter` provider.

## Verification

- `cargo fmt --all -- --check` passed during the fork session.
- Frontend TypeScript check passed during the fork session.
- GitHub Actions CI passed frontend typecheck, formatting, unit tests, backend `cargo fmt`, `cargo clippy`, and `cargo test`.
- Full local Windows backend `cargo check` / build was not completed on this machine because the Windows C++ Build Tools / Windows SDK linker environment is missing.

## Caveat

This is a preview source release. No binary installer is attached. The history sync makes conversations visible under the active provider bucket, but resuming very old provider-specific conversations may still depend on Codex session internals.
