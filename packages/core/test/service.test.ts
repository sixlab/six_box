import assert from 'node:assert/strict';
import test from 'node:test';
import { PassThrough } from 'node:stream';
import { runService } from '../src/service.ts';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import path from 'node:path';

const paths = {
  data: '/tmp/sixbox/data', cache: '/tmp/sixbox/cache', logs: '/tmp/sixbox/logs',
  plugins: '/tmp/sixbox/data/plugins', pluginData: '/tmp/sixbox/data/plugin-data',
  settings: '/tmp/sixbox/data/settings', database: '/tmp/sixbox/data/data',
  runtimes: '/tmp/sixbox/data/runtimes/node',
};
const request = (id: number, method: string, params: unknown = {}) =>
  JSON.stringify({ version: 1, type: 'request', id, method, params }) + '\n';
const init = request(1, 'initialize', { paths });

async function exchange(text: string) {
  const input = new PassThrough();
  const output = new PassThrough();
  let wire = '';
  output.on('data', chunk => { wire += chunk.toString(); });
  const running = runService(input, output);
  input.end(text);
  await running;
  return wire.trim().split('\n').filter(Boolean).map(line => JSON.parse(line));
}

test('初始化响应后报告 Ready，ping 及 shutdown 保持请求 ID', async () => {
  const messages = await exchange(init + request(2, 'ping') + request(3, 'shutdown'));
  assert.equal(messages.length, 4);
  assert.deepEqual(messages[0], { version: 1, type: 'response', id: 1, result: { initialized: true } });
  assert.equal(messages[1].name, 'ready');
  assert.equal(messages[1].payload.pid, process.pid);
  assert.equal(messages[1].payload.execPath, process.execPath);
  assert.equal(messages[2].id, 2);
  assert.equal(messages[2].result.alive, true);
  assert.deepEqual(messages[3].result, { stopping: true });
});
test('初始化前拒绝业务调用，初始化后拒绝重复初始化和未知方法', async () => {
  const messages = await exchange(request(4, 'ping') + init + request(5, 'initialize', { paths }) + request(6, 'deleteAll'));
  assert.equal(messages[0].error.code, 'NOT_INITIALIZED');
  assert.equal(messages[3].error.code, 'ALREADY_INITIALIZED');
  assert.equal(messages[4].error.code, 'METHOD_NOT_FOUND');
});
test('拒绝无效版本、相对目录和空请求 ID，且不回显输入', async () => {
  const messages = await exchange(request(1, 'initialize', { paths: { ...paths, data: '../secret' } })
    + '{"version":99,"type":"request","id":2,"method":"ping"}\n'
    + request(0, 'ping') + 'secret-invalid-json\n');
  assert.equal(messages.length, 4);
  assert.equal(messages[0].error.code, 'INVALID_PARAMS');
  assert.equal(messages[1].error.code, 'INVALID_REQUEST');
  assert.equal(messages[2].error.code, 'INVALID_REQUEST');
  assert.equal(messages[3].error.code, 'PARSE_ERROR');
  assert.ok(!JSON.stringify(messages).includes('secret'));
});
test('超长无换行输入立即拒绝，避免无限缓冲', async () => {
  await assert.rejects(exchange('x'.repeat(65537)), /FRAME_TOO_LARGE/);
});
test('EOF 中的半条消息不得执行，shutdown 后不得继续处理', async () => {
  await assert.rejects(exchange('{"version":1'), /TRUNCATED_FRAME/);
  const messages = await exchange(init + request(2, 'shutdown') + request(3, 'ping'));
  assert.equal(messages.length, 3);
});

test('真实管理服务在父进程关闭 stdin 后退出', { timeout: 5000 }, async () => {
  const child = spawn(process.execPath, [path.resolve(import.meta.dirname, '../src/index.ts')], { stdio: ['pipe', 'pipe', 'pipe'] });
  const exit = once(child, 'exit');
  child.stdin.end(init);
  const [code, signal] = await exit;
  assert.equal(code, 0);
  assert.equal(signal, null);
});
