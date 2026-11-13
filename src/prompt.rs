//! Prompt and status-line rendering.

use crate::theme::{self, ACCENT, CHROME, ERROR, OK, TEXT, TEXT_MUTED};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

/// Snapshot of values shown in the chrome around the input line.
#[derive(Debug, Clone)]
pub struct PromptState {
    pub cwd: PathBuf,
    pub last_exit: Option<i32>,
    pub username: String,
    pub hostname: String,
}

impl PromptState {
    pub fn fresh() -> Self {
        Self {
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            last_exit: None,
            username: whoami_user(),
            hostname: whoami_host(),
        }
    }

    pub fn refresh_cwd(&mut self) {
        if let Ok(cwd) = env::current_dir() {
            self.cwd = cwd;
        }
    }
}

/// Compact status line: cwd + last exit code, in cold chrome colors.
pub fn draw_status(out: &mut impl Write, state: &PromptState) -> io::Result<()> {
    theme::write_styled(out, CHROME, None, "┌─ ")?;
    theme::write_styled(out, TEXT_MUTED, None, "cwd ")?;
    theme::write_styled(out, TEXT, None, &display_path(&state.cwd))?;

    match state.last_exit {
        Some(0) => {
            theme::write_styled(out, TEXT_MUTED, None, "  ·  exit ")?;
            theme::write_styled(out, OK, None, "0")?;
        }
        Some(code) => {
            theme::write_styled(out, TEXT_MUTED, None, "  ·  exit ")?;
            theme::write_styled(out, ERROR, None, &code.to_string())?;
        }
        None => {
            theme::write_styled(out, TEXT_MUTED, None, "  ·  ready")?;
        }
    }
    out.write_all(b"\r\n")?;
    Ok(())
}

/// Draw `user@host path › ` then leave the cursor ready for input.
pub fn draw_prompt(out: &mut impl Write, state: &PromptState) -> io::Result<()> {
    theme::write_styled(out, CHROME, None, "└─ ")?;
    theme::write_styled(out, ACCENT, None, &state.username)?;
    theme::write_styled(out, TEXT_MUTED, None, "@")?;
    theme::write_styled(out, TEXT, None, &state.hostname)?;
    out.write_all(b" ")?;
    theme::write_styled(out, TEXT_MUTED, None, &short_cwd(&state.cwd))?;
    out.write_all(b" ")?;
    theme::write_accent_bold(out, "› ")?;
    out.flush()?;
    Ok(())
}

fn display_path(path: &std::path::Path) -> String {
    path.display().to_string()
}

/// Prefer `~` when the cwd is under the home directory — keeps the prompt short.
fn short_cwd(path: &std::path::Path) -> String {
    if let Some(home) = home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            if stripped.as_os_str().is_empty() {
                return "~".into();
            }
            return format!("~/{}", stripped.display());
        }
    }
    path.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
}

fn whoami_user() -> String {
    env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "user".into())
}

fn whoami_host() -> String {
    // Avoid pulling an extra crate for hostname; env is good enough for chrome.
    env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".into())
}
