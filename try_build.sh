#!/bin/bash

# 确保你的项目根目录下的 Cargo.toml 中定义了 server 和 client 的名称
# 如果你的主二进制文件 (来自 src/main.rs) 在 Cargo.toml 中的名称是 'tacklebox' 或其他，
# 请相应地修改 SERVER_NAME 变量。
SERVER_NAME="tackle_box"  # 假设 src/main.rs 编译出的文件名为 server
CLIENT_NAME="client"  # 对应 src/bin/client.rs
IMAGE_NAME="musl-builder"
TARGET_DIR="target/x86_64-unknown-linux-musl/release"

# --- 1. 构建镜像 (编译代码) ---
echo "--- 1. 编译项目并创建 builder 镜像 ---"
# 使用 buildx 确保在 macOS 上能正确编译 Linux/amd64 目标
docker buildx build \
    --platform linux/amd64 \
    -t ${IMAGE_NAME} \
    -f Dockerfile.musl .

# 检查构建是否成功
if [ $? -ne 0 ]; then
    echo "❌ Docker 构建失败，请检查 Dockerfile 或依赖项错误。"
    exit 1
fi

# --- 2. 创建一个临时容器实例 (用于访问文件) ---
echo "--- 2. 创建临时容器实例 ---"
CONTAINER_ID=$(docker create ${IMAGE_NAME})

# 检查容器创建是否成功
if [ -z "${CONTAINER_ID}" ]; then
    echo "❌ 无法创建 Docker 容器。"
    exit 1
fi

# --- 3. 提取 Server 二进制文件 (来自 main.rs) ---
echo "--- 3a. 复制 Server (${SERVER_NAME}) 到当前目录 ---"
docker cp ${CONTAINER_ID}:/app/${TARGET_DIR}/${SERVER_NAME} ./${SERVER_NAME}_linux_musl

# --- 4. 提取 Client 二进制文件 (来自 src/bin/client.rs) ---
echo "--- 3b. 复制 Client (${CLIENT_NAME}) 到当前目录 ---"
docker cp ${CONTAINER_ID}:/app/${TARGET_DIR}/${CLIENT_NAME} ./${CLIENT_NAME}_linux_musl

# --- 5. 清理临时容器 ---
echo "--- 4. 清理临时容器 ---"
docker rm ${CONTAINER_ID}

echo ""
echo "================================================="
echo "✅ 编译和提取成功！"
echo "Linux Musl Server 文件位于: ./${SERVER_NAME}_linux_musl"
echo "Linux Musl Client 文件位于: ./${CLIENT_NAME}_linux_musl"
echo "================================================="
