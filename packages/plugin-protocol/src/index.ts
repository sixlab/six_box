// M1 的宿主/管理服务协议；插件 SDK 留待 M2 按实际需求定义。
export const PROTOCOL_VERSION = 1;
export const MAX_FRAME_BYTES = 65536;

export interface AppPaths {
  data: string;
  cache: string;
  logs: string;
  plugins: string;
  pluginData: string;
  settings: string;
  database: string;
  runtimes: string;
}
export interface ProtocolError { code: string; message: string }
export interface Request {
  version: 1;
  type: 'request';
  id: number;
  method: string;
  params: unknown;
}
export type Response = { version: 1; type: 'response'; id: number | null } & (
  { result: unknown; error?: never } | { error: ProtocolError; result?: never }
);
export interface ReadyEvent {
  version: 1;
  type: 'event';
  name: 'ready';
  payload: { pid: number; nodeVersion: string; execPath: string };
}
export type RuntimePhase = 'detecting' | 'starting' | 'ready' | 'recovering' | 'unavailable' | 'failed' | 'stopping' | 'stopped';
export interface RuntimeStatus {
  phase: RuntimePhase;
  source: 'bundled' | 'managed' | 'system' | 'custom';
  nodePath: string;
  nodeVersion: string | null;
  servicePid: number | null;
  restarts: number;
  error: ProtocolError | null;
}
export interface Snapshot {
  paths: AppPaths | null;
  runtime: RuntimeStatus;
}
