# --- 编译阶段 (Builder Stage) ---
# 使用官方 rust:slim 镜像，它基于 Debian/Ubuntu (glibc)，安装 C 库更容易。
# 修正：将不存在的 'latest-slim' 标签替换为可靠的稳定 'slim' 版本。
FROM rust:1.90.0-slim AS builder

# 1. 安装必要的 C 库和依赖
# 安装 PostgreSQL (libpq) 和 OpenSSL (libssl) 的开发库。
# 在 glibc 环境下，apt-get 可以完美安装这些依赖。
RUN apt-get update \
    && apt-get install -y \
        libpq-dev \
        libssl-dev \
        ca-certificates \
        pkg-config \
        protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY .sqlx ./.sqlx/
# 2. 告诉 SQLX 宏使用离线模式, 避免连接数据库
ENV SQLX_OFFLINE=true
# 复制 Cargo 文件以利用 Docker 缓存
COPY Cargo.toml Cargo.lock ./

# 首次运行下载依赖，并编译依赖项，以利用 Docker 缓存
# 目标：x86_64-unknown-linux-gnu (标准的动态链接 Linux 服务器)

COPY build.rs ./
COPY proto ./proto



# 首次运行下载依赖，并编译依赖项，以利用 Docker 缓存
# 目标：x86_64-unknown-linux-gnu (标准的动态链接 Linux 服务器)
# 注：现在 build.rs 已经存在，它会运行并生成 tonic 所需的代码。
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-gnu

# 复制源代码 (替换临时的 src/main.rs)
COPY src ./src
# 编译最终的动态链接二进制文件 (server 和 client)

RUN cargo build --release --target x86_64-unknown-linux-gnu
