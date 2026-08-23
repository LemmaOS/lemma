# AGENTS.md

## 个人编码习惯

- 缩进一律使用 4 个空格（不用 Tab）
- 注释保持简练、简短，只在必要处添加
- 配置文件与项目骨架优先用官方生成命令（如 `cargo new`、`buf init`、`npm create`），不手写
- 模块用同名 .rs 文件（`auth.rs` + `auth/` 目录），不用 `mod.rs`

## Git 提交习惯

- Conventional Commits，提交信息用英语
- 按逻辑块整合提交，一个完整的功能/主题一个提交，避免细碎密集

## 开发流程

- just 是统一任务入口：例如 `proto-lint` / `proto-build` / `proto-gen`，`rust-lint` / `rust-test` / `rust-fmt` / `rust-dev`
- proto 契约变更后四连验证：`just proto-lint && just proto-build && just proto-gen && cargo build`
- 提交前 `cargo fmt --all`；clippy 与测试必须全绿
- TS 生成物不入 git（例如： `web/src/gen` 已 ignore），克隆后 `just proto-gen` 再构建 web

## 代码约定

- `lemma-db` 只是存储内核（连接池、迁移、共享实体）；领域查询住在各领域 crate（users/tokens → lemma-auth，providers → lemma-providers，conversations → lemma-conversations）
- conversations/messages 表的所有 UPDATE 必须显式 `sync_seq = nextval('sync_seq')`（列默认值只作用于 INSERT）
- 集成测试用 `#[sqlx::test]`（每测试独立临时库）；跨 crate 测试带 `migrations = "../lemma-db/migrations"`；测试文件顶部 `#![allow(clippy::unwrap_used)]`
- handler 直测：`ServiceRequest::from_parts(&view, &bytes)` + `RequestContext::new(headers)`，不走 HTTP

## 已知环境问题

- 数据库连接串用 `127.0.0.1`，不用 `localhost`（疑似本机 IPv6 被防火墙黑洞）
- rust-analyzer 对 refining_impl_trait 有误报（报 `impl Encodable<...>` 类型不符）——cargo / clippy 绿即为正确，忽略红线
