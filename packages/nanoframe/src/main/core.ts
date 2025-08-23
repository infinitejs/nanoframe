import { execa, ExecaChildProcess } from 'execa'
import { Readable } from 'node:stream'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import fs from 'node:fs'
import { createRequire } from 'node:module'

export type Json = any

type RpcPending = { resolve: (v: Json) => void; reject: (e: Error) => void }

export class RpcClient {
  private child: ExecaChildProcess
  private nextId = 1
  private pending = new Map<number, RpcPending>()
  private notifyCb?: (method: string, params: Json) => void

  constructor(child: ExecaChildProcess) {
    this.child = child
    if (!child.stdout || !child.stdin) throw new Error('Child process pipes not attached')
    this.readLoop(child.stdout)
  }

  private async readLoop(stream: Readable) {
    let buf = ''
    for await (const chunk of stream) {
      buf += chunk.toString('utf8')
      let nl
      while ((nl = buf.indexOf('\n')) !== -1) {
        const line = buf.slice(0, nl)
        buf = buf.slice(nl + 1)
        if (!line.trim()) continue
        try {
          const msg = JSON.parse(line)
          if ('id' in msg && ('result' in msg || 'error' in msg)) {
            const id = typeof msg.id === 'number' ? msg.id : undefined
            if (id && this.pending.has(id)) {
              const { resolve, reject } = this.pending.get(id)!
              this.pending.delete(id)
              if (msg.error) reject(new Error(msg.error.message))
              else resolve(msg.result)
            }
          } else if (msg && typeof msg.method === 'string') {
            this.notifyCb?.(msg.method, msg.params)
          }
        } catch {
          // ignore parse errors
        }
      }
    }
  }

  call(method: string, params: Json): Promise<Json> {
    const id = this.nextId++
    const payload = JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n'
    this.child.stdin!.write(payload)
    return new Promise((resolve, reject) => { this.pending.set(id, { resolve, reject }) })
  }

  kill(signal: number | undefined = undefined) { this.child.kill(signal) }

  onNotify(cb: (method: string, params: Json) => void) { this.notifyCb = cb }

  static launch(): { rpc: RpcClient; child: ExecaChildProcess; stopKeepAlive: () => void } {
    const isDev = process.env.NANOF_DEV === '1'
    let child: ExecaChildProcess
    const here = fileURLToPath(new URL('.', import.meta.url))
    const coreDir = path.resolve(here, '../../../nanoframe-core')
    const platformBin = resolvePlatformPackageBinary()
    const hasLocalCore = fs.existsSync(path.join(coreDir, 'Cargo.toml'))
    const preferLocalCargo = isDev || (hasLocalCore && process.env.NANOF_FORCE_PLATFORM !== '1')
    if (preferLocalCargo) child = execa('cargo', ['run', '--quiet', '--release', '--bin', 'nanoframe-core'], { stdio: ['pipe', 'pipe', 'inherit'], cwd: coreDir })
    else if (platformBin) child = execa(platformBin, [], { stdio: ['pipe', 'pipe', 'inherit'] })
    else child = execa('nanoframe-core', [], { stdio: ['pipe', 'pipe', 'inherit'] })
    const rpc = new RpcClient(child)
    const keepAlive = setInterval(() => {}, 1 << 30)
    const stopKeepAlive = () => clearInterval(keepAlive)
    child.on('exit', (code: number | null) => {
      stopKeepAlive()
      const exitCode = typeof code === 'number' ? code : 0
      setTimeout(() => process.exit(exitCode), 0)
    })
    return { rpc, child, stopKeepAlive }
  }
}

function resolvePlatformPackageBinary(): string | null {
  const binName = process.platform === 'win32' ? 'nanoframe-core.exe' : 'nanoframe-core'
  const map: Record<string, string> = {
    'win32-x64': '@nanoframe/core-win32-x64',
    'win32-arm64': '@nanoframe/core-win32-arm64',
    'darwin-x64': '@nanoframe/core-darwin-x64',
    'darwin-arm64': '@nanoframe/core-darwin-arm64',
    'linux-x64': '@nanoframe/core-linux-x64',
  }
  const key = `${process.platform}-${process.arch}`
  const pkg = map[key]
  if (!pkg) return null
  try {
    const req = createRequire(import.meta.url)
    const pkgJsonPath = req.resolve(`${pkg}/package.json`)
    const pkgDir = path.dirname(pkgJsonPath)
    const full = path.join(pkgDir, 'bin', binName)
    if (fs.existsSync(full)) return full
  } catch {}
  return null
}

export function withTimeout<T>(p: Promise<T>, ms: number, err: Error): Promise<T> {
  let to: NodeJS.Timeout
  return new Promise((resolve, reject) => {
    to = setTimeout(() => reject(err), ms)
    p.then(v => { clearTimeout(to); resolve(v) }, e => { clearTimeout(to); reject(e) })
  })
}
