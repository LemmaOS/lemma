# AGENTS.md

## 个人编码习惯

- 缩进一律使用 4 个空格（不用 Tab）
- 注释保持简练、简短，只在必要处添加
- 配置文件与项目骨架优先用官方生成命令（如 `cargo new`、`buf init`、`npm create`），不手写
- 模块用同名 .rs 文件（`auth.rs` + `auth/` 目录），不用 `mod.rs`

## Git 提交习惯

- Conventional Commits，提交信息用英语
- 按逻辑块整合提交，一个完整的功能/主题一个提交，避免细碎密集
