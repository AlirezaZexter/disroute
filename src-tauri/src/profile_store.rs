use crate::model::ProxyProfile;
use std::path::Path;

pub fn save(dir: &Path, profile: &ProxyProfile) -> Result<(), String> {
    profile.validate()?;
    let mut bytes = serde_json::to_vec(profile).map_err(|_| "ساخت پروفایل ناموفق بود.")?;
    let result = protect(&bytes, false);
    bytes.fill(0);
    let encrypted = result?;
    std::fs::create_dir_all(dir).map_err(|_| "پوشه ذخیره‌سازی قابل دسترسی نیست.")?;
    let pending = dir.join("profile.dpapi.pending");
    std::fs::write(&pending, encrypted).map_err(|_| "ذخیره امن پروفایل ناموفق بود.")?;
    std::fs::rename(&pending, dir.join("profile.dpapi"))
        .map_err(|_| "جایگزینی پروفایل ناموفق بود.")?;
    Ok(())
}

pub fn load(dir: &Path) -> Result<Option<ProxyProfile>, String> {
    let path = dir.join("profile.dpapi");
    let encrypted = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("خواندن پروفایل ذخیره‌شده ناموفق بود.".into()),
    };
    let mut bytes = protect(&encrypted, true)?;
    let profile = serde_json::from_slice::<ProxyProfile>(&bytes);
    bytes.fill(0);
    let profile = profile.map_err(|_| "پروفایل ذخیره‌شده معتبر نیست؛ دوباره وارد کنید.")?;
    profile.validate()?;
    Ok(Some(profile))
}

pub fn forget(dir: &Path) -> Result<(), String> {
    for name in ["profile.dpapi", "profile.dpapi.pending"] {
        match std::fs::remove_file(dir.join(name)) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err("حذف پروفایل ذخیره‌شده ناموفق بود.".into()),
        }
    }
    Ok(())
}

#[cfg(windows)]
fn protect(bytes: &[u8], decrypt: bool) -> Result<Vec<u8>, String> {
    use windows::Win32::{
        Foundation::{LocalFree, HLOCAL},
        Security::Cryptography::{
            CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
        },
    };
    let input = CRYPT_INTEGER_BLOB {
        cbData: u32::try_from(bytes.len()).map_err(|_| "پروفایل بیش از حد بزرگ است.")?,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        let result = if decrypt {
            CryptUnprotectData(
                &input,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptProtectData(
                &input,
                windows::core::w!("DisRoute profile"),
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        result.map_err(|_| "حفاظت ویندوز قابل دسترسی نیست؛ از همان حساب ویندوز استفاده کنید یا پروفایل را دوباره وارد کنید.")?;
        let buffer = std::slice::from_raw_parts_mut(output.pbData, output.cbData as usize);
        let result = buffer.to_vec();
        for byte in buffer {
            std::ptr::write_volatile(byte, 0);
        }
        let _ = LocalFree(Some(HLOCAL(output.pbData.cast())));
        Ok(result)
    }
}

#[cfg(not(windows))]
fn protect(_: &[u8], _: bool) -> Result<Vec<u8>, String> {
    Err("ذخیره امن فقط در ویندوز در دسترس است.".into())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn encrypted_save_restore_replace_forget() {
        let dir =
            std::env::temp_dir().join(format!("disroute-profile-test-{}", std::process::id()));
        let mut profile = ProxyProfile {
            name: "test".into(),
            vless_link: "vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls"
                .into(),
        };
        assert!(load(&dir).unwrap().is_none());
        save(&dir, &profile).unwrap();
        let encrypted = std::fs::read(dir.join("profile.dpapi")).unwrap();
        assert!(!String::from_utf8_lossy(&encrypted).contains("vless://"));
        assert_eq!(load(&dir).unwrap().unwrap().vless_link, profile.vless_link);
        profile.name = "updated".into();
        save(&dir, &profile).unwrap();
        assert_eq!(load(&dir).unwrap().unwrap().name, "updated");
        std::fs::write(dir.join("profile.dpapi"), b"corrupt").unwrap();
        assert!(load(&dir).is_err());
        forget(&dir).unwrap();
        forget(&dir).unwrap();
        assert!(load(&dir).unwrap().is_none());
        std::fs::remove_dir(dir).unwrap();
    }
}
