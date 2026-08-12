# 运行时镜像
FROM debian:bullseye-slim

# 临时禁用 APT 的 GPG 验证以解决密钥环问题
# 安装运行时依赖（这些包来自官方 Debian 仓库）
RUN echo 'Acquire::AllowInsecureRepositories "true";' > /etc/apt/apt.conf.d/99allow-insecure \
    && echo 'APT::Get::AllowUnauthenticated "true";' >> /etc/apt/apt.conf.d/99allow-insecure \
    && apt-get update --allow-releaseinfo-change \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl1.1 \
    && rm -f /etc/apt/apt.conf.d/99allow-insecure \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -m -u 1000 appuser

# 设置工作目录
WORKDIR /app

# 构建参数：指定要复制的二进制文件名
ARG BINARY_NAME

# 从本地根目录复制已编译的二进制文件到固定路径
COPY --chown=appuser:appuser ${BINARY_NAME} /app/app

# 设置文件权限
RUN mkdir -p /app/assets/secret /app/configs \
    && chown -R appuser:appuser /app/assets /app/configs \
    && chmod +x /app/app

# 切换到非 root 用户
USER appuser

# 统一入口，配置文件路径通过 docker-compose 的 command 参数传入
ENTRYPOINT ["/app/app"]
