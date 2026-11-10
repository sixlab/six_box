#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::Serialize;
use sixbox_lib::{
    lifecycle::ExitGate,
    logging::Logger,
    paths::AppPaths,
    protocol::RuntimeError,
    runtime::{RuntimeConfig, RuntimeManager, RuntimeStatus},
    single_instance::{self, Acquisition, SingleInstance},
};
use std::{path::PathBuf, sync::Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

struct HostState {
    paths: Option<AppPaths>,
    runtime: Option<RuntimeManager>,
    fallback: RuntimeStatus,
    logger: Option<Logger>,
    exit: ExitGate,
    instance: Mutex<Option<SingleInstance>>,
}
#[derive(Serialize)]
struct Snapshot {
    paths: Option<AppPaths>,
    runtime: RuntimeStatus,
}
fn authorize(window: &tauri::WebviewWindow) -> Result<(), RuntimeError> {
    if window.label() != "main" {
        return Err(RuntimeError::new("UNAUTHORIZED", "仅主窗口可管理运行时"));
    }
    Ok(())
}
#[tauri::command]
fn get_snapshot(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, HostState>,
) -> Result<Snapshot, RuntimeError> {
    authorize(&window)?;
    Ok(Snapshot {
        paths: state.paths.clone(),
        runtime: state
            .runtime
            .as_ref()
            .map(|r| r.status())
            .unwrap_or_else(|| state.fallback.clone()),
    })
}
#[tauri::command]
async fn retry_runtime(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, HostState>,
) -> Result<(), RuntimeError> {
    authorize(&window)?;
    match &state.runtime {
        Some(runtime) => runtime.retry().await,
        None => Err(RuntimeError::new(
            "DIRECTORY_UNAVAILABLE",
            "请修复存储目录后重新打开应用",
        )),
    }
}
#[tauri::command]
fn quit_app(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), RuntimeError> {
    authorize(&window)?;
    request_quit(&app);
    Ok(())
}
fn request_quit(app: &tauri::AppHandle) {
    let state = app.state::<HostState>();
    if !state.exit.begin() {
        return;
    }
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = handle.state::<HostState>();
        if let Some(runtime) = &state.runtime {
            if let Err(error) = runtime.shutdown().await {
                eprintln!("SixBox 停止服务失败：{}", error.message);
            }
        }
        state.exit.finish();
        handle.exit(0);
    });
}
fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window
            .unminimize()
            .and_then(|_| window.show())
            .and_then(|_| window.set_focus())
        {
            eprintln!("SixBox 无法显示主窗口：{error}");
        }
    }
}
fn runtime_files(app: &tauri::App) -> Result<(PathBuf, PathBuf), String> {
    if cfg!(debug_assertions) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        Ok((
            root.join("binaries/node-aarch64-apple-darwin"),
            root.join("resources/service.cjs"),
        ))
    } else {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let parent = exe.parent().ok_or("无法解析程序路径")?;
        Ok((
            parent.join("node"),
            app.path()
                .resource_dir()
                .map_err(|e| e.to_string())?
                .join("runtime/service.cjs"),
        ))
    }
}
fn initialize(app: &tauri::App) -> HostState {
    let mut state = HostState {
        paths: None,
        runtime: None,
        fallback: RuntimeStatus::initial(PathBuf::new()),
        logger: None,
        exit: ExitGate::default(),
        instance: Mutex::new(None),
    };
    let result = (|| -> Result<(), String> {
        let (node, service) = runtime_files(app)?;
        state.fallback.node_path = node.clone();
        let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
        let handle = app.handle().clone();
        let instance = single_instance::acquire(&data, move || {
            let focus = handle.clone();
            if let Err(error) = handle.run_on_main_thread(move || show_main(&focus)) {
                eprintln!("SixBox 聚焦失败：{error}");
            }
        })?;
        match instance {
            Acquisition::Secondary => std::process::exit(0),
            Acquisition::Primary(guard) => {
                *state.instance.lock().map_err(|e| e.to_string())? = Some(guard)
            }
        }
        let paths = AppPaths::initialize(
            data,
            app.path().app_cache_dir().map_err(|e| e.to_string())?,
            app.path().app_log_dir().map_err(|e| e.to_string())?,
        )?;
        let logger = Logger::open(&paths.logs)?;
        logger.write("host.start", &format!("SixBox pid={}", std::process::id()));
        let runtime = tauri::async_runtime::block_on(async {
            RuntimeManager::start(
                RuntimeConfig::bundled(node, service),
                paths.clone(),
                logger.clone(),
            )
        });
        state.paths = Some(paths);
        state.logger = Some(logger);
        state.runtime = Some(runtime);
        Ok(())
    })();
    if let Err(message) = result {
        state.fallback.phase = "failed".into();
        state.fallback.error = Some(RuntimeError::new("DIRECTORY_UNAVAILABLE", message));
    }
    state
}
fn main() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            retry_runtime,
            quit_app
        ])
        .setup(|app| {
            app.manage(initialize(app));
            #[cfg(unix)]
            {
                let (mut terminate, mut interrupt) = tauri::async_runtime::block_on(async {
                    Ok::<_, std::io::Error>((
                        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?,
                        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?,
                    ))
                })?;
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::select! { _=terminate.recv()=>{}, _=interrupt.recv()=>{} }
                    request_quit(&handle);
                });
            }
            let show = MenuItem::with_id(app, "show", "打开 SixBox", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "完全退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            // M1 只提供恢复主窗口和退出入口；完整托盘菜单属于 M3。
            let tray = TrayIconBuilder::new()
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/tray.png"
                ))?)
                .icon_as_template(true)
                .tooltip("SixBox 工具箱")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main(app),
                    "quit" => request_quit(app),
                    _ => {}
                })
                .build(app);
            if let Err(error) = tray {
                if let Some(logger) = &app.state::<HostState>().logger {
                    logger.write("tray.unavailable", &error.to_string());
                }
            }
            show_main(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" && !window.state::<HostState>().exit.may_exit() {
                    api.prevent_close();
                    if let Err(error) = window.hide() {
                        eprintln!("SixBox 隐藏窗口失败：{error}");
                    } else if let Some(logger) = &window.state::<HostState>().logger {
                        logger.write("window.hidden", "main");
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("SixBox 桌面宿主初始化失败");
    app.run(|app, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            if !app.state::<HostState>().exit.may_exit() {
                api.prevent_exit();
                request_quit(app);
            }
        }
        tauri::RunEvent::Reopen { .. } => show_main(app),
        tauri::RunEvent::Exit => {
            let state = app.state::<HostState>();
            // macOS 原生 Quit 可直接触发 Exit，跳过 ExitRequested。
            // 在最终退出回调返回前等待有界清理，不能依赖进程退出时的 Drop。
            if !state.exit.may_exit() {
                state.exit.begin();
                if let Some(logger) = &state.logger {
                    logger.write("host.native-exit", "等待管理服务清理");
                }
                if let Some(runtime) = &state.runtime {
                    if let Err(error) = tauri::async_runtime::block_on(runtime.shutdown()) {
                        eprintln!("SixBox 最终退出清理失败：{}", error.message);
                    }
                }
                state.exit.finish();
            }
            state.instance.lock().unwrap().take();
        }
        _ => {}
    });
}
