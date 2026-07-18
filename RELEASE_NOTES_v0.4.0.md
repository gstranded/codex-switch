# Codex Switch v0.4.0

此版本修复 Codex Desktop 自动重启和 Windows/macOS 跨电脑迁移，并将聊天归档升级为完整的 Codex 数据归档。

## 主要更新

- Windows 重启不再终止 `resources/codex.exe` 后端子进程，而是像手动退出一样，向官方 Codex/ChatGPT 桌面主程序发送正常关闭请求，确认退出后再通过系统应用 ID 重新打开，避免“ChatGPT 已意外停止”。
- macOS 会识别 Codex 的实际应用进程和 bundle id，正常退出后重新打开；未运行时也可直接启动已安装的 Codex。
- 导出归档现在同时包含会话、标题/线程索引、API Provider、Base URL、API Key 与官方登录配置。
- 导入会把 SQLite 中来自 Windows 或 macOS 的绝对 `rollout_path` 重写为目标电脑的真实会话路径，修复“文件已导入但 Codex 不显示聊天”的问题。
- 导入恢复当前供应商并写回 live 配置，随后把全部历史同步到当前 provider；以后继续切换 provider 时聊天仍会跟随同步。
- v1 纯聊天归档继续兼容导入。
- 自动同步备份改为固定的 `codex-auto-history-sync-v2` 增量目录：已有 JSONL 不重复备份，只有新线程出现时才更新同一个 SQLite 快照，不再按每次切换无限增加时间戳目录。

## 安全提示

新版 Codex 数据归档包含 API Key 和登录凭据，请作为敏感文件保管，不要上传到公开仓库或发送给他人。

## 验证

- 前端 TypeScript、格式检查和 400 项 Vitest 测试全部通过。
- Rust 格式、Clippy 和完整测试在 GitHub CI 通过。
- 新增跨平台路径改写、Provider/Key 回环恢复、重复导入、后续 provider 切换、增量备份与重启 hook 测试。

## 下载

- Windows x64：Setup EXE、MSI、Portable ZIP
- macOS universal：DMG、ZIP
- Linux x64：AppImage、DEB、RPM

安装包仍为未签名预览构建。Windows 可能显示 SmartScreen 提示；macOS 首次打开可能需要右键选择“打开”。
