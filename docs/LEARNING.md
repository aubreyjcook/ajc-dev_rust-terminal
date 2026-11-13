# Learning the ajc-term codebase

This guide is for someone who already knows programming basics and wants to learn by reading the Rust code. Skim the map first, then follow the runtime loop.

## Crate map

| File | Role |
|------|------|
| `src/main.rs` | Binary entry — calls `ajc_term::run()` |
| `src/lib.rs` | Public library surface + the REPL loop |
| `src/theme.rs` | Cold gray/blue color constants and styled writes |
| `src/banner.rs` | Startup splash art |
| `src/prompt.rs` | Status line + prompt (`user@host … ›`) |
| `src/input.rs` | Raw-mode line editor (keys → buffer) |
| `src/builtins.rs` | In-process commands (`cd`, `help`, …) |
| `src/shell.rs` | Spawn the host shell for everything else |

The **library + binary** split matters: other terminal-driven programs can depend on `ajc_term` and reuse modules without going through `main`.

## Runtime loop (the heart)

Open `src/lib.rs` and find `run()`. Mentally walk this sequence:

```text
enable raw mode
paint background + banner
loop:
  draw status + prompt
  read_line()
    Ctrl+C  → clear line, continue
    Ctrl+D  → break
    Enter   → submit text
  try builtins
    Quit    → break
    handled → update exit code, continue
  disable raw mode
  host shell execute
  enable raw mode
  update exit code
restore cooked mode (RawModeGuard Drop)
```

### Why raw mode?

In “cooked” mode the OS line-buffers input and handles editing. For a custom prompt and per-key control we need **raw mode**: every key arrives as an event.

- Crossterm terminal docs: https://docs.rs/crossterm/latest/crossterm/terminal/index.html
- We wrap enable/disable in `RawModeGuard` (`src/input.rs`) so a panic or early return still restores the terminal via `Drop`.

### Why leave raw mode around `shell::execute`?

Many child programs expect a normal terminal. Leaving raw mode for the duration of the child avoids broken line editing inside those tools. See the comments around `terminal::disable_raw_mode` in `lib.rs`.

## How commands actually run

`src/shell.rs` builds a `std::process::Command`:

- **Windows:** `cmd.exe /C <line>`
- **Unix:** `$SHELL -c <line>` (fallback `/bin/sh -c`)

Stdio is **inherited**, so output streams straight to your terminal (including colors from tools like `git`).

Primary source: https://doc.rust-lang.org/std/process/struct.Command.html

### Why is `cd` a builtin?

`Command` starts a **child** process. If that child changes directory and exits, *our* process cwd is unchanged. `builtins::cd` calls `std::env::set_current_dir` in the parent so the next prompt and the next spawn see the new path.

## Theme plumbing

`theme.rs` defines RGB colors as crossterm `Color::Rgb { r, g, b }` values and helpers like `write_styled`. Prompt, banner, help, and errors all go through those helpers so the cold palette stays consistent.

Crossterm styling overview: https://docs.rs/crossterm/latest/crossterm/style/index.html

## Input handling details worth noticing

In `input.rs`:

- We filter `KeyEventKind::Press` so Windows does not insert each character twice (Press + Release).
- Backspace is implemented as the classic `BS space BS` sequence rather than relying on a full TUI toolkit.
- When stdin is **not** a TTY (piped input), `read_line` falls back to ordinary `read_line` buffering and skips raw mode — handy for scripts and smoke tests (`std::io::IsTerminal`).
- History and tab completion are intentionally absent — natural next extensions to this module.

## Suggested reading order

1. `main.rs` → `lib.rs` (`run`)
2. `prompt.rs` + `banner.rs` + `theme.rs` (what you see)
3. `input.rs` (how keys become a string)
4. `builtins.rs` then `shell.rs` (what happens after Enter)

## Rust language anchors

If you are still getting comfortable with Rust while reading this project:

- Ownership and `Drop` (the raw-mode guard): https://doc.rust-lang.org/book/ch15-03-drop.html
- Error handling with `Result` / `?`: https://doc.rust-lang.org/book/ch09-00-error-handling.html
- Modules and `pub use` / crate layout: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html

## Extension points

When you want to grow the project without a rewrite:

- New builtins → add arms in `builtins::try_handle`
- New colors / themes → expand `theme.rs` (config file can come later)
- History / completion → grow `input.rs`’s buffer logic
- Alternate front-ends → call library modules from a different binary
