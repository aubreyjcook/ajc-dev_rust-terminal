//! Cold gray/blue visual palette for the shell chrome.
//!
//! Keeping colors in one place makes it easy for other terminal-driven programs
//! built on this crate to reuse (or swap) the look without hunting through UI code.

use crossterm::style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor, Print, SetAttribute, Attribute};
use crossterm::{queue, ExecutableCommand};
use std::io::{self, Write};

/// Dark slate used as the ambient background tint for chrome lines.
pub const BG_MUTED: Color = Color::Rgb { r: 15, g: 23, b: 42 }; // slate-950-ish

/// Primary readable text — light cool gray.
pub const TEXT: Color = Color::Rgb { r: 203, g: 213, b: 225 }; // slate-300

/// Dimmer secondary text (paths, hints).
pub const TEXT_MUTED: Color = Color::Rgb { r: 100, g: 116, b: 139 }; // slate-500

/// Accent for the prompt glyph and highlights.
pub const ACCENT: Color = Color::Rgb { r: 56, g: 189, b: 248 }; // sky-400

/// Slightly deeper blue for borders / chrome rules.
pub const CHROME: Color = Color::Rgb { r: 71, g: 85, b: 105 }; // slate-600 with blue cast

/// Muted cold red for errors (sparingly — still in the cold family).
pub const ERROR: Color = Color::Rgb { r: 248, g: 113, b: 113 }; // red-400, desaturated feel on dark bg

/// Success / ok exit hint.
pub const OK: Color = Color::Rgb { r: 125, g: 211, b: 252 }; // sky-300

/// Write `text` in `fg` (and optional background), then reset styles.
pub fn write_styled(
    out: &mut impl Write,
    fg: Color,
    bg: Option<Color>,
    text: &str,
) -> io::Result<()> {
    queue!(out, SetForegroundColor(fg))?;
    if let Some(bg) = bg {
        queue!(out, SetBackgroundColor(bg))?;
    }
    queue!(out, Print(text), ResetColor)?;
    Ok(())
}

/// Write bold accent text then reset.
pub fn write_accent_bold(out: &mut impl Write, text: &str) -> io::Result<()> {
    queue!(
        out,
        SetForegroundColor(ACCENT),
        SetAttribute(Attribute::Bold),
        Print(text),
        SetAttribute(Attribute::Reset),
        ResetColor
    )?;
    Ok(())
}

/// Apply the cold background to the whole screen once at startup.
///
/// Crossterm's `SetBackgroundColor` only affects subsequent cells; clearing
/// after setting it paints the viewport with our muted slate.
pub fn paint_screen(out: &mut impl Write) -> io::Result<()> {
    out.execute(SetBackgroundColor(BG_MUTED))?;
    out.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    out.execute(crossterm::cursor::MoveTo(0, 0))?;
    out.execute(ResetColor)?;
    Ok(())
}
