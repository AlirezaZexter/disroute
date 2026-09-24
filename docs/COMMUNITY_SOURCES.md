# Community Quick Connect sources

DisRoute is a client, not a proxy service. A source publishes an authorized list; the listed endpoints remain operated by their respective third parties.

## Default source registry

[`community-sources.json`](community-sources.json) seeds the first-run registry with [Radikal Top 100](https://github.com/0xRadikal/Free-v2ray-Configs) and its jsDelivr mirror. The source operator publishes this subscription for client use under [MIT](https://github.com/0xRadikal/Free-v2ray-Configs/blob/main/LICENSE). Reviewed 2026-09-25. Radikal aggregates public third-party endpoints; its license and source-side health labels are not evidence of server ownership, privacy or availability on the user's network. The mirror is the same pool, not an independent provider.

The registry also includes the Germany and Netherlands subscriptions from [Au1rxx/free-vpn-subscriptions](https://github.com/Au1rxx/free-vpn-subscriptions), published under [MIT](https://github.com/Au1rxx/free-vpn-subscriptions/blob/main/LICENSE). These are independently maintained feeds, not DisRoute-operated servers. Country names describe the upstream list, not independently verified exit geolocation.

No credentials are embedded. Users can edit, disable or delete sources without rebuilding. Deleted defaults are not re-added after initialization. No background fetch occurs until the warning is acknowledged. One-click connect refreshes sources (or uses stale cached data), runs local tests and tries the ranked working candidates. A failed preflight preserves the existing engine; switching can reset Discord voice/TCP sessions.

## Supported providers

- Version 1 JSON manifests over HTTPS, including GitHub Raw URLs and GitHub release assets
- Standard newline or Base64 subscription URLs
- Local version 1 manifest files
- User-added HTTPS URLs

Each source has its own enable switch, refresh interval, timeout, attribution, last refresh, error state, and optional expected SHA-256. The source registry and last-known-good cache are protected with Windows DPAPI and can be cleared from the UI.

## Manifest example

```json
{
  "version": 1,
  "id": "example-community",
  "generatedAt": 1789560000,
  "source": { "name": "Example operator", "url": "https://example.org/disroute" },
  "configurations": [
    {
      "id": "de-01",
      "uri": "vless://REDACTED@example.org:443?security=tls",
      "protocol": "vless",
      "country": "DE",
      "supportsUdp": true,
      "addedAt": 1789560000,
      "expiresAt": 1792152000
    }
  ]
}
```

The canonical schema is [`community-manifest.schema.json`](community-manifest.schema.json). Unknown versions, unsupported protocols, oversized input, expired entries, malformed credentials, unsafe fields, and private/local endpoints are rejected. DNS answers are checked again before a health test to mitigate hostnames resolving to private addresses.

DisRoute never passes a downloaded document to sing-box. It parses each URI and generates a constrained internal configuration with a loopback-only SOCKS listener and no system proxy. Health checks use isolated processes and unique ports, perform certificate-verified Discord HTTPS requests through a mandatory SOCKS proxy using Rustls, and separately try UDP via SOCKS5 UDP Associate. UDP reachability does not certify an actual Discord voice session or stream quality. Temporary files and processes are removed after every test. Subscription parsing skips invalid/unsupported entries and deduplicates the generated outbound configuration; a wholly invalid refresh preserves the last-known-good cache.

Scans use four workers and consider at most 100 candidates. The 180-second scan deadline stops new candidate tests; an in-flight bounded request may finish afterward. Cancellation is checked between operations. No latency or availability guarantee is made.

Only publish or add sources whose operator explicitly permits redistribution. Never commit live credentials to this repository or its tests.
