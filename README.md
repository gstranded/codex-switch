# Codex Switch

<p>
  <a href="README_EN.md">English</a> |
  <strong>中文</strong>
</p>

Codex Switch 是面向 Codex 的供应商切换工具，核心目标是解决 Codex 在不同 `model_provider` 之间切换后，本地聊天记录不可见的问题。

## 下载

请到 [Releases](https://github.com/gstranded/codex-switch/releases) 下载。

Windows x64 当前提供：

- `Codex-Switch-0.1.0-Windows-x64-Setup.exe`：常规安装包，推荐大多数用户使用。
- `Codex-Switch-0.1.0-Windows-x64.msi`：MSI 安装包，适合偏好 MSI 或企业部署的场景。
- `Codex-Switch-0.1.0-Windows-x64-Portable.zip`：便携版，解压后运行 `codex-switch.exe`，不会安装到系统。
- `SHA256SUMS.txt`：校验文件完整性。

当前是 preview 版本，Windows 可能提示“未知发布者”或 SmartScreen，这是因为安装包尚未做代码签名。

## 主要改动

- 应用名称改为 Codex Switch。
- 支持供应商添加、配置管理和一键切换流程。
- Codex 供应商切换成功后，自动同步本地聊天记录的 provider bucket。
- 会重写 Codex `.jsonl` 会话元数据里的 `session_meta.payload.model_provider`。
- 会重写 Codex `state_5.sqlite` 中 `threads.model_provider` 的 provider bucket。
- 重写前会创建备份。

## 聊天记录同步逻辑

Codex 供应商切换成功后，Codex Switch 会：

1. 从当前 Codex `config.toml` 读取激活中的 `model_provider`。
2. 从 live config、已保存供应商配置、JSONL 会话元数据、SQLite thread 行和内置历史 provider id 中收集旧 bucket。
3. 把匹配到的本地历史 bucket 改写到当前激活的 provider id。
4. 如果没有任何内容需要同步，则不会创建多余备份。

备份路径：

```text
~/.cc-switch/backups/codex-auto-history-sync-v1/<timestamp>/
```

现阶段仍保留 legacy `~/.cc-switch` 作为配置目录，方便复用已有供应商数据。后续如果彻底迁移到 `~/.codex-switch`，需要做兼容迁移逻辑。

## 限制

这个功能的目标是让旧会话在切换供应商后重新出现在 Codex 历史列表中。它不能保证所有旧会话都能跨供应商继续恢复运行，因为 Codex 可能在会话数据里保存 provider 相关或加密内容。

## 本地开发

```powershell
pnpm install
pnpm typecheck
pnpm tauri dev
```

Windows 本地完整后端检查需要安装 Visual Studio Build Tools C++ workload 和 Windows SDK：

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
```

## 验证状态

- GitHub Actions CI 已通过 frontend typecheck、format、unit tests。
- GitHub Actions CI 已通过 backend `cargo fmt`、`cargo clippy`、`cargo test`。
- Windows x64 release workflow 已成功构建并上传安装包。
- 本机没有完整 Windows C++/SDK 链接环境，因此没有在本机完成 Windows 后端构建。

## 项目说明

Codex Switch 会继续保留必要的 legacy 数据兼容逻辑，避免已有配置和会话历史在迁移过程中丢失。
