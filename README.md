# ajc-term

Cold-themed interactive shell wrapper written in Rust. Type commands at a styled prompt; builtins like `cd` run in-process, everything else goes through your host shell. Meant as a small, reusable base for other terminal-driven programs.

## Quick start

```bash
cargo run
```

## Docs

- [User guide](docs/USER_GUIDE.md) — install, keys, builtins, theme notes
- [Learning guide](docs/LEARNING.md) — architecture walkthrough for reading the code

## Layout

```text
src/
  main.rs      binary entry
  lib.rs       REPL loop (library surface)
  theme.rs     gray/blue palette
  banner.rs    startup splash
  prompt.rs    status + prompt
  input.rs     line editor
  builtins.rs  cd / help / clear / …
  shell.rs     host shell spawn
```

## License

MIT OR Apache-2.0
