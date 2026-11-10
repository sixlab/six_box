use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub struct Logger {
    inner: Arc<Mutex<PathBuf>>,
}
impl Logger {
    pub fn open(root: &Path) -> Result<Self, String> {
        let path = root.join("host.jsonl");
        for index in 0..=3 {
            let file = if index == 0 {
                path.clone()
            } else {
                path.with_extension(format!("jsonl.{index}"))
            };
            if fs::metadata(&file)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|age| age > Duration::from_secs(7 * 86400))
            {
                fs::remove_file(file).map_err(|e| e.to_string())?;
            }
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            inner: Arc::new(Mutex::new(path)),
        })
    }
    pub fn write(&self, event: &str, message: &str) {
        // 日志故障不能终止宿主；仍向 stderr 报告，不静默丢弃。
        if let Err(error) = self.append(event, message) {
            eprintln!("SixBox 日志写入失败：{error}");
        }
    }
    fn append(&self, event: &str, message: &str) -> Result<(), String> {
        let path = self.inner.lock().map_err(|e| e.to_string())?;
        let line = serde_json::json!({"timestamp":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(), "event":event, "message":message.chars().take(2048).collect::<String>()}).to_string() + "\n";
        if fs::metadata(&*path).map(|m| m.len()).unwrap_or(0) + line.len() as u64 > 1024 * 1024 {
            for index in (1..=3).rev() {
                let to = path.with_extension(format!("jsonl.{index}"));
                let from = if index == 1 {
                    path.clone()
                } else {
                    path.with_extension(format!("jsonl.{}", index - 1))
                };
                if from.exists() {
                    fs::rename(from, to).map_err(|e| e.to_string())?;
                }
            }
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*path)
            .and_then(|mut f| f.write_all(line.as_bytes()))
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::tests::TempDir;
    #[test]
    fn bounds_logs_and_keeps_rotation_backups() {
        let root = TempDir::new();
        let logger = Logger::open(&root.0).unwrap();
        for _ in 0..3000 {
            logger.write("test", &"x".repeat(4096));
        }
        let files: Vec<_> = std::fs::read_dir(&root.0).unwrap().collect();
        assert!((2..=4).contains(&files.len()));
        for file in files {
            assert!(file.unwrap().metadata().unwrap().len() <= 1_050_000);
        }
        let text = std::fs::read_to_string(root.0.join("host.jsonl")).unwrap();
        assert!(text
            .lines()
            .all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok()));
    }
}
