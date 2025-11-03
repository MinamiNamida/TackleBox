# 使用官方 Python 基础镜像，推荐使用 slim 版本
FROM python:3.10-slim-bullseye

# 设置工作目录
WORKDIR /app

# --- 镜像加速配置 ---
# 配置 Debian 镜像源 (清华 TUNA)
RUN echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian bullseye main contrib non-free" > /etc/apt/sources.list \
    && echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian-security bullseye-security main contrib non-free" >> /etc/apt/sources.list \
    && echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian bullseye-updates main contrib non-free" >> /etc/apt/sources.list \
    && echo "[global]\nindex-url = https://pypi.tuna.tsinghua.edu.cn/simple" > /etc/pip.conf

# --- 基础工具安装 ---
# 安装 build-essential 等，用于编译一些依赖（如 numpy/grpcio 可能会用到）
RUN apt-get update && apt-get install -y --no-install-recommends \
       git curl build-essential \
       python3-dev \
       python3-pip \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# --- Python 依赖安装 ---
RUN pip install --no-cache-dir \
    grpcio \
    grpcio-tools \
    
    # 基础开发和数据处理工具
    jupyterlab \
    ipykernel \
    numpy \
    pandas \
    matplotlib \
    
    gymnasium[atari] \
    pettingzoo \
    rlcard 
    
# 暴露 gRPC 服务的端口
EXPOSE 50051
EXPOSE 50050
# 暴露 Jupyter 端口
EXPOSE 8888
EXPOSE 3000

# 启动命令：进入 JupyterLab
CMD ["jupyter", "lab", "--ip=0.0.0.0", "--allow-root", "--NotebookApp.token=''"]

# 建议：将你的代码文件 COPY 到 /app 目录下，并设置启动命令
# CMD ["python", "your_agent_server.py"]
