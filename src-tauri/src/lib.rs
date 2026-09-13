mod engine;
mod model;

use engine::EngineManager;
use model::{AppStatus, ProxyProfile};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
fn get_status(app: AppHandle, manager: State<'_, Mutex<EngineManager>>) -> AppStatus {
    manager.lock().expect("engine lock poisoned").status(&app)
}

#[tauri::command]
fn save_profile(app: AppHandle, profile: ProxyProfile) -> Result<(), String> {
    profile.validate()?;
    let mut safe_profile = profile;
    safe_profile.password.clear();
    let data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&safe_profile).map_err(|e| e.to_string())?;
    std::fs::write(data_dir.join("profile.json"), json).map_err(|e| e.to_string())
}

#[tauri::command]
fn start_tunnel(
    app: AppHandle,
    profile: ProxyProfile,
    manager: State<'_, Mutex<EngineManager>>,
) -> Result<AppStatus, String> {
    profile.validate()?;
    manager.lock().map_err(|_| "engine lock poisoned")?.start(&app, &profile)
}

#[tauri::command]
fn stop_tunnel(
    app: AppHandle,
    manager: State<'_, Mutex<EngineManager>>,
) -> Result<AppStatus, String> {
    manager.lock().map_err(|_| "engine lock poisoned")?.stop(&app)
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(EngineManager::default()))
        .invoke_handler(tauri::generate_handler![get_status, save_profile, start_tunnel, stop_tunnel])
        .run(tauri::generate_context!())
        .expect("error while running DisRoute");
}

