use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyProfile {
    pub name: String,
    pub vless_link: String,
}

impl ProxyProfile {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("نام پروفایل نمی‌تواند خالی باشد.".into());
        }
        parse_vless_link(&self.vless_link)?;
        Ok(())
    }

    pub fn proxifyre_config(&self) -> serde_json::Value {
        serde_json::json!({
            "logLevel": "Info",
            "bypassLan": true,
            "proxies": [{
                "appNames": ["Discord.exe", "DiscordCanary.exe", "DiscordPTB.exe"],
                "socks5ProxyEndpoint": "127.0.0.1:2080",
                "username": "",
                "password": "",
                "socks5Transport": "TCP",
                "tlsServerName": "",
                "tlsPinnedSha256": "",
                "tlsAllowInvalidCertificate": false,
                "supportedProtocols": ["TCP", "UDP"],
                "supportedAddressFamilies": ["IPv4", "IPv6"]
            }],
            "excludes": []
        })
    }

    pub fn sing_box_config(&self) -> Result<serde_json::Value, String> {
        let outbound = parse_vless_link(&self.vless_link)?;
        Ok(serde_json::json!({
            "log": { "level": "info", "timestamp": true },
            "inbounds": [{
                "type": "mixed",
                "tag": "discord-local",
                "listen": "127.0.0.1",
                "listen_port": 2080,
                "set_system_proxy": false
            }],
            "outbounds": [outbound],
            "route": { "final": "vless-out", "auto_detect_interface": true }
        }))
    }
}

fn parse_vless_link(value: &str) -> Result<serde_json::Value, String> {
    let url = url::Url::parse(value.trim()).map_err(|_| "لینک VLESS معتبر نیست.")?;
    if url.scheme() != "vless" {
        return Err("لینک باید با vless:// شروع شود.".into());
    }
    let uuid = url.username();
    if !looks_like_uuid(uuid) {
        return Err("UUID لینک VLESS معتبر نیست.".into());
    }
    let host = url.host_str().ok_or("آدرس سرور در لینک وجود ندارد.")?;
    let port = url.port().ok_or("پورت سرور در لینک وجود ندارد.")?;
    let query: std::collections::HashMap<String, String> = url.query_pairs().into_owned().collect();
    if matches!(
        query.get("allowInsecure").map(String::as_str),
        Some("1" | "true")
    ) {
        return Err(
            "برای امنیت، لینک‌هایی که بررسی گواهی TLS را غیرفعال می‌کنند پذیرفته نمی‌شوند.".into(),
        );
    }

    let mut outbound = serde_json::json!({
        "type": "vless", "tag": "vless-out", "server": host,
        "server_port": port, "uuid": uuid, "packet_encoding": "xudp"
    });
    if let Some(flow) = query.get("flow").filter(|v| !v.is_empty()) {
        if flow != &"xtls-rprx-vision" {
            return Err(format!("Flow پشتیبانی‌نشده: {flow}"));
        }
        outbound["flow"] = serde_json::json!(flow);
    }

    let security = query.get("security").map(String::as_str).unwrap_or("none");
    if security == "tls" || security == "reality" {
        let server_name = query
            .get("sni")
            .or_else(|| query.get("serverName"))
            .map(String::as_str)
            .unwrap_or(host);
        let mut tls = serde_json::json!({ "enabled": true, "server_name": server_name });
        if let Some(alpn) = query.get("alpn").filter(|v| !v.is_empty()) {
            tls["alpn"] = serde_json::json!(alpn.split(',').collect::<Vec<_>>());
        }
        if security == "reality" {
            let public_key = query
                .get("pbk")
                .or_else(|| query.get("publicKey"))
                .filter(|v| !v.is_empty())
                .ok_or("کلید عمومی Reality در لینک وجود ندارد.")?;
            tls["reality"] = serde_json::json!({ "enabled": true, "public_key": public_key, "short_id": query.get("sid").or_else(|| query.get("shortId")).map(String::as_str).unwrap_or("") });
            tls["utls"] = serde_json::json!({ "enabled": true, "fingerprint": query.get("fp").map(String::as_str).unwrap_or("chrome") });
        }
        outbound["tls"] = tls;
    } else if security != "none" {
        return Err(format!("Security پشتیبانی‌نشده: {security}"));
    }

    let transport_type = query.get("type").map(String::as_str).unwrap_or("tcp");
    match transport_type {
        "tcp" | "raw" => {}
        "ws" => {
            let mut transport = serde_json::json!({ "type": "ws", "path": query.get("path").map(String::as_str).unwrap_or("/") });
            if let Some(host_header) = query.get("host").filter(|v| !v.is_empty()) {
                transport["headers"] = serde_json::json!({ "Host": host_header });
            }
            outbound["transport"] = transport;
        }
        "grpc" => {
            outbound["transport"] = serde_json::json!({ "type": "grpc", "service_name": query.get("serviceName").map(String::as_str).unwrap_or("") })
        }
        "http" => {
            outbound["transport"] = serde_json::json!({ "type": "http", "host": query.get("host").filter(|v| !v.is_empty()).map(|v| vec![v.as_str()]).unwrap_or_default(), "path": query.get("path").map(String::as_str).unwrap_or("/") })
        }
        other => return Err(format!("نوع انتقال VLESS پشتیبانی‌نشده: {other}")),
    }
    Ok(outbound)
}

fn looks_like_uuid(value: &str) -> bool {
    value.len() == 36
        && value.chars().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == '-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
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
        ProxyProfile { name: "Discord".into(), vless_link: "vless://00000000-0000-4000-8000-000000000000@example.com:443?security=reality&type=tcp&flow=xtls-rprx-vision&sni=cdn.example.com&pbk=public-key&sid=abcd&fp=chrome#Discord".into() }
    }

    #[test]
    fn generated_config_routes_only_discord_variants() {
        let config = profile().proxifyre_config();
        let apps = config["proxies"][0]["appNames"].as_array().unwrap();
        assert!(apps
            .iter()
            .all(|name| name.as_str().unwrap().starts_with("Discord")));
        assert_eq!(
            config["proxies"][0]["supportedProtocols"],
            serde_json::json!(["TCP", "UDP"])
        );
    }

    #[test]
    fn rejects_non_vless_links() {
        let mut invalid = profile();
        invalid.vless_link = "https://example.com".into();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn parses_reality_settings_without_disabling_certificate_checks() {
        let config = profile().sing_box_config().unwrap();
        let outbound = &config["outbounds"][0];
        assert_eq!(outbound["type"], "vless");
        assert_eq!(outbound["tls"]["reality"]["public_key"], "public-key");
        assert_eq!(outbound["tls"]["insecure"], serde_json::Value::Null);
    }
}
