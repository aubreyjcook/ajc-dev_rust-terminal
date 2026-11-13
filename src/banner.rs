//! Startup splash — a small bit of flavor before the first prompt.

use crate::theme::{self, ACCENT, CHROME, TEXT, TEXT_MUTED};
use std::io::{self, Write};

const BANNER_LINES: &[&str] = &[
    r"     _    _  ____      _____ _____ ____  __  __ ",
    r"    / \  | |/ ___|    |_   _| ____|  _ \|  \/  |",
    r"   / _ \ | | |   _____  | | |  _| | |_) | |\/| |",
    r"  / ___ \| | |__|_____| | | | |___|  _ <| |  | |",
    r" /_/   \_\_|\____|      |_| |_____|_| \_\_|  |_|",
];

/// Draw the cold-themed startup banner and a one-line tagline.
pub fn draw(out: &mut impl Write) -> io::Result<()> {
    theme::write_styled(out, CHROME, None, &format!("{}\r\n", "─".repeat(52)))?;

    for line in BANNER_LINES {
        theme::write_styled(out, ACCENT, None, line)?;
        out.write_all(b"\r\n")?;
    }

    theme::write_styled(out, CHROME, None, &format!("{}\r\n", "─".repeat(52)))?;
    theme::write_styled(out, TEXT, None, "  cold shell wrapper")?;
    theme::write_styled(out, TEXT_MUTED, None, "  ·  type ")?;
    theme::write_styled(out, ACCENT, None, "help")?;
    theme::write_styled(out, TEXT_MUTED, None, "  ·  ")?;
    theme::write_styled(out, ACCENT, None, "exit")?;
    theme::write_styled(out, TEXT_MUTED, None, " to quit\r\n\r\n")?;
    out.flush()?;
    Ok(())
}
