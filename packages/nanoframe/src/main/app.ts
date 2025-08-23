import { createNanoEvents } from 'nanoevents'
import { RpcClient, withTimeout } from './core'

export class AppImpl {
  #rpc!: RpcClient
  #emitter = createNanoEvents<{ ready: () => void, windowAllClosed: () => void }>()
  whenReady: Promise<void>
  #stopKeepAlive?: () => void

  constructor() { this.whenReady = this.bootstrap() }

  private async bootstrap() {
    const { rpc, stopKeepAlive } = RpcClient.launch()
    this.#rpc = rpc
    this.#stopKeepAlive = stopKeepAlive
    this.#rpc.onNotify((method, params) => {
      if (method === 'window.closed') {
        this.#emitter.emit('windowAllClosed')
      }
    })
    await withTimeout(this.#rpc.call('ping', {}), 10_000, new Error('nanoframe-core ping timeout'))
    this.#emitter.emit('ready')
  }

  on(event: 'ready' | 'windowAllClosed', cb: () => void) { this.#emitter.on(event as any, cb as any) }

  get rpc() { return this.#rpc }

  async createWindow(opts: { title?: string; width?: number; height?: number; url?: string; html?: string; iconPath?: string }) {
    const { BrowserWindow } = await import('./window')
    return BrowserWindow.create(opts as any)
  }

  async openDialog(opts: { title?: string; directory?: boolean; multiple?: boolean; filters?: { name?: string; extensions?: string[] }[] }) {
    await this.whenReady
    return this.#rpc.call('dialog.open', opts)
  }

  async saveDialog(opts: { title?: string; defaultFileName?: string }) {
    await this.whenReady
    const params: any = { ...opts }
    if ('defaultFileName' in params) { params.default_file_name = params.defaultFileName; delete params.defaultFileName }
    return this.#rpc.call('dialog.save', params)
  }

  async getPath(name: 'home' | 'temp' | 'appData' | 'userData', appName?: string) {
    await this.whenReady
    return this.#rpc.call('app.getPath', { name, app_name: appName })
  }

  async openExternal(target: string) { await this.#rpc.call('shell.openExternal', { target }) }
  async writeClipboardText(text: string) { await this.#rpc.call('clipboard.writeText', { text }) }
  async readClipboardText(): Promise<{ text: string }> { return this.#rpc.call('clipboard.readText', {}) }

  quit() {
    if (this.#stopKeepAlive) this.#stopKeepAlive()
    this.#rpc.kill()
  }
}

export const app = new AppImpl()
