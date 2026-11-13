//! Binary entry for `ajc-term`.
//!
//! Kept thin on purpose: all behavior lives in the `ajc_term` library so other
//! programs can reuse the same REPL or individual modules.

fn main() {
    if let Err(err) = ajc_term::run() {
        // If we fail before/without raw mode, plain stderr is the safest channel.
        eprintln!("ajc-term exited with error: {err:#}");
        std::process::exit(1);
    }
}
