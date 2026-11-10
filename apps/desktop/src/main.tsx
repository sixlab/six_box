import { StrictMode, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import type { Snapshot, RuntimePhase } from '@sixbox/plugin-protocol';
import './style.css';

const labels: Record<RuntimePhase, string> = {
  detecting: '正在检测运行环境', starting: '正在启动服务', ready: '运行环境已就绪',
  recovering: '正在恢复服务', unavailable: '运行环境不可用', failed: '服务暂时不可用',
  stopping: '正在停止服务', stopped: '服务已停止',
};
function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const refresh = async () => {
      try {
        const value = await invoke<Snapshot>('get_snapshot');
        if (!cancelled) { setSnapshot(value); setError(''); }
      } catch { if (!cancelled) setError('暂时无法获取应用状态，请尝试重新打开 SixBox。'); }
      finally { if (!cancelled) timer = setTimeout(refresh, 1000); }
    };
    void refresh();
    return () => { cancelled = true; clearTimeout(timer); };
  }, []);
  const retry = async () => {
    setBusy(true);
    try { await invoke('retry_runtime'); setError(''); }
    catch (reason) { setError(typeof reason === 'object' && reason && 'message' in reason ? String(reason.message) : '重试失败，请稍后再试。'); }
    finally { setBusy(false); }
  };
  const runtime = snapshot?.runtime;
  const unavailable = runtime?.phase === 'unavailable' || runtime?.phase === 'failed';
  const directories = snapshot?.paths;
  return <div className="shell">
    <aside className="sidebar">
      <div className="brand"><span className="logo" aria-hidden="true">{Array.from({ length: 6 }, (_, i) => <i key={i} />)}</span><strong>SixBox</strong></div>
      <div className="nav-current"><span aria-hidden="true">◈</span> 工具箱</div>
      <div className="sidebar-footer"><span>SixLab</span><small>六楼实验室</small><small>v0.1.0 · M1</small></div>
    </aside>
    <main>
      <header><div><p className="eyebrow">YOUR EVERYDAY TOOLBOX</p><h1>常用工具，一盒搞定。</h1><p className="intro">欢迎使用 SixBox 工具箱。</p></div><span className="version">M1 基础版本</span></header>
      <section className="welcome" aria-labelledby="tools-title"><div className="box-mark" aria-hidden="true">＋</div><h2 id="tools-title">工具箱，准备就绪</h2><p>这里将收纳你的常用工具。<br />当前版本提供应用基础与运行环境诊断。</p><span className="quiet-tag">插件功能将在后续版本开放</span></section>
      <section className="runtime" aria-labelledby="runtime-title">
        <div className="section-head"><h2 id="runtime-title">运行环境</h2><span className={`status ${runtime?.phase === 'ready' ? 'healthy' : unavailable ? 'warning' : ''}`} role="status"><i />{runtime ? labels[runtime.phase] : '正在连接应用'}</span></div>
        <p className="runtime-copy">{unavailable ? '基础界面仍可正常使用。恢复运行环境后，即可使用依赖它的功能。' : '内置 Node.js 为工具提供本地运行环境。'}</p>
        {runtime?.error && <div className="notice" role="alert">{runtime.error.message}</div>}
        {error && <div className="notice" role="alert">{error}</div>}
        <div className="runtime-bottom"><span>Node.js <strong>{runtime?.nodeVersion?.replace(/^v/, '') ?? '等待检测'}</strong><span className="divider">/</span> 随应用内置</span>{unavailable && <button onClick={() => void retry()} disabled={busy || !directories}>{busy ? '正在重试…' : '重新检测'}</button>}</div>
        <details><summary>诊断信息与存储位置</summary><dl>
          <dt>应用标识</dt><dd>cn.sixlab.box</dd>
          <dt>运行时路径</dt><dd>{runtime?.nodePath || '—'}</dd>
          <dt>管理服务</dt><dd>{runtime?.servicePid ? `PID ${runtime.servicePid}` : '未运行'} · 自动恢复 {runtime?.restarts ?? 0} 次</dd>
          {runtime?.error && <><dt>错误代码</dt><dd>{runtime.error.code}</dd></>}
          <dt>应用数据</dt><dd>{directories?.data ?? '不可用'}</dd>
          <dt>缓存</dt><dd>{directories?.cache ?? '不可用'}</dd>
          <dt>日志</dt><dd>{directories?.logs ?? '不可用'}</dd>
        </dl></details>
      </section>
      <footer><span>本地优先 · 由 SixLab 开发</span><button className="text-button" onClick={() => { void invoke('quit_app').catch(() => setError('退出失败，请重试。')); }}>完全退出 SixBox</button></footer>
    </main>
  </div>;
}
createRoot(document.getElementById('root')!).render(<StrictMode><App /></StrictMode>);
