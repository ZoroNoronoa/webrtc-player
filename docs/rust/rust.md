# rust

## 查看本机 triple

```bash
$ rustup show
Default host: x86_64-pc-windows-msvc
rustup home:  C:\Users\51267\.rustup

installed toolchains
--------------------
stable-x86_64-pc-windows-msvc (active, default)

active toolchain
----------------
name: stable-x86_64-pc-windows-msvc
active because: it's the default toolchain
installed targets:
  x86_64-pc-windows-msvc
```

## workspace

```bash
# 初始化
cd examples
cargo init --name whep-player-examples

# 添加依赖
# 1. 进到 member 目录
cd examples
cargo add color_eyre
# 2. 在 workspace 目录
cargo add -p whep-player-examples tracing

# 编译
cargo build --release -p whep-player-examples --example tracing

# 运行
cargo run --release -p whep-player-examples --example tracing
```
