# Contributing to Nanoframe

Thanks for your interest! This guide explains how to set up your environment and contribute effectively.

## Development setup

Requirements:
- Node.js 20+
- pnpm 10+
- Rust stable toolchain

Bootstrap:
```powershell
pnpm install
pnpm -r build
```

Run the example during development:
```powershell
$env:NANOF_DEV = "1"
pnpm -C examples/hello-world dev
```

Build core:
```powershell
cargo build -r
```

## Package scripts

- `pnpm -r build` – builds TS packages

## Commit and PR guidelines

- Keep PRs focused and small when possible
- Include tests for user-facing changes
- Update docs (README) for new APIs
- Link related issues (e.g., "Fixes #123")
- The CI must be green before merging

## Coding standards

- TypeScript: follow ESLint and existing code style
- Rust: `cargo fmt` and `cargo clippy` where applicable

## Release process (maintainers)

- Update versions in `package.json`/`Cargo.toml`
- Publish prebuilt core packages first, then JS SDK
- Tag and draft release notes

## Reporting issues

Use our GitHub issue templates. Provide reproduction steps and environment info.
