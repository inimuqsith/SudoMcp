use std::process::Command;

use anyhow::{anyhow, Result};

pub fn prompt_password(message: &str) -> Result<Vec<u8>> {
    #[cfg(target_os = "macos")]
    {
        prompt_darwin(message)
    }
    #[cfg(target_os = "linux")]
    {
        prompt_linux(message)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = message;
        Err(anyhow!("unsupported OS {}", std::env::consts::OS))
    }
}

#[cfg(target_os = "macos")]
fn prompt_darwin(message: &str) -> Result<Vec<u8>> {
    // AppleScript string literals: backslash and double-quote need escaping.
    // Real newlines pass through fine.
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        "display dialog \"{escaped}\" default answer \"\" with hidden answer with title \"Claude Code sudo-mcp\" with icon caution\ntext returned of result"
    );
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(script)
        .output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "osascript exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

#[cfg(target_os = "linux")]
fn prompt_linux(message: &str) -> Result<Vec<u8>> {
    // Patch lokal: helper absolut (ssh-askpass root-owned, dialog GTK lewat PyGObject).
    // Tidak ada pencarian $PATH, jadi tidak ada jalur substitusi helper.
    const ASKPASS: &str = "/usr/local/bin/ssh-askpass";
    let output = Command::new(ASKPASS).arg(message).output()?;
    if !output.status.success() {
        return Err(anyhow!("{ASKPASS} exited {}", output.status));
    }
    Ok(output.stdout)
}
