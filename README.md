# Codex Switch

<p>
  <a href="README_EN.md">English</a> |
  <strong>中文</strong>
</p>

Codex Switch 是一个基于 CC Switch 二次开发的 Codex 供应商切换工具。它的核心目标很直接：在一个地方维护你的官方 Codex/ChatGPT 登录态和自定义 API Provider，然后一键切换，并尽量让本地聊天记录在切换后继续可见，不再因为 provider 改了就“掉历史”。

## 下载最新版本

请到 [Releases / Latest](https://github.com/gstranded/codex-switch/releases/latest) 下载最新版。

当前最新版提供：

- `Codex-Switch-0.4.1-Windows-x64-Setup.exe`：Windows 常规安装包，推荐大多数 Windows 用户使用。
- `Codex-Switch-0.4.1-Windows-x64.msi`：Windows MSI 安装包，适合偏好 MSI 或企业部署的场景。
- `Codex-Switch-0.4.1-Windows-x64-Portable.zip`：Windows 便携版，解压后运行 `codex-switch.exe`。
- `Codex-Switch-0.4.1-macOS-universal.dmg`：macOS 安装包，同时支持 Apple Silicon 和 Intel Mac。
- `Codex-Switch-0.4.1-macOS-universal.zip`：macOS `.app` 压缩包，解压后运行。
- `Codex-Switch-0.4.1-Linux-x64.AppImage`：Linux 便携版，赋予执行权限后运行。
- `Codex-Switch-0.4.1-Linux-x64.deb`：Debian / Ubuntu 安装包。
- `Codex-Switch-0.4.1-Linux-x64.rpm`：Fedora / RHEL / openSUSE 安装包。
- `SHA256SUMS.txt`：校验文件完整性。

当前预览版本尚未代码签名或 Apple 公证。Windows 可能提示“未知发布者”或 SmartScreen；macOS 可能需要右键点击应用并选择打开。

## 核心功能

- **官方登录态管理**：保存和恢复 Codex 官方登录状态，适合使用 OpenAI / ChatGPT 官方账号的场景。
- **API Provider 管理**：添加自定义 Base URL、API Key、模型和 provider 配置，集中维护不同供应商。
- **一键切换**：在 Codex Switch 里选择目标配置后，一键写入当前 Codex 配置。
- **聊天记录同步**：切换到官方登录态或任意 API Provider 后，自动同步本地 Codex 历史记录的 provider bucket，让旧对话继续出现在历史列表里。
- **重启确认**：Codex 切换成功后，可选择立即重启 Codex Desktop，让新配置立刻生效。
- **跨电脑完整迁移**：可一次导出和导入 Codex 聊天、API Provider、Key 与官网登录配置；导入记录会安全合并并同步到当前供应商。
- **增量安全备份**：改写聊天记录索引前仅补充缺少的会话备份，不再为每次切换创建一整套时间戳副本。
- **兼容旧数据**：现阶段保留必要的 legacy 数据目录和迁移逻辑，尽量复用已有 CC Switch 配置。

## 典型使用方式

1. 在 Codex Switch 中保存一个官方 Codex/ChatGPT 登录态，或者添加你的 API Provider。
2. 为 API Provider 填写 Base URL、API Key、模型和名称。
3. 点击需要使用的配置并切换。
4. 选择是否立即重启 Codex。选择重启会立即加载新配置；选择暂不重启则在下次手动打开 Codex 时生效。
5. 打开 Codex，当前 provider 已切换，原来的本地聊天记录仍会同步到当前 provider 下显示。

这意味着你可以同时维护官方账号、OpenRouter、DeepSeek、其他兼容 OpenAI API 的服务，按需一键切换，而不是每次手动改配置、丢历史列表。

## 聊天记录为什么不会掉

Codex 的本地聊天历史会按 `model_provider` 分 bucket。很多切换工具只改当前 `config.toml`，所以 Codex 切到新 provider 后，只会看新 bucket，旧聊天记录就像消失了一样。

Codex Switch 在供应商切换成功后会自动做同步：

1. 读取当前 Codex `config.toml` 中激活的 `model_provider`。
2. 从 live config、已保存供应商配置、JSONL 会话元数据、SQLite thread 行和内置历史 provider id 中收集旧 bucket。
3. 将 Codex `.jsonl` 会话元数据里的 `session_meta.payload.model_provider` 改写到当前 provider。
4. 将 Codex `state_5.sqlite` 中 `threads.model_provider` 改写到当前 provider。
5. 已有会话只保留一份基线备份；发现新会话时才补充，且没有内容需要同步时不会写备份。

备份路径：

```text
~/.cc-switch/backups/codex-auto-history-sync-v2/
```

现阶段仍保留 legacy `~/.cc-switch` 作为配置目录，主要是为了兼容旧配置和历史数据。后续如果迁移到 `~/.codex-switch`，会做兼容迁移。

## 跨电脑迁移 Codex 数据

在 **设置 -> 数据 -> Codex 数据迁移** 中，可在原电脑导出 `.zip` 归档，再在新电脑导入。归档包含 Codex 会话 JSONL、会话标题、线程索引、API Provider、Key 与官方登录配置。该文件包含敏感凭据，不应上传到公开位置或转发给他人。

导入时会保留本机已有会话、跳过重复会话，并把 Windows/macOS/Linux 的源路径改写为当前电脑的实际会话路径；随后恢复供应商配置并立即把会话同步到当前供应商。之后再次切换供应商时，这些聊天记录仍会走同一套自动同步逻辑。

## 注意事项

- 这个功能的目标是让旧会话在切换供应商后重新出现在 Codex 历史列表中。
- 跨 provider 恢复非常旧的会话仍可能受 Codex 内部数据、provider 特定字段或加密内容影响。
- 当前版本提供 Windows x64、macOS universal 和 Linux x64 安装包。
- 安装包暂未签名，首次安装可能需要手动允许运行。
- 部分 Linux 发行版运行 AppImage 可能需要安装 FUSE；如果 AppImage 不适合当前系统，可以使用 `.deb` 或 `.rpm`。

## 致谢

Codex Switch 基于 CC Switch 二次开发。感谢 CC Switch 原项目和贡献者提供的基础能力与灵感。

本项目会保留必要的兼容逻辑，并围绕 Codex 官方登录态、API Provider 切换和聊天记录同步继续迭代。
