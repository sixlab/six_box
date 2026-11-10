import type { Readable, Writable } from 'node:stream';
import path from 'node:path';
import { MAX_FRAME_BYTES, PROTOCOL_VERSION } from '../../plugin-protocol/src/index.ts';
import type { AppPaths, Request, Response, ReadyEvent } from '../../plugin-protocol/src/index.ts';

function isObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}
function isRequest(value: unknown): value is Request {
  return isObject(value) && value.version === PROTOCOL_VERSION && value.type === 'request'
    && Number.isSafeInteger(value.id) && Number(value.id) > 0 && typeof value.method === 'string';
}
function validPaths(value: unknown): value is AppPaths {
  const keys = ['data', 'cache', 'logs', 'plugins', 'pluginData', 'settings', 'database', 'runtimes'];
  return isObject(value) && keys.every(key => typeof value[key] === 'string'
    && path.isAbsolute(value[key]) && !value[key].includes('\0'));
}

export async function runService(input: Readable, output: Writable): Promise<void> {
  let initialized = false;
  let pending = Buffer.alloc(0);
  const send = (message: Response | ReadyEvent): Promise<void> => new Promise((resolve, reject) => {
    output.write(JSON.stringify(message) + '\n', error => error ? reject(error) : resolve());
  });
  const fail = (id: number | null, code: string, message: string) =>
    send({ version: 1, type: 'response', id, error: { code, message } });

  // 不使用 readline：它会无限缓存没有换行的输入。
  for await (const chunk of input) {
    const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    let start = 0;
    while (start < bytes.length) {
      const newline = bytes.indexOf(10, start);
      const end = newline === -1 ? bytes.length : newline;
      if (pending.length + end - start > MAX_FRAME_BYTES) throw new Error('FRAME_TOO_LARGE');
      pending = Buffer.concat([pending, bytes.subarray(start, end)]);
      if (newline === -1) break;
      start = end + 1;
      let message: unknown;
      try { message = JSON.parse(pending.toString('utf8')); }
      catch { pending = Buffer.alloc(0); await fail(null, 'PARSE_ERROR', '消息不是有效 JSON'); continue; }
      pending = Buffer.alloc(0);
      if (!isRequest(message)) { await fail(null, 'INVALID_REQUEST', '协议版本或请求格式无效'); continue; }
      const { id, method, params } = message;
      const reply = (result: unknown) => send({ version: 1, type: 'response', id, result });
      if (method === 'initialize') {
        if (initialized) { await fail(id, 'ALREADY_INITIALIZED', '服务已初始化'); continue; }
        if (!isObject(params) || !validPaths(params.paths)) {
          await fail(id, 'INVALID_PARAMS', '初始化需要 Rust 解析的绝对目录'); continue;
        }
        // M1 尚无插件业务，不创建或重新推导 Rust 管理的目录。
        initialized = true;
        await reply({ initialized: true });
        await send({ version: 1, type: 'event', name: 'ready', payload: {
          pid: process.pid, nodeVersion: process.version, execPath: process.execPath,
        } });
      } else if (method === 'shutdown') {
        await reply({ stopping: true });
        return;
      } else if (!initialized) {
        await fail(id, 'NOT_INITIALIZED', '服务尚未初始化');
      } else if (method === 'ping') {
        await reply({ alive: true });
      } else {
        await fail(id, 'METHOD_NOT_FOUND', '未知管理服务方法');
      }
    }
  }
  if (pending.length) throw new Error('TRUNCATED_FRAME');
}
