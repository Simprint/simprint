<div align="center">
  <img src="./public/assets/logo.png" alt="Simprint Logo" width="120" />
  <h1>Simprint</h1>
  <p>Desktop workspace for browser environments, proxy resources, and automation workflows.</p>
  <p>
    <img alt="License AGPLv3" src="https://img.shields.io/badge/license-AGPLv3-67e8f9?style=flat-square&labelColor=0f172a" />
    <img alt="Desktop Tauri 2" src="https://img.shields.io/badge/desktop-Tauri%202-f59e0b?style=flat-square&labelColor=0f172a" />
    <img alt="UI React 19" src="https://img.shields.io/badge/ui-React%2019-60a5fa?style=flat-square&labelColor=0f172a" />
    <img alt="Runtime Rust 2024" src="https://img.shields.io/badge/runtime-Rust%202024-f97316?style=flat-square&labelColor=0f172a" />
  </p>
  <table border="1" cellspacing="0" cellpadding="12">
    <tr>
      <td align="center">
        <strong>👉 Join the Simprint Community</strong><br />
        <sub>Join now for updates, questions, and contributor discussions.</sub><br />
        <sub>👇 Pick a group below</sub><br /><br />
        <a href="https://t.me/simprintapp"><img alt="Telegram Community" src="https://img.shields.io/badge/Telegram-@simprintapp-27a7e7?style=for-the-badge" /></a>
        <img alt="QQ Group 1105174006" src="https://img.shields.io/badge/QQ%20Group-1105174006-12b7f5?style=for-the-badge" />
      </td>
    </tr>
  </table>
  <p>
    <strong>English</strong> | <a href="./README.zh-CN.md">简体中文</a>
  </p>
</div>

<p align="center">
  <img src="./docs/assets/demo-gif.gif" alt="Simprint product demo" width="100%" />
</p>

---

## Introduction

Simprint is a desktop workspace for browser-driven operations, designed to organize browser profiles, proxy resources, automation flows, and local runtime capabilities in one place.

It is intended for individuals and teams that need to maintain multiple browser work environments over time, including scenarios such as cross-border operations, account management, automated task execution, and shared resource coordination. With a unified desktop entry point, Simprint helps manage environment lifecycles more consistently, connect external resources, and build reusable workflows around daily operations.

## Why Simprint?

Most browser automation and browser workspace products are still shaped by a few recurring limitations:

- Closed-source products that hide implementation details and reduce long-term trust.
- Cloud-only products that force operational workflows and sensitive data into third-party infrastructure.
- Anti-user product decisions that restrict ownership, portability, and control over browser environments.
- Rigid systems that are difficult to extend, automate, or integrate into custom workflows.

Simprint is being built to take a different direction: an open, programmable browser workspace for developers, researchers, operators, and automation-heavy teams. The goal is to make browser environments easier to control locally, easier to integrate with surrounding tools, and easier to evolve as workflows become more technical and more AI-assisted.

## Features

- **Isolated browser environments**: Run multiple browser workspaces with separated state and operational boundaries.
- **Persistent browser profiles**: Keep long-lived browser profiles, account context, and workspace state organized over time.
- **Proxy orchestration**: Connect, assign, and manage proxy resources across environments and workflow scenarios.
- **Fingerprint configuration**: Control environment-level browser characteristics and continue refining fingerprint-related behavior.
- **Local automation runtime**: Build and run repeatable browser workflows for daily operations and task execution.
- **Chromium-based desktop runtime**: Run the workspace through a local Chromium-oriented Tauri + Rust desktop runtime with integrated frontend and system-level services.
- **Syncer**: Coordinate multiple running environments, choose a master session, and mirror interaction flows across selected windows.
- **RPC bridge**: Use the built-in Tauri command bridge between the React frontend and Rust services for desktop-native operations and orchestration.
- **Local API**: Expose workspace resources such as environments, proxies, tags, groups, and browser kernels through a local runtime API.
- **MCP**: Run a local Model Context Protocol service so external AI clients can connect to Simprint-managed tools and workspace resources.

## Quick Start

### Prerequisites

- Node.js 20+
- `pnpm`
- Rust toolchain
- Tauri system prerequisites for your platform

### One-line self-hosted server install

Linux servers can bootstrap the self-hosted backend with:

```bash
curl -fsSL https://raw.githubusercontent.com/Simprint/simprint/main/deploy/install-server.sh | bash # Update the client config afterwards, for example: base_url = http://127.0.0.1:40041/api/
```

### Run locally

```bash
pnpm install
cp src-tauri/config.example.toml src-tauri/config.development.toml
cargo tauri dev --features development
```

## Status

Simprint was originally developed as a commercial product. It is now being transitioned into an open-source project, and the current open-source edition is intended to provide all core functionality without feature gating.

Some billing-related UI, upgrade prompts, or commercial entry points may still appear in the product as remnants of the previous commercial model. These interfaces are being phased out and will be removed over time, while the related functionality will remain openly available in the community edition.

## Roadmap

- **AI workflows**: Expand AI-assisted operational flows and agent-oriented task orchestration.
- **Private deployment**: Completed. Self-hosted and enterprise-controlled deployment support is now available.
- **Fingerprint research**: Continue refining browser environment controls, compatibility, and research depth.
- **Automation SDK**: Provide a more reusable interface for building and integrating automation capabilities.

## Data Use

Most users of the open-source edition will rely on community-hosted services, and related data may therefore be stored on community-managed servers. The community does not proactively disclose user data to third parties, but each user remains responsible for assessing their own data risk and should avoid submitting sensitive information whenever possible.

## Contributing

Simprint is in an active open-source transition, and we are looking for early contributors and future maintainers.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup, contribution workflow, and pull request expectations.

Current high-value contribution areas include:

- Bug reports with reproducible steps and environment details
- Build, packaging, and CI improvements
- Documentation, onboarding, and local development experience
- Frontend UX polish and workflow consistency
- Tests, regression coverage, and release verification

Issues and pull requests are welcome. If you are interested in contributing on a longer horizon, please open an issue or discussion to introduce yourself and mention the areas you want to help maintain.

## Friend Links

- [LINUX DO - 新的理想型社区](https://linux.do/)

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPLv3).

If you want to use Simprint in a way that does not comply with the AGPLv3 obligations, including distributing modified versions or providing modified versions as a closed-source service, please contact us for a commercial license.
