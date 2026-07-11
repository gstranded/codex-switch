# Codex Switch v0.3.0

此版本把供应商切换后的生效流程和跨电脑聊天记录迁移补齐了。

## 新增功能

- Codex 供应商切换成功后，会弹出“现在重启 / 暂不重启”确认。
- 在 Windows 上，选择“重启 Codex”会定位正在运行的 Codex Desktop 主进程，关闭并按原路径重新启动；不会重启 Codex Switch 本身。
- macOS 支持重启已运行的 Codex Desktop；Linux 会保留切换结果并提示用户手动重开 Codex 客户端。
- 设置 -> 数据新增“Codex 聊天记录”导入导出。
- 聊天归档为 ZIP，只包含会话 JSONL、会话标题索引与必要的 `state_5.sqlite` 线程索引；不会导出 API Key、登录凭据或供应商配置。
- 导入会校验归档、阻止路径穿越、合并会话并按会话 ID 去重，不覆盖本机已有会话。
- 导入完成后会自动同步到当前 active provider；以后切换到其他 provider 时，导入的历史记录也会继续同步并显示。

## 验证

- 新增归档回环测试：导出 JSONL 和 SQLite 会话，导入到隔离目录，切换到目标 provider 后验证 JSONL 与 SQLite 的 provider bucket 都会更新。
- 新增切换后重启确认测试，确保只有用户确认后才调用 Codex Desktop 重启命令。

## 下载

- Windows x64：Setup EXE、MSI、Portable ZIP
- macOS universal：DMG、ZIP
- Linux x64：AppImage、DEB、RPM

安装包仍为未签名预览构建。Windows 可能显示 SmartScreen 提示；macOS 首次打开可能需要右键选择“打开”。
