# ajc-term User Guide

`ajc-term` is an interactive shell wrapper with a cold gray/blue look. You type commands at a prompt; most of them are handed to your host shell (`cmd.exe` on Windows, `$SHELL` / `sh` elsewhere). A few builtins run inside `ajc-term` itself.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain with Cargo)
- A terminal that supports ANSI colors (Windows Terminal, modern conhost, or a typical Unix terminal)

## Install & run

From the repository root:

```bash
cargo run
```

Or build a release binary:

```bash
cargo build --release
./target/release/ajc-term
```

On Windows PowerShell the release binary is `.\target\release\ajc-term.exe`.

## What you see

1. A blue ASCII **banner** on a dark slate background
2. A **status line** with your current directory and last exit code
3. A **prompt** shaped like `user@host path ›`

Colors stay in the cold range: slate grays and sky blues. Errors use a muted cold red.

## Builtins

These are handled by `ajc-term` (not the host shell):

| Command | What it does |
|---------|----------------|
| `help` | List builtins |
| `cd [path]` | Change directory in this process (bare `cd` goes home) |
| `clear` / `cls` | Clear the screen and repaint the cold background |
| `theme` | Preview the palette swatches |
| `exit` / `quit` | Leave the shell |

Anything else is forwarded to the host shell, for example:

```text
› dir
› echo hello
› cargo --version
```

## Keys

| Key | Action |
|-----|--------|
| Enter | Run the current line |
| Backspace | Delete the previous character |
| Ctrl+C | Cancel the current line (does not quit) |
| Ctrl+D | Quit when the line is empty |

## Tips

- Use `cd` from the builtin list so later commands see the new directory. Running `cd` only inside a host-shell one-liner would not stick.
- Exit codes on the status line come from the last builtin or host-shell command (`0` is success).
- Child programs inherit the terminal; tools that print their own colors should still work.

## Theme notes

The palette lives in `src/theme.rs`. There is no config file yet — change the RGB constants there (or call `theme` at the prompt to preview).
