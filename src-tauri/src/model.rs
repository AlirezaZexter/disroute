use base64::{engine::general_purpose, Engine as _};
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const OUTBOUND_TAG: &str = "proxy-out";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyProfile {
    pub name: String,
    #[serde(alias = "vlessLink")]
    pub config_link: String,
}

impl ProxyProfile {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("نام پروفایل نمی‌تواند خالی باشد.".into());
        }
        parse_proxy_link(&self.config_link)?;
        Ok(())
    }

    pub fn proxifyre_config(&self) -> serde_json::Value {
        serde_json::json!({
            "logLevel": "Warning",
            "bypassLan": false,
            "proxies": [{
                "appNames": ["Discord.exe", "DiscordCanary.exe", "DiscordPTB.exe", "\\Discord\\Update.exe"],
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
        let outbound = parse_proxy_link(&self.config_link)?;
        // ISP DNS may return a private block-page IP. Recover only known
        // Discord TLS names, then let the remote proxy resolve the real host.
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
                "outbound": OUTBOUND_TAG, "override_address": domain }));
        }
        Ok(serde_json::json!({
            "log": { "level": "info", "timestamp": true },
            "inbounds": [{
                "type": "mixed", "tag": "discord-local", "listen": "127.0.0.1",
                "listen_port": 2081, "set_system_proxy": false
            }],
            "outbounds": [outbound],
            "route": { "final": OUTBOUND_TAG, "auto_detect_interface": true, "rules": rules }
        }))
    }
}

fn parse_proxy_link(value: &str) -> Result<serde_json::Value, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("کانفیگ اتصال خالی است.".into());
    }
    let scheme = value
        .split_once("://")
        .map(|(scheme, _)| scheme.to_ascii_lowercase())
        .ok_or("فرمت کانفیگ شناخته نشد.")?;
    match scheme.as_str() {
        "vless" => parse_vless_link(value),
        "vmess" => parse_vmess_link(value),
        "trojan" => parse_trojan_link(value),
        "ss" => parse_shadowsocks_link(value),
        _ => Err(
            "پروتکل پشتیبانی نمی‌شود. از VLESS، VMess، Trojan یا Shadowsocks استفاده کنید.".into(),
        ),
    }
}

fn parse_vless_link(value: &str) -> Result<serde_json::Value, String> {
    let url = parse_url(value, "VLESS")?;
    let uuid = decoded(url.username(), "شناسه VLESS")?;
    if !looks_like_uuid(&uuid) {
        return Err("UUID لینک VLESS معتبر نیست.".into());
    }
    let host = required_host(&url)?;
    let port = required_port(&url)?;
    let query = query_map(&url);
    reject_insecure(&query)?;
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
        "type": "vless", "tag": OUTBOUND_TAG, "server": host,
        "server_port": port, "uuid": uuid, "packet_encoding": packet_encoding
    });
    if let Some(flow) = query.get("flow").filter(|v| !v.is_empty()) {
        if flow != "xtls-rprx-vision" {
            return Err(format!("Flow پشتیبانی‌نشده: {flow}"));
        }
        outbound["flow"] = serde_json::json!(flow);
    }
    apply_tls_from_query(&mut outbound, &query, &host, false)?;
    apply_url_transport(&mut outbound, &query)?;
    Ok(outbound)
}

fn parse_trojan_link(value: &str) -> Result<serde_json::Value, String> {
    let url = parse_url(value, "Trojan")?;
    let password = decoded(url.username(), "رمز Trojan")?;
    if password.is_empty() {
        return Err("رمز Trojan در لینک وجود ندارد.".into());
    }
    let host = required_host(&url)?;
    let port = required_port(&url)?;
    let query = query_map(&url);
    reject_insecure(&query)?;
    let mut outbound = serde_json::json!({
        "type": "trojan", "tag": OUTBOUND_TAG, "server": host,
        "server_port": port, "password": password
    });
    apply_tls_from_query(&mut outbound, &query, &host, true)?;
    apply_url_transport(&mut outbound, &query)?;
    Ok(outbound)
}

fn parse_vmess_link(value: &str) -> Result<serde_json::Value, String> {
    let encoded = value
        .split_once("://")
        .map(|(_, body)| body)
        .ok_or("لینک VMess معتبر نیست.")?;
    let encoded = encoded.split('#').next().unwrap_or(encoded).trim();
    let decoded = decode_base64(encoded).map_err(|_| "محتوای Base64 لینک VMess معتبر نیست.")?;
    let json: serde_json::Value =
        serde_json::from_slice(&decoded).map_err(|_| "ساختار JSON لینک VMess معتبر نیست.")?;
    let server = json_text(&json, "add").ok_or("آدرس سرور در لینک VMess وجود ندارد.")?;
    let port = json_u16(&json, "port").ok_or("پورت VMess معتبر نیست.")?;
    let uuid = json_text(&json, "id").ok_or("UUID در لینک VMess وجود ندارد.")?;
    if !looks_like_uuid(&uuid) {
        return Err("UUID لینک VMess معتبر نیست.".into());
    }
    if json_bool(&json, "allowInsecure") {
        return Err(
            "برای امنیت، کانفیگ‌هایی که بررسی گواهی TLS را غیرفعال می‌کنند پذیرفته نمی‌شوند.".into(),
        );
    }
    let security = json_text(&json, "scy")
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "auto".into());
    let alter_id = json_u64(&json, "aid").unwrap_or(0);
    let mut outbound = serde_json::json!({
        "type": "vmess", "tag": OUTBOUND_TAG, "server": server,
        "server_port": port, "uuid": uuid, "security": security,
        "alter_id": alter_id, "packet_encoding": "xudp"
    });
    let network = json_text(&json, "net").unwrap_or_else(|| "tcp".into());
    let path = json_text(&json, "path").unwrap_or_default();
    let host = json_text(&json, "host").unwrap_or_default();
    apply_transport(&mut outbound, &network, &path, &host)?;
    let tls_mode = json_text(&json, "tls").unwrap_or_default();
    if !tls_mode.is_empty() && tls_mode != "none" {
        if tls_mode != "tls" {
            return Err(format!("Security پشتیبانی‌نشده در VMess: {tls_mode}"));
        }
        let server_name = json_text(&json, "sni")
            .filter(|value| !value.is_empty())
            .or_else(|| {
                host.split(',')
                    .next()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| server.clone());
        let mut tls = serde_json::json!({ "enabled": true, "server_name": server_name });
        if let Some(alpn) = json_text(&json, "alpn").filter(|v| !v.is_empty()) {
            tls["alpn"] = serde_json::json!(alpn.split(',').map(str::trim).collect::<Vec<_>>());
        }
        if let Some(fp) = json_text(&json, "fp").filter(|v| !v.is_empty()) {
            tls["utls"] = serde_json::json!({ "enabled": true, "fingerprint": fp });
        }
        outbound["tls"] = tls;
    }
    Ok(outbound)
}

fn parse_shadowsocks_link(value: &str) -> Result<serde_json::Value, String> {
    let body = value
        .split_once("://")
        .map(|(_, body)| body)
        .ok_or("لینک Shadowsocks معتبر نیست.")?;
    let without_fragment = body.split('#').next().unwrap_or(body);
    let (endpoint, query) = without_fragment
        .split_once('?')
        .unwrap_or((without_fragment, ""));
    if url::form_urlencoded::parse(query.as_bytes())
        .any(|(key, value)| key == "plugin" && !value.is_empty())
    {
        return Err("افزونه‌های Shadowsocks در این نسخه پشتیبانی نمی‌شوند.".into());
    }
    let normalized = if endpoint.contains('@') {
        endpoint.to_owned()
    } else {
        String::from_utf8(
            decode_base64(endpoint).map_err(|_| "محتوای Base64 لینک Shadowsocks معتبر نیست.")?,
        )
        .map_err(|_| "متن لینک Shadowsocks معتبر نیست.")?
    };
    let parsed = url::Url::parse(&format!("ss://{normalized}"))
        .map_err(|_| "ساختار لینک Shadowsocks معتبر نیست.")?;
    let host = required_host(&parsed)?;
    let port = required_port(&parsed)?;
    let (method, password) = if let Some(password) = parsed.password() {
        (
            decoded(parsed.username(), "روش رمزنگاری Shadowsocks")?,
            decoded(password, "رمز Shadowsocks")?,
        )
    } else {
        let encoded_credentials = decoded(parsed.username(), "مشخصات Shadowsocks")?;
        let credentials = String::from_utf8(
            decode_base64(&encoded_credentials)
                .map_err(|_| "مشخصات Base64 لینک Shadowsocks معتبر نیست.")?,
        )
        .map_err(|_| "مشخصات Shadowsocks معتبر نیست.")?;
        let (method, password) = credentials
            .split_once(':')
            .ok_or("روش رمزنگاری یا رمز Shadowsocks وجود ندارد.")?;
        (method.to_owned(), password.to_owned())
    };
    if method.is_empty() || password.is_empty() {
        return Err("روش رمزنگاری یا رمز Shadowsocks وجود ندارد.".into());
    }
    Ok(serde_json::json!({
        "type": "shadowsocks", "tag": OUTBOUND_TAG, "server": host,
        "server_port": port, "method": method, "password": password
    }))
}

fn apply_tls_from_query(
    outbound: &mut serde_json::Value,
    query: &HashMap<String, String>,
    host: &str,
    default_enabled: bool,
) -> Result<(), String> {
    let security = query
        .get("security")
        .map(String::as_str)
        .unwrap_or(if default_enabled { "tls" } else { "none" });
    if security == "none" {
        return Ok(());
    }
    if security != "tls" && security != "reality" {
        return Err(format!("Security پشتیبانی‌نشده: {security}"));
    }
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
        tls["reality"] = serde_json::json!({
            "enabled": true, "public_key": public_key,
            "short_id": query.get("sid").or_else(|| query.get("shortId")).map(String::as_str).unwrap_or("")
        });
        tls["utls"] = serde_json::json!({
            "enabled": true, "fingerprint": query.get("fp").map(String::as_str).unwrap_or("chrome")
        });
    } else if let Some(fp) = query.get("fp").filter(|v| !v.is_empty()) {
        tls["utls"] = serde_json::json!({ "enabled": true, "fingerprint": fp });
    }
    outbound["tls"] = tls;
    Ok(())
}

fn apply_url_transport(
    outbound: &mut serde_json::Value,
    query: &HashMap<String, String>,
) -> Result<(), String> {
    if let Some(header) = query
        .get("headerType")
        .filter(|value| !value.is_empty() && *value != "none")
    {
        return Err(format!("Header type پشتیبانی‌نشده: {header}"));
    }
    let network = query.get("type").map(String::as_str).unwrap_or("tcp");
    let path = query.get("path").map(String::as_str).unwrap_or("");
    let host = query.get("host").map(String::as_str).unwrap_or("");
    if network == "grpc" {
        return apply_transport(
            outbound,
            network,
            query.get("serviceName").map(String::as_str).unwrap_or(path),
            host,
        );
    }
    apply_transport(outbound, network, path, host)
}

fn apply_transport(
    outbound: &mut serde_json::Value,
    network: &str,
    path: &str,
    host: &str,
) -> Result<(), String> {
    match network {
        "" | "tcp" | "raw" => Ok(()),
        "ws" => {
            let mut transport = serde_json::json!({ "type": "ws", "path": if path.is_empty() { "/" } else { path } });
            if !host.is_empty() {
                transport["headers"] = serde_json::json!({ "Host": host });
            }
            outbound["transport"] = transport;
            Ok(())
        }
        "grpc" => {
            outbound["transport"] = serde_json::json!({ "type": "grpc", "service_name": path });
            Ok(())
        }
        "http" | "h2" => {
            let hosts: Vec<&str> = host
                .split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .collect();
            outbound["transport"] = serde_json::json!({
                "type": "http", "host": hosts, "path": if path.is_empty() { "/" } else { path }
            });
            Ok(())
        }
        "httpupgrade" => {
            outbound["transport"] = serde_json::json!({
                "type": "httpupgrade", "host": host, "path": if path.is_empty() { "/" } else { path }
            });
            Ok(())
        }
        other => Err(format!("نوع انتقال پشتیبانی‌نشده: {other}")),
    }
}

fn parse_url(value: &str, protocol: &str) -> Result<url::Url, String> {
    url::Url::parse(value.trim()).map_err(|_| format!("لینک {protocol} معتبر نیست."))
}

fn required_host(url: &url::Url) -> Result<String, String> {
    url.host_str()
        .filter(|host| !host.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "آدرس سرور در لینک وجود ندارد.".into())
}

fn required_port(url: &url::Url) -> Result<u16, String> {
    url.port()
        .ok_or_else(|| "پورت سرور در لینک وجود ندارد.".into())
}

fn query_map(url: &url::Url) -> HashMap<String, String> {
    url.query_pairs().into_owned().collect()
}

fn reject_insecure(query: &HashMap<String, String>) -> Result<(), String> {
    if matches!(
        query.get("allowInsecure").map(String::as_str),
        Some("1" | "true")
    ) {
        return Err(
            "برای امنیت، کانفیگ‌هایی که بررسی گواهی TLS را غیرفعال می‌کنند پذیرفته نمی‌شوند.".into(),
        );
    }
    Ok(())
}

fn decoded(value: &str, label: &str) -> Result<String, String> {
    percent_decode_str(value)
        .decode_utf8()
        .map(|value| value.into_owned())
        .map_err(|_| format!("{label} معتبر نیست."))
}

fn decode_base64(value: &str) -> Result<Vec<u8>, base64::DecodeError> {
    let value = value.trim();
    for engine in [
        &general_purpose::STANDARD,
        &general_purpose::STANDARD_NO_PAD,
        &general_purpose::URL_SAFE,
        &general_purpose::URL_SAFE_NO_PAD,
    ] {
        if let Ok(bytes) = engine.decode(value) {
            return Ok(bytes);
        }
    }
    general_purpose::STANDARD.decode(value)
}

fn json_text(value: &serde_json::Value, key: &str) -> Option<String> {
    match value.get(key)? {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn json_u16(value: &serde_json::Value, key: &str) -> Option<u16> {
    json_text(value, key)?.parse().ok()
}
fn json_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    json_text(value, key)?.parse().ok()
}
fn json_bool(value: &serde_json::Value, key: &str) -> bool {
    matches!(json_text(value, key).as_deref(), Some("1" | "true"))
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
        ProxyProfile { name: "Discord".into(), config_link: "vless://00000000-0000-4000-8000-000000000000@example.com:443?security=reality&type=tcp&flow=xtls-rprx-vision&sni=cdn.example.com&pbk=public-key&sid=abcd&fp=chrome#Discord".into() }
    }

    fn vmess_link() -> String {
        let json = serde_json::json!({
            "v": "2", "ps": "Discord", "add": "example.com", "port": "443",
            "id": "00000000-0000-4000-8000-000000000000", "aid": "0", "scy": "auto",
            "net": "ws", "host": "cdn.example.com", "path": "/socket", "tls": "tls",
            "sni": "cdn.example.com", "fp": "chrome"
        });
        format!(
            "vmess://{}",
            general_purpose::STANDARD.encode(json.to_string())
        )
    }

    #[test]
    fn generated_config_routes_only_discord_variants() {
        let config = profile().proxifyre_config();
        let apps = config["proxies"][0]["appNames"].as_array().unwrap();
        assert!(apps.iter().all(|name| {
            let name = name.as_str().unwrap();
            name.starts_with("Discord") || name == "\\Discord\\Update.exe"
        }));
        assert_eq!(
            config["proxies"][0]["supportedProtocols"],
            serde_json::json!(["TCP", "UDP"])
        );
    }

    #[test]
    fn accepts_vless_vmess_trojan_and_shadowsocks_links() {
        let links = [
            profile().config_link,
            vmess_link(),
            "trojan://secret@example.com:443?security=tls&sni=cdn.example.com&type=grpc&serviceName=discord".into(),
            "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ=@example.com:8388#Discord".into(),
        ];
        let expected = ["vless", "vmess", "trojan", "shadowsocks"];
        for (link, kind) in links.into_iter().zip(expected) {
            let mut candidate = profile();
            candidate.config_link = link;
            candidate.validate().unwrap();
            assert_eq!(
                candidate.sing_box_config().unwrap()["outbounds"][0]["type"],
                kind
            );
            assert_eq!(
                candidate.sing_box_config().unwrap()["route"]["final"],
                OUTBOUND_TAG
            );
        }
    }

    #[test]
    fn parses_vmess_transport_and_tls() {
        let outbound = parse_proxy_link(&vmess_link()).unwrap();
        assert_eq!(outbound["transport"]["type"], "ws");
        assert_eq!(outbound["transport"]["headers"]["Host"], "cdn.example.com");
        assert_eq!(outbound["tls"]["server_name"], "cdn.example.com");
        assert_eq!(outbound["packet_encoding"], "xudp");
    }

    #[test]
    fn migrates_legacy_saved_profile_field() {
        let legacy = r#"{"name":"Discord","vlessLink":"vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls"}"#;
        let restored: ProxyProfile = serde_json::from_str(legacy).unwrap();
        assert!(restored.config_link.starts_with("vless://"));
        assert!(serde_json::to_string(&restored)
            .unwrap()
            .contains("configLink"));
    }

    #[test]
    fn rejects_unknown_or_insecure_links() {
        for link in [
            "https://example.com",
            "trojan://secret@example.com:443?security=tls&allowInsecure=1",
        ] {
            let mut invalid = profile();
            invalid.config_link = link.into();
            assert!(invalid.validate().is_err());
        }
    }

    #[test]
    fn recovers_discord_names_from_poisoned_dns() {
        let config = profile().sing_box_config().unwrap();
        assert_eq!(config["inbounds"][0]["listen_port"], 2081);
        let rules = config["route"]["rules"].as_array().unwrap();
        for domain in ["discord.com", "updates.discord.com", "gateway.discord.gg"] {
            assert!(rules
                .iter()
                .any(|rule| rule["domain"] == serde_json::json!([domain])
                    && rule["override_address"] == domain
                    && rule["outbound"] == OUTBOUND_TAG));
        }
    }

    #[test]
    fn respects_vless_packet_encoding() {
        let mut packetaddr = profile();
        packetaddr.config_link = packetaddr
            .config_link
            .replace("#Discord", "&packetEncoding=packetaddr#Discord");
        assert_eq!(
            packetaddr.sing_box_config().unwrap()["outbounds"][0]["packet_encoding"],
            "packetaddr"
        );
        let mut disabled = profile();
        disabled.config_link = disabled
            .config_link
            .replace("#Discord", "&packetEncoding=none#Discord");
        assert_eq!(
            disabled.sing_box_config().unwrap()["outbounds"][0]["packet_encoding"],
            ""
        );
    }
}
