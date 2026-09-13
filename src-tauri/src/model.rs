use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyProfile {
    pub name: String,
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub tls_server_name: String,
}

impl ProxyProfile {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("نام پروفایل نمی‌تواند خالی باشد.".into());
        }
        let (host, port) = split_endpoint(self.endpoint.trim())?;
        if host.is_empty() || port == 0 {
            return Err("آدرس SOCKS5 معتبر نیست.".into());
        }
        if self.use_tls && self.tls_server_name.trim().is_empty() {
            return Err("برای TLS نام سرور را وارد کنید.".into());
        }
        Ok(())
    }

    pub fn engine_config(&self) -> serde_json::Value {
        serde_json::json!({
            "logLevel": "Info",
            "bypassLan": true,
            "proxies": [{
                "appNames": ["Discord.exe", "DiscordCanary.exe", "DiscordPTB.exe"],
                "socks5ProxyEndpoint": self.endpoint.trim(),
                "username": self.username,
                "password": self.password,
                "socks5Transport": if self.use_tls { "TLS" } else { "TCP" },
                "tlsServerName": if self.use_tls { self.tls_server_name.trim() } else { "" },
                "tlsPinnedSha256": "",
                "tlsAllowInvalidCertificate": false,
                "supportedProtocols": ["TCP", "UDP"],
                "supportedAddressFamilies": ["IPv4", "IPv6"]
            }],
            "excludes": []
        })
    }
}

fn split_endpoint(value: &str) -> Result<(&str, u16), String> {
    let (host, port) = if value.starts_with('[') {
        let end = value.find(']').ok_or("آدرس IPv6 معتبر نیست.")?;
        let host = &value[1..end];
        let port = value.get(end + 2..).ok_or("پورت وارد نشده است.")?;
        (host, port)
    } else {
        value.rsplit_once(':').ok_or("آدرس باید به شکل host:port باشد.")?
    };
    let port = port.parse::<u16>().map_err(|_| "پورت باید بین ۱ تا ۶۵۵۳۵ باشد.")?;
    Ok((host, port))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub status: &'static str,
    pub engine_ready: bool,
    pub is_elevated: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> ProxyProfile {
        ProxyProfile { name: "Discord".into(), endpoint: "127.0.0.1:1080".into(), username: "u".into(), password: "secret".into(), use_tls: false, tls_server_name: "".into() }
    }

    #[test]
    fn generated_config_routes_only_discord_variants() {
        let config = profile().engine_config();
        let apps = config["proxies"][0]["appNames"].as_array().unwrap();
        assert!(apps.iter().all(|name| name.as_str().unwrap().starts_with("Discord")));
        assert_eq!(config["proxies"][0]["supportedProtocols"], serde_json::json!(["TCP", "UDP"]));
    }

    #[test]
    fn rejects_missing_port() {
        let mut invalid = profile();
        invalid.endpoint = "proxy.example.com".into();
        assert!(invalid.validate().is_err());
    }
}

