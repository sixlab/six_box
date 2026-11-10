use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPaths {
    pub data: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
    pub plugins: PathBuf,
    pub plugin_data: PathBuf,
    pub settings: PathBuf,
    pub database: PathBuf,
    pub runtimes: PathBuf,
}
impl AppPaths {
    pub fn initialize(data: PathBuf, cache: PathBuf, logs: PathBuf) -> Result<Self, String> {
        if [&data, &cache, &logs].iter().any(|p| !p.is_absolute()) {
            return Err("应用目录必须是绝对路径".into());
        }
        let paths = Self {
            plugins: data.join("plugins"),
            plugin_data: data.join("plugin-data"),
            settings: data.join("settings"),
            database: data.join("data"),
            runtimes: data.join("runtimes/node"),
            data,
            cache,
            logs,
        };
        for path in [
            &paths.data,
            &paths.cache,
            &paths.logs,
            &paths.plugins,
            &paths.plugin_data,
            &paths.settings,
            &paths.database,
            &paths.runtimes,
        ] {
            let mut builder = std::fs::DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder
                .create(path)
                .map_err(|e| format!("无法初始化 {}：{e}", path.display()))?;
        }
        Ok(paths)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    pub struct TempDir(pub PathBuf);
    impl TempDir {
        pub fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "sixbox-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        pub fn paths(&self) -> AppPaths {
            AppPaths::initialize(
                self.0.join("app"),
                self.0.join("cache"),
                self.0.join("logs"),
            )
            .unwrap()
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn creates_separated_directories_and_preserves_existing_data() {
        let root = TempDir::new();
        let paths = root.paths();
        for path in [
            &paths.data,
            &paths.cache,
            &paths.logs,
            &paths.plugins,
            &paths.plugin_data,
            &paths.settings,
            &paths.database,
            &paths.runtimes,
        ] {
            assert!(path.is_dir(), "{}", path.display());
        }
        std::fs::write(paths.settings.join("preferences.json"), "existing").unwrap();
        root.paths();
        assert_eq!(
            std::fs::read_to_string(paths.settings.join("preferences.json")).unwrap(),
            "existing"
        );
    }
    #[test]
    fn rejects_relative_roots_and_existing_files_without_overwriting() {
        let root = TempDir::new();
        assert!(
            AppPaths::initialize("relative".into(), root.0.join("cache"), root.0.join("logs"))
                .is_err()
        );
        let occupied = root.0.join("occupied");
        std::fs::write(&occupied, "keep").unwrap();
        assert!(
            AppPaths::initialize(occupied.clone(), root.0.join("cache"), root.0.join("logs"))
                .is_err()
        );
        assert_eq!(std::fs::read_to_string(occupied).unwrap(), "keep");
    }
}
