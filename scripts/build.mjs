import { build, context } from 'esbuild';
import fs from 'node:fs/promises';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '..');
export async function buildService() {
  await build({ absWorkingDir: root, entryPoints: ['packages/core/src/index.ts'], bundle: true, platform: 'node', format: 'cjs', target: 'node24', outfile: 'apps/desktop/src-tauri/resources/service.cjs' });
}
export async function buildFrontend(dev = false) {
  const outdir = path.join(root, 'apps/desktop/dist');
  await fs.mkdir(outdir, { recursive: true });
  await fs.copyFile(path.join(root, 'apps/desktop/index.html'), path.join(outdir, 'index.html'));
  const options = { absWorkingDir: root, entryPoints: ['apps/desktop/src/main.tsx'], bundle: true, format: 'esm', target: 'safari16.4', outdir, entryNames: 'app', minify: !dev, sourcemap: dev, define: { 'process.env.NODE_ENV': JSON.stringify(dev ? 'development' : 'production') } };
  if (!dev) return build(options);
  const ctx = await context(options);
  await ctx.watch();
  await ctx.serve({ host: '127.0.0.1', port: 1420, servedir: outdir });
  return ctx;
}
if (process.argv[1] === import.meta.filename) {
  await buildService();
  if (!process.argv.includes('--service-only')) await buildFrontend();
}
