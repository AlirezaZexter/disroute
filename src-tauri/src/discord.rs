use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
struct DiscordChannel {
    process_name: &'static str,
    shortcut_names: &'static [&'static str],
}

#[cfg(windows)]
const CHANNELS: &[DiscordChannel] = &[
    DiscordChannel {
        process_name: "Discord.exe",
        shortcut_names: &["Discord.lnk", "Discord Inc\\Discord.lnk"],
    },
    DiscordChannel {
        process_name: "DiscordCanary.exe",
        shortcut_names: &["Discord Canary.lnk", "Discord Inc\\Discord Canary.lnk"],
    },
    DiscordChannel {
        process_name: "DiscordPTB.exe",
        shortcut_names: &["Discord PTB.lnk", "Discord Inc\\Discord PTB.lnk"],
    },
];

#[cfg(windows)]
fn hidden_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null());
    command
}

#[cfg(windows)]
fn is_running(process_name: &str) -> bool {
    let filter = format!("IMAGENAME eq {process_name}");
    hidden_command("tasklist.exe")
        .args(["/FI", &filter, "/FO", "CSV", "/NH"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .to_ascii_lowercase()
                .contains(&format!("\"{}\"", process_name.to_ascii_lowercase()))
        })
        .unwrap_or(false)
}

#[cfg(windows)]
fn find_shortcut(channel: &DiscordChannel) -> Option<PathBuf> {
    let programs = PathBuf::from(std::env::var_os("APPDATA")?)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs");
    channel
        .shortcut_names
        .iter()
        .map(|name| programs.join(name))
        .find(|path| path.is_file())
}

#[cfg(windows)]
pub fn restart() -> Result<String, String> {
    let channel = CHANNELS
        .iter()
        .find(|channel| is_running(channel.process_name))
        .or_else(|| {
            CHANNELS
                .iter()
                .find(|channel| find_shortcut(channel).is_some())
        })
        .ok_or("میان‌بُر Discord در Start Menu پیدا نشد. Discord را یک‌بار نصب یا Repair کنید.")?;
    let shortcut = find_shortcut(channel)
        .ok_or("میان‌بُر Discord در Start Menu پیدا نشد. Discord را یک‌بار نصب یا Repair کنید.")?;

    // Discord can keep a headless Electron process after its window closes.
    // Ending only the selected Discord channel clears that stale instance;
    // DisRoute and its networking engines remain untouched.
    let _ = hidden_command("taskkill.exe")
        .args(["/IM", channel.process_name, "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    thread::sleep(Duration::from_millis(600));

    // Ask the user's Explorer shell to open the regular Start Menu shortcut.
    // This avoids inheriting DisRoute's Administrator token.
    hidden_command("explorer.exe")
        .arg(shortcut)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("اجرای دوبارهٔ Discord ناموفق بود: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        if is_running(channel.process_name) {
            return Ok("Discord با یک نمونهٔ تازه اجرا شد؛ اتصال DisRoute روشن ماند.".into());
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err("Discord در زمان مقرر اجرا نشد. میان‌بُر Discord را بررسی کنید.".into())
}

#[cfg(not(windows))]
pub fn restart() -> Result<String, String> {
    Err("راه‌اندازی مجدد Discord فقط در Windows پشتیبانی می‌شود.".into())
}
