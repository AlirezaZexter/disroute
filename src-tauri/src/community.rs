use crate::model::ProxyProfile;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::RandomState, HashMap, HashSet};
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MANIFEST_VERSION: u32 = 1;
pub const DEFAULT_MAX_MANIFEST_BYTES: usize = 1_048_576;
pub const DEFAULT_MAX_CONFIGS: usize = 500;
static STORE_WRITE: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind {
    GithubRaw,
    GithubRelease,
    Subscription,
    LocalFile,
    Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CommunitySource {
    pub id: String,
    pub name: String,
    pub location: String,
    pub attribution: String,
    pub kind: SourceKind,
    pub enabled: bool,
    #[serde(default = "default_refresh_minutes")]
    pub refresh_interval_minutes: u32,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u32,
    #[serde(default)]
    pub expected_sha256: Option<String>,
    #[serde(default)]
    pub redistribution_authorized: bool,
    #[serde(default)]
    pub last_successful_refresh: Option<u64>,
    #[serde(default)]
    pub last_error: Option<String>,
}

fn default_refresh_minutes() -> u32 {
    60
}
fn default_timeout_seconds() -> u32 {
    12
}

impl CommunitySource {
    pub fn validate(&self) -> Result<(), String> {
        if !self.redistribution_authorized {
            return Err("برای این منبع، اجازهٔ بازنشر تأیید نشده است.".into());
        }
        if self.id.is_empty()
            || self.id.len() > 80
            || !self
                .id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        {
            return Err("شناسهٔ منبع معتبر نیست.".into());
        }
        if self.name.trim().is_empty() || self.name.chars().count() > 120 {
            return Err("نام منبع معتبر نیست.".into());
        }
        if self.attribution.trim().is_empty() || self.attribution.chars().count() > 240 {
            return Err("نام یا نشانی ارائه‌دهنده باید مشخص باشد.".into());
        }
        if self.location.len() > 2048 {
            return Err("نشانی منبع بیش از حد طولانی است.".into());
        }
        match self.kind {
            SourceKind::LocalFile => {
                if self.location.trim().is_empty() {
                    return Err("مسیر فایل محلی خالی است.".into());
                }
            }
            _ => {
                let url = url::Url::parse(&self.location).map_err(|_| "نشانی منبع معتبر نیست.")?;
                if url.scheme() != "https" {
                    return Err("منابع راه‌دور باید از HTTPS استفاده کنند.".into());
                }
                if !url.username().is_empty() || url.password().is_some() {
                    return Err("قرار دادن رمز در نشانی منبع مجاز نیست.".into());
                }
            }
        }
        if !(5..=120).contains(&self.timeout_seconds) {
            return Err("مهلت منبع باید بین ۵ تا ۱۲۰ ثانیه باشد.".into());
        }
        if !(5..=10080).contains(&self.refresh_interval_minutes) {
            return Err("فاصلهٔ نوسازی باید بین ۵ دقیقه تا ۷ روز باشد.".into());
        }
        if let Some(checksum) = &self.expected_sha256 {
            if checksum.len() != 64 || !checksum.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err("مقدار SHA-256 معتبر نیست.".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub id: String,
    #[serde(default)]
    pub generated_at: Option<u64>,
    pub source: ManifestSource,
    pub configurations: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ManifestSource {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ManifestEntry {
    pub id: String,
    pub uri: String,
    pub protocol: String,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub supports_udp: Option<bool>,
    #[serde(default)]
    pub added_at: Option<u64>,
    #[serde(default)]
    pub expires_at: Option<u64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub id: String,
    pub source_id: String,
    pub source_name: String,
    pub attribution: String,
    pub uri: String,
    pub protocol: String,
    pub country: Option<String>,
    pub supports_udp: Option<bool>,
    pub added_at: Option<u64>,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SourceResult {
    pub source_id: String,
    pub source_name: String,
    pub attribution: String,
    pub candidates: Vec<Candidate>,
    pub last_successful_refresh: Option<u64>,
    pub stale: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CommunitySnapshot {
    pub sources: Vec<CommunitySource>,
    pub candidates: Vec<Candidate>,
    pub stale: bool,
    pub refreshed_at: Option<u64>,
    pub acknowledged_warning: bool,
    pub automatic_failover: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CommunityState {
    #[serde(default)]
    defaults_initialized: bool,
    #[serde(default)]
    source_catalog_version: u32,
    sources: Vec<CommunitySource>,
    cache: HashMap<String, SourceResult>,
    acknowledged_warning: bool,
    automatic_failover: bool,
    #[serde(default)]
    history: HashMap<String, CandidateHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CandidateHistory {
    pub successes: u32,
    pub disconnects: u32,
    pub consecutive_failures: u32,
    pub next_retry_at: Option<u64>,
    #[serde(default)]
    pub last_latency_ms: Option<u64>,
    #[serde(default)]
    pub tested_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResult {
    pub candidate_id: String,
    pub source_id: String,
    pub source_name: String,
    pub attribution: String,
    pub protocol: String,
    pub country: Option<String>,
    pub working: bool,
    pub median_latency_ms: Option<u64>,
    pub jitter_ms: Option<u64>,
    pub failure_rate: f64,
    pub udp_available: Option<bool>,
    pub label: String,
    pub score: i64,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_at: Option<u64>,
}

pub struct CommunityStore {
    dir: PathBuf,
}

impl CommunityStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn snapshot(&self) -> Result<CommunitySnapshot, String> {
        let state = self.load()?;
        let mut candidates = Vec::new();
        let mut stale = false;
        let mut refreshed_at = None;
        let now = now_secs();
        for source in state.sources.iter().filter(|source| source.enabled) {
            if let Some(cache) = state.cache.get(&source.id) {
                candidates.extend(
                    cache
                        .candidates
                        .iter()
                        .filter(|candidate| candidate.expires_at.is_none_or(|expiry| expiry > now))
                        .cloned(),
                );
                stale |= cache.stale || source_due(source, now);
                refreshed_at = refreshed_at.max(cache.last_successful_refresh);
            }
        }
        // Interleave sources before the global cap: one large subscription
        // must not consume all 500 slots and hide the smaller regional feeds.
        let mut candidates = deduplicate(interleave_sources(candidates));
        candidates.truncate(DEFAULT_MAX_CONFIGS);
        Ok(CommunitySnapshot {
            sources: state.sources,
            candidates,
            stale,
            refreshed_at,
            acknowledged_warning: state.acknowledged_warning,
            automatic_failover: state.automatic_failover,
        })
    }

    pub fn save_sources(&self, sources: Vec<CommunitySource>) -> Result<CommunitySnapshot, String> {
        let _guard = STORE_WRITE.lock().map_err(|_| "Community storage busy")?;
        if sources.len() > 50 {
            return Err("حداکثر ۵۰ منبع قابل ثبت است.".into());
        }
        let mut ids = HashSet::new();
        for source in &sources {
            source.validate()?;
            if !ids.insert(source.id.clone()) {
                return Err("شناسهٔ منبع تکراری است.".into());
            }
        }
        let mut state = self.load()?;
        state.sources = sources;
        self.save(&state)?;
        self.snapshot()
    }

    pub fn set_preferences(
        &self,
        acknowledged_warning: bool,
        automatic_failover: bool,
    ) -> Result<CommunitySnapshot, String> {
        let _guard = STORE_WRITE.lock().map_err(|_| "Community storage busy")?;
        let mut state = self.load()?;
        state.acknowledged_warning = acknowledged_warning;
        state.automatic_failover = automatic_failover;
        self.save(&state)?;
        self.snapshot()
    }

    pub fn refresh(&self) -> Result<CommunitySnapshot, String> {
        self.refresh_selected(false)
    }

    fn refresh_selected(&self, only_due: bool) -> Result<CommunitySnapshot, String> {
        let _guard = STORE_WRITE.lock().map_err(|_| "Community storage busy")?;
        let mut state = self.load()?;
        if !state.acknowledged_warning {
            return Err("ابتدا شرایط اتصال Community را بخوانید و تأیید کنید.".into());
        }
        let now = now_secs();
        // Four bounded workers, rather than a timeout for every source in series.
        // Keep storage serialized so clear/disable cannot be undone by an old fetch.
        let sources: Vec<_> = state
            .sources
            .iter()
            .filter(|source| {
                source.enabled
                    && (!only_due
                        || source_due(source, now)
                        || !state.cache.contains_key(&source.id))
            })
            .cloned()
            .collect();
        let fetched = fetch_sources(&sources);
        for (id, fetched) in fetched {
            let Some(source) = state.sources.iter_mut().find(|source| source.id == id) else {
                continue;
            };
            match fetched {
                Ok(mut result) => {
                    result.last_successful_refresh = Some(now);
                    source.last_successful_refresh = Some(now);
                    source.last_error = None;
                    state.cache.insert(source.id.clone(), result);
                }
                Err(error) => {
                    let error = redact_secrets(&error);
                    source.last_error = Some(error.clone());
                    if let Some(cached) = state.cache.get_mut(&source.id) {
                        cached.stale = true;
                        cached.error = Some(error);
                    }
                }
            }
        }
        self.save(&state)?;
        self.snapshot()
    }

    pub fn refresh_due(&self) -> Result<(), String> {
        let state = self.load()?;
        let now = now_secs();
        let due = state.acknowledged_warning
            && state.sources.iter().any(|source| source_due(source, now));
        if due {
            self.refresh_selected(true).map(|_| ())
        } else {
            Ok(())
        }
    }

    pub fn clear(&self) -> Result<CommunitySnapshot, String> {
        let _guard = STORE_WRITE.lock().map_err(|_| "Community storage busy")?;
        let mut state = self.load()?;
        state.cache.clear();
        state.history.clear();
        for source in &mut state.sources {
            source.last_successful_refresh = None;
        }
        self.save(&state)?;
        self.snapshot()
    }

    pub fn candidate(&self, id: &str) -> Result<Candidate, String> {
        self.snapshot()?
            .candidates
            .into_iter()
            .find(|candidate| candidate.id == id)
            .ok_or_else(|| "کانفیگ انتخاب‌شده در کش معتبر وجود ندارد.".into())
    }

    pub fn history(&self) -> Result<HashMap<String, CandidateHistory>, String> {
        Ok(self.load()?.history)
    }

    pub fn record_scan(&self, results: &[HealthResult]) -> Result<(), String> {
        let _guard = STORE_WRITE.lock().map_err(|_| "Community storage busy")?;
        let mut state = self.load()?;
        let now = now_secs();
        for result in results {
            let h = state
                .history
                .entry(result.candidate_id.clone())
                .or_default();
            h.tested_at = Some(now);
            h.last_latency_ms = if result.working {
                result.median_latency_ms
            } else {
                None
            };
            if result.working {
                h.consecutive_failures = 0;
                h.next_retry_at = None;
            } else {
                // A failed probe is not a dropped user connection.
                let disconnects = h.disconnects;
                record_failure(h, now);
                h.disconnects = disconnects;
            }
        }
        state.history.retain(|_, h| {
            h.tested_at
                .is_none_or(|t| now.saturating_sub(t) < 7 * 86400)
        });
        self.save(&state)
    }

    pub fn record_result(&self, id: &str, success: bool) -> Result<(), String> {
        let _guard = STORE_WRITE.lock().map_err(|_| "Community storage busy")?;
        let mut state = self.load()?;
        let history = state.history.entry(id.to_owned()).or_default();
        if success {
            history.successes = history.successes.saturating_add(1);
            history.consecutive_failures = 0;
            history.next_retry_at = None;
        } else {
            record_failure(history, now_secs());
        }
        self.save(&state)
    }

    fn path(&self) -> PathBuf {
        self.dir.join("community.dpapi")
    }
    fn load(&self) -> Result<CommunityState, String> {
        let encrypted = match std::fs::read(self.path()) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let mut state = CommunityState::default();
                initialize_sources(&mut state)?;
                return Ok(state);
            }
            Err(_) => return Err("خواندن تنظیمات Community ناموفق بود.".into()),
        };
        let mut plain = crate::profile_store::protect(&encrypted, true)?;
        let parsed = serde_json::from_slice::<CommunityState>(&plain)
            .map_err(|_| "دادهٔ ذخیره‌شدهٔ Community معتبر نیست.".to_string());
        plain.fill(0);
        let mut state = parsed?;
        initialize_sources(&mut state)?;
        Ok(state)
    }
    fn save(&self, state: &CommunityState) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir).map_err(|_| "ساخت پوشهٔ Community ناموفق بود.")?;
        let mut plain = serde_json::to_vec(state).map_err(|_| "ساخت دادهٔ Community ناموفق بود.")?;
        let encrypted = crate::profile_store::protect(&plain, false);
        plain.fill(0);
        let encrypted = encrypted?;
        let pending = self.dir.join("community.dpapi.pending");
        std::fs::write(&pending, encrypted).map_err(|_| "ذخیرهٔ Community ناموفق بود.")?;
        std::fs::rename(pending, self.path())
            .map_err(|_| "جایگزینی تنظیمات Community ناموفق بود.".to_string())
    }
}

fn initialize_sources(state: &mut CommunityState) -> Result<(), String> {
    if !state.defaults_initialized || state.source_catalog_version < 2 {
        let defaults: Vec<CommunitySource> =
            serde_json::from_str(include_str!("../../docs/community-sources.json"))
                .map_err(|_| "فهرست منابع پیش‌فرض معتبر نیست.")?;
        for source in defaults {
            source.validate()?;
            // Existing users keep removed/disabled original feeds. Only the
            // newly introduced regional/stability feeds migrate once.
            if state.defaults_initialized
                && !matches!(
                    source.id.as_str(),
                    "au1rxx-tr" | "au1rxx-fr" | "au1rxx-ae" | "au1rxx-stable"
                )
            {
                continue;
            }
            if !state.sources.iter().any(|item| item.id == source.id) {
                state.sources.push(source);
            }
        }
        state.defaults_initialized = true;
        state.source_catalog_version = 2;
    }
    Ok(())
}

fn source_due(source: &CommunitySource, now: u64) -> bool {
    source.enabled
        && source
            .last_successful_refresh
            .map(|last| now.saturating_sub(last) >= source.refresh_interval_minutes as u64 * 60)
            .unwrap_or(true)
}

pub(crate) fn interleave_sources(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
    let mut offsets = HashMap::<String, usize>::new();
    candidates.sort_by_cached_key(|c| {
        let offset = offsets.entry(c.source_id.clone()).or_default();
        let key = *offset;
        *offset += 1;
        key
    });
    candidates
}

fn fetch_sources(sources: &[CommunitySource]) -> Vec<(String, Result<SourceResult, String>)> {
    let mut results = Vec::new();
    for batch in sources.chunks(4) {
        std::thread::scope(|scope| {
            let workers: Vec<_> = batch
                .iter()
                .map(|source| {
                    (
                        source.id.clone(),
                        scope.spawn(move || {
                            fetch_source(source, DEFAULT_MAX_MANIFEST_BYTES, DEFAULT_MAX_CONFIGS)
                        }),
                    )
                })
                .collect();
            for (id, worker) in workers {
                results.push((
                    id,
                    worker
                        .join()
                        .unwrap_or_else(|_| Err("Source worker stopped".into())),
                ));
            }
        });
    }
    results
}

fn fetch_source(
    source: &CommunitySource,
    max_bytes: usize,
    max_configs: usize,
) -> Result<SourceResult, String> {
    source.validate()?;
    let bytes = match source.kind {
        SourceKind::LocalFile => read_limited(Path::new(&source.location), max_bytes)?,
        _ => fetch_https(&source.location, source.timeout_seconds, max_bytes)?,
    };
    verify_checksum(&bytes, source.expected_sha256.as_deref())?;
    let candidates = match source.kind {
        SourceKind::Subscription => parse_subscription(&bytes, source, max_configs)?,
        _ => parse_manifest(&bytes, source, max_configs)?,
    };
    Ok(SourceResult {
        source_id: source.id.clone(),
        source_name: source.name.clone(),
        attribution: source.attribution.clone(),
        candidates,
        last_successful_refresh: None,
        stale: false,
        error: None,
    })
}

fn read_limited(path: &Path, max_bytes: usize) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path).map_err(|_| "فایل منبع خوانده نشد.")?;
    if metadata.len() > max_bytes as u64 {
        return Err("حجم منبع از سقف مجاز بیشتر است.".into());
    }
    std::fs::read(path).map_err(|_| "فایل منبع خوانده نشد.".into())
}

fn verify_checksum(bytes: &[u8], expected: Option<&str>) -> Result<(), String> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let actual = sha256_hex(bytes);
    if !actual.eq_ignore_ascii_case(expected) {
        return Err("بررسی SHA-256 منبع ناموفق بود.".into());
    }
    Ok(())
}

pub fn parse_manifest(
    bytes: &[u8],
    source: &CommunitySource,
    max_configs: usize,
) -> Result<Vec<Candidate>, String> {
    if bytes.len() > DEFAULT_MAX_MANIFEST_BYTES {
        return Err("حجم manifest از سقف مجاز بیشتر است.".into());
    }
    let manifest: Manifest =
        serde_json::from_slice(bytes).map_err(|_| "ساختار JSON manifest معتبر نیست.")?;
    if manifest.version != MANIFEST_VERSION {
        return Err(format!(
            "نسخهٔ manifest پشتیبانی نمی‌شود: {}",
            manifest.version
        ));
    }
    if manifest.configurations.len() > max_configs {
        return Err("تعداد کانفیگ‌ها از سقف مجاز بیشتر است.".into());
    }
    let now = now_secs();
    manifest
        .configurations
        .into_iter()
        .map(|entry| validate_entry(entry, source, now))
        .collect()
}

fn parse_subscription(
    bytes: &[u8],
    source: &CommunitySource,
    max_configs: usize,
) -> Result<Vec<Candidate>, String> {
    if bytes.len() > DEFAULT_MAX_MANIFEST_BYTES {
        return Err("حجم منبع از سقف مجاز بیشتر است.".into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "متن subscription معتبر نیست.")?
        .trim();
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let decoded = general_purpose::STANDARD
        .decode(&compact)
        .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(&compact))
        .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(&compact))
        .ok()
        .and_then(|value| String::from_utf8(value).ok())
        .unwrap_or_else(|| text.to_owned());
    let mut result = Vec::new();
    for (index, uri) in decoded
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .enumerate()
    {
        if result.len() >= max_configs {
            break;
        }
        let mut protocol = uri
            .split_once("://")
            .map(|v| v.0.to_ascii_lowercase())
            .unwrap_or_default();
        if protocol == "ss" {
            protocol = "shadowsocks".into();
        }
        if let Ok(mut candidate) = validate_entry(
            ManifestEntry {
                id: format!("subscription-{index}"),
                uri: uri.to_owned(),
                protocol,
                country: subscription_country(&source.location),
                supports_udp: None,
                added_at: None,
                expires_at: None,
                source: None,
                checksum: None,
            },
            source,
            now_secs(),
        ) {
            candidate.id = format!(
                "{}:{}",
                source.id,
                sha256_hex(
                    candidate
                        .uri
                        .split('#')
                        .next()
                        .unwrap_or(&candidate.uri)
                        .as_bytes()
                )
            );
            result.push(candidate);
        }
    }
    if result.is_empty() {
        return Err("این منبع کانفیگ معتبر و پشتیبانی‌شده‌ای ندارد.".into());
    }
    Ok(deduplicate(result))
}

fn subscription_country(location: &str) -> Option<String> {
    // A source hint, never proof of geography or a substitute for local RTT.
    let url = url::Url::parse(location).ok()?;
    let country = url
        .path()
        .rsplit('/')
        .next()?
        .strip_prefix("v2ray-base64-")?
        .strip_suffix(".txt")?;
    (country.len() == 2 && country.bytes().all(|c| c.is_ascii_uppercase()))
        .then(|| country.to_owned())
}

fn validate_entry(
    entry: ManifestEntry,
    source: &CommunitySource,
    now: u64,
) -> Result<Candidate, String> {
    if entry.id.is_empty() || entry.id.len() > 120 {
        return Err("شناسهٔ کانفیگ معتبر نیست.".into());
    }
    if entry.expires_at.is_some_and(|value| value <= now) {
        return Err("manifest شامل کانفیگ منقضی‌شده است.".into());
    }
    let profile = ProxyProfile {
        name: entry.id.clone(),
        config_link: entry.uri.clone(),
    };
    let endpoint = profile.endpoint(false)?;
    if !entry.protocol.eq_ignore_ascii_case(&endpoint.protocol) {
        return Err("پروتکل اعلام‌شده با URI یکسان نیست.".into());
    }
    if let Some(checksum) = entry.checksum.as_deref() {
        verify_checksum(entry.uri.as_bytes(), Some(checksum))?;
    }
    Ok(Candidate {
        id: format!("{}:{}", source.id, entry.id),
        source_id: source.id.clone(),
        source_name: source.name.clone(),
        attribution: source.attribution.clone(),
        uri: entry.uri,
        protocol: endpoint.protocol,
        country: entry.country.filter(|value| value.len() <= 32),
        supports_udp: entry.supports_udp,
        added_at: entry.added_at,
        expires_at: entry.expires_at,
    })
}

pub fn deduplicate(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| {
            let profile = ProxyProfile {
                name: "dedup".into(),
                config_link: candidate.uri.clone(),
            };
            let key = profile
                .sing_box_config()
                .ok()
                .map(|config| config["outbounds"].to_string())
                .unwrap_or_else(|| candidate.uri.trim().to_owned());
            seen.insert(key)
        })
        .collect()
}

pub fn rank(
    mut results: Vec<HealthResult>,
    history: &HashMap<String, CandidateHistory>,
    now: u64,
) -> Vec<HealthResult> {
    for result in &mut results {
        let h = history
            .get(&result.candidate_id)
            .cloned()
            .unwrap_or_default();
        let latency = result.median_latency_ms.unwrap_or(10_000).min(10_000) as i64;
        let jitter = result.jitter_ms.unwrap_or(5_000).min(5_000) as i64;
        let udp = match result.udp_available {
            Some(true) => 250,
            Some(false) => -200,
            None => -40,
        };
        result.score = if result.working { 10_000 } else { -10_000 }
            - latency
            - jitter * 2
            - (result.failure_rate.clamp(0.0, 1.0) * 3000.0) as i64
            + udp
            + h.successes.min(20) as i64 * 15
            - h.disconnects.min(20) as i64 * 80;
        if h.next_retry_at.is_some_and(|until| until > now) {
            result.score -= 20_000;
        }
        if let Some(added_at) = result.added_at {
            let age_days = now.saturating_sub(added_at) / 86_400;
            result.score -= age_days.min(90) as i64 * 4;
        }
        result.label = if !result.working {
            "Offline"
        } else if result.failure_rate > 0.25 || jitter > 250 {
            "Unstable"
        } else if result.udp_available == Some(false) {
            "UDP unavailable"
        } else if latency < 250 {
            "Fast"
        } else {
            "Working"
        }
        .into();
    }
    // RandomState uses a process-local randomized seed, so equal scores do not
    // send every installation to the same endpoint during the same time window.
    let random = RandomState::new();
    results.sort_by_key(|result| {
        (
            std::cmp::Reverse(result.score / 50),
            random.hash_one(&result.candidate_id),
        )
    });
    results
}

pub fn record_failure(history: &mut CandidateHistory, now: u64) {
    history.consecutive_failures = history.consecutive_failures.saturating_add(1);
    history.disconnects = history.disconnects.saturating_add(1);
    let exponent = history.consecutive_failures.saturating_sub(1).min(8);
    history.next_retry_at = Some(now.saturating_add(5u64.saturating_mul(2u64.pow(exponent))));
}

pub fn redact_secrets(value: &str) -> String {
    let mut output = Vec::new();
    for token in value.split_whitespace() {
        if token.contains("://") {
            output.push("[redacted-uri]");
        } else if token.len() == 36 && token.chars().filter(|c| *c == '-').count() == 4 {
            output.push("[redacted-id]");
        } else {
            output.push(token);
        }
    }
    output.join(" ")
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn fetch_https(location: &str, timeout_seconds: u32, max_bytes: usize) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let _ = rustls::crypto::ring::default_provider().install_default();
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .https_only(true)
        .timeout(std::time::Duration::from_secs(timeout_seconds as u64))
        .redirect(reqwest::redirect::Policy::limited(3))
        .user_agent("DisRoute Community/1")
        .build()
        .map_err(|_| "آماده‌سازی دریافت HTTPS ناموفق بود.")?;
    let response = client
        .get(location)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|_| "دریافت منبع HTTPS ناموفق بود.")?;
    if response
        .content_length()
        .is_some_and(|size| size > max_bytes as u64)
    {
        return Err("حجم منبع از سقف مجاز بیشتر است.".into());
    }
    let mut bytes = Vec::new();
    response
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "خواندن پاسخ منبع ناموفق بود.")?;
    if bytes.len() > max_bytes {
        return Err("حجم منبع از سقف مجاز بیشتر است.".into());
    }
    Ok(bytes)
}

// Dependency-free SHA-256 used for manifest integrity checks.
fn sha256_hex(input: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut message = input.to_vec();
    let bit_len = (message.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for chunk in message.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (index, bytes) in chunk.chunks_exact(4).enumerate() {
            w[index] = u32::from_be_bytes(bytes.try_into().unwrap());
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (value, add) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *value = value.wrapping_add(add);
        }
    }
    h.iter().map(|value| format!("{value:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "downloads authorized public source and tests current third-party endpoints"]
    fn live_community_scan() {
        let mut state = CommunityState::default();
        initialize_sources(&mut state).unwrap();
        let started = std::time::Instant::now();
        let mut candidates = Vec::new();
        for (id, result) in fetch_sources(&state.sources) {
            match result {
                Ok(result) => {
                    eprintln!(
                        "Source {id}: {} validated candidates",
                        result.candidates.len()
                    );
                    candidates.extend(result.candidates);
                }
                Err(error) => eprintln!("Source {id}: {}", redact_secrets(&error)),
            }
        }
        let mut candidates = deduplicate(interleave_sources(candidates));
        candidates.truncate(DEFAULT_MAX_CONFIGS);
        eprintln!(
            "Fetch duration {:?}; {} candidates",
            started.elapsed(),
            candidates.len()
        );
        let exe = std::env::var_os("DISROUTE_TEST_ENGINE").expect("engine path");
        let root = std::env::temp_dir().join(format!("disroute-live-{}", std::process::id()));
        let results = crate::health::scan(
            candidates,
            exe.into(),
            root.clone(),
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            HashMap::new(),
        )
        .unwrap();
        let healthy: Vec<_> = results.iter().filter(|item| item.working).collect();
        eprintln!(
            "Tested {}; working {}; best measured latency {:?}",
            results.len(),
            healthy.len(),
            healthy.first().and_then(|item| item.median_latency_ms)
        );
        eprintln!(
            "Total duration {:?}; UDP working {}",
            started.elapsed(),
            healthy
                .iter()
                .filter(|r| r.udp_available == Some(true))
                .count()
        );
        let mut errors = std::collections::BTreeMap::new();
        for result in &results {
            if !result.working {
                *errors
                    .entry(
                        result
                            .error
                            .clone()
                            .unwrap_or_else(|| "HTTPS samples failed".into()),
                    )
                    .or_insert(0) += 1;
            }
        }
        eprintln!("Failure categories: {errors:?}");
        if root.exists() {
            assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
            std::fs::remove_dir(root).unwrap();
        }
        assert!(
            !healthy.is_empty(),
            "No working endpoint on this network during this scan"
        );
    }

    #[test]
    fn default_sources_are_valid_and_removals_persist() {
        let mut state = CommunityState::default();
        initialize_sources(&mut state).unwrap();
        assert_eq!(state.sources.len(), 8);
        state.sources.clear();
        initialize_sources(&mut state).unwrap();
        assert!(state.sources.is_empty());
    }

    #[test]
    fn catalog_upgrade_preserves_disabled_and_removed_sources() {
        let mut state = CommunityState {
            defaults_initialized: true,
            ..Default::default()
        };
        let mut disabled = source();
        disabled.id = "au1rxx-tr".into();
        disabled.enabled = false;
        state.sources.push(disabled);
        initialize_sources(&mut state).unwrap();
        assert_eq!(state.sources.len(), 4);
        assert!(
            !state
                .sources
                .iter()
                .find(|s| s.id == "au1rxx-tr")
                .unwrap()
                .enabled
        );
        assert!(!state.sources.iter().any(|s| s.id == "radikal-top100"));
        state.sources.clear();
        initialize_sources(&mut state).unwrap();
        assert!(state.sources.is_empty());
    }

    #[test]
    fn regional_hint_is_a_bounded_country_code() {
        assert_eq!(
            subscription_country("https://example.org/v2ray-base64-TR.txt"),
            Some("TR".into())
        );
        assert_eq!(
            subscription_country("https://example.org/v2ray-base64-0001.txt"),
            None
        );
    }

    #[test]
    fn large_source_cannot_starve_smaller_sources() {
        let mut candidates =
            parse_subscription(b"trojan://test@example.com:443", &source(), 10).unwrap();
        let one = candidates[0].clone();
        candidates.extend(vec![one.clone(); 600]);
        let mut other = one;
        other.source_id = "small".into();
        candidates.push(other);
        assert_eq!(interleave_sources(candidates)[1].source_id, "small");
    }

    #[test]
    fn mixed_subscription_keeps_valid_entries_and_stable_ids() {
        let good = "vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls";
        let first = parse_subscription(
            format!("# title\nhy2://bad\n{good}#one\n{good}#two").as_bytes(),
            &source(),
            10,
        )
        .unwrap();
        assert_eq!(first.len(), 1);
        let second = parse_subscription(good.as_bytes(), &source(), 10).unwrap();
        assert_eq!(first[0].id, second[0].id);
        assert!(parse_subscription(b"vless://bad@127.0.0.1:443", &source(), 10).is_err());
    }

    #[test]
    fn subscription_limits_retained_candidates() {
        let text = "trojan://test@example.com:443\ntrojan://test2@example.net:443";
        assert_eq!(
            parse_subscription(text.as_bytes(), &source(), 1)
                .unwrap()
                .len(),
            1
        );
    }
    fn source() -> CommunitySource {
        CommunitySource {
            id: "test".into(),
            name: "Test".into(),
            location: "C:\\manifest.json".into(),
            attribution: "Test operator".into(),
            kind: SourceKind::LocalFile,
            enabled: true,
            refresh_interval_minutes: 60,
            timeout_seconds: 10,
            expected_sha256: None,
            redistribution_authorized: true,
            last_successful_refresh: None,
            last_error: None,
        }
    }
    fn manifest(uri: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({ "version": 1, "id": "test", "source": { "name": "Test", "url": "https://example.com" }, "configurations": [{ "id": "one", "uri": uri, "protocol": "vless", "supportsUdp": true }] })).unwrap()
    }

    #[test]
    fn parses_v1_manifest() {
        assert_eq!(
            parse_manifest(
                &manifest(
                    "vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls"
                ),
                &source(),
                10
            )
            .unwrap()
            .len(),
            1
        );
    }
    #[test]
    fn rejects_unknown_manifest_version() {
        let bytes = serde_json::to_vec(&serde_json::json!({"version": 99, "id":"x", "source":{"name":"x","url":"x"}, "configurations":[]})).unwrap();
        assert!(parse_manifest(&bytes, &source(), 10)
            .unwrap_err()
            .contains("نسخه"));
    }
    #[test]
    fn rejects_unsafe_endpoint() {
        assert!(parse_manifest(
            &manifest("vless://00000000-0000-4000-8000-000000000000@127.0.0.1:443?security=tls"),
            &source(),
            10
        )
        .is_err());
    }
    #[test]
    fn rejects_oversized_manifest() {
        assert!(
            parse_manifest(&vec![b' '; DEFAULT_MAX_MANIFEST_BYTES + 1], &source(), 10).is_err()
        );
    }
    #[test]
    fn deduplicates_credentials() {
        let one = parse_manifest(
            &manifest("vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls"),
            &source(),
            10,
        )
        .unwrap()
        .pop()
        .unwrap();
        assert_eq!(deduplicate(vec![one.clone(), one]).len(), 1);
    }
    #[test]
    fn redacts_uris_and_ids() {
        let value = redact_secrets(
            "failed vless://secret@example.com 00000000-0000-4000-8000-000000000000",
        );
        assert!(!value.contains("secret"));
        assert!(!value.contains("00000000"));
    }
    #[test]
    fn applies_exponential_backoff() {
        let mut history = CandidateHistory::default();
        record_failure(&mut history, 100);
        assert_eq!(history.next_retry_at, Some(105));
        record_failure(&mut history, 100);
        assert_eq!(history.next_retry_at, Some(110));
    }
    #[test]
    fn sha256_matches_standard_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
    #[test]
    fn ranks_working_and_rotates_equivalent_candidates() {
        let make = |id: &str, working| HealthResult {
            candidate_id: id.into(),
            source_id: "s".into(),
            source_name: "s".into(),
            attribution: "a".into(),
            protocol: "vless".into(),
            country: None,
            working,
            median_latency_ms: Some(100),
            jitter_ms: Some(10),
            failure_rate: 0.0,
            udp_available: Some(true),
            label: String::new(),
            score: 0,
            error: None,
            added_at: None,
        };
        let first = rank(vec![make("a", true), make("b", false)], &HashMap::new(), 0);
        assert_eq!(first[0].candidate_id, "a");
        let first_ids: HashSet<String> = (0..100)
            .map(|bucket| {
                rank(
                    vec![make("a", true), make("b", true)],
                    &HashMap::new(),
                    bucket * 300,
                )[0]
                .candidate_id
                .clone()
            })
            .collect();
        assert_eq!(first_ids.len(), 2);
    }

    #[test]
    fn local_provider_fetches_a_real_manifest_file() {
        let path =
            std::env::temp_dir().join(format!("disroute-manifest-{}.json", std::process::id()));
        std::fs::write(
            &path,
            manifest("vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls"),
        )
        .unwrap();
        let mut local = source();
        local.location = path.to_string_lossy().into_owned();
        let result = fetch_source(&local, DEFAULT_MAX_MANIFEST_BYTES, 10).unwrap();
        assert_eq!(result.candidates.len(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cache_refresh_interval_is_enforced() {
        let mut value = source();
        value.last_successful_refresh = Some(1_000);
        assert!(!source_due(&value, 1_000 + 59 * 60));
        assert!(source_due(&value, 1_000 + 60 * 60));
    }
}
