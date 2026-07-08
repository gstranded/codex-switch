# Codex Switch v0.2.0

这个版本重点补齐 macOS 和 Linux 的发布支持。Codex Switch 现在不只提供 Windows 包，也会在 GitHub Releases 中提供 macOS universal 和 Linux x64 版本，方便不同系统的用户直接下载测试。

## 下载

当前版本提供以下文件：

- `Codex-Switch-0.2.0-Windows-x64-Setup.exe`：Windows 常规安装包，推荐大多数 Windows 用户使用。
- `Codex-Switch-0.2.0-Windows-x64.msi`：Windows MSI 安装包。
- `Codex-Switch-0.2.0-Windows-x64-Portable.zip`：Windows 便携版，解压后运行 `codex-switch.exe`。
- `Codex-Switch-0.2.0-macOS-universal.dmg`：macOS 安装包，同时支持 Apple Silicon 和 Intel Mac。
- `Codex-Switch-0.2.0-macOS-universal.zip`：macOS `.app` 压缩包，解压后运行。
- `Codex-Switch-0.2.0-Linux-x64.AppImage`：Linux 便携版，赋予执行权限后运行。
- `Codex-Switch-0.2.0-Linux-x64.deb`：Debian / Ubuntu 安装包。
- `Codex-Switch-0.2.0-Linux-x64.rpm`：Fedora / RHEL / openSUSE 安装包。
- `SHA256SUMS.txt`：文件校验。

当前预览版本尚未代码签名或 Apple 公证。Windows 可能提示“未知发布者”或 SmartScreen；macOS 可能需要右键点击应用并选择打开。

## 新增

- 新增 macOS universal Release 构建，覆盖 Apple Silicon 和 Intel Mac。
- 新增 Linux x64 Release 构建，提供 AppImage、deb 和 rpm。
- Release workflow 改为一次发布 Windows、macOS、Linux 资产。
- README 下载说明改为多平台说明。

## 核心能力保持不变

- 在一个地方维护官方 Codex/ChatGPT 登录态和多个 API Provider。
- 支持保存 Base URL、API Key、模型和 provider 配置。
- 支持一键切换到官方登录态或任意 API Provider。
- 切换成功后自动同步 Codex 本地聊天记录 bucket，让历史列表不因为 provider 切换而消失。

## 注意

- 这个功能的目标是让旧会话在切换供应商后重新出现在 Codex 历史列表中。
- 跨 provider 恢复非常旧的会话仍可能受 Codex 内部数据、provider 特定字段或加密内容影响。
- 部分 Linux 发行版运行 AppImage 可能需要安装 FUSE；如果 AppImage 不适合当前系统，可以使用 `.deb` 或 `.rpm`。

## 致谢

Codex Switch 基于 CC Switch 二次开发。感谢 CC Switch 原项目和贡献者提供的基础能力与灵感。
