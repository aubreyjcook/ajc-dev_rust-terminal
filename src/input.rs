//! Minimal line editor on top of crossterm's raw-mode key events.
//!
//! We intentionally keep this small (no history file / tab completion yet) so
//! the control flow is easy to follow. Extending later means growing this
//! module rather than scattering key handling through the REPL loop.

use crate::theme::TEXT;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Print, ResetColor, SetForegroundColor};
use crossterm::{queue, terminal};
use std::io::{self, BufRead, IsTerminal, Write};
use std::time::Duration;

/// Outcome of one line-editing session (until Enter / quit / interrupt).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineResult {
    /// User submitted a line (may be empty).
    Submit(String),
    /// Ctrl+C — clear the line and redraw prompt; do not exit.
    Interrupt,
    /// Ctrl+D on an empty line, or an explicit quit signal from the editor.
    Eof,
}

/// Read one line from the user.
///
/// Interactive TTYs use the crossterm key loop (caller should be in raw mode).
/// Piped / redirected stdin uses ordinary line reads so scripts and smoke tests work.
pub fn read_line(out: &mut impl Write) -> io::Result<LineResult> {
    if io::stdin().is_terminal() {
        read_line_raw(out)
    } else {
        read_line_piped()
    }
}

fn read_line_piped() -> io::Result<LineResult> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    match handle.read_line(&mut buffer)? {
        0 => Ok(LineResult::Eof),
        _ => {
            // Strip trailing newline characters from the pipe.
            while buffer.ends_with('\n') || buffer.ends_with('\r') {
                buffer.pop();
            }
            Ok(LineResult::Submit(buffer))
        }
    }
}

/// Read one editable line, echoing characters in the cold text color.
///
/// Caller must already be in raw mode. We only handle the keys listed in the
/// MVP plan: printable chars, Backspace, Enter, Ctrl+C, Ctrl+D.
fn read_line_raw(out: &mut impl Write) -> io::Result<LineResult> {
    let mut buffer = String::new();

    loop {
        // Poll so we can eventually add timeouts / redraws without blocking forever.
        if !event::poll(Duration::from_millis(500))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        // On Windows, crossterm can emit Press and Release; act only on Press
        // so we do not double-insert characters.
        // See: https://docs.rs/crossterm/latest/crossterm/event/enum.KeyEventKind.html
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match handle_key(key, &mut buffer, out)? {
            Some(result) => return Ok(result),
            None => continue,
        }
    }
}

fn handle_key(
    key: KeyEvent,
    buffer: &mut String,
    out: &mut impl Write,
) -> io::Result<Option<LineResult>> {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            // Clear visual line residue then signal interrupt to the REPL.
            clear_input_echo(out, buffer.len())?;
            buffer.clear();
            out.write_all(b"^C\r\n")?;
            out.flush()?;
            Ok(Some(LineResult::Interrupt))
        }
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
            if buffer.is_empty() {
                out.write_all(b"\r\n")?;
                out.flush()?;
                Ok(Some(LineResult::Eof))
            } else {
                Ok(None)
            }
        }
        (_, KeyCode::Enter) => {
            out.write_all(b"\r\n")?;
            out.flush()?;
            Ok(Some(LineResult::Submit(std::mem::take(buffer))))
        }
        (_, KeyCode::Backspace) => {
            if buffer.pop().is_some() {
                // Move left, overwrite with space, move left again — classic terminal erase.
                queue!(out, Print("\u{8} \u{8}"))?;
                out.flush()?;
            }
            Ok(None)
        }
        (_, KeyCode::Char(c)) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            buffer.push(c);
            queue!(out, SetForegroundColor(TEXT), Print(c), ResetColor)?;
            out.flush()?;
            Ok(None)
        }
        (_, KeyCode::Esc) => {
            // Ignore lone Esc for now; useful hook for future modes.
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn clear_input_echo(out: &mut impl Write, len: usize) -> io::Result<()> {
    for _ in 0..len {
        queue!(out, Print("\u{8} \u{8}"))?;
    }
    out.flush()?;
    Ok(())
}

/// Whether this process is attached to an interactive terminal for input.
pub fn is_interactive() -> bool {
    io::stdin().is_terminal()
}

/// RAII guard: enable raw mode on create (TTY only), disable on drop.
///
/// Raw mode is required so we receive keys immediately instead of line-buffered
/// stdin. Restoring cooked mode on drop keeps the user's terminal usable even
/// if the REPL returns early or panics through `Drop`.
/// Docs: https://docs.rs/crossterm/latest/crossterm/terminal/index.html
pub struct RawModeGuard {
    active: bool,
}

impl RawModeGuard {
    pub fn enter() -> io::Result<Self> {
        if is_interactive() {
            terminal::enable_raw_mode()?;
            Ok(Self { active: true })
        } else {
            Ok(Self { active: false })
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = terminal::disable_raw_mode();
        }
    }
}
