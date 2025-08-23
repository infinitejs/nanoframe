declare module 'execa' {
  export type ExecaChildProcess = any
  export function execa(cmd: string, args?: string[], opts?: any): ExecaChildProcess
}
declare module 'nanoevents' {
  export function createNanoEvents<T extends Record<string, any>>(): any
}
declare module 'node:stream' {
  import * as stream from 'stream'
  export = stream
}
declare module 'node:path' {
  import * as path from 'path'
  export = path
}
declare module 'node:url' {
  export function fileURLToPath(u: any): string
}

declare var process: any
declare var importMeta: any
