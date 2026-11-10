import { spawnSync } from 'node:child_process';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '..');
const result = spawnSync('cargo', process.argv.slice(2), {
  cwd: path.join(root, 'apps/desktop/src-tauri'), stdio: 'inherit',
  env: { ...process.env, CARGO_HOME: path.join(root, '.cache/cargo'), CARGO_TARGET_DIR: path.join(root, '.cache/target'),
    CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS: 'fallback' },
});
if (result.error) console.error(result.error.message);
process.exit(result.status ?? 1);
