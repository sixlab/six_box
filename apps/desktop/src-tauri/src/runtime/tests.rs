use super::*;
use crate::paths::tests::TempDir;
use tokio::time::{sleep, timeout};

fn config() -> RuntimeConfig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config = RuntimeConfig::bundled(
        root.join("binaries/node-aarch64-apple-darwin"),
        root.join("resources/service.cjs"),
    );
    config.request_timeout = Duration::from_millis(400);
    config.heartbeat = Duration::from_millis(100);
    config.backoff = Duration::from_millis(20);
    config
}
async fn phase(manager: &RuntimeManager, expected: &str) -> RuntimeStatus {
    timeout(Duration::from_secs(10), async {
        loop {
            let status = manager.status();
            if status.phase == expected {
                return status;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("expected {expected}, got {:?}", manager.status()))
}
fn manager(config: RuntimeConfig, temp: &TempDir) -> RuntimeManager {
    let paths = temp.paths();
    RuntimeManager::start(config, paths.clone(), Logger::open(&paths.logs).unwrap())
}
#[tokio::test]
async fn real_bundled_node_handshakes_and_shuts_down() {
    let root = TempDir::new();
    let manager = manager(config(), &root);
    let ready = phase(&manager, "ready").await;
    assert_eq!(ready.node_version.as_deref(), Some("v24.22.2"));
    assert!(ready.service_pid.is_some());
    assert_eq!(manager.retry().await.unwrap_err().code, "RUNTIME_BUSY");
    manager.shutdown().await.unwrap();
    assert_eq!(manager.status().phase, "stopped");
    assert!(manager.status().service_pid.is_none());
}
#[tokio::test]
async fn missing_node_can_be_retried_after_restoring_binary() {
    let root = TempDir::new();
    let original = config();
    let mut cfg = original.clone();
    cfg.node = root.0.join("node");
    let manager = manager(cfg.clone(), &root);
    assert_eq!(
        phase(&manager, "unavailable").await.error.unwrap().code,
        "NODE_MISSING"
    );
    std::os::unix::fs::symlink(original.node, &cfg.node).unwrap();
    manager.retry().await.unwrap();
    phase(&manager, "ready").await;
    manager.shutdown().await.unwrap();
}
#[tokio::test]
async fn unexecutable_and_wrong_version_nodes_are_unavailable() {
    use std::os::unix::fs::PermissionsExt;
    for executable in [false, true] {
        let root = TempDir::new();
        let mut cfg = config();
        cfg.node = root.0.join("node");
        std::fs::write(
            &cfg.node,
            "#!/bin/sh\necho '{\"version\":\"v1\",\"arch\":\"arm64\",\"platform\":\"darwin\"}'\n",
        )
        .unwrap();
        if executable {
            std::fs::set_permissions(&cfg.node, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let manager = manager(cfg, &root);
        assert_eq!(
            phase(&manager, "unavailable").await.error.unwrap().code,
            if executable {
                "NODE_INCOMPATIBLE"
            } else {
                "NODE_EXEC_FAILED"
            }
        );
        manager.shutdown().await.unwrap();
    }
}
#[tokio::test]
async fn startup_timeout_and_protocol_failure_stop_after_three_retries() {
    for script in [
        "setInterval(()=>{},1000)",
        "process.stdout.write('invalid\\n');setInterval(()=>{},1000)",
        "process.exit(23)",
    ] {
        let root = TempDir::new();
        let mut cfg = config();
        cfg.service = root.0.join("fault.cjs");
        std::fs::write(&cfg.service, script).unwrap();
        let manager = manager(cfg, &root);
        let status = phase(&manager, "failed").await;
        assert_eq!(status.restarts, 3);
        assert!(status.service_pid.is_none());
        assert!(status.error.is_some());
        manager.shutdown().await.unwrap();
    }
}
#[tokio::test]
async fn crashed_service_is_replaced_and_hung_service_is_killed() {
    let root = TempDir::new();
    let manager = manager(config(), &root);
    let pid = phase(&manager, "ready").await.service_pid.unwrap();
    assert!(std::process::Command::new("/bin/kill")
        .args(["-KILL", &pid.to_string()])
        .status()
        .unwrap()
        .success());
    timeout(Duration::from_secs(5), async {
        loop {
            let s = manager.status();
            if s.phase == "ready" && s.service_pid != Some(pid) {
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    let pid = manager.status().service_pid.unwrap();
    std::process::Command::new("/bin/kill")
        .args(["-STOP", &pid.to_string()])
        .status()
        .unwrap();
    timeout(Duration::from_secs(3), manager.shutdown())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(manager.status().phase, "stopped");
}

#[tokio::test]
async fn hung_ready_service_is_automatically_recovered() {
    let root = TempDir::new();
    let manager = manager(config(), &root);
    let pid = phase(&manager, "ready").await.service_pid.unwrap();
    std::process::Command::new("/bin/kill")
        .args(["-STOP", &pid.to_string()])
        .status()
        .unwrap();
    timeout(Duration::from_secs(5), async {
        loop {
            let status = manager.status();
            if status.phase == "ready" && status.service_pid != Some(pid) {
                assert_eq!(status.restarts, 1);
                assert!(status.error.is_none());
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    manager.shutdown().await.unwrap();
}

#[tokio::test]
async fn concurrent_shutdown_callers_both_wait_for_cleanup() {
    let root = TempDir::new();
    let manager = manager(config(), &root);
    let pid = phase(&manager, "ready").await.service_pid.unwrap();
    std::process::Command::new("/bin/kill")
        .args(["-STOP", &pid.to_string()])
        .status()
        .unwrap();
    let (first, second) = tokio::join!(manager.shutdown(), manager.shutdown());
    assert!(first.is_ok());
    assert!(
        second.is_ok(),
        "后续退出调用必须等待同一次清理完成：{second:?}"
    );
    assert_eq!(manager.status().phase, "stopped");
}
