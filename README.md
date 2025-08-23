<div align="center">
<img src="./assets/logo.png" width="100" height="100" />

<h1>Nanoframe</h1>
<p>Lightweight, fast desktop apps with system webviews. Nanoframe pairs a tiny Rust core (tao + wry) with a TypeScript API, connected via JSON‑RPC over stdio. Think “Electron‑like DX without bundling Chromium”.</p>
</div>

Status: experimental, but usable for prototypes and internal tools.

## Highlights

- System webviews (no bundled Chromium)
- Cross‑platform Rust core with prebuilt binaries per OS/arch
- Simple Node/TypeScript API: `app` + `BrowserWindow`
- JSON‑RPC transport over stdio (no local servers)

## Repository layout

- `packages/nanoframe` – TypeScript SDK (Node entry point)
- `packages/nanoframe-core` – Rust core (`nanoframe-core` binary)
- `packages/nanoframe-core-*` – Prebuilt platform binary packages
- `examples/hello-world` – Minimal example using the SDK

## Prerequisites

- Node.js 20+
- pnpm 19+
- Rust stable toolchain (for local dev or when building core from source)
- Windows: WebView2 runtime installed; macOS: WebKit is present; Linux: wry dependencies (WebKitGTK)

## Quick start (run the example)

Windows PowerShell:

```powershell
pnpm install
cargo build -r
$env:NANOF_DEV = "1"  # prefer local cargo-built core during development
pnpm -C examples/hello-world dev
```

On macOS/Linux, use your shell’s equivalent of setting `NANOF_DEV=1`.

The example will open a window pointed at https://example.com, print to the webview console, then demo a few window APIs.

## Using Nanoframe in your app

Install from npm (using pnpm here):

```bash
npm install nanoframe
# or
pnpm add nanoframe
# or
yarn add nanoframe
# or
bun add nanoframe
```

Create a window:

```ts
import { app, BrowserWindow } from 'nanoframe';

await app.whenReady;
const win = await BrowserWindow.create({
	title: 'My App',
	width: 1024,
	height: 768,
	url: 'https://example.com'
});
await win.openDevTools();
```

See the full package API in [packages/nanoframe/README.md](packages/nanoframe/README.md).

## How it works

- The JS SDK launches the `nanoframe-core` process.
- Calls from JS are sent via JSON‑RPC over stdio to the Rust core.
- The Rust core manages windows (tao) and webviews (wry), then returns results back over the same channel.
- In dev, set `NANOF_DEV=1` to run the local cargo build; otherwise the SDK resolves a platform binary package when available.

Environment variables:

- `NANOF_DEV=1` – Prefer local cargo build of `nanoframe-core`.
- `NANOF_FORCE_PLATFORM=1` – Force using a platform binary package even if a local core is present.

## Development

```powershell
pnpm install
pnpm -r build   # builds TS packages
cargo build -r  # builds the Rust core
```

Run the example during development:

```powershell
$env:NANOF_DEV = "1"
pnpm -C examples/hello-world dev
```

## Contributing

We welcome issues and PRs. Please read:

- [CONTRIBUTING.md](CONTRIBUTING.md) – setup, development flow, commit/PR conventions
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) – community standards
- [SECURITY.md](SECURITY.md) – how to report vulnerabilities

## License

This project is licensed under the Infinite Clause License (ICL). See [LICENSE](LICENSE) for details.
