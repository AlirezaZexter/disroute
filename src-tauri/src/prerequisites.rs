use crate::engine::{hidden_command, is_elevated, EngineManager};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const INSTALLER: &str = "ProxiFyre-2.6.1-win-x64-setup.exe";
const HASH: &str = "c08cbb5c15acd04d77d7c330712ae366d2a9a8e2a290586e8fe9aa73c78a1908";
const MAX_INSTALLER_SIZE: u64 = 16 * 1024 * 1024;

#[derive(Default)]
pub struct SetupControl(pub Mutex<()>);

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PrerequisiteStatus {
    pub packet_filter: bool,
    pub visual_cpp: bool,
    pub dot_net: bool,
    pub webview: bool,
    pub pending_reboot: bool,
    pub elevated: bool,
    pub engines: bool,
    pub installer_available: bool,
    pub supported_platform: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub status: PrerequisiteStatus,
    pub outcome: &'static str,
}

pub fn check(app: &AppHandle) -> Result<PrerequisiteStatus, String> {
    let mut status = platform_check()?;
    let engine = EngineManager::default().engine_dir(app)?;
    status.engines =
        engine.join("ProxiFyre.exe").is_file() && engine.join("sing-box.exe").is_file();
    status.elevated = is_elevated();
    status.installer_available = installer_path(app).is_ok();
    Ok(status)
}

#[cfg(windows)]
fn platform_check() -> Result<PrerequisiteStatus, String> {
    if std::env::consts::ARCH != "x86_64" {
        return Ok(PrerequisiteStatus::default());
    }
    let root = std::env::var_os("SystemRoot").ok_or("مسیر Windows پیدا نشد.")?;
    let exe = PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let encoded = encode_script(include_str!("../windows/prerequisites.ps1"));
    let mut command = hidden_command(exe);
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-EncodedCommand",
        &encoded,
    ]);
    let bytes = bounded_output(&mut command, Duration::from_secs(15))?;
    let text = String::from_utf8_lossy(&bytes);
    let mut status: PrerequisiteStatus =
        serde_json::from_str(text.trim_start_matches('\u{feff}').trim())
            .map_err(|_| "نتیجهٔ بررسی پیش‌نیازها خوانده نشد. دوباره بررسی کنید.")?;
    status.supported_platform = true;
    Ok(status)
}

#[cfg(not(windows))]
fn platform_check() -> Result<PrerequisiteStatus, String> {
    Ok(PrerequisiteStatus::default())
}

fn encode_script(script: &str) -> String {
    STANDARD.encode(
        script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>(),
    )
}

fn bounded_output(command: &mut Command, timeout: Duration) -> Result<Vec<u8>, String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "بررسی پیش‌نیازها اجرا نشد.")?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err("بررسی Windows ناموفق بود؛ دوباره بررسی کنید.".into());
                }
                let mut output = Vec::new();
                if let Some(stdout) = child.stdout.take() {
                    stdout
                        .take(8193)
                        .read_to_end(&mut output)
                        .map_err(|_| "خواندن نتیجه ناموفق بود.")?;
                }
                if output.len() > 8192 {
                    return Err("نتیجهٔ بررسی بیش از حد بزرگ بود.".into());
                }
                return Ok(output);
            }
            Ok(None) if start.elapsed() < timeout => std::thread::sleep(Duration::from_millis(100)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("بررسی پیش‌نیازها طولانی شد. دوباره بررسی کنید.".into());
            }
        }
    }
}

fn installer_path(app: &AppHandle) -> Result<PathBuf, String> {
    // Resolve only application-owned fixed paths; no URL or path is accepted
    // from the webview, community data, or user configuration.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let portable = parent.join("prerequisites").join(INSTALLER);
            if portable.is_file() {
                return Ok(portable);
            }
        }
    }
    let bundled = app
        .path()
        .resource_dir()
        .map_err(|_| "مسیر منابع برنامه پیدا نشد.")?
        .join("resources/prerequisites")
        .join(INSTALLER);
    if bundled.is_file() {
        return Ok(bundled);
    }
    Err("نصب‌کنندهٔ پیش‌نیاز همراه این نسخه نیست؛ آخرین فایل نصب DisRoute را دریافت کنید.".into())
}

fn verify_installer(path: &Path) -> Result<std::fs::File, String> {
    // Deny concurrent writes/deletion while checking and launching on Windows.
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1); // FILE_SHARE_READ
    }
    let mut file = options.open(path).map_err(|_| "فایل نصب‌کننده باز نشد.")?;
    let size = file
        .metadata()
        .map_err(|_| "فایل نصب‌کننده خوانده نشد.")?
        .len();
    if size == 0 || size > MAX_INSTALLER_SIZE {
        return Err("اندازهٔ نصب‌کننده معتبر نیست.".into());
    }
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 16384];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| "بررسی فایل نصب‌کننده ناموفق بود.")?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    if format!("{:x}", hash.finalize()) != HASH {
        return Err("فایل نصب‌کننده تغییر کرده است؛ نصب متوقف شد. DisRoute را از صفحهٔ Releases دوباره دریافت کنید.".into());
    }
    Ok(file)
}

fn installer_outcome(code: Option<i32>) -> Result<&'static str, String> {
    match code {
        Some(0) => Ok("completed"),
        Some(3010 | 1641) => Ok("restartRequired"),
        Some(1602) => Ok("cancelled"),
        Some(value) => Err(format!("نصب پیش‌نیازها کامل نشد (کد {value}). پیام نصب‌کننده را بررسی کنید و دوباره امتحان کنید.")),
        None => Err("نصب‌کننده بدون نتیجه بسته شد. دوباره بررسی کنید.".into()),
    }
}

pub fn install(app: &AppHandle) -> Result<InstallResult, String> {
    let control = app.state::<SetupControl>();
    let _guard = control
        .0
        .try_lock()
        .map_err(|_| "یک عملیات اتصال یا نصب در حال اجراست؛ پس از پایان دوباره امتحان کنید.")?;
    let status = check(app)?;
    if !status.supported_platform {
        return Err("نصب پیش‌نیازها فقط روی Windows x64 در دسترس است.".into());
    }
    if !status.elevated {
        return Err("اجازهٔ Administrator لازم است؛ نسخهٔ نصب‌شده را دوباره باز کنید و پیام Windows را تأیید کنید.".into());
    }
    if !status.dot_net {
        return Err("ابتدا .NET Framework 4.7.2 یا جدیدتر را از Windows Update نصب کنید، سپس دوباره بررسی کنید.".into());
    }
    if app
        .state::<Mutex<EngineManager>>()
        .lock()
        .map_err(|_| "وضعیت موتور خوانده نشد.")?
        .status(app)
        .status
        == "connected"
    {
        return Err("پیش از نصب پیش‌نیازها اتصال را قطع کنید.".into());
    }
    if status.packet_filter && status.visual_cpp {
        return Ok(InstallResult {
            status,
            outcome: "alreadyInstalled",
        });
    }
    if status.pending_reboot {
        return Ok(InstallResult {
            status,
            outcome: "restartRequired",
        });
    }
    let path = installer_path(app)?;
    let _verified_file = verify_installer(&path)?;
    // Interactive upstream installer: user sees its terms, download progress,
    // cancellation and restart requests. No quiet flags or service start.
    let mut child = Command::new(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "نصب‌کننده باز نشد. Antivirus یا پیام Windows را بررسی کنید.")?;
    let outcome = loop {
        if let Some(status) = child.try_wait().map_err(|_| "نتیجهٔ نصب‌کننده خوانده نشد.")?
        {
            break installer_outcome(status.code())?;
        }
        // A user may leave the terms/download dialog open. Keep the operation
        // gate until the interactive installer exits; never kill it mid-install
        // or allow a second installation/connection while it is still running.
        std::thread::sleep(Duration::from_millis(300));
    };
    let status = check(app)?;
    let outcome = if outcome == "completed" && (!status.packet_filter || !status.visual_cpp) {
        "incomplete"
    } else {
        outcome
    };
    Ok(InstallResult { status, outcome })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_installer_success_restart_cancel_and_failure() {
        assert_eq!(installer_outcome(Some(0)).unwrap(), "completed");
        assert_eq!(installer_outcome(Some(3010)).unwrap(), "restartRequired");
        assert_eq!(installer_outcome(Some(1641)).unwrap(), "restartRequired");
        assert_eq!(installer_outcome(Some(1602)).unwrap(), "cancelled");
        assert!(installer_outcome(Some(1603)).is_err());
        assert!(installer_outcome(None).is_err());
    }
    #[test]
    fn script_is_passed_as_utf16_not_shell_interpolated() {
        let value = "فارسی ' $() ;";
        let bytes = STANDARD.decode(encode_script(value)).unwrap();
        let units = bytes
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes([v[0], v[1]]))
            .collect::<Vec<_>>();
        assert_eq!(String::from_utf16(&units).unwrap(), value);
    }
    #[test]
    fn rejects_tampered_installer() {
        let path =
            std::env::temp_dir().join(format!("disroute-prereq-test-{}", std::process::id()));
        std::fs::write(&path, b"not an installer").unwrap();
        assert!(verify_installer(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn rejects_empty_and_oversized_installers() {
        let path =
            std::env::temp_dir().join(format!("disroute-prereq-size-{}", std::process::id()));
        std::fs::write(&path, []).unwrap();
        assert!(verify_installer(&path).unwrap_err().contains("اندازه"));
        let file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.set_len(MAX_INSTALLER_SIZE + 1).unwrap();
        drop(file);
        assert!(verify_installer(&path).unwrap_err().contains("اندازه"));
        std::fs::remove_file(path).unwrap();
    }
    #[cfg(windows)]
    #[test]
    fn prerequisite_probe_has_a_bounded_timeout() {
        let exe = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let mut command = hidden_command(exe);
        command.args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-EncodedCommand",
            &encode_script("Start-Sleep -Seconds 30"),
        ]);
        let start = Instant::now();
        assert!(bounded_output(&mut command, Duration::from_millis(50)).is_err());
        assert!(start.elapsed() < Duration::from_secs(5));
    }
    #[test]
    fn detects_windows_prerequisites_without_mutation() {
        let result = platform_check().unwrap();
        #[cfg(windows)]
        assert_eq!(
            result.supported_platform,
            std::env::consts::ARCH == "x86_64"
        );
        #[cfg(not(windows))]
        assert!(!result.supported_platform);
    }

    #[test]
    #[ignore = "prepare checksum-pinned installer resources first"]
    fn verifies_actual_bundled_prerequisite_installer() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources/prerequisites")
            .join(INSTALLER);
        let verified = verify_installer(&path).unwrap();
        assert!(verified.metadata().unwrap().len() > 0);
    }
}
