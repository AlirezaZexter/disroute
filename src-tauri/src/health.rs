use crate::community::{rank, Candidate, CandidateHistory, HealthResult};
use crate::engine::hidden_command;
use crate::model::{validate_endpoint_host, ProxyProfile};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

const SAMPLE_COUNT: usize = 3;
const MAX_SCAN_CANDIDATES: usize = 100;
const WORKERS: usize = 4;
const TOTAL_SCAN_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct ScanControl {
    cancelled: Arc<AtomicBool>,
}

impl ScanControl {
    pub fn begin(&mut self) -> Arc<AtomicBool> {
        self.cancel();
        self.cancelled = Arc::new(AtomicBool::new(false));
        self.cancelled.clone()
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

pub fn scan(
    candidates: Vec<Candidate>,
    sing_box_exe: PathBuf,
    runtime_root: PathBuf,
    cancelled: Arc<AtomicBool>,
    history: HashMap<String, CandidateHistory>,
) -> Result<Vec<HealthResult>, String> {
    if candidates.is_empty() {
        return Err("هیچ کانفیگی برای آزمایش وجود ندارد.".into());
    }
    let mut candidates = candidates;
    use std::hash::BuildHasher;
    let random = std::collections::hash_map::RandomState::new();
    let now = crate::community::now_secs();
    candidates.retain(|candidate| {
        !history
            .get(&candidate.id)
            .and_then(|h| h.next_retry_at)
            .is_some_and(|until| until > now)
    });
    candidates.sort_by_key(|candidate| {
        let h = history.get(&candidate.id);
        let recent_latency = h
            .filter(|h| h.tested_at.is_some_and(|t| now.saturating_sub(t) < 86400))
            .and_then(|h| h.last_latency_ms);
        (
            recent_latency.unwrap_or(u64::MAX) / 100,
            std::cmp::Reverse(history.get(&candidate.id).map(|h| h.successes).unwrap_or(0)),
            random.hash_one(&candidate.id),
        )
    });
    let candidates = crate::community::interleave_sources(candidates);
    let queue = Arc::new(Mutex::new(VecDeque::from(
        candidates
            .into_iter()
            .take(MAX_SCAN_CANDIDATES)
            .collect::<Vec<_>>(),
    )));
    let results = Arc::new(Mutex::new(Vec::new()));
    let started = Instant::now();
    let deadline = started + TOTAL_SCAN_TIMEOUT;
    let mut workers = Vec::new();
    for _ in 0..WORKERS {
        let queue = queue.clone();
        let results = results.clone();
        let exe = sing_box_exe.clone();
        let root = runtime_root.clone();
        let cancelled = cancelled.clone();
        workers.push(thread::spawn(move || loop {
            if cancelled.load(Ordering::Relaxed) || Instant::now() >= deadline {
                break;
            }
            if results
                .lock()
                .is_ok_and(|values| shortlist_ready(&values, started.elapsed()))
            {
                break;
            }
            let candidate = queue.lock().ok().and_then(|mut queue| queue.pop_front());
            let Some(candidate) = candidate else {
                break;
            };
            let result = test_candidate(candidate, &exe, &root, &cancelled);
            if let Ok(mut values) = results.lock() {
                values.push(result);
            }
        }));
    }
    for worker in workers {
        let _ = worker.join();
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err("آزمایش اتصال لغو شد.".into());
    }
    let values = Arc::try_unwrap(results)
        .map_err(|_| "جمع‌آوری نتیجه‌ها ناموفق بود.")?
        .into_inner()
        .map_err(|_| "جمع‌آوری نتیجه‌ها ناموفق بود.")?;
    Ok(rank(values, &history, crate::community::now_secs()))
}

fn test_candidate(
    candidate: Candidate,
    sing_box_exe: &Path,
    runtime_root: &Path,
    cancelled: &AtomicBool,
) -> HealthResult {
    let mut result = HealthResult {
        candidate_id: candidate.id.clone(),
        source_id: candidate.source_id.clone(),
        source_name: candidate.source_name.clone(),
        attribution: candidate.attribution.clone(),
        protocol: candidate.protocol.clone(),
        country: candidate.country.clone(),
        working: false,
        median_latency_ms: None,
        jitter_ms: None,
        failure_rate: 1.0,
        udp_available: None,
        label: "Untested".into(),
        score: 0,
        error: None,
        added_at: candidate.added_at,
    };
    match test_candidate_inner(&candidate, sing_box_exe, runtime_root, cancelled, false) {
        Ok((samples, failures, udp)) => {
            let successful = samples.len();
            let attempts = successful + failures;
            result.working = successful >= 2;
            result.failure_rate = if attempts == 0 {
                1.0
            } else {
                failures as f64 / attempts as f64
            };
            if !samples.is_empty() {
                let mut sorted = samples;
                sorted.sort_unstable();
                let median = sorted[sorted.len() / 2];
                result.median_latency_ms = Some(median);
                result.jitter_ms = Some(
                    sorted
                        .iter()
                        .map(|sample| sample.abs_diff(median))
                        .sum::<u64>()
                        / sorted.len() as u64,
                );
            }
            result.udp_available = udp;
        }
        Err(error) => result.error = Some(crate::community::redact_secrets(&error)),
    }
    result
}

fn shortlist_ready(results: &[HealthResult], elapsed: Duration) -> bool {
    let stable = |r: &&HealthResult| {
        r.working && r.failure_rate == 0.0 && r.jitter_ms.is_some_and(|j| j <= 250)
    };
    let voice = results
        .iter()
        .filter(stable)
        .filter(|r| r.udp_available == Some(true))
        .count();
    voice >= 3 || (elapsed >= Duration::from_secs(20) && results.iter().filter(stable).count() >= 4)
}

fn test_candidate_inner(
    candidate: &Candidate,
    sing_box_exe: &Path,
    runtime_root: &Path,
    cancelled: &AtomicBool,
    allow_private: bool,
) -> Result<(Vec<u64>, usize, Option<bool>), String> {
    if cancelled.load(Ordering::Relaxed) {
        return Err("آزمایش لغو شد.".into());
    }
    let profile = ProxyProfile {
        name: candidate.id.clone(),
        config_link: candidate.uri.clone(),
    };
    let endpoint = profile.endpoint(allow_private)?;
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let address_to_resolve = (endpoint.host.clone(), endpoint.port);
    thread::spawn(move || {
        let _ = tx.send(
            address_to_resolve
                .to_socket_addrs()
                .map(|items| items.collect::<Vec<SocketAddr>>()),
        );
    });
    let addresses = rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "مهلت یافتن آدرس سرور تمام شد.")?
        .map_err(|_| "نام سرور resolve نشد.")?;
    if addresses.is_empty() {
        return Err("برای سرور هیچ IP معتبری پیدا نشد.".into());
    }
    for address in &addresses {
        validate_endpoint_host(&address.ip().to_string(), allow_private)?;
    }
    let reachable = addresses
        .iter()
        .take(4)
        .find(|address| {
            !cancelled.load(Ordering::Relaxed)
                && TcpStream::connect_timeout(address, Duration::from_secs(2)).is_ok()
        })
        .ok_or("سرور روی پورت اعلام‌شده در دسترس نیست.")?;

    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))
        .map_err(|_| "اختصاص پورت آزمایش ناموفق بود.")?;
    let port = listener
        .local_addr()
        .map_err(|_| "خواندن پورت آزمایش ناموفق بود.")?
        .port();
    drop(listener);
    let test_id = format!(
        "{}-{}-{port}",
        std::process::id(),
        crate::community::now_secs()
    );
    let dir = runtime_root.join(test_id);
    std::fs::create_dir_all(&dir).map_err(|_| "ساخت پوشهٔ موقت آزمایش ناموفق بود.")?;
    let config_path = dir.join("config.json");
    let mut cleanup = TestProcess { child: None, dir };
    let mut config = profile.sing_box_config_for_port(port)?;
    config["outbounds"][0]["server"] = serde_json::json!(reachable.ip().to_string());
    let config = serde_json::to_vec(&config).map_err(|_| "ساخت کانفیگ داخلی آزمایش ناموفق بود.")?;
    std::fs::write(&config_path, config).map_err(|_| "نوشتن کانفیگ موقت آزمایش ناموفق بود.")?;
    cleanup.child = Some(
        hidden_command(sing_box_exe)
            .args(["check", "-c"])
            .arg(&config_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "اعتبارسنجی موتور آزمایش اجرا نشد.")?,
    );
    let check_deadline = Instant::now() + Duration::from_secs(3);
    let check = loop {
        if cancelled.load(Ordering::Relaxed) || Instant::now() >= check_deadline {
            return Err("مهلت اعتبارسنجی موتور تمام شد.".into());
        }
        if let Some(status) = cleanup
            .child
            .as_mut()
            .unwrap()
            .try_wait()
            .map_err(|_| "موتور آزمایش متوقف شد.")?
        {
            break status;
        }
        thread::sleep(Duration::from_millis(25));
    };
    if !check.success() {
        return Err("موتور، کانفیگ تولیدشده را نپذیرفت.".into());
    }
    cleanup.child = Some(
        hidden_command(sing_box_exe)
            .args(["run", "-c"])
            .arg(&config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "نمونهٔ موقت sing-box اجرا نشد.")?,
    );
    wait_for_port(port, &mut cleanup.child, cancelled)?;

    let mut samples = Vec::new();
    let mut failures = 0;
    for _ in 0..SAMPLE_COUNT {
        if cancelled.load(Ordering::Relaxed) {
            return Err("آزمایش لغو شد.".into());
        }
        match https_sample(port) {
            Ok(value) => samples.push(value),
            Err(_) => failures += 1,
        }
        if samples.is_empty() && failures >= 1 {
            break;
        }
    }
    let udp = if samples.is_empty() {
        None
    } else if candidate.supports_udp == Some(false) {
        Some(false)
    } else {
        Some(test_udp_dns(port).is_ok())
    };
    Ok((samples, failures, udp))
}

struct TestProcess {
    child: Option<Child>,
    dir: PathBuf,
}
impl Drop for TestProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn wait_for_port(
    port: u16,
    child: &mut Option<Child>,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if cancelled.load(Ordering::Relaxed) {
            return Err("آزمایش لغو شد.".into());
        }
        if child
            .as_mut()
            .and_then(|child| child.try_wait().ok())
            .flatten()
            .is_some()
        {
            return Err("sing-box هنگام آماده‌سازی متوقف شد.".into());
        }
        if TcpStream::connect_timeout(
            &SocketAddr::from(([127, 0, 0, 1], port)),
            Duration::from_millis(150),
        )
        .is_ok()
        {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(80));
    }
    Err("مهلت آماده‌شدن sing-box تمام شد.".into())
}

fn https_sample(port: u16) -> Result<u64, String> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let proxy = format!("socks5h://127.0.0.1:{port}");
    let started = Instant::now();
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .proxy(reqwest::Proxy::all(&proxy).map_err(|_| "نشانی پروکسی آزمایش معتبر نیست.")?)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .https_only(true)
        .user_agent("DisRoute/Community")
        .build()
        .map_err(|_| "آماده‌سازی HTTPS ناموفق بود.")?;
    // TLS verifies discord.com, and this client has exactly one mandatory SOCKS
    // proxy with environment/system proxy discovery disabled. No direct fallback.
    let response = client
        .get("https://discord.com/api/v10/gateway")
        .send()
        .map_err(|_| "پاسخ HTTPS معتبر از Discord دریافت نشد.")?;
    if !response.status().is_success() {
        return Err("پاسخ HTTPS معتبر نبود.".into());
    }
    Ok(started.elapsed().as_millis().min(u64::MAX as u128) as u64)
}

pub(crate) fn active_https_available() -> bool {
    // Require two failures before replacing an otherwise running tunnel.
    https_sample(2080).is_ok() || https_sample(2080).is_ok()
}

fn test_udp_dns(port: u16) -> Result<(), String> {
    let mut control = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_secs(2),
    )
    .map_err(|_| "SOCKS برای UDP پاسخ نداد.")?;
    control.set_read_timeout(Some(Duration::from_secs(4))).ok();
    control.set_write_timeout(Some(Duration::from_secs(2))).ok();
    control
        .write_all(&[5, 1, 0])
        .map_err(|_| "شروع SOCKS UDP ناموفق بود.")?;
    let mut greeting = [0; 2];
    control
        .read_exact(&mut greeting)
        .map_err(|_| "پاسخ SOCKS UDP دریافت نشد.")?;
    if greeting != [5, 0] {
        return Err("SOCKS UDP پذیرفته نشد.".into());
    }
    control
        .write_all(&[5, 3, 0, 1, 0, 0, 0, 0, 0, 0])
        .map_err(|_| "UDP associate ارسال نشد.")?;
    let mut head = [0u8; 4];
    control
        .read_exact(&mut head)
        .map_err(|_| "پاسخ UDP associate دریافت نشد.")?;
    if head[1] != 0 {
        return Err("UDP associate رد شد.".into());
    }
    let relay = match head[3] {
        1 => {
            let mut ip = [0u8; 4];
            control
                .read_exact(&mut ip)
                .map_err(|_| "آدرس relay ناقص است.")?;
            let mut p = [0u8; 2];
            control
                .read_exact(&mut p)
                .map_err(|_| "پورت relay ناقص است.")?;
            SocketAddr::from((ip, u16::from_be_bytes(p)))
        }
        _ => return Err("نوع آدرس relay پشتیبانی نمی‌شود.".into()),
    };
    let relay = if relay.ip().is_unspecified() {
        SocketAddr::from(([127, 0, 0, 1], relay.port()))
    } else {
        relay
    };
    let socket = UdpSocket::bind(("127.0.0.1", 0)).map_err(|_| "ساخت سوکت UDP ناموفق بود.")?;
    socket.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let mut packet = vec![0, 0, 0, 1, 1, 1, 1, 1, 0, 53];
    packet.extend_from_slice(&[
        0x42, 0x10, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 7, b'd', b'i', b's', b'c', b'o', b'r', b'd', 3,
        b'c', b'o', b'm', 0, 0, 1, 0, 1,
    ]);
    socket
        .connect(relay)
        .map_err(|_| "اتصال به relay ناموفق بود.")?;
    socket
        .send_to(&packet, relay)
        .map_err(|_| "ارسال UDP آزمایشی ناموفق بود.")?;
    let mut response = [0u8; 1500];
    let size = socket
        .recv(&mut response)
        .map_err(|_| "پاسخ UDP آزمایشی دریافت نشد.")?;
    if !valid_udp_reply(&response[..size]) {
        return Err("پاسخ UDP معتبر نبود.".into());
    }
    Ok(())
}

fn valid_udp_reply(response: &[u8]) -> bool {
    response.len() >= 22
        && response[..10] == [0, 0, 0, 1, 1, 1, 1, 1, 0, 53]
        && response[10..12] == [0x42, 0x10]
        && response[12] & 0x80 != 0
        && response[13] & 0x0f == 0
        && response[14..16] == [0, 1]
        && u16::from_be_bytes([response[16], response[17]]) > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn early_completion_requires_stable_shortlist_not_one_lucky_probe() {
        let result = HealthResult {
            candidate_id: "test".into(),
            source_id: "test".into(),
            source_name: "Test".into(),
            attribution: "Local test".into(),
            protocol: "vless".into(),
            country: None,
            working: true,
            median_latency_ms: Some(100),
            jitter_ms: Some(10),
            failure_rate: 0.0,
            udp_available: Some(true),
            label: String::new(),
            score: 0,
            error: None,
            added_at: None,
        };
        assert!(!shortlist_ready(&[result.clone()], Duration::from_secs(30)));
        assert!(shortlist_ready(
            &vec![result.clone(); 3],
            Duration::from_secs(2)
        ));
        let mut no_udp = result.clone();
        no_udp.udp_available = Some(false);
        assert!(!shortlist_ready(
            &vec![no_udp.clone(); 4],
            Duration::from_secs(2)
        ));
        assert!(shortlist_ready(&vec![no_udp; 4], Duration::from_secs(20)));
        let mut unstable = result;
        unstable.failure_rate = 1.0 / 3.0;
        assert!(!shortlist_ready(
            &vec![unstable; 5],
            Duration::from_secs(30)
        ));
    }

    #[test]
    fn udp_requires_matching_dns_response() {
        assert!(!valid_udp_reply(&[0; 100]));
        let mut reply = vec![
            0, 0, 0, 1, 1, 1, 1, 1, 0, 53, 0x42, 0x10, 0x81, 0x80, 0, 1, 0, 1, 0, 0, 0, 0,
        ];
        assert!(valid_udp_reply(&reply));
        reply[10] = 0x43;
        assert!(!valid_udp_reply(&reply));
    }

    #[test]
    #[ignore = "requires DISROUTE_TEST_ENGINE pointing to the pinned sing-box binary"]
    fn isolated_engine_authentication_and_cleanup() {
        let exe = PathBuf::from(std::env::var_os("DISROUTE_TEST_ENGINE").expect("engine path"));
        let root =
            std::env::temp_dir().join(format!("disroute-integration-{}", std::process::id()));
        let free_port = || {
            let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
            l.local_addr().unwrap().port()
        };
        let server_port = free_port();
        let mut server = TestProcess {
            child: None,
            dir: root.join("server"),
        };
        std::fs::create_dir_all(&server.dir).unwrap();
        let server_config = server.dir.join("config.json");
        std::fs::write(&server_config, serde_json::to_vec(&serde_json::json!({
            "log": {"disabled": true},
            "inbounds": [{"type":"shadowsocks", "listen":"127.0.0.1", "listen_port":server_port, "method":"aes-128-gcm", "password":"local-test-password"}],
            "outbounds": [{"type":"direct"}]
        })).unwrap()).unwrap();
        server.child = Some(
            hidden_command(&exe)
                .args(["run", "-c"])
                .arg(&server_config)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let cancelled = AtomicBool::new(false);
        wait_for_port(server_port, &mut server.child, &cancelled).unwrap();
        for (password, expected) in [
            ("local-test-password", true),
            ("wrong-test-password", false),
        ] {
            let target = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
            let target_port = target.local_addr().unwrap().port();
            target.set_nonblocking(true).unwrap();
            let target_thread = thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(4);
                while Instant::now() < deadline {
                    if let Ok((mut stream, _)) = target.accept() {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(1)))
                            .unwrap();
                        let mut request = [0; 1024];
                        let _ = stream.read(&mut request);
                        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\ndisroute-marker");
                        return;
                    }
                    thread::sleep(Duration::from_millis(20));
                }
            });
            let port = free_port();
            let mut client = TestProcess {
                child: None,
                dir: root.join(format!("client-{port}")),
            };
            std::fs::create_dir_all(&client.dir).unwrap();
            let config_path = client.dir.join("config.json");
            let profile = ProxyProfile {
                name: "local test".into(),
                config_link: format!("ss://aes-128-gcm:{password}@127.0.0.1:{server_port}"),
            };
            std::fs::write(
                &config_path,
                serde_json::to_vec(&profile.sing_box_config_for_port(port).unwrap()).unwrap(),
            )
            .unwrap();
            client.child = Some(
                hidden_command(&exe)
                    .args(["run", "-c"])
                    .arg(&config_path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap(),
            );
            wait_for_port(port, &mut client.child, &cancelled).unwrap();
            let _ = rustls::crypto::ring::default_provider().install_default();
            let http = reqwest::blocking::Client::builder()
                .no_proxy()
                .proxy(reqwest::Proxy::all(format!("socks5h://127.0.0.1:{port}")).unwrap())
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap();
            let result = http
                .get(format!("http://127.0.0.1:{target_port}"))
                .send()
                .and_then(|r| r.text());
            assert_eq!(result.is_ok_and(|body| body == "disroute-marker"), expected);
            let dir = client.dir.clone();
            drop(client);
            assert!(!dir.exists());
            assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
            target_thread.join().unwrap();
        }
        drop(server);
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn cancellation_stops_empty_scan_work() {
        let cancelled = Arc::new(AtomicBool::new(true));
        let result = scan(
            vec![Candidate {
                id: "x".into(),
                source_id: "s".into(),
                source_name: "s".into(),
                attribution: "a".into(),
                uri: "vless://x".into(),
                protocol: "vless".into(),
                country: None,
                supports_udp: None,
                added_at: None,
                expires_at: None,
            }],
            PathBuf::from("missing"),
            std::env::temp_dir(),
            cancelled,
            HashMap::new(),
        );
        assert!(result.unwrap_err().contains("لغو"));
    }
}
