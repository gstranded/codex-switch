# Codex Switch v0.4.1

此补丁版本修复 Windows 上切换供应商后无法正常重启 Codex Desktop 的问题。

## 修复内容

- 不再把 Codex Desktop 启动的 `codex.exe app-server` 或其他 Codex CLI 后台进程误认为桌面主程序。
- 只向真正持有 Codex 主窗口的 `ChatGPT.exe` / `Codex.exe` 发送正常关闭请求，避免在关闭前直接报错。
- 重新启动后等待 Codex 主窗口出现；若 15 秒内未成功打开，会返回明确错误，不再误报“重启成功”。
- 仍然不会强制终止 Codex 进程，确保正常退出流程有机会保存当前状态。

## 下载

- Windows x64：Setup EXE、MSI、Portable ZIP
- macOS universal：DMG、ZIP
- Linux x64：AppImage、DEB、RPM

安装包仍为未签名预览构建。Windows 可能显示 SmartScreen 提示；macOS 首次打开可能需要右键选择“打开”。
