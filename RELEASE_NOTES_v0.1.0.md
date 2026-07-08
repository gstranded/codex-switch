# Codex Switch v0.1.0 preview

这是 Codex provider 切换预览版，重点是切换 Codex 供应商后自动同步本地聊天记录 bucket。

## 下载

Windows x64 提供以下文件：

- `Codex-Switch-0.1.0-Windows-x64-Setup.exe`：常规安装包，推荐大多数用户使用。
- `Codex-Switch-0.1.0-Windows-x64.msi`：MSI 安装包。
- `Codex-Switch-0.1.0-Windows-x64-Portable.zip`：便携版，解压后运行 `codex-switch.exe`。
- `SHA256SUMS.txt`：文件校验。

当前安装包尚未代码签名，Windows 可能提示“未知发布者”或 SmartScreen。

## 新增

- Codex provider 切换成功后，自动同步 Codex 本地会话历史。
- 将 JSONL 中的 `session_meta.payload.model_provider` 改写为当前激活 provider。
- 将 Codex `state_5.sqlite` 中的 `threads.model_provider` 改写为当前激活 provider。
- 改写前自动备份到 `~/.cc-switch/backups/codex-auto-history-sync-v1/<timestamp>/`。
- 新增 Rust 测试，覆盖从 `openai`、`deepseek`、`custom` bucket 同步到激活的 `openrouter` provider。

## 验证

- GitHub Actions CI 已通过 frontend typecheck、format、unit tests。
- GitHub Actions CI 已通过 backend `cargo fmt`、`cargo clippy`、`cargo test`。
- Windows x64 release workflow 已成功构建并上传安装包。
- 本机没有完整 Windows C++ Build Tools / Windows SDK 链接环境，因此没有在本机完成 Windows 后端构建。

## 注意

历史同步会让旧会话在当前 provider bucket 下可见，但跨 provider 恢复非常旧的会话仍可能受 Codex 会话内部数据影响。
