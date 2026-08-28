# AGENTS.md

## 个人编码习惯

- 缩进一律使用 4 个空格（不用 Tab）
- 注释保持简练、简短，只在必要处添加
- 配置文件与项目骨架优先用官方生成命令（如 `cargo new`、`buf init`、`npm create`），不手写
- 模块用同名 .rs 文件（`auth.rs` + `auth/` 目录），不用 `mod.rs`

## Git 提交习惯

- Conventional Commits，提交信息用英语
- 按逻辑块整合提交，一个完整的功能/主题一个提交，避免细碎密集
- 提交必须原子化：一笔只做一件完整的事，且任何一笔检出后都能正常编译、测试全绿（保证 bisect 与回滚不踩坑）
- 只在明确要求时才提交 / 推送，不主动 commit

## 开发流程

- just 是统一任务入口；跑 `just --list` 查看所有可用任务
- proto 契约变更后，完整走一遍 lint、构建与代码生成（各有对应 just 任务），最后 `cargo build` 确认编译
- 新 proto 文件必须注册进 `crates/lemma-proto/build.rs` 的文件清单：TS 侧 buf 自动扫目录，Rust 侧是显式清单——漏注册四连验证照样全绿，到引用时才炸
- 提交前 `cargo fmt --all`；clippy 与测试必须全绿
- 覆盖率用 cargo-llvm-cov 测（just 已封装对应任务）；结果可疑时多半是陈旧计数，清缓存后重测
- 生成物不入 git（如 `web/src/gen` 已 ignore）；新环境先重新生成代码，再构建 web

## 代码约定

- `lemma-db` 只是存储内核（连接池、迁移、共享实体）；领域查询住在各领域 crate（users/tokens → lemma-auth，providers → lemma-providers，conversations → lemma-conversations，s3 配置 → lemma-archive）
- 入库凭证一律 lemma-crypto 密封、出库脱敏回显；前端密钥框不回填脱敏串（留空=保持）——回填值被保存会当成真密钥重新密封，密钥静默损坏
- S3 桶必须预先存在，测试连接只探测不建桶——自动建桶是刻意删掉的，别再加回
- conversations/messages 表的所有 UPDATE 必须显式 `sync_seq = nextval('sync_seq')`（列默认值只作用于 INSERT）
- 集成测试用 `#[sqlx::test]`（每测试独立临时库）；跨 crate 测试带 `migrations = "../lemma-db/migrations"`
- 测试直调 handler（ServiceRequest / RequestContext），不起 HTTP 服务

## GitHub 与项目管理

- 仓库 `LemmaOS/lemma` 为 private（将来开源会连带 issue 公开，措辞按可公开标准写）
- 待办 / feature list 走 org project「Lemma Product 2026」：建 issue 挂进板，卡片用 Status 流转
- issue 内容只描述问题本身，不出现内部代号（C2、M3 这类），正文能省则省；关闭前把正文补成完整描述（方案 + 关联提交）
- 项目板只当 feature list 用，不做日期排期；Roadmap / Iteration 字段是刻意删的，别再加回

## 已知环境问题

- 数据库连接串用 `127.0.0.1`，不用 `localhost`：localhost 同时解析出 ::1（常被先试），本机 ::1 被防火墙黑洞——实测 Docker 已监听 `[::]:5432` 仍连接超时
