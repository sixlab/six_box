import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { spawn, execFileSync } from 'node:child_process';
import { setTimeout as delay } from 'node:timers/promises';

const root = path.resolve(import.meta.dirname, '..');
const original = path.resolve(process.argv[2] ?? path.join(root, '.cache/target/aarch64-apple-darwin/release/bundle/macos/SixBox.app'));
const runDir = path.join(root, 'artifacts', `verify-${Date.now()}`);
const log = path.join(os.homedir(), 'Library/Logs/cn.sixlab.box/host.jsonl');
const children = new Set();
const services = new Set();
const results = [];
await fs.mkdir(runDir, { recursive: true });

function alive(pid) { try { process.kill(pid, 0); return true; } catch (error) { if (error.code === 'ESRCH') return false; throw error; } }
async function waitFor(predicate, label, milliseconds = 15000) {
  const end = Date.now() + milliseconds;
  while (Date.now() < end) { const result = await predicate(); if (result) return result; await delay(50); }
  throw new Error(`等待超时：${label}`);
}
async function states(pid) {
  let text;
  try { text = await fs.readFile(log, 'utf8'); } catch (error) { if (error.code === 'ENOENT') return []; throw error; }
  const records = text.split('\n').filter(Boolean).flatMap(line => { try { return [JSON.parse(line)]; } catch { return []; } });
  const start = records.findLastIndex(item => item.event === 'host.start' && item.message === `SixBox pid=${pid}`);
  if (start < 0) return [];
  return records.slice(start + 1).filter(item => item.event === 'runtime.state').map(item => JSON.parse(item.message));
}
async function state(pid, phase, predicate = () => true) {
  return waitFor(async () => {
    const latest = (await states(pid)).at(-1);
    if (latest?.servicePid) services.add(latest.servicePid);
    return latest?.phase === phase && predicate(latest) ? latest : false;
  }, `${pid} ${phase}`);
}
function launch(bundle = original) {
  const child = spawn(path.join(bundle, 'Contents/MacOS/sixbox'), [], {
    env: { ...process.env, PATH: '/usr/bin:/bin', NODE_OPTIONS: '--require /sixbox-must-not-load.cjs', NODE_PATH: '/sixbox-must-not-load' },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let output = '';
  child.stdout.on('data', chunk => { output += chunk; });
  child.stderr.on('data', chunk => { output += chunk; });
  child.on('error', error => { output += error.message; });
  child.on('close', () => { void fs.writeFile(path.join(runDir, `host-${child.pid}.log`), output); });
  children.add(child);
  return child;
}
async function exited(child) { await waitFor(() => child.exitCode !== null || child.signalCode !== null, `宿主 ${child.pid} 退出`); }
async function stop(child) { child.kill('SIGTERM'); await exited(child); assert.equal(child.exitCode, 0); }
function pass(name, details = {}) { results.push({ name, passed: true, ...details }); console.log(`PASS ${name}`); }

try {
  assert.equal(execFileSync('/usr/libexec/PlistBuddy', ['-c', 'Print :CFBundleIdentifier', path.join(original, 'Contents/Info.plist')], { encoding: 'utf8' }).trim(), 'cn.sixlab.box');
  execFileSync('/usr/bin/codesign', ['--verify', '--deep', '--strict', original]);
  pass('正式包标识和 ad-hoc 签名');

  const first = launch();
  let ready = await state(first.pid, 'ready');
  assert.equal(ready.nodeVersion, 'v24.22.2');
  assert.equal(ready.nodePath, path.join(original, 'Contents/MacOS/node'));
  pass('无用户 Node PATH、移除 Node 注入环境后内置服务启动', { hostPid: first.pid, servicePid: ready.servicePid });

  const second = launch();
  await exited(second);
  assert.equal(second.exitCode, 0);
  assert.equal((await state(first.pid, 'ready')).servicePid, ready.servicePid);
  pass('重复启动退出第二实例且不重复启动服务');

  process.kill(ready.servicePid, 'SIGKILL');
  const previous = ready.servicePid;
  ready = await state(first.pid, 'ready', value => value.servicePid !== previous);
  assert.equal(ready.restarts, 1);
  pass('正式包管理服务崩溃后恢复，宿主保持运行');

  process.kill(ready.servicePid, 'SIGSTOP');
  const hung = ready.servicePid;
  ready = await state(first.pid, 'ready', value => value.servicePid !== hung);
  assert.equal(ready.restarts, 2);
  await waitFor(() => !alive(hung), '挂起服务被回收');
  pass('正式包心跳超时恢复并回收挂起服务');

  process.kill(ready.servicePid, 'SIGSTOP');
  first.kill('SIGTERM');
  await delay(100);
  first.kill('SIGTERM');
  await exited(first);
  await waitFor(() => !alive(ready.servicePid), '退出时强制回收服务');
  pass('挂起服务时重复退出请求，宿主和服务均停止');

  const concurrent = Array.from({ length: 6 }, () => launch());
  await delay(2500);
  const running = concurrent.filter(child => child.exitCode === null && child.signalCode === null);
  assert.equal(running.length, 1);
  const owner = running[0];
  const ownerReady = await state(owner.pid, 'ready');
  await stop(owner);
  await waitFor(() => !alive(ownerReady.servicePid), '并发启动测试服务停止');
  pass('同时启动 6 份正式包，仅一个宿主及管理服务');

  const missing = path.join(runDir, 'MissingNode.app');
  execFileSync('/bin/cp', ['-cR', original, missing]);
  await fs.unlink(path.join(missing, 'Contents/MacOS/node'));
  execFileSync('/usr/bin/codesign', ['--force', '--deep', '--sign', '-', missing], { stdio: 'pipe' });
  const noNode = launch(missing);
  const unavailable = await state(noNode.pid, 'unavailable');
  assert.equal(unavailable.error.code, 'NODE_MISSING');
  assert.equal(unavailable.servicePid, null);
  assert.ok(alive(noNode.pid));
  await stop(noNode);
  pass('删除包内 Node 后宿主存活且报告 NODE_MISSING（界面另行人工验收）');

  const orphan = launch();
  const orphanReady = await state(orphan.pid, 'ready');
  orphan.kill('SIGKILL');
  await exited(orphan);
  await waitFor(() => !alive(orphanReady.servicePid), '父进程被终止后服务 EOF 退出');
  const recovered = launch();
  const recoveredReady = await state(recovered.pid, 'ready');
  await stop(recovered);
  await waitFor(() => !alive(recoveredReady.servicePid), '重启服务退出');
  pass('宿主异常退出后服务自行停止，文件锁可重新获取');
} catch (error) {
  results.push({ name: '验收中断', passed: false, error: String(error) });
  process.exitCode = 1;
  console.error(error);
} finally {
  for (const child of children) if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
  for (const pid of services) if (alive(pid)) process.kill(pid, 'SIGKILL');
  await fs.writeFile(path.join(runDir, 'results.json'), JSON.stringify({ platform: `${process.platform}-${process.arch}`, bundle: original, results }, null, 2));
  console.log(`验收记录：${path.join(runDir, 'results.json')}`);
}
