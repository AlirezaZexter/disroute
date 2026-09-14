mod engine;
mod model;
mod profile_store;
mod voice_proxy;

use engine::EngineManager;
use model::{AppStatus, ProxyProfile};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

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
) -> Result<AppStatus, String> {
    profile.validate()?;
    manager
        .lock()
        .map_err(|_| "engine lock poisoned")?
        .start(&app, &profile)
}

#[tauri::command]
fn stop_tunnel(
    app: AppHandle,
    manager: State<'_, Mutex<EngineManager>>,
) -> Result<AppStatus, String> {
    manager
        .lock()
        .map_err(|_| "engine lock poisoned")?
        .stop(&app)
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(EngineManager::default()))
        .setup(|app| {
            use tauri::{
                menu::{Menu, MenuItem},
                tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
            };
            let show = MenuItem::with_id(app, "show", "باز کردن DisRoute", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "قطع اتصال و خروج", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let mut tray = TrayIconBuilder::with_id("disroute-tray")
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
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
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
            stop_tunnel
        ])
        .run(tauri::generate_context!())
        .expect("error while running DisRoute");
}
