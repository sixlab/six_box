fn main() {
    #[cfg(feature = "desktop")]
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&["get_snapshot", "retry_runtime", "quit_app"]),
    ))
    .expect("生成 Tauri 配置失败");
}
