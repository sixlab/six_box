# SixBox

**All Your Tools. One Box.**

[简体中文](README.md) · English

SixBox is a local-first desktop toolbox by **SixLab（六楼实验室）**, designed to bring everyday utilities, plugins, and quick actions into one application.

[GitHub](https://github.com/sixlab/six_box) · [Report an issue](https://github.com/sixlab/six_box/issues) · [AGPL-3.0-only](LICENSE)

## Project status

SixBox is in early development. **M1, the project and runtime foundation, is complete.** The current build and validation target is **macOS on Apple Silicon**.

Implemented:

- A Tauri 2 + React + TypeScript desktop shell with runtime status and diagnostics.
- Rust-managed system data directories, single-instance behavior, windows, and application shutdown.
- A bundled Node.js runtime; end users do not need to install Node.js.
- Service startup, health checks, failure recovery, manual retry, and shutdown cleanup.
- A usable shell and diagnostics even when the bundled Node.js executable is missing.

The plugin system, full management pages, global quick panel, and individual tools are not implemented yet. This version is intended for development and validation; a production installer for end users is not available yet.

## Development requirements

| Dependency | Requirement |
|---|---|
| Operating system | macOS, Apple Silicon / arm64 |
| Native toolchain | Xcode Command Line Tools |
| Rust | 1.87 or later, with `clippy` and `rustfmt` |
| Node.js | 24.x, version 24.14.0 or later |
| pnpm | 12.15.0 |

```bash
git clone https://github.com/sixlab/six_box.git
cd six_box
pnpm install --frozen-lockfile
pnpm dev
```

The first run downloads and verifies the official Node.js **24.22.2** distribution for bundling. Your development Node.js / pnpm installation is used for building; the application runs its own bundled Node.js.

See the [development guide](docs/DEVELOPMENT.md) (Chinese) for offline runtime preparation, project structure, and the communication protocol.

## Checks and builds

```bash
pnpm check          # Type checks, TypeScript / Rust tests, clippy, and rustfmt
pnpm build          # Build the release application for macOS arm64
pnpm verify:bundle  # Verify real application processes; quit SixBox first
```

Build output:

```text
.cache/target/aarch64-apple-darwin/release/bundle/macos/SixBox.app
```

Builds currently use a local ad-hoc signature. Developer ID signing, notarization, and a DMG installer are not configured. The declared minimum is macOS 13.5; testing has covered macOS 26.6.2 only. Other macOS versions have not been validated. Windows, Linux, and Intel Macs are not supported yet.

`verify:bundle` requires a macOS graphical session and uses standard system application directories. It only terminates processes started during that run and does not clear existing user data. Results are saved to `artifacts/verify-*/results.json`.

## Roadmap

| Milestone | Scope | Status |
|---|---|---|
| M1 | Project, desktop shell, bundled runtime, and system directories | Complete |
| M2 | Plugin SDK, installation, and lifecycle management | Planned |
| M3 | Independent windows, management pages, and a unified tray | Planned |
| M4 | Command registry and a global quick panel | Planned |
| M5 | Network monitoring and JSON tools | Planned |
| M6 | Safe manual updates, long-running tests, and release validation | Planned |

Accounts, cloud synchronization, and Pro features are future plans outside M1. The official server is planned as a separately maintained, closed-source component.

## Documentation and contributions

- [Product requirements](docs/PRD.md) (Chinese)
- [Architecture](docs/ARCHITECTURE.md) (Chinese)
- [Development, protocol, and builds](docs/DEVELOPMENT.md) (Chinese)
- [M1 validation report and limitations](docs/M1_REPORT.md) (Chinese)
- [PoC review and reuse decisions](docs/POC_REVIEW.md) (Chinese)
- [Contributing](CONTRIBUTING.md) (Chinese / English)
- [Licensing and commercial use](docs/LICENSING.md) (Chinese / English)
- [Third-party components](THIRD_PARTY_NOTICES.md) (Chinese / English)

Use GitHub Issues to report problems or discuss ideas. Please read the contribution guide before submitting code. The current development scope remains M1.

## License

Copyright © 2026 SixLab（六楼实验室）.

Except for third-party components with their own notices, this repository is licensed under the **GNU Affero General Public License v3.0 only**, identified by `AGPL-3.0-only`. See [LICENSE](LICENSE) for the full terms and [COPYRIGHT](COPYRIGHT) for the project license notice.

**The AGPL permits commercial use.** Distribution of the software or derivative versions, and remote network interaction with modified versions, carry corresponding source obligations as specified by the license. No noncommercial restriction is added. Independent official server software and separately licensed products do not automatically become open source because this repository is public; see the [licensing guide](docs/LICENSING.md).
