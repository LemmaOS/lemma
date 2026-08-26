# AGENTS.md

## 个人编码习惯

- 缩进一律使用 4 个空格（不用 Tab）
- 注释保持简练、简短，只在必要处添加
- 配置文件与项目骨架优先用官方生成命令（如 `cargo new`、`buf init`、`npm create`），不手写
- 模块用同名 .rs 文件（`auth.rs` + `auth/` 目录），不用 `mod.rs`

## Git 提交习惯

- Conventional Commits，提交信息用英语
- 按逻辑块整合提交，一个完整的功能/主题一个提交，避免细碎密集
- 只在明确要求时才提交 / 推送，不主动 commit

## 开发流程

- just 是统一任务入口；跑 `just --list` 查看所有可用任务
- proto 契约变更后，完整走一遍 lint、构建与代码生成（各有对应 just 任务），最后 `cargo build` 确认编译
- 提交前 `cargo fmt --all`；clippy 与测试必须全绿
- 覆盖率用 cargo-llvm-cov 测（just 已封装对应任务）；结果可疑时多半是陈旧计数，清缓存后重测
- 生成物不入 git（如 `web/src/gen` 已 ignore）；新环境先重新生成代码，再构建 web

## 代码约定

- `lemma-db` 只是存储内核（连接池、迁移、共享实体）；领域查询住在各领域 crate（users/tokens → lemma-auth，providers → lemma-providers，conversations → lemma-conversations）
- conversations/messages 表的所有 UPDATE 必须显式 `sync_seq = nextval('sync_seq')`（列默认值只作用于 INSERT）
- 集成测试用 `#[sqlx::test]`（每测试独立临时库）；跨 crate 测试带 `migrations = "../lemma-db/migrations"`
- 测试直调 handler（ServiceRequest / RequestContext），不起 HTTP 服务

## GitHub 与项目管理

- 仓库 `LemmaOS/lemma` 为 private（将来开源会连带 issue 公开，措辞按可公开标准写）
- 待办 / feature list 走 org project「Lemma Product 2026」：建 issue 挂进板，卡片用 Status 流转
- issue 内容只描述问题本身，不出现内部代号（C2、M3 这类），正文能省则省
- 项目板只当 feature list 用，不做日期排期；Roadmap / Iteration 字段是刻意删的，别再加回

## 已知环境问题

- 数据库连接串用 `127.0.0.1`，不用 `localhost`：localhost 同时解析出 ::1（常被先试），本机 ::1 被防火墙黑洞——实测 Docker 已监听 `[::]:5432` 仍连接超时
