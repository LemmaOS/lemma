# 默认列出所有配方
default:
    @just --list

# [proto] 静态检查
proto-lint:
    cd proto && buf lint

# [proto] 编译验证
proto-build:
    cd proto && buf build

# [proto] 代码生成
proto-gen:
    npm run gen:proto

# [rust] 开发运行
rust-dev:
    cargo run

# [rust] clippy 检查
rust-lint:
    cargo clippy --workspace --all-targets

# [rust] 测试
rust-test:
    cargo test --workspace

# [rust] 格式化
rust-fmt:
    cargo fmt --all
