import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '..');
const version = '24.22.2';
const archiveName = `node-v${version}-darwin-arm64.tar.gz`;
const archiveHash = 'aa9549349e47dffaa42dcf95d6e06ade8c4bdafc980979883b88767670df385a';
const binaryHash = 'b6fb419f0f43bb2fbca273700294dd2f3b685a64b60e8417c295f47646a70e4a';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

export async function prepareRuntime() {
  if (process.platform !== 'darwin' || process.arch !== 'arm64') throw new Error('M1 仅构建 macOS Apple Silicon');
  const cache = path.join(root, '.cache/node');
  await fs.mkdir(cache, { recursive: true });
  const archive = process.env.SIXBOX_NODE_ARCHIVE || path.join(cache, archiveName);
  try { await fs.access(archive); }
  catch {
    if (process.env.SIXBOX_NODE_ARCHIVE) throw new Error('SIXBOX_NODE_ARCHIVE 文件不存在');
    const response = await fetch(`https://nodejs.org/dist/v${version}/${archiveName}`, { signal: AbortSignal.timeout(120000) });
    if (!response.ok) throw new Error(`下载 Node 失败：${response.status}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    if (sha256(bytes) !== archiveHash) throw new Error('Node 官方归档 SHA-256 不匹配');
    await fs.writeFile(archive, bytes);
  }
  if (sha256(await fs.readFile(archive)) !== archiveHash) throw new Error('Node 官方归档 SHA-256 不匹配');
  const staging = await fs.mkdtemp(path.join(cache, 'extract-'));
  try {
    execFileSync('/usr/bin/tar', ['-xzf', archive, '-C', staging]);
    const distribution = path.join(staging, `node-v${version}-darwin-arm64`);
    const source = path.join(distribution, 'bin/node');
    if (sha256(await fs.readFile(source)) !== binaryHash) throw new Error('Node 可执行文件 SHA-256 不匹配');
    const env = { ...process.env, NODE_OPTIONS: '', NODE_PATH: '', PATH: '/usr/bin:/bin' };
    const info = JSON.parse(execFileSync(source, ['-p', 'JSON.stringify({version:process.version,arch:process.arch,platform:process.platform})'], { encoding: 'utf8', env, timeout: 5000 }));
    if (info.version !== `v${version}` || info.arch !== 'arm64' || info.platform !== 'darwin') throw new Error('Node 版本或架构错误');
    const libraries = execFileSync('/usr/bin/otool', ['-L', source], { encoding: 'utf8' }).split('\n').slice(1).filter(line => line.trim());
    if (libraries.some(line => !/^\s*\/(usr\/lib|System\/Library)\//.test(line))) throw new Error('Node 依赖非系统动态库');
    const tauri = path.join(root, 'apps/desktop/src-tauri');
    await fs.mkdir(path.join(tauri, 'binaries'), { recursive: true });
    await fs.mkdir(path.join(tauri, 'resources'), { recursive: true });
    const target = path.join(tauri, 'binaries/node-aarch64-apple-darwin');
    await fs.copyFile(source, target);
    await fs.chmod(target, 0o755);
    await fs.copyFile(path.join(distribution, 'LICENSE'), path.join(tauri, 'resources/NODE-LICENSE'));
    await fs.writeFile(path.join(tauri, 'resources/node-runtime.json'), JSON.stringify({ ...info, archive: archiveName, archiveSha256: archiveHash, binarySha256: binaryHash }, null, 2) + '\n');
    console.log(`已准备 Node ${info.version}（darwin arm64，SHA-256 已校验）`);
  } finally { await fs.rm(staging, { recursive: true, force: true }); }
}
if (process.argv[1] === import.meta.filename) await prepareRuntime();
