use crate::{logging::Logger, paths::AppPaths, protocol::RuntimeError};
use serde::Serialize;
use std::{path::PathBuf, time::Duration};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::{sleep, timeout};
mod session;
use session::Session;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub phase: String,
    pub source: String,
    pub node_path: PathBuf,
    pub node_version: Option<String>,
    pub service_pid: Option<u32>,
    pub restarts: u32,
    pub error: Option<RuntimeError>,
}
impl RuntimeStatus {
    pub fn initial(node: PathBuf) -> Self {
        Self {
            phase: "detecting".into(),
            source: "bundled".into(),
            node_path: node,
            node_version: None,
            service_pid: None,
            restarts: 0,
            error: None,
        }
    }
}
#[derive(Clone)]
pub struct RuntimeConfig {
    pub node: PathBuf,
    pub service: PathBuf,
    pub request_timeout: Duration,
    pub heartbeat: Duration,
    pub backoff: Duration,
}
impl RuntimeConfig {
    pub fn bundled(node: PathBuf, service: PathBuf) -> Self {
        Self {
            node,
            service,
            request_timeout: Duration::from_secs(3),
            heartbeat: Duration::from_secs(5),
            backoff: Duration::from_millis(500),
        }
    }
}
enum Control {
    Retry(oneshot::Sender<Result<(), RuntimeError>>),
    Shutdown(oneshot::Sender<Result<(), RuntimeError>>),
}
pub struct RuntimeManager {
    status: watch::Receiver<RuntimeStatus>,
    control: mpsc::Sender<Control>,
}
impl RuntimeManager {
    pub fn start(config: RuntimeConfig, paths: AppPaths, logger: Logger) -> Self {
        let (tx, status) = watch::channel(RuntimeStatus::initial(config.node.clone()));
        let (control, rx) = mpsc::channel(16);
        tokio::spawn(supervise(config, paths, logger, tx, rx));
        Self { status, control }
    }
    pub fn status(&self) -> RuntimeStatus {
        self.status.borrow().clone()
    }
    pub async fn retry(&self) -> Result<(), RuntimeError> {
        let (tx, rx) = oneshot::channel();
        self.control
            .send(Control::Retry(tx))
            .await
            .map_err(|_| RuntimeError::new("MANAGER_STOPPED", "运行时管理器已停止"))?;
        rx.await
            .map_err(|_| RuntimeError::new("MANAGER_STOPPED", "运行时管理器已停止"))?
    }
    pub async fn shutdown(&self) -> Result<(), RuntimeError> {
        if self.status().phase == "stopped" {
            return Ok(());
        }
        let (tx, rx) = oneshot::channel();
        if self.control.send(Control::Shutdown(tx)).await.is_ok() {
            if let Ok(result) = rx.await {
                return result;
            }
        }
        // 第一个调用完成后 actor 会关闭队列；并发调用仍以最终状态为准。
        if self.status().phase == "stopped" {
            Ok(())
        } else {
            Err(RuntimeError::new("MANAGER_STOPPED", "运行时管理器异常停止"))
        }
    }
}

fn publish(tx: &watch::Sender<RuntimeStatus>, state: &RuntimeStatus, logger: &Logger) {
    logger.write(
        "runtime.state",
        &serde_json::to_string(state).unwrap_or_default(),
    );
    tx.send_replace(state.clone());
}
fn stopped(tx: &watch::Sender<RuntimeStatus>, state: &mut RuntimeStatus, logger: &Logger) {
    state.phase = "stopped".into();
    state.service_pid = None;
    publish(tx, state, logger);
}
// 只有 failed/unavailable 接受重试，串行控制队列防止重复子进程。
async fn idle(
    rx: &mut mpsc::Receiver<Control>,
    tx: &watch::Sender<RuntimeStatus>,
    state: &mut RuntimeStatus,
    logger: &Logger,
) -> bool {
    match rx.recv().await {
        Some(Control::Retry(reply)) => {
            let _ = reply.send(Ok(()));
            true
        }
        Some(Control::Shutdown(reply)) => {
            stopped(tx, state, logger);
            let _ = reply.send(Ok(()));
            false
        }
        None => {
            stopped(tx, state, logger);
            false
        }
    }
}
async fn supervise(
    config: RuntimeConfig,
    paths: AppPaths,
    logger: Logger,
    tx: watch::Sender<RuntimeStatus>,
    mut rx: mpsc::Receiver<Control>,
) {
    loop {
        let mut state = RuntimeStatus::initial(config.node.clone());
        publish(&tx, &state, &logger);
        match session::probe(&config).await {
            Ok(version) => state.node_version = Some(version),
            Err(error) => {
                state.phase = "unavailable".into();
                state.error = Some(error);
                publish(&tx, &state, &logger);
                if idle(&mut rx, &tx, &mut state, &logger).await {
                    continue;
                } else {
                    return;
                }
            }
        }
        for attempt in 0..=3 {
            state.phase = "starting".into();
            state.restarts = attempt;
            state.service_pid = None;
            publish(&tx, &state, &logger);
            let failure = match Session::launch(&config, &paths, logger.clone()).await {
                Ok(mut session) => {
                    state.phase = "ready".into();
                    state.service_pid = session.child.id();
                    state.error = None;
                    publish(&tx, &state, &logger);
                    loop {
                        tokio::select! {
                            command = rx.recv() => match command {
                                Some(Control::Retry(reply)) => { let _ = reply.send(Err(RuntimeError::new("RUNTIME_BUSY", "管理服务已在运行"))); }
                                command => {
                                    state.phase="stopping".into(); publish(&tx,&state,&logger);
                                    let result=session.stop().await;
                                    stopped(&tx,&mut state,&logger);
                                    if let Some(Control::Shutdown(reply))=command { let _ = reply.send(result); }
                                    return;
                                }
                            },
                            result = session.child.wait() => {
                                break RuntimeError::new("SERVICE_EXITED",match result {Ok(status)=>format!("管理服务退出：{status}"),Err(error)=>error.to_string()});
                            }
                            _ = sleep(config.heartbeat) => {
                                match session.request("ping",serde_json::json!({})).await {
                                    Ok(value) if value["alive"] == true => {},
                                    result => {
                                        let error = result.err().unwrap_or_else(||RuntimeError::new("PROTOCOL_ERROR","服务健康响应无效"));
                                        if let Err(cleanup)=session.stop().await { logger.write("service.cleanup-failed",&cleanup.message); }
                                        break error;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(error) => error,
            };
            state.error = Some(failure);
            state.service_pid = None;
            state.phase = if attempt == 3 { "failed" } else { "recovering" }.into();
            publish(&tx, &state, &logger);
            if attempt < 3 {
                let delay = sleep(config.backoff * 2_u32.pow(attempt));
                tokio::pin!(delay);
                loop {
                    tokio::select! {
                        _ = &mut delay => break,
                        command=rx.recv() => match command {
                            Some(Control::Retry(reply))=>{ let _ = reply.send(Err(RuntimeError::new("RUNTIME_BUSY","服务正在自动恢复"))); }
                            command=> { stopped(&tx,&mut state,&logger); if let Some(Control::Shutdown(reply))=command {let _=reply.send(Ok(()));} return; }
                        }
                    }
                }
            }
        }
        if !idle(&mut rx, &tx, &mut state, &logger).await {
            return;
        }
    }
}

#[cfg(test)]
mod tests;
