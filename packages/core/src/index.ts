import { runService } from './service.ts';

// 断开父进程 stdin 会结束服务；stdout 严格保留给协议。
process.stdout.on('error', () => process.exit(1));
process.on('SIGTERM', () => process.stdin.destroy());
process.on('SIGINT', () => process.stdin.destroy());
void runService(process.stdin, process.stdout).catch(error => {
  console.error(error instanceof Error ? error.message : 'SERVICE_FAILURE');
  process.exitCode = 1;
});
