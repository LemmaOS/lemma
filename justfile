# 默认列出所有配方
default:
    @just --list

# [proto] 契约静态检查
proto-lint:
    cd proto && buf lint

# [proto] 契约编译验证
proto-build:
    cd proto && buf build

# [proto] 契约代码生成
proto-gen:
    npm run gen:proto

# [rust] 服务端开发服务器
rust-dev:
    cargo run

# [rust] 服务端构建
rust-build:
    cargo build --workspace

# [rust] 服务端 clippy 检查
rust-lint:
    cargo clippy --workspace --all-targets

# [rust] 服务端测试
rust-test:
    cargo test --workspace

# [rust] 服务端代码格式化
rust-fmt:
    cargo fmt --all

# [web] 网页前端构建
web-build:
    cd web && npm run build

# [web] 网页前端开发服务器
web-dev:
    cd web && npm run dev

# [web] 网页前端测试
web-test:
    cd web && npm test

# [web] 网页前端eslint 检查
web-lint:
    cd web && npm run lint

# [web] 网页前端代码格式化
web-fmt:
    cd web && npm run format
