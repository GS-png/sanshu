use anyhow::Result;
use std::process::Command;
use std::fs;
use std::path::Path;

use crate::mcp::types::PopupRequest;

/// Create UI popup
///
/// Prefers UI command in same directory as MCP server, falls back to global
pub fn create_tauri_popup(request: &PopupRequest) -> Result<String> {
    // Create temp request file - cross platform
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("mcp_request_{}.json", request.id));
    let ui_log_file = temp_dir.join(format!("devkit_ui_mcp_{}.log", request.id));
    let request_json = serde_json::to_string_pretty(request)?;
    fs::write(&temp_file, request_json)?;

    // Find UI command path
    let command_path = find_ui_command()?;

    // Execute UI command
    let output = Command::new(&command_path)
        .env("MCP_LOG_FILE", ui_log_file.to_string_lossy().to_string())
        .arg("--mcp-request")
        .arg(temp_file.to_string_lossy().to_string())
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to start UI process ({}): {}", command_path, e))?;

    // Cleanup temp file
    let _ = fs::remove_file(&temp_file);

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout);
        let response = response.trim();
        if response.is_empty() {
            Ok("CANCELLED".to_string())
        } else {
            Ok(response.to_string())
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "UI process failed ({}). Exit: {}\nUI log file: {}\n--- stderr ---\n{}\n--- stdout ---\n{}",
            command_path,
            output.status,
            ui_log_file.display(),
            stderr.trim(),
            stdout.trim()
        );
    }
}

fn ui_candidate_names() -> &'static [&'static str] {
    #[cfg(windows)]
    {
        &["devkit-ui.exe", "devkit-ui"]
    }

    #[cfg(not(windows))]
    {
        &["devkit-ui"]
    }
}

/// Find UI command path
///
/// Priority: same directory -> global -> development
pub fn find_ui_command() -> Result<String> {
    let ui_path_override = std::env::var("DEVKIT_UI_PATH")
        .or_else(|_| std::env::var("MCP_UI_PATH"))
        .ok()
        .map(std::path::PathBuf::from);

    let ui_mode = std::env::var("DEVKIT_UI_MODE")
        .or_else(|_| std::env::var("MCP_UI_MODE"))
        .unwrap_or_default();
    let explicit_debug = matches!(ui_mode.as_str(), "debug" | "dev");

    if let Some(path) = ui_path_override {
        if path.exists() && is_executable(&path) {
            return Ok(path.to_string_lossy().to_string());
        }
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            for name in ui_candidate_names() {
                let p = exe_dir.join(name);
                if p.exists() && is_executable(&p) {
                    return Ok(p.to_string_lossy().to_string());
                }
            }
        }
    }
    let prefer_debug = explicit_debug;
    if prefer_debug {
        let repo_debug_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("debug");
        for name in ui_candidate_names() {
            let p = repo_debug_dir.join(name);
            if p.exists() && is_executable(&p) {
                return Ok(p.to_string_lossy().to_string());
            }
        }

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(target_dir) = current_exe
                .ancestors()
                .find(|p| p.file_name().and_then(|s| s.to_str()) == Some("target"))
            {
                let debug_dir = target_dir.join("debug");
                for name in ui_candidate_names() {
                    let p = debug_dir.join(name);
                    if p.exists() && is_executable(&p) {
                        return Ok(p.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    let repo_release_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("release");
    for name in ui_candidate_names() {
        let p = repo_release_dir.join(name);
        if p.exists() && is_executable(&p) {
            return Ok(p.to_string_lossy().to_string());
        }
    }

    // 1. Try UI command in same directory as MCP server
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            if let Some(target_dir) = current_exe
                .ancestors()
                .find(|p| p.file_name().and_then(|s| s.to_str()) == Some("target"))
            {
                let release_dir = target_dir.join("release");
                for name in ui_candidate_names() {
                    let p = release_dir.join(name);
                    if p.exists() && is_executable(&p) {
                        return Ok(p.to_string_lossy().to_string());
                    }
                }
            }
            for name in ui_candidate_names() {
                let p = exe_dir.join(name);
                if p.exists() && is_executable(&p) {
                    return Ok(p.to_string_lossy().to_string());
                }
            }
        }
    }

    // 2. Try global command
    for name in ui_candidate_names() {
        let candidate = name.trim_end_matches(".exe");
        if test_command_available(candidate) {
            return Ok(candidate.to_string());
        }
    }

    // 3. Return detailed error
    anyhow::bail!(
        "UI command not found. Tried names: {:?}\n\
         You can explicitly set UI path via env: DEVKIT_UI_PATH or MCP_UI_PATH\n\
         Please ensure either:\n\
         1. UI is installed / in PATH (e.g. devkit-ui)\n\
         2. Or UI exe is in the same directory as devkit-mcp\n\
         3. Or set DEVKIT_UI_PATH/MCP_UI_PATH to full path of the UI executable",
        ui_candidate_names()
    )
}

/// Test if command is available
fn test_command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if file is executable
fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        // Windows 上检查文件扩展名
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
    }
}
