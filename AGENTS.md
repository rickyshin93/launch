# AGENTS.md

## Project Overview

`on` — A CLI tool to restore your full dev environment with one command (terminal panes, editor, browser).

- Language: Rust (edition 2021)
- Platforms: macOS, Linux
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt --check`

## Code Standards

- `unsafe` is forbidden (`unsafe_code = "forbid"`)
- Clippy pedantic warnings enabled, see `Cargo.toml [lints.clippy]`
- Formatting config in `rustfmt.toml`
- Ensure `cargo clippy` and `cargo fmt --check` pass before committing

## Project Structure

```
src/
  main.rs          — CLI entry point (clap)
  lib.rs           — Library entry point
  config.rs        — YAML config parsing (~/.on/<project>.yaml)
  process.rs       — Process orchestration & PID tracking
  state.rs         — Runtime state management
  iterm.rs         — iTerm2 AppleScript backend (macOS)
  tmux.rs          — tmux backend (macOS/Linux)
  editor.rs        — Editor launching
  browser.rs       — Browser opening (open/xdg-open)
  git.rs           — Git status checks
  port.rs          — Port conflict detection
```

## Terminal Backends

- **iTerm2** — macOS only, uses AppleScript via `osascript`
- **tmux** — cross-platform, uses `tmux` CLI commands
- Config `terminal.type` selects backend (default: `iterm` on macOS, `tmux` on Linux)

## Notes

- Config path: `~/.on/<project>.yaml`
- Legacy `iterm:` config key still supported (auto-converted to `terminal:`)
- Keep README.md in sync when changing CLI arguments
