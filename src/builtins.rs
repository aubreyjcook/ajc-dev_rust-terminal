//! In-process builtins that must not go through the host shell.
//!
//! `cd` is the classic example: changing directory in a child process would not
//! affect our cwd. Keeping these here is also the extension point for future
//! builtins (alias, theme, history, …).

use crate::theme::{self, ACCENT, ERROR, TEXT, TEXT_MUTED};
use anyhow::{Context, Result};
use std::env;
use std::io::Write;
use std::path::Path;

/// What the REPL should do after a builtin runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinOutcome {
    /// Continue the REPL loop; optional exit code updates the status line.
    Continue { exit_code: i32 },
    /// Leave the REPL cleanly.
    Quit,
}

/// If `line` starts with a known builtin, handle it and return `Some`.
/// Otherwise return `None` so the caller can send the line to the host shell.
pub fn try_handle(line: &str, out: &mut impl Write) -> Result<Option<BuiltinOutcome>> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(Some(BuiltinOutcome::Continue { exit_code: 0 }));
    }

    let mut parts = line.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();

    match cmd {
        "exit" | "quit" => Ok(Some(BuiltinOutcome::Quit)),
        "cd" => Ok(Some(cd(args.as_slice())?)),
        "clear" | "cls" => Ok(Some(clear(out)?)),
        "help" => Ok(Some(help(out)?)),
        "theme" => Ok(Some(theme_preview(out)?)),
        _ => Ok(None),
    }
}

fn cd(args: &[&str]) -> Result<BuiltinOutcome> {
    let target = if let Some(path) = args.first() {
        Path::new(path).to_path_buf()
    } else {
        // Bare `cd` → home directory (USERPROFILE on Windows, HOME elsewhere).
        env::var_os("USERPROFILE")
            .or_else(|| env::var_os("HOME"))
            .map(Into::into)
            .context("HOME / USERPROFILE is not set")?
    };

    env::set_current_dir(&target)
        .with_context(|| format!("cd: cannot access '{}'", target.display()))?;

    Ok(BuiltinOutcome::Continue { exit_code: 0 })
}

fn clear(out: &mut impl Write) -> Result<BuiltinOutcome> {
    crate::theme::paint_screen(out)?;
    out.flush()?;
    Ok(BuiltinOutcome::Continue { exit_code: 0 })
}

fn help(out: &mut impl Write) -> Result<BuiltinOutcome> {
    theme::write_styled(out, ACCENT, None, "ajc-term builtins\r\n")?;
    theme::write_styled(out, TEXT_MUTED, None, "─────────────────\r\n")?;
    write_help_line(out, "help", "show this list")?;
    write_help_line(out, "cd [path]", "change directory (in-process)")?;
    write_help_line(out, "clear / cls", "repaint the cold background")?;
    write_help_line(out, "theme", "preview the gray/blue palette")?;
    write_help_line(out, "exit / quit", "leave the shell")?;
    theme::write_styled(
        out,
        TEXT_MUTED,
        None,
        "\r\nAnything else is forwarded to the host shell.\r\n\r\n",
    )?;
    out.flush()?;
    Ok(BuiltinOutcome::Continue { exit_code: 0 })
}

fn write_help_line(out: &mut impl Write, name: &str, desc: &str) -> Result<()> {
    theme::write_styled(out, ACCENT, None, &format!("  {name:<14}"))?;
    theme::write_styled(out, TEXT, None, &format!("{desc}\r\n"))?;
    Ok(())
}

fn theme_preview(out: &mut impl Write) -> Result<BuiltinOutcome> {
    theme::write_styled(out, TEXT, None, "palette preview\r\n")?;
    swatch(out, "text   ", theme::TEXT)?;
    swatch(out, "muted  ", theme::TEXT_MUTED)?;
    swatch(out, "accent ", theme::ACCENT)?;
    swatch(out, "chrome ", theme::CHROME)?;
    swatch(out, "ok     ", theme::OK)?;
    swatch(out, "error  ", ERROR)?;
    out.write_all(b"\r\n")?;
    out.flush()?;
    Ok(BuiltinOutcome::Continue { exit_code: 0 })
}

fn swatch(out: &mut impl Write, label: &str, color: crossterm::style::Color) -> Result<()> {
    theme::write_styled(out, theme::TEXT_MUTED, None, &format!("  {label}"))?;
    theme::write_styled(out, color, None, "████  sample\r\n")?;
    Ok(())
}

/// Print a failure without unwinding the REPL.
pub fn report_error(out: &mut impl Write, err: &anyhow::Error) -> Result<()> {
    theme::write_styled(out, ERROR, None, "error: ")?;
    theme::write_styled(out, TEXT, None, &format!("{err:#}\r\n"))?;
    out.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_is_a_builtin() {
        let mut buf = Vec::new();
        let outcome = try_handle("help", &mut buf).unwrap();
        assert_eq!(outcome, Some(BuiltinOutcome::Continue { exit_code: 0 }));
        let text = String::from_utf8_lossy(&buf);
        assert!(text.contains("builtins"));
    }

    #[test]
    fn unknown_falls_through() {
        let mut buf = Vec::new();
        let outcome = try_handle("echo hi", &mut buf).unwrap();
        assert_eq!(outcome, None);
    }

    #[test]
    fn exit_quits() {
        let mut buf = Vec::new();
        let outcome = try_handle("exit", &mut buf).unwrap();
        assert_eq!(outcome, Some(BuiltinOutcome::Quit));
    }
}
