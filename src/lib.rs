//! `ajc_term` — cold-themed interactive shell wrapper.
//!
//! This library is the reusable surface: other terminal-driven programs can
//! depend on the modules (theme, prompt, input, shell, builtins) or call
//! [`run`] for the full REPL experience.

pub mod banner;
pub mod builtins;
pub mod input;
pub mod prompt;
pub mod shell;
pub mod theme;

use anyhow::Result;
use builtins::BuiltinOutcome;
use crossterm::terminal;
use input::{LineResult, RawModeGuard};
use prompt::PromptState;
use std::io::{self, Write};

/// Run the interactive shell until the user quits.
///
/// Lifecycle:
/// 1. Enter raw mode when stdin is a TTY (restored via [`RawModeGuard`] on exit)
/// 2. Paint cold background + banner
/// 3. Loop: status → prompt → read line → builtin or host shell
pub fn run() -> Result<()> {
    let mut stdout = io::stdout();

    // Enable ANSI on older Windows consoles when possible.
    // Crossterm handles the VT enable dance internally for styling/cursor APIs.
    let raw = RawModeGuard::enter()?;

    theme::paint_screen(&mut stdout)?;
    banner::draw(&mut stdout)?;

    let mut state = PromptState::fresh();

    loop {
        state.refresh_cwd();
        prompt::draw_status(&mut stdout, &state)?;
        prompt::draw_prompt(&mut stdout, &state)?;

        let line = match input::read_line(&mut stdout)? {
            LineResult::Submit(s) => s,
            LineResult::Interrupt => {
                state.last_exit = Some(130); // conventional SIGINT-ish code
                continue;
            }
            LineResult::Eof => break,
        };

        // Builtins first — especially `cd`, which must mutate *our* process.
        match builtins::try_handle(&line, &mut stdout) {
            Ok(Some(BuiltinOutcome::Quit)) => break,
            Ok(Some(BuiltinOutcome::Continue { exit_code })) => {
                state.last_exit = Some(exit_code);
                continue;
            }
            Ok(None) => { /* fall through to host shell */ }
            Err(err) => {
                let _ = builtins::report_error(&mut stdout, &err);
                state.last_exit = Some(1);
                continue;
            }
        }

        // Child processes often expect cooked (non-raw) mode for line editing
        // and signal handling. Temporarily leave raw mode around the spawn.
        if raw.is_active() {
            terminal::disable_raw_mode()?;
        }
        let code = match shell::execute(line.trim()) {
            Ok(code) => code,
            Err(err) => {
                if raw.is_active() {
                    terminal::enable_raw_mode()?;
                }
                let _ = builtins::report_error(&mut stdout, &err);
                state.last_exit = Some(1);
                continue;
            }
        };
        if raw.is_active() {
            terminal::enable_raw_mode()?;
        }
        state.last_exit = Some(code);
    }

    // Farewell line in theme colors, then leave the guard to restore cooked mode.
    stdout.write_all(b"\r\n")?;
    theme::write_styled(&mut stdout, theme::TEXT_MUTED, None, "until next time.\r\n")?;
    stdout.flush()?;
    Ok(())
}
