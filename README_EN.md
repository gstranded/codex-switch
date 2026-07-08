# Codex Switch

<p>
  <strong>English</strong> |
  <a href="README.md">中文</a>
</p>

Codex Switch is a Codex provider switcher forked from CC Switch. Its main purpose is simple: keep your official Codex/ChatGPT login state and custom API providers in one place, switch between them with one click, and keep local Codex conversations visible after the switch.

## Latest Download

Download the latest build from [Releases / Latest](https://github.com/gstranded/codex-switch/releases/latest).

Current Windows x64 assets:

- `Codex-Switch-0.1.0-Windows-x64-Setup.exe`: standard installer, recommended for most users.
- `Codex-Switch-0.1.0-Windows-x64.msi`: MSI installer for users or deployment flows that prefer MSI.
- `Codex-Switch-0.1.0-Windows-x64-Portable.zip`: portable build. Extract and run `codex-switch.exe`.
- `SHA256SUMS.txt`: checksums for release assets.

The installer is not code-signed yet. Windows may show an unknown publisher or SmartScreen warning.

## Core Features

- **Official login state management**: save and restore the official Codex login state for OpenAI / ChatGPT account workflows.
- **API provider management**: add custom Base URLs, API keys, models, and provider names in one place.
- **One-click switching**: select a saved provider and apply it to the active Codex configuration.
- **Conversation history sync**: after switching to either an official login state or an API provider, Codex Switch syncs local Codex history buckets so older conversations remain visible.
- **Automatic backups**: creates backups before rewriting local history indexes.
- **Legacy data compatibility**: keeps the compatibility paths needed to reuse existing CC Switch data during this fork stage.

## Typical Workflow

1. Save an official Codex/ChatGPT login state in Codex Switch, or add your API provider.
2. For API providers, enter the Base URL, API key, model, and provider name.
3. Click the configuration you want to use and switch.
4. Open Codex. The active provider is changed, and existing local conversations should still appear under the current provider.

This lets you maintain official login, OpenRouter, DeepSeek, and other OpenAI-compatible API services side by side without manually editing config files or losing the history list after each switch.

## Why History Stays Visible

Codex stores local conversation history by `model_provider` bucket. Many switchers only update the active `config.toml`, so after switching to a new provider, Codex looks at a different bucket and older conversations appear to be gone.

After a provider switch succeeds, Codex Switch automatically:

1. Reads the active `model_provider` from the live Codex `config.toml`.
2. Finds known provider buckets from live config, saved provider configs, JSONL session metadata, SQLite thread rows, and built-in legacy provider ids.
3. Rewrites `session_meta.payload.model_provider` in Codex `.jsonl` session metadata to the active provider.
4. Rewrites `threads.model_provider` in Codex `state_5.sqlite` to the active provider.
5. Skips backup creation when there is nothing to synchronize.

Backups are stored under:

```text
~/.cc-switch/backups/codex-auto-history-sync-v1/<timestamp>/
```

The app intentionally keeps the legacy `~/.cc-switch` storage path for now to preserve compatibility with existing configuration and history data. A future `~/.codex-switch` migration should include compatibility migration logic.

## Notes

- The goal is to keep old conversations visible in the Codex history list after provider switching.
- Resuming very old conversations across providers may still be affected by Codex internal data, provider-specific fields, or encrypted content.
- The current release focuses on Windows x64.
- The installer is unsigned, so Windows may ask you to allow it on first run.

## Credits

Codex Switch is developed as a fork of CC Switch. Thanks to the original CC Switch project and contributors for the foundation and inspiration.

This project keeps the compatibility logic that matters while continuing to iterate on official login state switching, API provider switching, and Codex history synchronization.
