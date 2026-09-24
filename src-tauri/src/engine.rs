use crate::model::{AppStatus, ProxyProfile};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub(crate) fn hidden_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    command
}

#[derive(Default)]
pub struct EngineManager {
    active_profile: Option<ProxyProfile>,
    connection_notice: Option<String>,
    voice_proxy: Option<crate::voice_proxy::VoiceProxy>,
    proxifyre: Option<Child>,
    sing_box: Option<Child>,
    runtime_dir: Option<PathBuf>,
}

impl EngineManager {
    pub(crate) fn engine_dir(&self, app: &AppHandle) -> Result<PathBuf, String> {
        if let Ok(path) = std::env::var("DISROUTE_ENGINE_DIR") {
            return Ok(PathBuf::from(path));
        }

        // Portable distributions keep both engines beside DisRoute.exe.
        // Prefer that directory when it exists, while retaining AppData as the
        // development/installed-build fallback.
        if let Ok(executable) = std::env::current_exe() {
            if let Some(parent) = executable.parent() {
                let portable_engine = parent.join("engine");
                if portable_engine.is_dir() {
                    return Ok(portable_engine);
                }
            }
        }

        // Installer builds bundle the pinned engines as Tauri resources so
        // every signed application update can replace them atomically too.
        if let Ok(resource_dir) = app.path().resource_dir() {
            let bundled_engine = resource_dir.join("resources").join("engine");
            if bundled_engine.is_dir() {
                return Ok(bundled_engine);
            }
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
        let running = proxy_running && tunnel_running && self.voice_proxy.is_some();
        if !running {
            self.active_profile = None;
            self.connection_notice = None;
            self.voice_proxy.take();
            stop_child(&mut self.proxifyre);
            stop_child(&mut self.sing_box);
            if let Some(dir) = &self.runtime_dir {
                let _ = std::fs::remove_file(dir.join("sing-box.json"));
            }
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
                self.connection_notice.clone().unwrap_or_else(|| {
                    "موتور فعال است؛ مسیر Discord از طریق پروکسی برقرار شد.".into()
                })
            } else if engine_ready {
                "موتور آماده است؛ مشخصات اتصال را وارد کنید.".into()
            } else {
                "موتور شبکه هنوز نصب نشده است.".into()
            },
        }
    }

    pub fn start(&mut self, app: &AppHandle, profile: &ProxyProfile) -> Result<AppStatus, String> {
        let previous = self.active_profile.clone();
        match self.start_inner(app, profile) {
            Ok(status) => {
                self.active_profile = Some(profile.clone());
                Ok(status)
            }
            Err(error) => {
                if let Some(previous) = previous {
                    if previous.config_link != profile.config_link
                        && self.start_inner(app, &previous).is_ok()
                    {
                        self.active_profile = Some(previous);
                        return Err(format!("{error} اتصال قبلی دوباره برقرار شد."));
                    }
                }
                self.active_profile = None;
                Err(error)
            }
        }
    }

    fn start_inner(
        &mut self,
        app: &AppHandle,
        profile: &ProxyProfile,
    ) -> Result<AppStatus, String> {
        if self.proxifyre.is_some() || self.sing_box.is_some() || self.voice_proxy.is_some() {
            self.stop(app)?;
        }
        let engine_dir = self.engine_dir(app)?;
        let proxifyre_exe = engine_dir.join("ProxiFyre.exe");
        let sing_box_exe = engine_dir.join("sing-box.exe");
        let proxy_config_path = engine_dir.join("app-config.json");
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
        ensure_firewall_rules(&proxifyre_exe)?;
        // Keep runtime secrets outside the portable/shareable application folder.
        let engine_dir = app
            .path()
            .app_local_data_dir()
            .map_err(|e| e.to_string())?
            .join("runtime");
        std::fs::create_dir_all(&engine_dir).map_err(|e| e.to_string())?;
        self.runtime_dir = Some(engine_dir.clone());
        let proxy_config =
            serde_json::to_string_pretty(&profile.proxifyre_config()).map_err(|e| e.to_string())?;
        let tunnel_config =
            serde_json::to_string_pretty(&profile.sing_box_config()?).map_err(|e| e.to_string())?;
        // ProxiFyre locates this non-secret routing file beside its executable.
        std::fs::write(proxy_config_path, proxy_config)
            .map_err(|e| format!("ذخیرهٔ تنظیمات ProxiFyre ناموفق بود: {e}"))?;
        std::fs::write(engine_dir.join("sing-box.json"), tunnel_config)
            .map_err(|e| format!("ذخیرهٔ تنظیمات sing-box ناموفق بود: {e}"))?;

        let check = hidden_command(&sing_box_exe)
            .current_dir(&engine_dir)
            .args(["check", "-c", "sing-box.json"])
            .output()
            .map_err(|e| format!("اعتبارسنجی sing-box اجرا نشد: {e}"))?;
        if !check.status.success() {
            return Err(format!(
                "کانفیگ پروکسی پذیرفته نشد: {}",
                String::from_utf8_lossy(&check.stderr)
            ));
        }
        self.sing_box = Some(
            hidden_command(&sing_box_exe)
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

        self.voice_proxy = match crate::voice_proxy::VoiceProxy::start(2080, 2081) {
            Ok(proxy) => Some(proxy),
            Err(error) => {
                stop_child(&mut self.sing_box);
                let _ = std::fs::remove_file(engine_dir.join("sing-box.json"));
                return Err(format!(
                    "راه‌اندازی مسیر وویس روی پورت ۲۰۸۰ ناموفق بود: {error}"
                ));
            }
        };
        let child = match hidden_command(&proxifyre_exe)
            .current_dir(&engine_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                self.voice_proxy.take();
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
            self.voice_proxy.take();
            stop_child(&mut self.sing_box);
            return Err(format!(
                "موتور بلافاصله متوقف شد (کد خروج {code}). درایور و گزارش‌ها را بررسی کنید."
            ));
        }

        let https_warning = match probe_proxy() {
            Ok(warning) => warning,
            Err(error) => {
                self.voice_proxy.take();
                stop_child(&mut self.proxifyre);
                stop_child(&mut self.sing_box);
                let _ = std::fs::remove_file(engine_dir.join("sing-box.json"));
                return Err(format!("آزمایش واقعی پروکسی ناموفق بود: {error}"));
            }
        };
        self.connection_notice = https_warning;
        Ok(self.status(app))
    }

    pub fn stop(&mut self, app: &AppHandle) -> Result<AppStatus, String> {
        self.active_profile = None;
        self.connection_notice = None;
        self.voice_proxy.take();
        stop_child(&mut self.proxifyre);
        stop_child(&mut self.sing_box);
        if let Some(engine_dir) = &self.runtime_dir {
            let _ = std::fs::remove_file(engine_dir.join("sing-box.json"));
        }
        Ok(self.status(app))
    }
}

impl Drop for EngineManager {
    fn drop(&mut self) {
        self.voice_proxy.take();
        stop_child(&mut self.proxifyre);
        stop_child(&mut self.sing_box);
        if let Some(dir) = &self.runtime_dir {
            let _ = std::fs::remove_file(dir.join("sing-box.json"));
        }
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

fn probe_proxy() -> Result<Option<String>, String> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, 2080));
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(3))
        .map_err(|_| "موتور SOCKS محلی روی پورت ۲۰۸۰ پاسخ نمی‌دهد.")?;
    stream
        .set_read_timeout(Some(Duration::from_secs(12)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;

    stream
        .write_all(&[0x05, 0x01, 0x00])
        .map_err(|_| "ارسال handshake به SOCKS ناموفق بود.")?;
    let mut greeting = [0u8; 2];
    stream
        .read_exact(&mut greeting)
        .map_err(|_| "پاسخ handshake از SOCKS دریافت نشد.")?;
    if greeting != [0x05, 0x00] {
        return Err("موتور SOCKS روش اتصال بدون رمز را نپذیرفت.".into());
    }

    let host = b"discord.com";
    let mut request = vec![0x05, 0x01, 0x00, 0x03, host.len() as u8];
    request.extend_from_slice(host);
    request.extend_from_slice(&443u16.to_be_bytes());
    stream
        .write_all(&request)
        .map_err(|_| "درخواست آزمایشی Discord ارسال نشد.")?;

    let mut response = [0u8; 4];
    stream
        .read_exact(&mut response)
        .map_err(|_| "سرور پروکسی در زمان مقرر پاسخ نداد.")?;
    if response[0] != 0x05 || response[1] != 0x00 {
        return Err(format!(
            "تونل درخواست آزمایشی را رد کرد (کد SOCKS {}).",
            response[1]
        ));
    }
    drop(stream);
    let warning = "تونل پروکسی برقرار شد، اما تست تکمیلی HTTPS در این ویندوز کامل نشد. Discord را باز کنید؛ اگر وصل نشد، ساعت Windows و تنظیمات Firewall یا Antivirus را بررسی کنید.";
    let Some(system_root) = std::env::var_os("SystemRoot") else {
        return Ok(Some(warning.into()));
    };
    let curl = PathBuf::from(system_root).join("System32").join("curl.exe");
    let response = match hidden_command(&curl)
        .args([
            "--proxy",
            "socks5h://127.0.0.1:2080",
            "--connect-timeout",
            "5",
            "--max-time",
            "15",
            "--silent",
            "--show-error",
            "--output",
            "NUL",
            "--write-out",
            "%{http_code}",
            "--user-agent",
            "Mozilla/5.0",
            "https://discord.com/api/v10/gateway",
        ])
        .output()
    {
        Ok(response) => response,
        Err(_) => return Ok(Some(warning.into())),
    };

    // Any real HTTP response proves that TLS reached Discord through the proxy.
    // Requiring exactly 200 caused false failures for redirects and edge/WAF
    // responses that vary by Windows version and outbound server IP.
    if response.status.success() && is_http_response(&response.stdout) {
        Ok(None)
    } else {
        Ok(Some(warning.into()))
    }
}

fn is_http_response(value: &[u8]) -> bool {
    std::str::from_utf8(value)
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .is_some_and(|code| (100..600).contains(&code))
}

#[cfg(windows)]
fn ensure_firewall_rules(program: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    let program = program
        .to_str()
        .ok_or("مسیر ProxiFyre برای Firewall معتبر نیست.")?;
    let program = firewall_program(program)?;
    for (name, protocol) in [
        ("DisRoute ProxiFyre TCP", "TCP"),
        ("DisRoute ProxiFyre UDP", "UDP"),
    ] {
        // Command::args can be re-tokenized by netsh on Windows when an
        // argument itself contains spaces. raw_arg preserves the quoting that
        // netsh documents for rule names and executable paths.
        let exists = hidden_command("netsh.exe")
            .raw_arg(format!("advfirewall firewall show rule name=\"{name}\""))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        let operation = if exists {
            format!("set rule name=\"{name}\" new")
        } else {
            format!("add rule name=\"{name}\"")
        };
        let add_args = format!(
            "advfirewall firewall {operation} dir=in action=allow program=\"{program}\" enable=yes profile=any protocol={protocol}"
        );
        let result = hidden_command("netsh.exe")
            .raw_arg(add_args)
            .output()
            .map_err(|e| format!("اجرای تنظیم Firewall ناموفق بود: {e}"))?;
        if !result.status.success() {
            let detail = String::from_utf8_lossy(if result.stderr.is_empty() {
                &result.stdout
            } else {
                &result.stderr
            });
            return Err(format!(
                "ساخت مجوز Firewall برای {protocol} ناموفق بود: {}",
                detail.trim()
            ));
        }
    }
    Ok(())
}

// Tauri's resource resolver can return a verbatim (\\?\) filesystem path.
// CreateProcess accepts it, but Windows Firewall's application parser does not.
fn firewall_program(value: &str) -> Result<String, String> {
    let value = value.strip_prefix(r"\\?\").unwrap_or(value);
    let bytes = value.as_bytes();
    if bytes.len() < 4
        || !bytes[0].is_ascii_alphabetic()
        || &bytes[1..3] != b":\\"
        || value
            .chars()
            .any(|c| c.is_control() || matches!(c, '"' | '*' | '?' | '<' | '>' | '|'))
        || value.encode_utf16().count() >= 260
    {
        return Err("مسیر موتور برای Firewall معتبر نیست. برنامه را در یک مسیر کوتاه روی درایو محلی نصب کنید.".into());
    }
    Ok(value.to_owned())
}

#[cfg(not(windows))]
fn ensure_firewall_rules(_program: &std::path::Path) -> Result<(), String> {
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::is_http_response;

    #[test]
    #[ignore = "requires elevation; updates only DisRoute ProxiFyre TCP/UDP rules"]
    fn installed_firewall_roundtrip() {
        let path =
            std::env::var_os("DISROUTE_FIREWALL_TEST_PROGRAM").expect("set test program path");
        let path = std::fs::canonicalize(path).unwrap();
        super::ensure_firewall_rules(&path).unwrap();
        super::ensure_firewall_rules(&path).unwrap();
    }

    #[test]
    fn firewall_accepts_tauri_verbatim_drive_paths() {
        assert_eq!(
            super::firewall_program(r"\\?\C:\Program Files\DisRoute\ProxiFyre.exe").unwrap(),
            r"C:\Program Files\DisRoute\ProxiFyre.exe"
        );
        assert_eq!(
            super::firewall_program(r"C:\Users\کاربر\DisRoute\ProxiFyre.exe").unwrap(),
            r"C:\Users\کاربر\DisRoute\ProxiFyre.exe"
        );
    }

    #[test]
    fn firewall_rejects_device_relative_and_injected_paths() {
        for path in [
            r"\\.\PhysicalDrive0",
            r"engine\ProxiFyre.exe",
            "C:\\bad\"path.exe",
            "C:\\bad\npath.exe",
        ] {
            assert!(super::firewall_program(path).is_err());
        }
    }

    #[test]
    fn accepts_reachable_discord_http_responses() {
        for code in [b"200".as_slice(), b"301", b"403", b"503"] {
            assert!(is_http_response(code));
        }
    }

    #[test]
    fn rejects_missing_http_responses() {
        for code in [b"000".as_slice(), b"", b"timeout"] {
            assert!(!is_http_response(code));
        }
    }
}
