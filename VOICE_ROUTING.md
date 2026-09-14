# Voice DNS recovery

Path: Discord processes → ProxiFyre → loopback SOCKS bridge :2080 → sing-box :2081 → VLESS.

The ISP returned a block-page IP for dynamic `*.discord.media` hosts. HTTPS messages worked with fixed-name overrides, but voice WSS failed before UDP negotiation. An unauthenticated probe failed with local DNS and returned HTTP 101 with remote SOCKS DNS.

The bridge recovers validated `*.discord.media` SNI from IP-addressed TLS on ports 443, 2053, 2083, 2087, 2096 and 8443. It handles fragmented ClientHello records with a 64 KiB cap. It does not terminate TLS, alter certificates, log payloads, or modify system DNS. Other destinations retain their original address. Domain-addressed CONNECT and UDP ASSOCIATE are forwarded; UDP packets use the sing-box relay endpoint, not the bridge. SOCKS BIND/authentication/HTTP-proxy mode are not supported. At most 128 active control/TCP connections are accepted.

SNI inspection requires an early SOCKS success response for IP-addressed TLS; upstream failure then closes that stream. Consequently, SOCKS success is not a connectivity guarantee. Existing application readiness probes still verify actual Discord HTTPS responses. The bridge is stopped and its sockets shut down on disconnect, startup failure or engine shutdown.

Limitations: encrypted ClientHello, new voice TLS ports, missing SNI and a VLESS server without UDP/XUDP support need separate handling. The live WSS probe does not establish an authenticated voice call or prove bidirectional audio.

Tests: `cargo test --manifest-path src-tauri/Cargo.toml --features custom-protocol`. The ignored `live_voice_handshake` test explicitly requires an existing tunnel on :2080 and uses a temporary bridge on :22080; it sends no Discord account credentials. Its test endpoint may expire. Do not enable ignored tests in generic CI.
