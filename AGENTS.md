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

## Release

发版使用 `cargo-release`，一条命令完成版本 bump、commit、tag、push：

```bash
cargo release patch --execute --no-confirm   # 0.3.2 → 0.3.3
cargo release minor --execute --no-confirm   # 0.3.2 → 0.4.0
cargo release major --execute --no-confirm   # 0.3.2 → 1.0.0
```

推送 tag 后 CI (`.github/workflows/release.yml`) 自动完成：
1. 构建 macOS (aarch64) + Linux (x86_64) 二进制
2. 上传 `.tar.gz` + `.sha256` 到 GitHub Release
3. 更新 `rickyshin93/homebrew-tap` 仓库中的 `Formula/on.rb`

**不需要手动更新 homebrew formula，CI 全自动处理。**

## Common Clippy Pitfalls

CI runs `cargo clippy -- -D warnings`，以下模式会报错：

| 错误写法 | 正确写法 |
|---|---|
| `.map(\|o\| o.status.success()).unwrap_or(false)` | `.is_ok_and(\|o\| o.status.success())` |
| `.map(\|_\| true).unwrap_or(false)` | `.is_ok_and(\|_\| true)` |

## Adding Fields to Config Struct

`Config` 在测试中有手动构造的地方（`config.rs` 中的 `expand_tilde_paths` 测试等）。
每次给 `Config` 或其子 struct 新增字段时，必须同步更新所有手动构造的实例，补上新字段（通常是 `new_field: None`），否则 CI 编译会报 `missing field` 错误。

## Notes

- Config path: `~/.on/<project>.yaml`
- Legacy `iterm:` config key still supported (auto-converted to `terminal:`)
- Keep README.md in sync when changing CLI arguments
