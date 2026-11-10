import { spawn } from 'node:child_process';
import path from 'node:path';
import fs from 'node:fs/promises';
import { prepareRuntime } from './prepare-runtime.mjs';
import { buildService, buildFrontend } from './build.mjs';
const root = path.resolve(import.meta.dirname, '..');
const mode = process.argv[2];
if (!['dev', 'build'].includes(mode)) throw new Error('用法：desktop.mjs dev|build');
await prepareRuntime();
await buildService();
const frontend = await buildFrontend(mode === 'dev');
await fs.mkdir(path.join(root, '.cache/tmp'), { recursive: true });
const args = [path.join(root, 'node_modules/@tauri-apps/cli/tauri.js'), mode];
if (mode === 'build') args.push('--target', 'aarch64-apple-darwin', '--bundles', 'app');
const child = spawn(process.execPath, args, {
  cwd: path.join(root, 'apps/desktop'), stdio: 'inherit',
  env: { ...process.env, CARGO_HOME: path.join(root, '.cache/cargo'), CARGO_TARGET_DIR: path.join(root, '.cache/target'), CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS: 'fallback', TMPDIR: path.join(root, '.cache/tmp') },
});
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => child.kill(signal));
child.on('error', error => { console.error(error.message); process.exitCode = 1; });
child.on('close', async code => { if (frontend && 'dispose' in frontend) await frontend.dispose(); process.exitCode = code ?? 1; });
