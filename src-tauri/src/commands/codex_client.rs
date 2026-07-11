use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexClientRestartResult {
    pub restarted_processes: usize,
}

/// Restart the installed Codex Desktop client after a provider switch. This is
/// deliberately separate from restarting Codex Switch itself.
#[tauri::command]
pub async fn restart_codex_client() -> Result<CodexClientRestartResult, String> {
    tauri::async_runtime::spawn_blocking(restart_codex_desktop)
        .await
        .map_err(|e| format!("Restart Codex client failed: {e}"))?
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
fn windows_restart_script() -> &'static str {
    // `codex.exe` is an Electron process family. Only relaunch main processes
    // (helpers carry `--type=`), then Windows cleans up their child processes.
    r#"
$targets = @(Get-CimInstance Win32_Process -Filter "Name = 'codex.exe'" |
  Where-Object { $_.ExecutablePath -and $_.CommandLine -notmatch '(^|\s)--type=' } |
  Sort-Object ProcessId)
if ($targets.Count -eq 0) { exit 3 }
$paths = @($targets | ForEach-Object { $_.ExecutablePath } | Select-Object -Unique)
$targets | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction Stop }
Start-Sleep -Milliseconds 350
$paths | ForEach-Object { Start-Process -FilePath $_ -ErrorAction Stop | Out-Null }
Write-Output $targets.Count
"#
}

#[cfg(target_os = "windows")]
fn restart_codex_desktop() -> Result<CodexClientRestartResult, AppError> {
    use std::process::Command;

    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            windows_restart_script(),
        ])
        .output()
        .map_err(|e| AppError::Message(format!("Unable to invoke Windows process control: {e}")))?;
    if output.status.code() == Some(3) {
        return Err(AppError::Message(
            "Codex Desktop is not running. Open Codex manually when you are ready.".to_string(),
        ));
    }
    if !output.status.success() {
        return Err(AppError::Message(format!(
            "Codex Desktop restart failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let restarted_processes = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<usize>()
        .unwrap_or(1);
    Ok(CodexClientRestartResult {
        restarted_processes,
    })
}

#[cfg(target_os = "macos")]
fn restart_codex_desktop() -> Result<CodexClientRestartResult, AppError> {
    use std::process::Command;

    let running = Command::new("pgrep")
        .args(["-x", "Codex"])
        .status()
        .map_err(|e| AppError::Message(format!("Unable to inspect Codex Desktop: {e}")))?
        .success();
    if !running {
        return Err(AppError::Message(
            "Codex Desktop is not running. Open Codex manually when you are ready.".to_string(),
        ));
    }
    let quit = Command::new("osascript")
        .args(["-e", "tell application \"Codex\" to quit"])
        .status()
        .map_err(|e| AppError::Message(format!("Unable to close Codex Desktop: {e}")))?;
    if !quit.success() {
        return Err(AppError::Message(
            "Codex Desktop did not close cleanly".to_string(),
        ));
    }
    let open = Command::new("open")
        .args(["-a", "Codex"])
        .status()
        .map_err(|e| AppError::Message(format!("Unable to reopen Codex Desktop: {e}")))?;
    if !open.success() {
        return Err(AppError::Message(
            "Codex Desktop did not reopen".to_string(),
        ));
    }
    Ok(CodexClientRestartResult {
        restarted_processes: 1,
    })
}

#[cfg(target_os = "linux")]
fn restart_codex_desktop() -> Result<CodexClientRestartResult, AppError> {
    Err(AppError::Message(
        "Codex Desktop restart is unavailable on Linux. Close and reopen your Codex client to apply the switch."
            .to_string(),
    ))
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn restart_codex_desktop() -> Result<CodexClientRestartResult, AppError> {
    Err(AppError::Message(
        "Codex Desktop restart is not supported on this platform".to_string(),
    ))
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    #[test]
    fn restart_script_excludes_electron_helpers() {
        assert!(super::windows_restart_script().contains("--type="));
        assert!(super::windows_restart_script().contains("Start-Process"));
    }
}
