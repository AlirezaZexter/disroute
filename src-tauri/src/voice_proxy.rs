//! Loopback SOCKS bridge: recover Discord voice SNI without decrypting TLS.
//! UDP ASSOCIATE is forwarded unchanged; datagrams go directly to sing-box.
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex, Weak,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct VoiceProxy {
    stopped: Arc<AtomicBool>,
    sockets: Arc<Mutex<Vec<Weak<TcpStream>>>>,
    listener: Option<JoinHandle<()>>,
}

impl VoiceProxy {
    pub fn start(port: u16, upstream: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))?;
        listener.set_nonblocking(true)?;
        let stopped = Arc::new(AtomicBool::new(false));
        let sockets = Arc::new(Mutex::new(Vec::<Weak<TcpStream>>::new()));
        let (stop, tracked) = (stopped.clone(), sockets.clone());
        let active = Arc::new(AtomicUsize::new(0));
        let worker = thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((socket, _)) => {
                        // Windows accepts inherit the listener's nonblocking mode.
                        if socket.set_nonblocking(false).is_err() {
                            continue;
                        }
                        if active.load(Ordering::Relaxed) >= 128 {
                            continue;
                        }
                        let socket = Arc::new(socket);
                        let mut list = tracked.lock().unwrap();
                        list.retain(|s| s.strong_count() > 0);
                        list.push(Arc::downgrade(&socket));
                        drop(list);
                        active.fetch_add(1, Ordering::Relaxed);
                        let (active, tracked, stop) =
                            (active.clone(), tracked.clone(), stop.clone());
                        thread::spawn(move || {
                            let result = serve(socket.clone(), upstream, &tracked, &stop);
                            #[cfg(test)]
                            if let Err(error) = &result {
                                eprintln!("bridge test: {error}");
                            }
                            let _ = result;
                            let _ = socket.shutdown(Shutdown::Both);
                            active.fetch_sub(1, Ordering::Relaxed);
                        });
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            stopped,
            sockets,
            listener: Some(worker),
        })
    }
}

impl Drop for VoiceProxy {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        if let Some(worker) = self.listener.take() {
            let _ = worker.join();
        }
        for socket in self
            .sockets
            .lock()
            .unwrap()
            .iter()
            .filter_map(Weak::upgrade)
        {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid SOCKS/TLS input")
}

fn serve(
    client: Arc<TcpStream>,
    port: u16,
    tracked: &Mutex<Vec<Weak<TcpStream>>>,
    stop: &AtomicBool,
) -> io::Result<()> {
    client.set_read_timeout(Some(Duration::from_secs(5)))?;
    client.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut source = &*client;
    let mut hello = [0; 2];
    source.read_exact(&mut hello)?;
    if hello[0] != 5 || hello[1] == 0 {
        return Err(invalid());
    }
    let mut methods = vec![0; hello[1] as usize];
    source.read_exact(&mut methods)?;
    if !methods.contains(&0) {
        source.write_all(&[5, 255])?;
        return Err(invalid());
    }
    source.write_all(&[5, 0])?;
    let mut header = [0; 4];
    source.read_exact(&mut header)?;
    if header[0] != 5 || header[2] != 0 || ![1, 3].contains(&header[1]) {
        source.write_all(&[5, 7, 0, 1, 0, 0, 0, 0, 0, 0])?;
        return Err(invalid());
    }
    let mut address = Vec::new();
    match header[3] {
        1 => address.resize(4, 0),
        4 => address.resize(16, 0),
        3 => {
            let mut len = [0];
            source.read_exact(&mut len)?;
            address.push(len[0]);
            address.resize(1 + len[0] as usize, 0);
        }
        _ => return Err(invalid()),
    }
    let offset = usize::from(header[3] == 3);
    source.read_exact(&mut address[offset..])?;
    let mut destination_port = [0; 2];
    source.read_exact(&mut destination_port)?;
    // Only IP-addressed voice TLS on Discord's observed TLS ports needs recovery.
    // Leave UDP, domain requests and all other traffic untouched.
    let recover = header[1] == 1
        && [1, 4].contains(&header[3])
        && [443, 2053, 2083, 2087, 2096, 8443].contains(&u16::from_be_bytes(destination_port));
    let mut prefix = Vec::new();
    if recover {
        source.write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0])?;
        if let Some(domain) = read_client_hello(&mut source, &mut prefix)? {
            header[3] = 3;
            address = vec![domain.len() as u8];
            address.extend_from_slice(domain.as_bytes());
        }
    }
    let upstream = Arc::new(TcpStream::connect_timeout(
        &([127, 0, 0, 1], port).into(),
        Duration::from_secs(3),
    )?);
    {
        let mut list = tracked.lock().unwrap();
        if stop.load(Ordering::Acquire) {
            return Err(invalid());
        }
        list.push(Arc::downgrade(&upstream));
    }
    upstream.set_read_timeout(Some(Duration::from_secs(10)))?;
    upstream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let mut remote = &*upstream;
    remote.write_all(&[5, 1, 0])?;
    remote.read_exact(&mut hello)?;
    if hello != [5, 0] {
        return Err(invalid());
    }
    remote.write_all(&header)?;
    remote.write_all(&address)?;
    remote.write_all(&destination_port)?;
    let mut reply = [0; 4];
    remote.read_exact(&mut reply)?;
    let n = match reply[3] {
        1 => 4,
        4 => 16,
        3 => {
            let mut n = [0];
            remote.read_exact(&mut n)?;
            if !recover {
                source.write_all(&reply)?;
                source.write_all(&n)?;
            }
            n[0] as usize
        }
        _ => return Err(invalid()),
    };
    let mut tail = vec![0; n + 2];
    remote.read_exact(&mut tail)?;
    if !recover {
        if reply[3] != 3 {
            source.write_all(&reply)?;
        }
        source.write_all(&tail)?;
    }
    if reply[0] != 5 || reply[1] != 0 {
        return Err(invalid());
    }
    remote.write_all(&prefix)?;
    client.set_read_timeout(None)?;
    client.set_write_timeout(None)?;
    upstream.set_read_timeout(None)?;
    upstream.set_write_timeout(None)?;
    let (c, u) = (client.clone(), upstream.clone());
    let back = thread::spawn(move || {
        let _ = io::copy(&mut &*u, &mut &*c);
        let _ = c.shutdown(Shutdown::Both);
        let _ = u.shutdown(Shutdown::Both);
    });
    let _ = io::copy(&mut source, &mut remote);
    let _ = client.shutdown(Shutdown::Both);
    let _ = upstream.shutdown(Shutdown::Both);
    let _ = back.join();
    Ok(())
}

fn read_client_hello(input: &mut impl Read, raw: &mut Vec<u8>) -> io::Result<Option<String>> {
    let mut handshake = Vec::new();
    loop {
        let mut first = [0];
        input.read_exact(&mut first)?;
        raw.push(first[0]);
        if first[0] != 22 {
            return Ok(None);
        }
        let mut header = [0; 4];
        input.read_exact(&mut header)?;
        raw.extend_from_slice(&header);
        let size = u16::from_be_bytes([header[2], header[3]]) as usize;
        if header[0] != 3 || size == 0 || raw.len() + size > 65536 {
            return Err(invalid());
        }
        let start = raw.len();
        raw.resize(start + size, 0);
        input.read_exact(&mut raw[start..])?;
        handshake.extend_from_slice(&raw[start..]);
        if handshake.len() >= 4 {
            if handshake[0] != 1 {
                return Ok(None);
            }
            let len = ((handshake[1] as usize) << 16)
                | ((handshake[2] as usize) << 8)
                | handshake[3] as usize;
            if len > 65532 {
                return Err(invalid());
            }
            if handshake.len() >= len + 4 {
                return Ok(voice_sni(&handshake[4..4 + len]));
            }
        }
    }
}

fn take<'a>(data: &mut &'a [u8], n: usize) -> Option<&'a [u8]> {
    if data.len() < n {
        return None;
    }
    let (value, rest) = data.split_at(n);
    *data = rest;
    Some(value)
}
fn number(data: &mut &[u8]) -> Option<usize> {
    let b = take(data, 2)?;
    Some(u16::from_be_bytes([b[0], b[1]]) as usize)
}
fn voice_sni(mut data: &[u8]) -> Option<String> {
    take(&mut data, 34)?;
    let n = take(&mut data, 1)?[0] as usize;
    take(&mut data, n)?;
    let n = number(&mut data)?;
    take(&mut data, n)?;
    let n = take(&mut data, 1)?[0] as usize;
    take(&mut data, n)?;
    let n = number(&mut data)?;
    let mut extensions = take(&mut data, n)?;
    while !extensions.is_empty() {
        let kind = number(&mut extensions)?;
        let n = number(&mut extensions)?;
        let mut value = take(&mut extensions, n)?;
        if kind != 0 {
            continue;
        }
        let n = number(&mut value)?;
        let mut names = take(&mut value, n)?;
        while !names.is_empty() {
            let kind = take(&mut names, 1)?[0];
            let n = number(&mut names)?;
            let bytes = take(&mut names, n)?;
            if kind != 0 {
                continue;
            }
            let name = std::str::from_utf8(bytes).ok()?.to_ascii_lowercase();
            if name.len() <= 253
                && name.ends_with(".discord.media")
                && name.split('.').all(|s| {
                    !s.is_empty()
                        && s.len() <= 63
                        && !s.starts_with('-')
                        && !s.ends_with('-')
                        && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
                })
            {
                return Some(name);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hello(name: &str) -> Vec<u8> {
        let mut body = vec![3, 3];
        body.extend_from_slice(&[0; 32]);
        body.extend_from_slice(&[0, 0, 2, 0x13, 1, 1, 0]);
        let mut sni = Vec::new();
        sni.extend_from_slice(&((name.len() + 3) as u16).to_be_bytes());
        sni.push(0);
        sni.extend_from_slice(&(name.len() as u16).to_be_bytes());
        sni.extend_from_slice(name.as_bytes());
        body.extend_from_slice(&((sni.len() + 4) as u16).to_be_bytes());
        body.extend_from_slice(&[0, 0]);
        body.extend_from_slice(&(sni.len() as u16).to_be_bytes());
        body.extend(sni);
        body
    }
    #[test]
    fn dynamic_voice_names_only() {
        assert_eq!(
            voice_sni(&hello("c-ams13-new.discord.media")).as_deref(),
            Some("c-ams13-new.discord.media")
        );
        for name in [
            "evil.com",
            "discord.media.evil.com",
            "bad/host.discord.media",
            ".discord.media",
        ] {
            assert!(voice_sni(&hello(name)).is_none());
        }
        let bytes = hello("c-sin12-new.discord.media");
        for n in 0..bytes.len() {
            assert!(voice_sni(&bytes[..n]).is_none());
        }
    }
    #[test]
    fn fragmented_hello_preserves_encrypted_bytes() {
        let body = hello("c-new.discord.media");
        let mut handshake = vec![1, 0, 0, body.len() as u8];
        handshake.extend(body);
        let mut wire = Vec::new();
        for part in handshake.chunks(11) {
            wire.extend_from_slice(&[22, 3, 1]);
            wire.extend_from_slice(&(part.len() as u16).to_be_bytes());
            wire.extend_from_slice(part);
        }
        let mut copy = Vec::new();
        assert_eq!(
            read_client_hello(&mut &wire[..], &mut copy)
                .unwrap()
                .as_deref(),
            Some("c-new.discord.media")
        );
        assert_eq!(copy, wire);
    }
    #[test]
    fn udp_associate_keeps_upstream_relay_endpoint() {
        let upstream = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = upstream.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut socket, _) = upstream.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut greeting = [0; 3];
            socket.read_exact(&mut greeting).unwrap();
            assert_eq!(greeting, [5, 1, 0]);
            socket.write_all(&[5, 0]).unwrap();
            let mut request = [0; 10];
            socket.read_exact(&mut request).unwrap();
            assert_eq!(request, [5, 3, 0, 1, 0, 0, 0, 0, 0, 0]);
            socket
                .write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 8, 33])
                .unwrap();
            let mut eof = [0];
            assert_eq!(socket.read(&mut eof).unwrap(), 0);
        });
        let proxy = VoiceProxy::start(22083, port).unwrap();
        let mut client = TcpStream::connect("127.0.0.1:22083").unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        client.write_all(&[5, 1, 0]).unwrap();
        let mut greeting = [0; 2];
        client.read_exact(&mut greeting).unwrap();
        assert_eq!(greeting, [5, 0]);
        client.write_all(&[5, 3, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap();
        let mut reply = [0; 10];
        client.read_exact(&mut reply).unwrap();
        assert_eq!(reply, [5, 0, 0, 1, 127, 0, 0, 1, 8, 33]);
        drop(proxy);
        server.join().unwrap();
    }
    #[test]
    fn shutdown_releases_listener() {
        let proxy = VoiceProxy::start(22082, 2080).unwrap();
        drop(proxy);
        let _listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 22082)).unwrap();
    }
    #[test]
    #[ignore = "requires the user's active local tunnel; unauthenticated voice handshake only"]
    fn live_voice_handshake() {
        let _proxy = VoiceProxy::start(22080, 2080).unwrap();
        let output = std::process::Command::new("curl.exe")
            .args([
                "--proxy",
                "socks5://127.0.0.1:22080",
                "--resolve",
                "c-ams13-56d03f2b.discord.media:2053:10.10.34.36",
                "--http1.1",
                "--connect-timeout",
                "5",
                "--max-time",
                "8",
                "--silent",
                "--output",
                "NUL",
                "--write-out",
                "%{http_code}",
                "-H",
                "Connection: Upgrade",
                "-H",
                "Upgrade: websocket",
                "-H",
                "Sec-WebSocket-Version: 13",
                "-H",
                "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==",
                "https://c-ams13-56d03f2b.discord.media:2053/?v=8",
            ])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout), "101");
    }
}
