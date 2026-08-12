# Contributing to Simprint

Thanks for considering a contribution to Simprint.

This repository is in an active open-source transition. We welcome bug reports, documentation improvements, CI and packaging fixes, frontend refinements, test coverage, and well-scoped product improvements.

## Before You Start

- Search existing issues and discussions before opening a new one.
- For larger changes, open an issue or discussion first so the direction can be aligned before implementation.
- Keep pull requests focused. Small, reviewable changes move faster than broad refactors.

## Development Setup

### Prerequisites

- Node.js 20+
- `pnpm` 9
- Rust toolchain
- Tauri system prerequisites for your platform

### Local Run

From the repository root:

```bash
pnpm install
cp src-tauri/config.example.toml src-tauri/config.development.toml
cargo tauri dev --features development
```

If you only need frontend iteration, you can also use:

```bash
pnpm dev
```

### Local Backend

Many contributions do not require a local backend. Documentation updates, CI changes, build fixes, and part of the frontend and desktop-shell work can usually be developed without running the full server stack locally.

If your change touches server-backed flows, workspace resources, or API-dependent behavior, you may also need a self-hosted backend during development.

The current bootstrap command for the self-hosted server is documented in [README.md](./README.md). After the server is available, update your local `src-tauri/config.development.toml` so `base_url` points to your development server.

## Useful Commands

Run these from the repository root unless noted otherwise:

```bash
pnpm lint
pnpm format:check
pnpm rust:fmt:check
pnpm rust:check
```

If you changed build, packaging, or release-related code, also run:

```bash
node build.cjs
```

## Pull Request Expectations

- Describe what changed and why.
- Mention any user-facing behavior changes.
- Include validation steps you ran locally.
- Attach screenshots or recordings for UI changes when helpful.
- Avoid mixing unrelated refactors into the same PR.

If your change touches build, release, or workflow files, call that out explicitly in the PR description.

## Issues and Bug Reports

Good bug reports usually include:

- Operating system and environment details
- What you expected to happen
- What actually happened
- Clear reproduction steps
- Screenshots, logs, or error messages when available

## Scope and Collaboration

Current high-value contribution areas include:

- Reproducible bug reports and targeted bug fixes
- Build, packaging, and CI improvements
- Documentation and onboarding improvements
- Frontend UX polish and workflow consistency
- Tests, regression coverage, and release verification

The environment runtime is maintained in this repository under `src-tauri/crates/runtime`; the browser-kernel layer (`simprint-browser-kernel`) remains a separate component. Changes to environment lifecycle behavior should include tests in the embedded runtime crate whenever possible.

## Communication

- English and Chinese are both acceptable in issues and pull requests.
- If you want to contribute on a longer horizon, open an issue or discussion and briefly introduce the areas you want to help maintain.
