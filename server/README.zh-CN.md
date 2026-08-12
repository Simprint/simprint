<div align="center">
  <h1>Simprint Server</h1>
  <p>面向 Simprint 工作区、账号体系、环境管理、代理资源和运行时接口的自托管后端服务。</p>
  <p>
    <img alt="Language Rust 2024" src="https://img.shields.io/badge/language-Rust%202024-f97316?style=flat-square&labelColor=0f172a" />
    <img alt="Framework Axum 0.8" src="https://img.shields.io/badge/framework-Axum%200.8-60a5fa?style=flat-square&labelColor=0f172a" />
    <img alt="Database PostgreSQL" src="https://img.shields.io/badge/database-PostgreSQL-38bdf8?style=flat-square&labelColor=0f172a" />
    <img alt="Cache Redis" src="https://img.shields.io/badge/cache-Redis-f87171?style=flat-square&labelColor=0f172a" />
  </p>
  <p>
    <a href="./README.md">English</a> | <strong>简体中文</strong>
  </p>
</div>

---

## Introduction

Simprint Server 是 Simprint 客户端和私有化部署场景使用的后端服务。它负责暴露应用 API、管理认证与持久化数据、初始化加密和存储相关资源，并在启动时自动执行内嵌的数据库迁移。

它面向希望把 Simprint 部署在自有基础设施中的使用者，而不是依赖共享托管后端。服务通过本地 TOML 配置文件驱动，默认围绕 PostgreSQL、Redis 和兼容 S3 的对象存储来组织运行环境。

## Why Simprint Server?

想把 Simprint 作为自托管服务落地，通常不只是“起一个 HTTP 服务”这么简单：

- 你需要控制 API 可用性、认证凭据和对象存储基础设施。
- 你需要一套可以公开发布、但不会泄露真实环境配置的部署产物。
- 你需要在发布和重启过程中稳定地完成数据库结构升级。
- 你需要一个统一的后端入口来承载工作区、环境、代理和账号相关接口。

Simprint Server 的设计就是围绕这些约束展开的：单个 Rust 服务、配置优先的部署模型、内嵌数据库迁移，以及仅打包可公开示例配置的发布流程。

## Features

- **核心应用 API**：在一个进程中承载账号、工作区、团队、环境、代理、模板、偏好、消息、扩展和本地运行时等接口。
- **认证与密钥初始化**：支持登录相关流程、令牌刷新、白名单路由以及首次启动时的 RSA 密钥初始化。
- **内嵌数据库迁移**：在 HTTP 服务开始接收流量前自动执行 `sqlx` migrations。
- **兼容 S3 的对象存储集成**：为头像、扩展资源和版本相关文件接入外部对象存储。
- **基于 Redis 的运行时协同**：使用 Redis 承担运行时协同和缓存类服务能力。
- **面向 Docker 的发布打包**：生成包含 `Dockerfile`、`docker-compose.yml` 以及由 `configs/config.example.toml` 复制出的 `configs/config.toml` 的部署包。
- **配置优先的运行方式**：本地运行和容器运行都使用同一套 `-f <config.toml>` 启动模型。

## Quick Start

### Prerequisites

- Rust toolchain
- PostgreSQL 16+ 或兼容的 PostgreSQL 实例
- Redis 7+
- 兼容 S3 的对象存储
- 可选的 SMTP 服务，用于邮件相关流程

### 一键安装自托管服务端

Linux 服务器可直接执行：

```bash
curl -fsSL https://raw.githubusercontent.com/Simprint/simprint/main/deploy/install-server.sh | bash # 请修改客户端的配置， 如: base_url = http://127.0.0.1:40041/api/
```

### 本地运行

```bash
cp configs/config.example.toml configs/config.local.toml
# 修改 configs/config.local.toml
cargo run -- -f configs/config.local.toml
```

示例配置默认监听 `40041` 端口，并使用 `/api/v1` 作为接口前缀。

### 构建 Docker 发布包

使用：

```bash
uv run python build_docker.py
```

默认构建产物包括：

- `./simprint-server`
- `./simprint-server-docker-*.tar.gz`

也可以使用这些常见参数：

```bash
uv run python build_docker.py --clean
uv run python build_docker.py --no-package
uv run python build_docker.py --format zip
uv run python build_docker.py --dev --no-package
```

打包后的 `configs/config.toml` 来自仓库中的 `configs/config.example.toml`，真实环境配置文件不会被包含在对外发布的部署包中。

## Status

Simprint Server 最初是作为私有商业后端体系的一部分开发的。当前这个仓库正在为公开开源发布做整理，文档也在同步重写，以便外部使用者更容易理解和部署独立的自托管版本。

仓库中的部分模块划分和命名，仍然会带有早期内部部署模型的痕迹。当前方向是把面向客户端的网关服务整理成一个可以独立部署、便于公开协作的仓库。

## Contributing

这个仓库目前仍处于开源重构阶段，但已经欢迎通过 Issue 和 Pull Request 参与改进。

当前更有价值的贡献方向包括：

- 自托管部署文档和上手流程优化
- 测试覆盖和回归验证补充
- API 文档和路由级使用示例完善
- 打包、发布和 CI 流程改进

如果你准备快速建立上下文，建议先看这些入口：

- `src/main.rs`
- `src/cli.rs`
- `configs/config.example.toml`
- `build_docker.py`
- `docs/`

## License

本项目采用 GNU Affero General Public License v3.0 (AGPLv3) 进行许可。

如果你希望在不履行 AGPLv3 义务的前提下使用 Simprint Server，包括分发修改版本或以闭源服务形式提供修改版本，请联系获取商业许可。
