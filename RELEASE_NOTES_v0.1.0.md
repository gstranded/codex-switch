# Codex Switch v0.1.0

Codex Switch 是基于 CC Switch 二次开发的 Codex 供应商切换工具。本版本重点解决一个核心问题：在官方 Codex/ChatGPT 登录态和自定义 API Provider 之间切换后，本地聊天记录仍然能继续显示，不会因为 provider bucket 改变而从历史列表里“消失”。

## 下载

Windows x64 提供以下文件：

- `Codex-Switch-0.1.0-Windows-x64-Setup.exe`：常规安装包，推荐大多数用户使用。
- `Codex-Switch-0.1.0-Windows-x64.msi`：MSI 安装包。
- `Codex-Switch-0.1.0-Windows-x64-Portable.zip`：便携版，解压后运行 `codex-switch.exe`。
- `SHA256SUMS.txt`：文件校验。

当前安装包尚未代码签名，Windows 可能提示“未知发布者”或 SmartScreen。

## 主要功能

- 在一个地方维护官方 Codex/ChatGPT 登录态和多个 API Provider。
- 支持保存 Base URL、API Key、模型和 provider 配置。
- 支持一键切换到官方登录态或任意 API Provider。
- 切换成功后自动同步 Codex 本地聊天记录 bucket。
- 同步 JSONL 会话元数据中的 `session_meta.payload.model_provider`。
- 同步 Codex `state_5.sqlite` 中的 `threads.model_provider`。
- 改写历史索引前自动备份到 `~/.cc-switch/backups/codex-auto-history-sync-v1/<timestamp>/`。

## 使用场景

你可以在 Codex Switch 里同时维护官方账号、OpenRouter、DeepSeek 或其他兼容 OpenAI API 的服务。需要切换时直接点击对应配置，Codex Switch 会写入当前 Codex 配置，并同步本地历史记录到当前 provider 下。

## 注意

- 这个功能的目标是让旧会话在切换供应商后重新出现在 Codex 历史列表中。
- 跨 provider 恢复非常旧的会话仍可能受 Codex 内部数据、provider 特定字段或加密内容影响。
- 现阶段保留 `~/.cc-switch` 等 legacy 路径，用于兼容旧配置和历史数据。

## 致谢

Codex Switch 基于 CC Switch 二次开发。感谢 CC Switch 原项目和贡献者提供的基础能力与灵感。
