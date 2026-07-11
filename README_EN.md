# Codex Switch

<p>
  <strong>English</strong> |
  <a href="README.md">中文</a>
</p>

Codex Switch is a Codex provider switcher forked from CC Switch. Its main purpose is simple: keep your official Codex/ChatGPT login state and custom API providers in one place, switch between them with one click, and keep local Codex conversations visible after the switch.

## Latest Download

Download the latest build from [Releases / Latest](https://github.com/gstranded/codex-switch/releases/latest).

Current latest assets:

- `Codex-Switch-0.3.0-Windows-x64-Setup.exe`: standard Windows installer, recommended for most Windows users.
- `Codex-Switch-0.3.0-Windows-x64.msi`: Windows MSI installer for users or deployment flows that prefer MSI.
- `Codex-Switch-0.3.0-Windows-x64-Portable.zip`: Windows portable build. Extract and run `codex-switch.exe`.
- `Codex-Switch-0.3.0-macOS-universal.dmg`: macOS installer for both Apple Silicon and Intel Macs.
- `Codex-Switch-0.3.0-macOS-universal.zip`: zipped macOS `.app` bundle.
- `Codex-Switch-0.3.0-Linux-x64.AppImage`: portable Linux build. Mark it executable and run it.
- `Codex-Switch-0.3.0-Linux-x64.deb`: Debian / Ubuntu package.
- `Codex-Switch-0.3.0-Linux-x64.rpm`: Fedora / RHEL / openSUSE package.
- `SHA256SUMS.txt`: checksums for release assets.

This preview build is not code-signed or Apple-notarized yet. Windows may show an unknown publisher or SmartScreen warning; macOS may require right-clicking the app and choosing Open.

## Core Features

- **Official login state management**: save and restore the official Codex login state for OpenAI / ChatGPT account workflows.
- **API provider management**: add custom Base URLs, API keys, models, and provider names in one place.
- **One-click switching**: select a saved provider and apply it to the active Codex configuration.
- **Conversation history sync**: after switching to either an official login state or an API provider, Codex Switch syncs local Codex history buckets so older conversations remain visible.
- **Restart confirmation**: after a Codex switch, choose whether to restart Codex Desktop immediately so the new configuration takes effect.
- **Portable chat archives**: export and import Codex conversations between computers; imported sessions merge safely and are synchronized to the current provider.
- **Automatic backups**: creates backups before rewriting local history indexes.
- **Legacy data compatibility**: keeps the compatibility paths needed to reuse existing CC Switch data during this fork stage.

## Typical Workflow

1. Save an official Codex/ChatGPT login state in Codex Switch, or add your API provider.
2. For API providers, enter the Base URL, API key, model, and provider name.
3. Click the configuration you want to use and switch.
4. Choose whether to restart Codex now. A restart applies the selected configuration immediately; choosing not to restart leaves Codex running unchanged until you reopen it.
5. Open Codex. The active provider is changed, and existing local conversations should still appear under the current provider.

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

## Move Chat History Between Computers

In **Settings -> Data -> Codex Chat History**, export a `.zip` archive on the source computer and import it on the target computer. The archive contains Codex session JSONL files, session titles, and thread indexes only. It never includes API keys, provider settings, or login credentials.

On import, existing session IDs are kept, duplicate sessions are skipped, and newly imported sessions are immediately synchronized to the active provider. Later provider switches keep using the same history-sync path.

## Notes

- The goal is to keep old conversations visible in the Codex history list after provider switching.
- Resuming very old conversations across providers may still be affected by Codex internal data, provider-specific fields, or encrypted content.
- The current release provides Windows x64, macOS universal, and Linux x64 builds.
- Installers are unsigned, so the OS may ask you to allow the app on first run.
- Some Linux distributions may require FUSE for AppImage. If AppImage does not fit your system, use the `.deb` or `.rpm` package.

## Credits

Codex Switch is developed as a fork of CC Switch. Thanks to the original CC Switch project and contributors for the foundation and inspiration.

This project keeps the compatibility logic that matters while continuing to iterate on official login state switching, API provider switching, and Codex history synchronization.
