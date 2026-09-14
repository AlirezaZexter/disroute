use crate::model::{AppStatus, ProxyProfile};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct EngineManager {
    proxifyre: Option<Child>,
    sing_box: Option<Child>,
}

impl EngineManager {
    fn engine_dir(&self, app: &AppHandle) -> Result<PathBuf, String> {
        if let Ok(path) = std::env::var("DISROUTE_ENGINE_DIR") {
            return Ok(PathBuf::from(path));
        }
        Ok(app
            .path()
            .app_local_data_dir()
            .map_err(|e| e.to_string())?
            .join("engine"))
    }

    pub fn status(&mut self, app: &AppHandle) -> AppStatus {
        let proxy_running = child_running(&mut self.proxifyre);
        let tunnel_running = child_running(&mut self.sing_box);
        let running = proxy_running && tunnel_running;
        if proxy_running != tunnel_running {
            stop_child(&mut self.proxifyre);
            stop_child(&mut self.sing_box);
        }
        let engine_ready = self
            .engine_dir(app)
            .map(|p| p.join("ProxiFyre.exe").is_file() && p.join("sing-box.exe").is_file())
            .unwrap_or(false);
        AppStatus {
            status: if running { "connected" } else { "disconnected" },
            engine_ready,
            is_elevated: is_elevated(),
            message: if running {
                "فقط پردازش‌های Discord از پروکسی عبور می‌کنند.".into()
            } else if engine_ready {
                "موتور آماده است؛ مشخصات اتصال را وارد کنید.".into()
            } else {
                "موتور شبکه هنوز نصب نشده است.".into()
            },
        }
    }

    pub fn start(&mut self, app: &AppHandle, profile: &ProxyProfile) -> Result<AppStatus, String> {
        if self.proxifyre.is_some() || self.sing_box.is_some() {
            self.stop(app)?;
        }
        let engine_dir = self.engine_dir(app)?;
        let proxifyre_exe = engine_dir.join("ProxiFyre.exe");
        let sing_box_exe = engine_dir.join("sing-box.exe");
        if !proxifyre_exe.is_file() || !sing_box_exe.is_file() {
            return Err(format!(
                "فایل‌های ProxiFyre.exe و sing-box.exe باید در {} باشند.",
                engine_dir.display()
            ));
        }
        if !is_elevated() {
            return Err(
                "برای فعال‌کردن مسیریابی پردازشی، DisRoute را با دسترسی Administrator اجرا کنید."
                    .into(),
            );
        }
        let proxy_config =
            serde_json::to_string_pretty(&profile.proxifyre_config()).map_err(|e| e.to_string())?;
        let tunnel_config =
            serde_json::to_string_pretty(&profile.sing_box_config()?).map_err(|e| e.to_string())?;
        std::fs::write(engine_dir.join("app-config.json"), proxy_config)
            .map_err(|e| format!("ذخیرهٔ تنظیمات ProxiFyre ناموفق بود: {e}"))?;
        std::fs::write(engine_dir.join("sing-box.json"), tunnel_config)
            .map_err(|e| format!("ذخیرهٔ تنظیمات sing-box ناموفق بود: {e}"))?;

        let check = Command::new(&sing_box_exe)
            .current_dir(&engine_dir)
            .args(["check", "-c", "sing-box.json"])
            .output()
            .map_err(|e| format!("اعتبارسنجی sing-box اجرا نشد: {e}"))?;
        if !check.status.success() {
            return Err(format!(
                "کانفیگ VLESS پذیرفته نشد: {}",
                String::from_utf8_lossy(&check.stderr)
            ));
        }
        self.sing_box = Some(
            Command::new(&sing_box_exe)
                .current_dir(&engine_dir)
                .args(["run", "-c", "sing-box.json"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("اجرای sing-box ناموفق بود: {e}"))?,
        );
        thread::sleep(Duration::from_millis(250));
        if !child_running(&mut self.sing_box) {
            return Err("sing-box بلافاصله متوقف شد.".into());
        }

        let child = match Command::new(&proxifyre_exe)
            .current_dir(&engine_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                stop_child(&mut self.sing_box);
                let _ = std::fs::remove_file(engine_dir.join("sing-box.json"));
                return Err(format!("اجرای موتور ناموفق بود: {error}"));
            }
        };
        self.proxifyre = Some(child);
        thread::sleep(Duration::from_millis(350));
        if let Some(code) = self
            .proxifyre
            .as_mut()
            .and_then(|child| child.try_wait().ok())
            .flatten()
        {
            self.proxifyre = None;
            stop_child(&mut self.sing_box);
            return Err(format!(
                "موتور بلافاصله متوقف شد (کد خروج {code}). درایور و گزارش‌ها را بررسی کنید."
            ));
        }
        Ok(self.status(app))
    }

    pub fn stop(&mut self, app: &AppHandle) -> Result<AppStatus, String> {
        stop_child(&mut self.proxifyre);
        stop_child(&mut self.sing_box);
        if let Ok(engine_dir) = self.engine_dir(app) {
            let _ = std::fs::remove_file(engine_dir.join("sing-box.json"));
        }
        Ok(self.status(app))
    }
}

impl Drop for EngineManager {
    fn drop(&mut self) {
        stop_child(&mut self.proxifyre);
        stop_child(&mut self.sing_box);
    }
}

fn child_running(child: &mut Option<Child>) -> bool {
    let running = child
        .as_mut()
        .is_some_and(|process| process.try_wait().ok().flatten().is_none());
    if !running {
        *child = None;
    }
    running
}

fn stop_child(child: &mut Option<Child>) {
    if let Some(mut process) = child.take() {
        let _ = process.kill();
        let _ = process.wait();
    }
}

#[cfg(windows)]
fn is_elevated() -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token = Default::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0;
        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        )
        .is_ok();
        let _ = CloseHandle(token);
        result && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
fn is_elevated() -> bool {
    false
}
