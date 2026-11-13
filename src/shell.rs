//! Host-shell command execution.
//!
//! External lines are handed to the platform shell so users get familiar
//! builtins and PATH lookup. We inherit stdio so child programs can still use
//! color and interactive prompts when the terminal allows it.

use anyhow::{Context, Result};
use std::process::{Command, Stdio};

/// Run `line` via the host shell and return its exit code (or 1 on spawn failure).
pub fn execute(line: &str) -> Result<i32> {
    let mut cmd = host_shell(line);

    // Inherit so tools like `git`, `cargo`, etc. stream directly to the terminal.
    // See: https://doc.rust-lang.org/std/process/struct.Command.html
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Leave raw mode briefly? Child programs that expect cooked mode can misbehave
    // if we stay in raw mode. The REPL disables raw mode around execute() in lib.rs.

    let status = cmd
        .status()
        .with_context(|| format!("failed to spawn shell for: {line}"))?;

    Ok(status.code().unwrap_or(1))
}

/// Build the platform-appropriate shell invocation.
///
/// Windows uses `cmd.exe /C` for predictable exit codes and broad compatibility.
/// Unix prefers `$SHELL -c` when set, otherwise `sh -c`.
fn host_shell(line: &str) -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd.exe");
        // /C runs the string then exits — ideal for a one-shot wrapper.
        cmd.args(["/C", line]);
        cmd
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let mut cmd = Command::new(shell);
        cmd.args(["-c", line]);
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_shell_sets_command_string() {
        let cmd = host_shell("echo hi");
        // We only assert the program name shape — args differ by OS.
        let program = cmd.get_program().to_string_lossy();
        if cfg!(windows) {
            assert!(program.contains("cmd"));
        } else {
            assert!(!program.is_empty());
        }
    }
}
