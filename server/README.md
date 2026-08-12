<div align="center">
  <h1>Simprint Server</h1>
  <p>Self-hosted backend service for Simprint workspaces, accounts, environments, proxies, and related runtime APIs.</p>
  <p>
    <img alt="Language Rust 2024" src="https://img.shields.io/badge/language-Rust%202024-f97316?style=flat-square&labelColor=0f172a" />
    <img alt="Framework Axum 0.8" src="https://img.shields.io/badge/framework-Axum%200.8-60a5fa?style=flat-square&labelColor=0f172a" />
    <img alt="Database PostgreSQL" src="https://img.shields.io/badge/database-PostgreSQL-38bdf8?style=flat-square&labelColor=0f172a" />
    <img alt="Cache Redis" src="https://img.shields.io/badge/cache-Redis-f87171?style=flat-square&labelColor=0f172a" />
  </p>
  <p>
    <strong>English</strong> | <a href="./README.zh-CN.md">简体中文</a>
  </p>
</div>

---

## Introduction

Simprint Server is the backend service used by Simprint clients and self-hosted deployments. It exposes the application API, manages authentication and persistent data, initializes encryption and storage resources, and runs embedded database migrations during startup.

It is intended for operators who want to run Simprint inside their own infrastructure instead of depending on a shared hosted backend. The service is configured through a local TOML file and is designed to work with PostgreSQL, Redis, and S3-compatible object storage.

## Why Simprint Server?

Running Simprint in a self-hosted setup usually requires more than just an HTTP server:

- You need control over API availability, credentials, and storage infrastructure.
- You need deployment artifacts that are safe to publish without leaking real environment configuration.
- You need database schema upgrades to happen predictably during release and restart.
- You need one backend entry point that can serve workspace, environment, proxy, and account-related APIs together.

Simprint Server is built around those constraints: a single Rust service, a config-first deployment model, embedded migrations, and packaging that only ships a publish-safe example configuration.

## Features

- **Core application API**: Serves account, workspace, team, environment, proxy, template, preference, message, extension, and local runtime endpoints from one process.
- **Authentication and secret initialization**: Supports login-related flows, token refresh, route whitelists, and RSA secret bootstrap on first startup.
- **Embedded database migrations**: Executes `sqlx` migrations automatically before the HTTP server starts accepting traffic.
- **S3-compatible storage integration**: Configures external object storage for avatars, extension assets, and version-related files.
- **Redis-backed runtime coordination**: Uses Redis for runtime coordination and cache-oriented service flows.
- **Docker-oriented release packaging**: Generates a deployment archive with `Dockerfile`, `docker-compose.yml`, and `configs/config.toml` copied from `configs/config.example.toml`.
- **Config-first execution**: Runs locally and in containers with the same `-f <config.toml>` startup model.

## Quick Start

### Prerequisites

- Rust toolchain
- PostgreSQL 16+ or a compatible PostgreSQL instance
- Redis 7+
- S3-compatible object storage
- Optional SMTP server for email-related flows

### One-line self-hosted server install

Linux servers can bootstrap the self-hosted backend with:

```bash
curl -fsSL https://raw.githubusercontent.com/Simprint/simprint/main/deploy/install-server.sh | bash # Update the client config afterwards, for example: base_url = http://127.0.0.1:40041/api/
```

### Run locally

```bash
cp configs/config.example.toml configs/config.local.toml
# edit configs/config.local.toml
cargo run -- -f configs/config.local.toml
```

The example configuration listens on port `40041` and uses the `/api/v1` prefix by default.

### Build a Docker release package

Use:

```bash
uv run python build_docker.py
```

The default build produces:

- `./simprint-server`
- `./simprint-server-docker-*.tar.gz`

You can also use options such as:

```bash
uv run python build_docker.py --clean
uv run python build_docker.py --no-package
uv run python build_docker.py --format zip
uv run python build_docker.py --dev --no-package
```

The packaged `configs/config.toml` is generated from `configs/config.example.toml`, and real environment-specific config files are intentionally not included in the release archive.

## Status

Simprint Server was originally developed as part of a private commercial backend stack. This repository is now being prepared for a public open-source release, and the documentation is being rewritten to make standalone self-hosted deployment easier to understand.

Some modules and naming still reflect earlier internal deployment assumptions. The current direction is to keep the client-facing gateway service deployable as an independent repository with a cleaner public-facing setup.

## Contributing

This repository is still in an open-source refactoring phase, but issues and pull requests are welcome.

High-value contribution areas include:

- Self-hosted deployment docs and onboarding improvements
- Test coverage and regression verification
- API documentation and route-level usage examples
- Packaging, release, and CI improvements

Useful entry points when exploring the codebase:

- `src/main.rs`
- `src/cli.rs`
- `configs/config.example.toml`
- `build_docker.py`
- `docs/`

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPLv3).

If you want to use Simprint Server in a way that does not comply with the AGPLv3 obligations, including distributing modified versions or providing modified versions as a closed-source service, please contact us for a commercial license.
