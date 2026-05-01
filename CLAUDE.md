# CLAUDE.md

详细规范见 [AGENTS.md](./AGENTS.md)，本文件补充 Claude Code 专属配置。

## CI 要求

**提交前必须通过以下检查**（已通过 `.claude/settings.json` hook 自动执行）：

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

hook 会在每次 `git commit` 前自动运行，任一步骤失败则阻断提交。
