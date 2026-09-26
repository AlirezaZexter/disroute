mod community;
mod discord;
mod engine;
mod health;
mod model;
mod profile_store;
mod voice_proxy;

use community::{CommunitySnapshot, CommunitySource, HealthResult};
use engine::EngineManager;
use health::ScanControl;
use model::{AppStatus, ProxyProfile};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[derive(Default)]
struct FailoverPlan {
    remaining: Vec<String>,
    generation: u64,
}

impl FailoverPlan {
    fn clear(&mut self) {
        self.remaining.clear();
        self.generation = self.generation.wrapping_add(1);
    }
}

#[tauri::command]
fn hide_to_tray(window: tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn get_status(app: AppHandle, manager: State<'_, Mutex<EngineManager>>) -> AppStatus {
    manager.lock().expect("engine lock poisoned").status(&app)
}

#[tauri::command]
fn save_profile(app: AppHandle, profile: ProxyProfile) -> Result<(), String> {
    let data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    profile_store::save(&data_dir, &profile)
}

#[tauri::command]
fn load_profile(app: AppHandle) -> Result<Option<ProxyProfile>, String> {
    profile_store::load(&app.path().app_local_data_dir().map_err(|e| e.to_string())?)
}

#[tauri::command]
fn forget_profile(app: AppHandle) -> Result<(), String> {
    profile_store::forget(&app.path().app_local_data_dir().map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn start_tunnel(
    app: AppHandle,
    profile: ProxyProfile,
    manager: State<'_, Mutex<EngineManager>>,
    failover: State<'_, Mutex<FailoverPlan>>,
) -> Result<AppStatus, String> {
    profile.validate()?;
    failover
        .lock()
        .map_err(|_| "failover lock poisoned")?
        .clear();
    manager
        .lock()
        .map_err(|_| "engine lock poisoned")?
        .start(&app, &profile)
}

#[tauri::command]
fn stop_tunnel(
    app: AppHandle,
    manager: State<'_, Mutex<EngineManager>>,
    failover: State<'_, Mutex<FailoverPlan>>,
) -> Result<AppStatus, String> {
    failover
        .lock()
        .map_err(|_| "failover lock poisoned")?
        .clear();
    manager
        .lock()
        .map_err(|_| "engine lock poisoned")?
        .stop(&app)
}

#[tauri::command]
fn restart_discord() -> Result<String, String> {
    discord::restart()
}

fn community_store(app: &AppHandle) -> Result<community::CommunityStore, String> {
    Ok(community::CommunityStore::new(
        app.path()
            .app_local_data_dir()
            .map_err(|e| e.to_string())?
            .join("community"),
    ))
}

#[tauri::command]
fn community_snapshot(app: AppHandle) -> Result<CommunitySnapshot, String> {
    community_store(&app)?.snapshot()
}

#[tauri::command]
fn save_community_sources(
    app: AppHandle,
    sources: Vec<CommunitySource>,
) -> Result<CommunitySnapshot, String> {
    community_store(&app)?.save_sources(sources)
}

#[tauri::command]
fn set_community_preferences(
    app: AppHandle,
    acknowledged_warning: bool,
    automatic_failover: bool,
) -> Result<CommunitySnapshot, String> {
    community_store(&app)?.set_preferences(acknowledged_warning, automatic_failover)
}

#[tauri::command]
async fn refresh_community(app: AppHandle) -> Result<CommunitySnapshot, String> {
    let store = community_store(&app)?;
    tauri::async_runtime::spawn_blocking(move || store.refresh())
        .await
        .map_err(|_| "نوسازی منابع متوقف شد.".to_string())?
}

#[tauri::command]
async fn scan_community(
    app: AppHandle,
    manager: State<'_, Mutex<EngineManager>>,
    scan_control: State<'_, Mutex<ScanControl>>,
) -> Result<Vec<HealthResult>, String> {
    let store = community_store(&app)?;
    let snapshot = store.snapshot()?;
    if !snapshot.acknowledged_warning {
        return Err("ابتدا شرایط اتصال Community را تأیید کنید.".into());
    }
    let history = store.history()?;
    let engine_dir = manager
        .lock()
        .map_err(|_| "engine lock poisoned")?
        .engine_dir(&app)?;
    let sing_box = engine_dir.join("sing-box.exe");
    if !sing_box.is_file() {
        return Err("فایل sing-box.exe پیدا نشد.".into());
    }
    let runtime = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("community-tests");
    let cancelled = scan_control
        .lock()
        .map_err(|_| "scan lock poisoned")?
        .begin();
    tauri::async_runtime::spawn_blocking(move || {
        let results = health::scan(snapshot.candidates, sing_box, runtime, cancelled, history)?;
        store.record_scan(&results)?;
        Ok(results)
    })
    .await
    .map_err(|_| "آزمایش منابع متوقف شد.".to_string())?
}

#[tauri::command]
fn cancel_community_scan(scan_control: State<'_, Mutex<ScanControl>>) -> Result<(), String> {
    scan_control
        .lock()
        .map_err(|_| "scan lock poisoned")?
        .cancel();
    Ok(())
}

#[tauri::command]
fn clear_community_data(app: AppHandle) -> Result<CommunitySnapshot, String> {
    community_store(&app)?.clear()
}

#[tauri::command]
async fn connect_community(
    app: AppHandle,
    candidate_ids: Vec<String>,
) -> Result<AppStatus, String> {
    tauri::async_runtime::spawn_blocking(move || connect_community_inner(app, candidate_ids))
        .await
        .map_err(|_| "اتصال Community متوقف شد.".to_string())?
}

fn connect_community_inner(
    app: AppHandle,
    candidate_ids: Vec<String>,
) -> Result<AppStatus, String> {
    if candidate_ids.is_empty() {
        return Err("هیچ اتصال سالمی انتخاب نشده است.".into());
    }
    let store = community_store(&app)?;
    if !store.snapshot()?.acknowledged_warning {
        return Err("ابتدا شرایط اتصال Community را تأیید کنید.".into());
    }
    let manager = app.state::<Mutex<EngineManager>>();
    let failover = app.state::<Mutex<FailoverPlan>>();
    let generation = {
        let mut plan = failover.lock().map_err(|_| "failover lock poisoned")?;
        plan.clear();
        plan.generation
    };
    let mut last_error = "هیچ اتصال سالمی پیدا نشد.".to_string();
    let candidate_ids: Vec<String> = candidate_ids.into_iter().take(5).collect();
    for (index, id) in candidate_ids.iter().enumerate() {
        let candidate = store.candidate(&id)?;
        let history = store.history()?;
        if history
            .get(id)
            .and_then(|h| h.next_retry_at)
            .is_some_and(|until| until > community::now_secs())
        {
            continue;
        }
        let engine_dir = manager
            .lock()
            .map_err(|_| "engine lock poisoned")?
            .engine_dir(&app)?;
        // A separate temporary proxy must succeed before touching the active engine.
        let tested = health::scan(
            vec![candidate.clone()],
            engine_dir.join("sing-box.exe"),
            app.path()
                .app_local_data_dir()
                .map_err(|e| e.to_string())?
                .join("community-tests"),
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            history,
        )?;
        if !tested.iter().any(|result| result.working) {
            store.record_result(id, false)?;
            continue;
        }
        let profile = ProxyProfile {
            name: format!("Community · {}", candidate.source_name),
            config_link: candidate.uri,
        };
        let mut plan = failover.lock().map_err(|_| "failover lock poisoned")?;
        if plan.generation != generation {
            return Err("اتصال Community لغو شد.".into());
        }
        match manager
            .lock()
            .map_err(|_| "engine lock poisoned")?
            .start(&app, &profile)
        {
            Ok(status) => {
                let _ = store.record_result(&id, true);
                plan.remaining = candidate_ids[index + 1..].to_vec();
                return Ok(status);
            }
            Err(error) => {
                let _ = store.record_result(&id, false);
                last_error = community::redact_secrets(&error);
            }
        }
    }
    Err(last_error)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Mutex::new(EngineManager::default()))
        .manage(Mutex::new(ScanControl::default()))
        .manage(Mutex::new(FailoverPlan::default()))
        .setup(|app| {
            use tauri::{
                menu::{Menu, MenuItem},
                tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
            };
            let show = MenuItem::with_id(app, "show", "باز کردن DisRoute", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "قطع اتصال و خروج", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let tray = TrayIconBuilder::with_id("disroute-tray")
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/32x32.png"
                ))?)
                .tooltip("DisRoute — برای باز کردن کلیک کنید")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main(app),
                    "quit" => {
                        let app = app.clone();
                        std::thread::spawn(move || {
                            if let Ok(mut manager) = app.state::<Mutex<EngineManager>>().lock() {
                                let _ = manager.stop(&app);
                            }
                            app.exit(0);
                        });
                    }
                    _ => (),
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        show_main(tray.app_handle());
                    }
                });
            tray.build(app)?;
            let refresh_app = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
                if let Ok(store) = community_store(&refresh_app) {
                    let _ = store.refresh_due();
                }
            });
            let failover_app = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(15));
                let disconnected = failover_app
                    .state::<Mutex<EngineManager>>()
                    .lock()
                    .map(|mut manager| manager.status(&failover_app).status == "disconnected")
                    .unwrap_or(false);
                let Ok(store) = community_store(&failover_app) else {
                    continue;
                };
                if !store.snapshot().is_ok_and(|snapshot| {
                    snapshot.automatic_failover && snapshot.acknowledged_warning
                }) {
                    continue;
                }
                if failover_app
                    .state::<Mutex<FailoverPlan>>()
                    .lock()
                    .map(|plan| plan.remaining.is_empty())
                    .unwrap_or(true)
                {
                    continue;
                }
                if !disconnected && health::active_https_available() {
                    continue;
                }
                let next = failover_app
                    .state::<Mutex<FailoverPlan>>()
                    .lock()
                    .ok()
                    .and_then(|mut plan| {
                        if plan.remaining.is_empty() {
                            None
                        } else {
                            Some((plan.remaining.remove(0), plan.generation))
                        }
                    });
                let Some((id, generation)) = next else {
                    continue;
                };
                let Ok(store) = community_store(&failover_app) else {
                    continue;
                };
                let Ok(candidate) = store.candidate(&id) else {
                    continue;
                };
                let engine_dir = match failover_app.state::<Mutex<EngineManager>>().lock() {
                    Ok(manager) => match manager.engine_dir(&failover_app) {
                        Ok(path) => path,
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                };
                let runtime = match failover_app.path().app_local_data_dir() {
                    Ok(path) => path.join("community-tests"),
                    Err(_) => continue,
                };
                let result = health::scan(
                    vec![candidate.clone()],
                    engine_dir.join("sing-box.exe"),
                    runtime,
                    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                    store.history().unwrap_or_default(),
                );
                if result
                    .as_ref()
                    .ok()
                    .and_then(|items| items.first())
                    .is_some_and(|item| item.working)
                {
                    // Hold the plan until switching completes; a manual disconnect
                    // invalidates an in-flight preflight before it can reconnect.
                    let plan_state = failover_app.state::<Mutex<FailoverPlan>>();
                    let Ok(plan) = plan_state.lock() else {
                        continue;
                    };
                    if plan.generation != generation {
                        continue;
                    }
                    if !store
                        .snapshot()
                        .is_ok_and(|snapshot| snapshot.automatic_failover)
                    {
                        continue;
                    }
                    let profile = ProxyProfile {
                        name: format!("Community · {}", candidate.source_name),
                        config_link: candidate.uri,
                    };
                    if failover_app
                        .state::<Mutex<EngineManager>>()
                        .lock()
                        .ok()
                        .and_then(|mut manager| manager.start(&failover_app, &profile).ok())
                        .is_some()
                    {
                        let _ = store.record_result(&id, true);
                    } else {
                        let _ = store.record_result(&id, false);
                    }
                } else {
                    let _ = store.record_result(&id, false);
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.hide().is_ok() {
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            hide_to_tray,
            get_status,
            save_profile,
            load_profile,
            forget_profile,
            start_tunnel,
            stop_tunnel,
            restart_discord,
            community_snapshot,
            save_community_sources,
            set_community_preferences,
            refresh_community,
            scan_community,
            cancel_community_scan,
            clear_community_data,
            connect_community
        ])
        .run(tauri::generate_context!())
        .expect("error while running DisRoute");
}
