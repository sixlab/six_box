use super::*;
use crate::protocol::{read_frame, response_result, write_frame};
use serde_json::{json, Value};
use std::{
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::{
    io::{AsyncReadExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    task::JoinHandle,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
fn command(node: &PathBuf) -> Command {
    let mut command = Command::new(node);
    command
        .env_remove("NODE_OPTIONS")
        .env_remove("NODE_PATH")
        .env_remove("NODE_EXTRA_CA_CERTS")
        .env_remove("NODE_ICU_DATA")
        .env_remove("ELECTRON_RUN_AS_NODE")
        .env("PATH", "/usr/bin:/bin")
        .kill_on_drop(true);
    command
}
pub(super) async fn probe(config: &RuntimeConfig) -> Result<String, RuntimeError> {
    if !config.node.is_file() {
        return Err(RuntimeError::new(
            "NODE_MISSING",
            "未找到内置 Node.js，请重新安装 SixBox 完整版",
        ));
    }
    let mut child = command(&config.node)
        .args([
            "-p",
            "JSON.stringify({version:process.version,arch:process.arch,platform:process.platform})",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| RuntimeError::new("NODE_EXEC_FAILED", e))?;
    let output = child
        .stdout
        .take()
        .ok_or_else(|| RuntimeError::new("NODE_EXEC_FAILED", "无法读取 Node 输出"))?;
    let result = timeout(config.request_timeout, async {
        let info = read_frame(&mut BufReader::new(output)).await?;
        let status = child
            .wait()
            .await
            .map_err(|e| RuntimeError::new("NODE_EXEC_FAILED", e))?;
        if !status.success() {
            return Err(RuntimeError::new("NODE_EXEC_FAILED", "Node 检测进程失败"));
        }
        if info["version"] != "v24.22.2" || info["platform"] != "darwin" || info["arch"] != "arm64"
        {
            return Err(RuntimeError::new(
                "NODE_INCOMPATIBLE",
                "需要内置 Node v24.22.2（macOS arm64）",
            ));
        }
        Ok("v24.22.2".into())
    })
    .await
    .unwrap_or_else(|_| Err(RuntimeError::new("NODE_TIMEOUT", "Node 执行检测超时")));
    if result.is_err() {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
    result
}
pub(super) struct Session {
    pub child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    stderr: JoinHandle<()>,
    timeout: Duration,
    logger: Logger,
}
impl Session {
    pub async fn launch(
        config: &RuntimeConfig,
        paths: &AppPaths,
        logger: Logger,
    ) -> Result<Self, RuntimeError> {
        if !config.service.is_file() {
            return Err(RuntimeError::new(
                "SERVICE_MISSING",
                "管理服务资源缺失，请重新安装 SixBox",
            ));
        }
        let mut child = command(&config.node)
            .arg(&config.service)
            .current_dir(&paths.data)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RuntimeError::new("SERVICE_SPAWN_FAILED", e))?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| RuntimeError::new("SERVICE_SPAWN_FAILED", "stdin 不可用"))?;
        let output = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| RuntimeError::new("SERVICE_SPAWN_FAILED", "stdout 不可用"))?,
        );
        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| RuntimeError::new("SERVICE_SPAWN_FAILED", "stderr 不可用"))?;
        let log = logger.clone();
        let stderr = tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            loop {
                match stderr.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => log.write("service.stderr", &String::from_utf8_lossy(&buffer[..n])),
                    Err(error) => {
                        log.write("service.stderr-read-failed", &error.to_string());
                        break;
                    }
                }
            }
        });
        let mut session = Self {
            child,
            input,
            output,
            stderr,
            timeout: config.request_timeout,
            logger,
        };
        let handshake = timeout(config.request_timeout, async {
            let reply = session
                .request("initialize", json!({"paths":paths}))
                .await?;
            if reply["initialized"] != true {
                return Err(RuntimeError::new("PROTOCOL_ERROR", "初始化响应无效"));
            }
            let ready = read_frame(&mut session.output).await?;
            if ready["version"] != 1
                || ready["type"] != "event"
                || ready["name"] != "ready"
                || ready["payload"]["pid"].as_u64() != session.child.id().map(u64::from)
                || ready["payload"]["nodeVersion"] != "v24.22.2"
            {
                return Err(RuntimeError::new("PROTOCOL_ERROR", "Ready 消息无效"));
            }
            Ok(())
        })
        .await
        .unwrap_or_else(|_| Err(RuntimeError::new("READY_TIMEOUT", "管理服务启动超时")));
        if let Err(error) = handshake {
            session.force_stop().await?;
            return Err(error);
        }
        Ok(session)
    }
    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, RuntimeError> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        timeout(self.timeout, async {
            write_frame(
                &mut self.input,
                &json!({"version":1,"type":"request","id":id,"method":method,"params":params}),
            )
            .await?;
            response_result(read_frame(&mut self.output).await?, id)
        })
        .await
        .unwrap_or_else(|_| {
            Err(RuntimeError::new(
                "REQUEST_TIMEOUT",
                format!("管理服务请求超时：{method}"),
            ))
        })
    }
    pub async fn stop(&mut self) -> Result<(), RuntimeError> {
        let result = timeout(self.timeout, async {
            self.request("shutdown", json!({})).await?;
            self.child
                .wait()
                .await
                .map_err(|e| RuntimeError::new("WAIT_FAILED", e))?;
            Ok::<_, RuntimeError>(())
        })
        .await;
        if !matches!(result, Ok(Ok(()))) {
            self.logger.write(
                "service.force-stop",
                "服务未在停止期限内退出，执行终止并回收",
            );
            self.force_stop().await?;
        }
        Ok(())
    }
    async fn force_stop(&mut self) -> Result<(), RuntimeError> {
        if self
            .child
            .try_wait()
            .map_err(|e| RuntimeError::new("WAIT_FAILED", e))?
            .is_none()
        {
            self.child
                .kill()
                .await
                .map_err(|e| RuntimeError::new("KILL_FAILED", e))?;
        }
        self.child
            .wait()
            .await
            .map_err(|e| RuntimeError::new("WAIT_FAILED", e))?;
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.stderr.abort();
    }
}
