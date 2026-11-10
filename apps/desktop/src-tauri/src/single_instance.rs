use fs2::FileExt;
use std::{
    collections::hash_map::DefaultHasher,
    fs::{File, OpenOptions},
    hash::{Hash, Hasher},
    io::{Read, Write},
    os::unix::{
        fs::{FileTypeExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
pub struct SingleInstance {
    _lock: File,
    socket: PathBuf,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
pub enum Acquisition {
    Primary(SingleInstance),
    Secondary,
}
pub fn acquire(
    data: &Path,
    on_activation: impl Fn() + Send + 'static,
) -> Result<Acquisition, String> {
    // flock 是所有权保证；socket 只传聚焦通知，不参与判断实例是否存活。
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(data.join(".instance.lock"))
        .map_err(|e| e.to_string())?;
    let mut hash = DefaultHasher::new();
    data.hash(&mut hash);
    let socket = PathBuf::from(format!("/tmp/sixbox-{:016x}.sock", hash.finish()));
    if let Err(error) = lock.try_lock_exclusive() {
        if error.kind() != std::io::ErrorKind::WouldBlock {
            return Err(error.to_string());
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Ok(mut stream) = UnixStream::connect(&socket) {
                stream
                    .set_write_timeout(Some(Duration::from_millis(250)))
                    .map_err(|e| e.to_string())?;
                stream
                    .set_read_timeout(Some(Duration::from_millis(500)))
                    .map_err(|e| e.to_string())?;
                if stream.write_all(b"S").is_ok() {
                    // 等待确认，避免客户端过早关闭导致 macOS 接收端设置失效。
                    let mut ack = [0u8; 1];
                    if stream.read_exact(&mut ack).is_ok() && ack == *b"A" {
                        return Ok(Acquisition::Secondary);
                    }
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
        eprintln!("SixBox 已在运行，但未能发送聚焦通知");
        return Ok(Acquisition::Secondary);
    }
    if let Ok(metadata) = std::fs::symlink_metadata(&socket) {
        if !metadata.file_type().is_socket() {
            return Err("单实例通知路径被其他文件占用".into());
        }
        std::fs::remove_file(&socket).map_err(|e| e.to_string())?;
    }
    let listener = UnixListener::bind(&socket).map_err(|e| e.to_string())?;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let stop = Arc::new(AtomicBool::new(false));
    let stopping = stop.clone();
    let thread = thread::spawn(move || {
        while !stopping.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let configured = stream
                        .set_nonblocking(false)
                        .and_then(|_| stream.set_read_timeout(Some(Duration::from_millis(250))))
                        .and_then(|_| stream.set_write_timeout(Some(Duration::from_millis(250))));
                    if let Err(error) = configured {
                        eprintln!("SixBox 聚焦通道设置失败：{error}");
                    } else {
                        let mut byte = [0u8; 1];
                        match stream.read_exact(&mut byte) {
                            Ok(_) if byte == *b"S" => {
                                on_activation();
                                if let Err(error) = stream.write_all(b"A") {
                                    eprintln!("SixBox 聚焦确认失败：{error}");
                                }
                            }
                            Err(error) => eprintln!("SixBox 聚焦通知读取失败：{error}"),
                            _ => {}
                        }
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20))
                }
                Err(error) => {
                    eprintln!("SixBox 单实例通知失败：{error}");
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    });
    Ok(Acquisition::Primary(SingleInstance {
        _lock: lock,
        socket,
        stop,
        thread: Some(thread),
    }))
}
impl Drop for SingleInstance {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        if let Err(error) = std::fs::remove_file(&self.socket) {
            eprintln!("SixBox 单实例通知清理失败：{error}");
        }
        // 持有文件锁直至 socket 清理完成。不要删除锁文件，避免并发创建不同 inode。
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::tests::TempDir;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            mpsc, Arc, Barrier,
        },
        time::Duration,
    };
    #[test]
    fn second_instance_notifies_primary_and_releases_after_drop() {
        let root = TempDir::new();
        let (tx, rx) = mpsc::channel();
        let first = acquire(&root.0, move || {
            tx.send(()).unwrap();
        })
        .unwrap();
        assert!(matches!(first, Acquisition::Primary(_)));
        assert!(matches!(
            acquire(&root.0, || {}).unwrap(),
            Acquisition::Secondary
        ));
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        drop(first);
        assert!(matches!(
            acquire(&root.0, || {}).unwrap(),
            Acquisition::Primary(_)
        ));
    }
    #[test]
    fn simultaneous_launches_have_exactly_one_owner() {
        let root = TempDir::new();
        let barrier = Arc::new(Barrier::new(8));
        let owners = Arc::new(AtomicUsize::new(0));
        let mut threads = Vec::new();
        for _ in 0..8 {
            let (path, barrier, owners) = (root.0.clone(), barrier.clone(), owners.clone());
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                let instance = acquire(&path, || {}).unwrap();
                if matches!(instance, Acquisition::Primary(_)) {
                    owners.fetch_add(1, Ordering::SeqCst);
                }
                barrier.wait();
                drop(instance);
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(owners.load(Ordering::SeqCst), 1);
    }
}
