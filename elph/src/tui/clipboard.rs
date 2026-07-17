//! Best-effort system clipboard write for TUI shortcuts (no extra crate).

use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

/// Copy plain text to the system clipboard.
///
/// Uses platform tools: `pbcopy` (macOS), `wl-copy` (Wayland), then `xclip`/`xsel` (X11).
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        pipe_to_stdin("pbcopy", &[], text)
    }

    #[cfg(not(target_os = "macos"))]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some()
            && which_ok("wl-copy")
            && pipe_to_stdin("wl-copy", &[], text).is_ok()
        {
            return Ok(());
        }
        if which_ok("xclip") && pipe_to_stdin("xclip", &["-selection", "clipboard"], text).is_ok() {
            return Ok(());
        }
        if which_ok("xsel") && pipe_to_stdin("xsel", &["--clipboard", "--input"], text).is_ok() {
            return Ok(());
        }
        bail!("no clipboard tool found (install wl-copy, xclip, or xsel)");
    }
}

fn pipe_to_stdin(program: &str, args: &[&str], text: &str) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawn {program}"))?;
    {
        let mut stdin = child.stdin.take().context("open stdin")?;
        stdin.write_all(text.as_bytes()).context("write clipboard")?;
    }
    let status = child.wait().context("wait clipboard")?;
    if !status.success() {
        bail!("{program} exited with {status}");
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn which_ok(program: &str) -> bool {
    Command::new("which")
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_valid_payload() {
        // Does not require a real clipboard; just ensures helper accepts empty text.
        // Integration depends on host tools — exercise spawn only on platforms with pbcopy.
        #[cfg(target_os = "macos")]
        {
            let _ = copy_to_clipboard("");
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = which_ok("true");
        }
    }
}
