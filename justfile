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
