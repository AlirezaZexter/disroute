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
            "logLevel": "Warning",
            "bypassLan": false,
            "proxies": [{
                "appNames": [
                    "Discord.exe",
                    "DiscordCanary.exe",
                    "DiscordPTB.exe",
                    "\\Discord\\Update.exe"
                ],
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
        // ISP DNS may return a private block-page IP. Recover only known
        // Discord TLS names, then let the VLESS server resolve the real host.
        let mut rules = vec![serde_json::json!({
            "network": "tcp", "action": "sniff", "sniffer": ["tls", "http"], "timeout": "1s"
        })];
        for domain in [
            "discord.com",
            "discordapp.com",
            "updates.discord.com",
            "stable.discord.com",
            "canary.discord.com",
            "ptb.discord.com",
            "gateway.discord.gg",
            "cdn.discordapp.com",
            "media.discordapp.net",
            "images-ext-1.discordapp.net",
            "images-ext-2.discordapp.net",
        ] {
            rules.push(serde_json::json!({ "domain": [domain], "action": "route",
                "outbound": "vless-out", "override_address": domain }));
        }
        Ok(serde_json::json!({
            "log": { "level": "info", "timestamp": true },
            "inbounds": [{
                "type": "mixed",
                "tag": "discord-local",
                "listen": "127.0.0.1",
                "listen_port": 2081,
                "set_system_proxy": false
            }],
            "outbounds": [outbound],
            "route": { "final": "vless-out", "auto_detect_interface": true, "rules": rules }
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

    let packet_encoding = query
        .get("packetEncoding")
        .or_else(|| query.get("packet_encoding"))
        .map(String::as_str)
        .unwrap_or("xudp");
    let packet_encoding = match packet_encoding {
        "" | "none" => "",
        "xudp" => "xudp",
        "packetaddr" => "packetaddr",
        other => return Err(format!("Packet encoding پشتیبانی‌نشده: {other}")),
    };

    let mut outbound = serde_json::json!({
        "type": "vless", "tag": "vless-out", "server": host,
        "server_port": port, "uuid": uuid, "packet_encoding": packet_encoding
    });
    if let Some(flow) = query.get("flow").filter(|v| !v.is_empty()) {
        if flow != "xtls-rprx-vision" {
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
        assert_eq!(config["logLevel"], "Warning");
        let apps = config["proxies"][0]["appNames"].as_array().unwrap();
        assert!(apps.iter().all(|name| {
            let name = name.as_str().unwrap();
            name.starts_with("Discord") || name == "\\Discord\\Update.exe"
        }));
        assert!(apps.contains(&serde_json::json!("\\Discord\\Update.exe")));
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
    fn recovers_discord_names_from_poisoned_dns() {
        assert_eq!(profile().proxifyre_config()["bypassLan"], false);
        let config = profile().sing_box_config().unwrap();
        assert_eq!(config["inbounds"][0]["listen_port"], 2081);
        assert_eq!(config["inbounds"][0]["listen"], "127.0.0.1");
        assert_eq!(
            profile().proxifyre_config()["proxies"][0]["socks5ProxyEndpoint"],
            "127.0.0.1:2080"
        );
        let rules = config["route"]["rules"].as_array().unwrap();
        assert_eq!(rules[0]["action"], "sniff");
        for domain in ["discord.com", "updates.discord.com", "gateway.discord.gg"] {
            assert!(rules
                .iter()
                .any(|rule| rule["domain"] == serde_json::json!([domain])
                    && rule["override_address"] == domain
                    && rule["outbound"] == "vless-out"));
        }
    }

    #[test]
    fn parses_reality_settings_without_disabling_certificate_checks() {
        let config = profile().sing_box_config().unwrap();
        let outbound = &config["outbounds"][0];
        assert_eq!(outbound["type"], "vless");
        assert_eq!(outbound["tls"]["reality"]["public_key"], "public-key");
        assert_eq!(outbound["tls"]["insecure"], serde_json::Value::Null);
    }

    #[test]
    fn respects_packet_encoding_from_share_link() {
        let mut packetaddr = profile();
        packetaddr.vless_link = packetaddr
            .vless_link
            .replace("#Discord", "&packetEncoding=packetaddr#Discord");
        assert_eq!(
            packetaddr.sing_box_config().unwrap()["outbounds"][0]["packet_encoding"],
            "packetaddr"
        );

        let mut disabled = profile();
        disabled.vless_link = disabled
            .vless_link
            .replace("#Discord", "&packetEncoding=none#Discord");
        assert_eq!(
            disabled.sing_box_config().unwrap()["outbounds"][0]["packet_encoding"],
            ""
        );
    }

    #[test]
    fn rejects_unknown_packet_encoding() {
        let mut invalid = profile();
        invalid.vless_link = invalid
            .vless_link
            .replace("#Discord", "&packetEncoding=made-up#Discord");
        assert!(invalid.validate().is_err());
    }
}
