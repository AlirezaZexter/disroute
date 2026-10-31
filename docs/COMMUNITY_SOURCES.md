# Community Quick Connect sources

DisRoute is a client, not a proxy service. A source publishes an authorized list; the listed endpoints remain operated by their respective third parties.

## Default source registry

[`community-sources.json`](community-sources.json) seeds the first-run registry with [Radikal Top 100](https://github.com/0xRadikal/Free-v2ray-Configs) and its jsDelivr mirror. The source operator publishes this subscription for client use under [MIT](https://github.com/0xRadikal/Free-v2ray-Configs/blob/main/LICENSE). Reviewed 2026-09-25. Radikal aggregates public third-party endpoints; its license and source-side health labels are not evidence of server ownership, privacy or availability on the user's network. The mirror is the same pool, not an independent provider.

The registry also includes Germany, Netherlands, France, Turkey, UAE and the first stability shard from [Au1rxx/free-vpn-subscriptions](https://github.com/Au1rxx/free-vpn-subscriptions), published under [MIT](https://github.com/Au1rxx/free-vpn-subscriptions/blob/main/LICENSE). Reviewed 2026-09-26: its README explicitly describes publicly shared volunteer subscriptions and provides client subscription URLs; all added paths were checked live. These are third-party feeds, not DisRoute-operated servers. Country names describe the upstream list, not independently verified exit geolocation. No new unverified bulk scraper was added merely for having more entries.

Reviewed 2026-10-31: the registry additionally includes [PulseConfigs verified](https://github.com/drmikecrypto/PulseConfigs) ([MIT license](https://github.com/drmikecrypto/PulseConfigs/blob/main/LICENSE)) and [Radikal Fast](https://github.com/0xRadikal/Free-v2ray-Configs/tree/main/fast). Their maintainers publish subscriptions for compatible clients. Radikal Fast broadens the same operator's pool; it is not an independent provider. PulseConfigs states that its links are publicly shared third-party endpoints; neither the list license nor source-side tests establish operator trust or availability from Iran. Both URLs passed the production parser and provided working Discord HTTPS paths during the local review. Only DisRoute's existing VLESS/VMess/Trojan/Shadowsocks support is used. No downloaded code is executed, and no proxy credentials are committed.

Users can edit, disable or delete sources without rebuilding. Catalog version 3 adds only the two new feeds once; an earlier installation also receives any missed version-2 additions. Existing edited/disabled/deleted sources remain unchanged. No background fetch occurs until the warning is acknowledged. One-click connect reuses a fresh cache, otherwise refreshes sources (or uses stale cached data), runs local tests and tries the ranked working candidates. Refresh uses up to four concurrent requests; scheduled refresh only fetches due sources. A failed preflight preserves the existing engine; switching can reset Discord voice/TCP sessions.

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

Scans use four workers and consider at most 100 candidates, interleaving sources before the 500-entry cache cap and scan selection. Recent locally measured successes are tried first within each source; country is only a hint. A working result requires at least two successful Discord HTTPS samples, not a single lucky response. Early completion requires zero failed samples, median HTTPS latency at most 700 ms and jitter at most 150 ms: three such UDP-capable results end scheduling early; after 20 seconds, four such HTTPS results also suffice. Slower working candidates remain eligible when no faster option is found but no longer end scheduling prematurely. The 60-second scheduling deadline stops new candidate tests; in-flight bounded operations may finish afterward. Cancellation is checked between operations, and engine configuration validation has a three-second limit. An initial failed HTTPS sample is rejected promptly. Rankings still include measured median latency, jitter, failure rate and UDP, not ICMP or upstream ping claims. The result is the best tested candidate, not a promise to find the globally fastest endpoint. Local scan history is encrypted alongside the cache and removed by Clear.

Local observations on 2026-10-31 (not a controlled benchmark): the old catalog/policy found 4 working paths in 31.46 seconds with a best-ranked median of 809 ms. Two runs with the revised catalog/policy found 14 and 9 working paths in 41.46 and 25.92 seconds; the best-ranked candidate measured 526 ms in both. The second run included PulseConfigs (one working path, 398 ms) and Radikal Fast (two working paths, lowest median 694 ms). The fastest measured path need not rank first when jitter, failures, UDP and local history differ. Random scan order, changing public endpoints and this one network limit the comparison. These observations do not establish throughput, Discord call quality or lasting performance.

On 2026-09-26, a live local run fetched eight feeds in 1.66 seconds, tested 68 candidates, and found three working UDP-capable candidates in 39.16 seconds total. The best median HTTPS connection time was 538 ms. This is a single-network observation, not a speed, voice-quality or long-term stability guarantee.

Only publish or add sources whose operator explicitly permits redistribution. Never commit live credentials to this repository or its tests.
