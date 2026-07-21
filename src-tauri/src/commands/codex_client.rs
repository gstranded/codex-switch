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
    // Current Microsoft Store builds use ChatGPT.exe for the Electron shell and
    // resources\codex.exe for the app-server child. Closing/relaunching the child
    // crashes the shell, so only target the top-level desktop process.
    r#"
$ErrorActionPreference = 'Stop'
function Get-CodexDesktopWindows {
  @(Get-CimInstance Win32_Process |
    Where-Object {
      $_.ExecutablePath -and
      $_.Name -in @('ChatGPT.exe', 'Codex.exe') -and
      $_.CommandLine -notmatch '(^|\s)--type=' -and
      $_.ExecutablePath -notmatch '\\resources\\codex\.exe$' -and
      $_.ExecutablePath -notmatch 'Codex[ -]Switch'
    } |
    Where-Object {
      $process = Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue
      $process -and $process.MainWindowHandle -ne 0
    } |
    Sort-Object ProcessId)
}

$targets = @(Get-CodexDesktopWindows)
$paths = @($targets | ForEach-Object { $_.ExecutablePath } | Select-Object -Unique)
$startApp = Get-StartApps |
  Where-Object { $_.Name -match '^(Codex|ChatGPT)$' -and $_.Name -notmatch 'Switch' } |
  Sort-Object @{ Expression = { if ($_.Name -eq 'Codex') { 0 } else { 1 } } } |
  Select-Object -First 1

if ($targets.Count -eq 0 -and -not $startApp -and $paths.Count -eq 0) { exit 3 }

foreach ($target in $targets) {
  $process = Get-Process -Id $target.ProcessId -ErrorAction SilentlyContinue
  if ($process -and -not $process.CloseMainWindow()) {
    [Console]::Error.WriteLine("Codex Desktop has no closable main window (PID $($target.ProcessId)).")
    exit 4
  }
}

$deadline = [DateTime]::UtcNow.AddSeconds(15)
do {
  $remaining = @($targets | Where-Object { Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue })
  if ($remaining.Count -eq 0) { break }
  Start-Sleep -Milliseconds 150
} while ([DateTime]::UtcNow -lt $deadline)
if ($remaining.Count -gt 0) {
  [Console]::Error.WriteLine('Codex Desktop did not close cleanly; it was not force-terminated.')
  exit 4
}

if ($startApp) {
  Start-Process -FilePath 'explorer.exe' -ArgumentList "shell:AppsFolder\$($startApp.AppID)" | Out-Null
} elseif ($paths.Count -gt 0) {
  Start-Process -FilePath $paths[0] | Out-Null
} else {
  exit 3
}

$launchDeadline = [DateTime]::UtcNow.AddSeconds(15)
do {
  $launched = @(Get-CodexDesktopWindows)
  if ($launched.Count -gt 0) { break }
  Start-Sleep -Milliseconds 150
} while ([DateTime]::UtcNow -lt $launchDeadline)
if ($launched.Count -eq 0) {
  [Console]::Error.WriteLine('Codex Desktop was launched but no main window appeared.')
  exit 5
}

Write-Output "CODEX_SWITCH_RESTARTED:$([Math]::Max(1, $targets.Count))"
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
            "Codex Desktop is not installed or could not be found in the Start menu.".to_string(),
        ));
    }
    if output.status.code() == Some(4) {
        return Err(AppError::Message(
            "Codex Desktop did not close cleanly, so Codex Switch left it running instead of force-closing it. Quit Codex once and try again.".to_string(),
        ));
    }
    if output.status.code() == Some(5) {
        return Err(AppError::Message(
            "Codex Desktop was launched, but its main window did not appear within 15 seconds."
                .to_string(),
        ));
    }
    if !output.status.success() {
        return Err(AppError::Message(format!(
            "Codex Desktop restart failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let restarted_processes = String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.trim().strip_prefix("CODEX_SWITCH_RESTARTED:"))
        .and_then(|count| count.parse::<usize>().ok())
        .unwrap_or(1);
    Ok(CodexClientRestartResult {
        restarted_processes,
    })
}

#[cfg(target_os = "macos")]
fn restart_codex_desktop() -> Result<CodexClientRestartResult, AppError> {
    use std::process::Command;
    use std::thread;
    use std::time::{Duration, Instant};

    let running = macos_running_codex_app()?;
    let (app_name, bundle_id) = running
        .clone()
        .unwrap_or_else(|| ("Codex".to_string(), "com.openai.codex".to_string()));

    if running.is_some() {
        let script = format!("tell application {app_name:?} to quit");
        let quit = Command::new("osascript")
            .args(["-e", script.as_str()])
            .status()
            .map_err(|e| AppError::Message(format!("Unable to close Codex Desktop: {e}")))?;
        if !quit.success() {
            return Err(AppError::Message(
                "Codex Desktop did not accept the normal quit request".to_string(),
            ));
        }

        let deadline = Instant::now() + Duration::from_secs(15);
        while macos_process_bundle_id(&app_name)?.is_some() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(150));
        }
        if macos_process_bundle_id(&app_name)?.is_some() {
            return Err(AppError::Message(
                "Codex Desktop did not close cleanly; it was not force-terminated".to_string(),
            ));
        }
    }

    let open = Command::new("open")
        .args(["-b", bundle_id.as_str()])
        .status()
        .map_err(|e| AppError::Message(format!("Unable to reopen Codex Desktop: {e}")))?;
    if !open.success() {
        let fallback = Command::new("open")
            .args(["-a", app_name.as_str()])
            .status()
            .map_err(|e| AppError::Message(format!("Unable to reopen Codex Desktop: {e}")))?;
        if !fallback.success() {
            return Err(AppError::Message(
                "Codex Desktop is not installed or could not be reopened".to_string(),
            ));
        }
    }

    Ok(CodexClientRestartResult {
        restarted_processes: 1,
    })
}

#[cfg(target_os = "macos")]
fn macos_running_codex_app() -> Result<Option<(String, String)>, AppError> {
    for app_name in ["Codex", "ChatGPT"] {
        if let Some(bundle_id) = macos_process_bundle_id(app_name)? {
            if app_name == "Codex" || bundle_id.to_ascii_lowercase().contains("codex") {
                return Ok(Some((app_name.to_string(), bundle_id)));
            }
        }
    }
    Ok(None)
}

#[cfg(target_os = "macos")]
fn macos_process_bundle_id(app_name: &str) -> Result<Option<String>, AppError> {
    use std::process::Command;

    let script = format!(
        "tell application \"System Events\" to get bundle identifier of first application process whose name is {app_name:?}"
    );
    let output = Command::new("osascript")
        .args(["-e", script.as_str()])
        .output()
        .map_err(|e| AppError::Message(format!("Unable to inspect Codex Desktop: {e}")))?;
    if !output.status.success() {
        return Ok(None);
    }
    let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok((!bundle_id.is_empty()).then_some(bundle_id))
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
    fn restart_script_closes_shell_and_never_kills_the_app_server() {
        assert!(super::windows_restart_script().contains("--type="));
        assert!(super::windows_restart_script().contains("CloseMainWindow"));
        assert!(super::windows_restart_script().contains("MainWindowHandle -ne 0"));
        assert!(super::windows_restart_script().contains("Get-StartApps"));
        assert!(super::windows_restart_script().contains("shell:AppsFolder"));
        assert!(super::windows_restart_script().contains("resources\\codex"));
        assert!(super::windows_restart_script().contains("exit 5"));
        assert!(!super::windows_restart_script().contains("Stop-Process"));
    }
}
