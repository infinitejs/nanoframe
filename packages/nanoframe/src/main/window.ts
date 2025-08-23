import { app } from './app'

export interface BrowserWindowOptions {
  title?: string
  width?: number
  height?: number
  url?: string
  html?: string
  iconPath?: string
  resizable?: boolean
  alwaysOnTop?: boolean
  fullscreen?: boolean
  decorations?: boolean
  center?: boolean
  preload?: string
}

export class BrowserWindow {
  id: string
  private constructor(id: string) { this.id = id }

  static async create(opts: BrowserWindowOptions) {
    await app.whenReady
    const params: any = { ...opts }
    if ('iconPath' in params) { params.icon_path = params.iconPath; delete params.iconPath }
    const res = await app.rpc.call('createWindow', params)
    return new BrowserWindow(res.windowId)
  }

  async show() { await app.rpc.call('window.show', { windowId: this.id }) }
  async hide() { await app.rpc.call('window.hide', { windowId: this.id }) }
  async close() { await app.rpc.call('window.close', { windowId: this.id }) }
  async setIcon(iconPath: string) { await app.rpc.call('window.setIcon', { windowId: this.id, iconPath }) }
  async eval(code: string) { await app.rpc.call('webview.eval', { windowId: this.id, code }) }
  async openDevTools() { await app.rpc.call('webview.openDevtools', { windowId: this.id }) }
  async setTitle(title: string) { await app.rpc.call('window.setTitle', { windowId: this.id, title }) }
  async setSize(width: number, height: number) { await app.rpc.call('window.setSize', { windowId: this.id, width, height }) }
  async getSize(): Promise<{ width: number, height: number }> { return app.rpc.call('window.getSize', { windowId: this.id }) }
  async center() { await app.rpc.call('window.center', { windowId: this.id }) }
  async setAlwaysOnTop(value: boolean) { await app.rpc.call('window.setAlwaysOnTop', { windowId: this.id, value }) }
  async setResizable(value: boolean) { await app.rpc.call('window.setResizable', { windowId: this.id, value }) }
  async isVisible(): Promise<boolean> { return app.rpc.call('window.isVisible', { windowId: this.id }) }
  async focus() { await app.rpc.call('window.focus', { windowId: this.id }) }
  async maximize() { await app.rpc.call('window.maximize', { windowId: this.id }) }
  async minimize() { await app.rpc.call('window.minimize', { windowId: this.id }) }
  async unminimize() { await app.rpc.call('window.unminimize', { windowId: this.id }) }
  async setFullscreen(value: boolean) { await app.rpc.call('window.setFullscreen', { windowId: this.id, value }) }
  async isFullscreen(): Promise<boolean> { return app.rpc.call('window.isFullscreen', { windowId: this.id }) }
  async setDecorations(value: boolean) { await app.rpc.call('window.setDecorations', { windowId: this.id, value }) }
  async setPosition(x: number, y: number) { await app.rpc.call('window.setPosition', { windowId: this.id, x, y }) }
  async getPosition(): Promise<{ x: number, y: number }> { return app.rpc.call('window.getPosition', { windowId: this.id }) }
  async postMessage(payload: any) { await app.rpc.call('webview.postMessage', { windowId: this.id, payload }) }
}

export { BrowserWindow as BrowserWindowImpl }
