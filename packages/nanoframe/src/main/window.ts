import { app } from './app.js'

export interface BrowserWindowOptions {
  title?: string
  width?: number
  height?: number
  x?: number
  y?: number
  show?: boolean
  url?: string
  html?: string
  iconPath?: string
  resizable?: boolean
  alwaysOnTop?: boolean
  fullscreen?: boolean
  decorations?: boolean
  center?: boolean
  preload?: string
  minWidth?: number
  minHeight?: number
  maxWidth?: number
  maxHeight?: number
  contentSize?: boolean
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
  async setBounds(bounds: { x?: number, y?: number, width?: number, height?: number }) { await app.rpc.call('window.setBounds', { windowId: this.id, ...bounds }) }
  async getBounds(): Promise<{ x?: number, y?: number, width: number, height: number }> { return app.rpc.call('window.getBounds', { windowId: this.id }) }
  async center() { await app.rpc.call('window.center', { windowId: this.id }) }
  async setAlwaysOnTop(value: boolean) { await app.rpc.call('window.setAlwaysOnTop', { windowId: this.id, value }) }
  async setResizable(value: boolean) { await app.rpc.call('window.setResizable', { windowId: this.id, value }) }
  async setMinSize(width: number, height: number) { await app.rpc.call('window.setMinSize', { windowId: this.id, width, height }) }
  async setMaxSize(width: number, height: number) { await app.rpc.call('window.setMaxSize', { windowId: this.id, width, height }) }
  async isVisible(): Promise<boolean> { return app.rpc.call('window.isVisible', { windowId: this.id }) }
  async focus() { await app.rpc.call('window.focus', { windowId: this.id }) }
  async maximize() { await app.rpc.call('window.maximize', { windowId: this.id }) }
  async unmaximize() { await app.rpc.call('window.unmaximize', { windowId: this.id }) }
  async isMaximized(): Promise<boolean> { return app.rpc.call('window.isMaximized', { windowId: this.id }) }
  async minimize() { await app.rpc.call('window.minimize', { windowId: this.id }) }
  async unminimize() { await app.rpc.call('window.unminimize', { windowId: this.id }) }
  async restore() { await app.rpc.call('window.restore', { windowId: this.id }) }
  async setFullscreen(value: boolean) { await app.rpc.call('window.setFullscreen', { windowId: this.id, value }) }
  async isFullscreen(): Promise<boolean> { return app.rpc.call('window.isFullscreen', { windowId: this.id }) }
  async setDecorations(value: boolean) { await app.rpc.call('window.setDecorations', { windowId: this.id, value }) }
  async setPosition(x: number, y: number) { await app.rpc.call('window.setPosition', { windowId: this.id, x, y }) }
  async getPosition(): Promise<{ x: number, y: number }> { return app.rpc.call('window.getPosition', { windowId: this.id }) }
  async postMessage(payload: any) { await app.rpc.call('webview.postMessage', { windowId: this.id, payload }) }
  async screenshot(): Promise<{ base64Png: string }> { return app.rpc.call('webview.screenshot', { windowId: this.id }) }
  async requestUserAttention(critical = false) { await app.rpc.call('window.requestUserAttention', { windowId: this.id, critical }) }

  // Lightweight context-menu: inject a JS listener to show a custom menu via postMessage roundtrip
  async enableContextMenu(handlerChannel = 'context-menu') {
    // Wire renderer to post right-click info
    await this.eval(`
      (function(){
        if (window.__nanoframeCtxMenu) return; window.__nanoframeCtxMenu = true;
        window.addEventListener('contextmenu', (ev) => {
          const targetInfo = { tag: ev.target && (ev.target as HTMLElement).tagName, x: ev.clientX, y: ev.clientY };
          (window as any).ipc && (window as any).ipc.postMessage && (window as any).ipc.postMessage(JSON.stringify({ type: '${handlerChannel}', detail: targetInfo }));
        });
      })();
    `)
  }
}

export { BrowserWindow as BrowserWindowImpl }
