# nanoframe (TypeScript SDK)

TypeScript/Node API for Nanoframe, a lightweight desktop runtime backed by a Rust core using system webviews.

## Install

```sh
npm install nanoframe
# or
pnpm add nanoframe
# or
yarn add nanoframe
# or
bun add nanoframe
```

Peer/runtime requirements:
- Node.js 20+
- Platform core binary (auto-resolved) or local cargo-built `nanoframe-core`

## Quick usage

```ts
import { app, BrowserWindow } from 'nanoframe'

await app.whenReady
const win = await BrowserWindow.create({
  title: 'My App',
  width: 800,
  height: 600,
  url: 'https://example.com'
})
await win.openDevTools()
```

## API

### `app: App`

- `whenReady: Promise<void>` – resolves when core is reachable
- `on(event, cb)` – events: `ready`, `windowAllClosed`
- `createWindow(options)` – convenience wrapper, same as `BrowserWindow.create`
- `openDialog(options)` – open file/directory dialog
- `saveDialog(options)` – save dialog
- `getPath(name, appName?)` – resolve OS paths
- `openExternal(target)` – open URL in default handler
- `writeClipboardText(text)` / `readClipboardText()`
- `quit()` – terminate the child core process

Environment flags:
- `NANOF_DEV=1` – prefer local cargo build during development
- `NANOF_FORCE_PLATFORM=1` – force platform package binary when available

### `BrowserWindow`

Constructor is internal; use `BrowserWindow.create(options)`.

Options (subset):
- `title?: string`
- `width?: number`, `height?: number`
- `url?: string` | `html?: string`
- `iconPath?: string`
- `resizable?: boolean`, `alwaysOnTop?: boolean`, `fullscreen?: boolean`
- `decorations?: boolean`, `center?: boolean`
- `preload?: string` (reserved)

Methods:
- `show()`, `hide()`, `close()`
- `setIcon(path)`, `setTitle(title)`
- `setSize(w, h)`, `getSize()`
- `setPosition(x, y)`, `getPosition()`
- `center()`, `focus()`, `maximize()`, `minimize()`, `unminimize()`
- `setAlwaysOnTop(bool)`, `setResizable(bool)`, `setFullscreen(bool)`, `isFullscreen()`
- `setDecorations(bool)`, `isVisible()`
- `eval(code)`, `openDevTools()`
- `postMessage(payload)`

## How binaries are resolved

At runtime, the SDK tries the following in order:
1) If `NANOF_DEV=1` or a local `packages/nanoframe-core` cargo project exists and `NANOF_FORCE_PLATFORM` is not set, run `cargo run --release --bin nanoframe-core` inside that directory.
2) Otherwise, resolve a platform package (`@nanoframe/core-<platform>-<arch>`) and execute the embedded binary.
3) Fallback: try `nanoframe-core` on PATH.

## Example app

See `examples/hello-world` in the monorepo for an end-to-end minimal setup using TypeScript.

## License

ICL – see repository root [LICENSE](https://github.com/infinitejs/nanoframe/blob/main/LICENSE).
