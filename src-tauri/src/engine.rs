use crate::model::{AppStatus, ProxyProfile};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct EngineManager {
    child: Option<Child>,
}

impl EngineManager {
    fn engine_dir(&self, app: &AppHandle) -> Result<PathBuf, String> {
        if let Ok(path) = std::env::var("DISROUTE_ENGINE_DIR") {
            return Ok(PathBuf::from(path));
        }
        Ok(app.path().app_local_data_dir().map_err(|e| e.to_string())?.join("engine"))
    }

    pub fn status(&mut self, app: &AppHandle) -> AppStatus {
        let running = self.child.as_mut().and_then(|child| child.try_wait().ok()).flatten().is_none() && self.child.is_some();
        if !running { self.child = None; }
        let engine_ready = self.engine_dir(app).map(|p| p.join("ProxiFyre.exe").is_file()).unwrap_or(false);
        AppStatus {
            status: if running { "connected" } else { "disconnected" },
            engine_ready,
            is_elevated: is_elevated(),
            message: if running { "فقط پردازش‌های Discord از پروکسی عبور می‌کنند.".into() } else if engine_ready { "موتور آماده است؛ مشخصات اتصال را وارد کنید.".into() } else { "موتور شبکه هنوز نصب نشده است.".into() },
        }
    }

    pub fn start(&mut self, app: &AppHandle, profile: &ProxyProfile) -> Result<AppStatus, String> {
        if self.child.is_some() { return Ok(self.status(app)); }
        let engine_dir = self.engine_dir(app)?;
        let executable = engine_dir.join("ProxiFyre.exe");
        if !executable.is_file() {
            return Err(format!("موتور شبکه پیدا نشد: {}", executable.display()));
        }
        if !is_elevated() {
            return Err("برای فعال‌کردن مسیریابی پردازشی، DisRoute را با دسترسی Administrator اجرا کنید.".into());
        }
        let config = serde_json::to_string_pretty(&profile.engine_config()).map_err(|e| e.to_string())?;
        std::fs::write(engine_dir.join("app-config.json"), config).map_err(|e| format!("ذخیرهٔ تنظیمات موتور ناموفق بود: {e}"))?;
        let child = Command::new(&executable)
            .current_dir(&engine_dir)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().map_err(|e| format!("اجرای موتور ناموفق بود: {e}"))?;
        self.child = Some(child);
        thread::sleep(Duration::from_millis(350));
        if let Some(code) = self.child.as_mut().and_then(|child| child.try_wait().ok()).flatten() {
            self.child = None;
            return Err(format!("موتور بلافاصله متوقف شد (کد خروج {code}). درایور و گزارش‌ها را بررسی کنید."));
        }
        Ok(self.status(app))
    }

    pub fn stop(&mut self, app: &AppHandle) -> Result<AppStatus, String> {
        if let Some(mut child) = self.child.take() {
            child.kill().map_err(|e| format!("توقف موتور ناموفق بود: {e}"))?;
            let _ = child.wait();
        }
        Ok(self.status(app))
    }
}

#[cfg(windows)]
fn is_elevated() -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token = Default::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() { return false; }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0;
        let result = GetTokenInformation(token, TokenElevation, Some(&mut elevation as *mut _ as *mut _), std::mem::size_of::<TOKEN_ELEVATION>() as u32, &mut size).is_ok();
        let _ = CloseHandle(token);
        result && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
fn is_elevated() -> bool { false }

