# Troubleshooting

Use the newest DisRoute release and extract it into a fresh folder before debugging an older installation. Never post a complete proxy link, `profile.dpapi`, runtime configuration, packet capture, or unredacted log in a public issue.

## Connection does not start

1. Run `DisRoute.exe` as Administrator.
2. Confirm that the `engine` directory is beside the executable.
3. Install the bundled ProxiFyre prerequisite package and reboot Windows if its installer requests it.
4. Check the Windows clock, Firewall, and Antivirus if the optional HTTPS probe cannot complete.
5. Try the same share link in a trusted client. A working TCP connection does not prove UDP support.

## Discord does not reopen

Use **Restart Discord** inside DisRoute. It closes only Discord Stable/PTB/Canary processes and relaunches the installed Start Menu shortcut without stopping the tunnel.

## Voice and streaming

Voice may connect while screen sharing remains unstable because the two workloads have very different bandwidth and packet-loss requirements.

Check these in order:

1. Stop other uploads, cloud sync, torrents, and game updates on the same internet connection.
2. Try a geographically closer proxy server with lower packet loss and enough upstream bandwidth.
3. Confirm that the server supports UDP and the packet encoding carried by the share link. DisRoute recognizes `packetEncoding=xudp`, `packetEncoding=packetaddr`, and `packetEncoding=none`; when absent, XUDP remains the default.
4. Compare a raw TCP/Reality link with WebSocket or gRPC only when the server offers both. Every TCP-based option can suffer head-of-line blocking when the path loses packets.
5. Test the same server and Discord stream through another client. If upload is still poor, the bottleneck is outside DisRoute.

DisRoute does not rate-limit media. ProxiFyre sends the SOCKS5 UDP relay traffic directly to sing-box; the local voice bridge only inspects the initial TLS name for dynamic `*.discord.media` hosts. This means a slow stream is usually caused by the physical upload, server capacity, route quality, or UDP encapsulation rather than the React interface or the SNI bridge.

Relevant upstream documentation:

- [ProxiFyre TCP/UDP routing](https://github.com/wiresock/proxifyre#readme)
- [sing-box VLESS packet encoding](https://sing-box.sagernet.org/configuration/outbound/vless/)

## Games become slower

Generated rules do not add games or browsers to the proxy. They still share the same physical connection, so a Discord stream can saturate upload and increase bufferbloat and game latency. Pause the stream or apply upload QoS/SQM on the router to verify this distinction.

## Preparing a useful bug report

Include:

- DisRoute version and Windows version;
- Discord channel: Stable, PTB, or Canary;
- protocol and transport/security names without host, UUID, passwords, keys, or paths;
- whether text, voice, and streaming fail independently;
- exact visible error text;
- whether the same redacted setup works in another client.

Do not include credentials or identifying network data.
