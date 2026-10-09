# Live connection health

The 0.5.8 panel separates a running local engine from a successful Discord HTTPS
request. It does not measure ICMP ping, throughput, packet loss, or voice quality.

## Measurement and privacy

- Both personal and community sessions use the existing certificate-verified
  `https://discord.com/api/v10/gateway` probe through local SOCKS port 2080.
- System/environment proxy discovery and redirects are disabled. There is no
  direct fallback, no route mutation and no new telemetry service.
- The background loop waits 15 seconds between iterations. A probe has an
  8-second total timeout (5 seconds for connection). Setup and scans serialize
  with probes, so this is not an exact 15-second sampling schedule.
- A manual check uses the same path and operation lock. The last 24 results,
  session start time and recovery count are held in memory only. No credentials,
  endpoint details or browsing data are recorded in this monitor.
- Session duration is elapsed time since connection, not guaranteed uptime.
  Successful checks are a count, not a calculated reliability guarantee.
- Results older than 45 seconds are not displayed as current success. The UI
  polls even while the engine is disconnected and does not overlap slow polls.

## Community recovery

Three consecutive failed checks are required before trying a replacement, only
when the community warning was acknowledged, automatic failover is enabled and
the existing candidate queue has alternatives. Personal connections never switch
to a community endpoint automatically.

The existing isolated preflight remains in place. A candidate must pass before
the existing engine switch/rollback path runs. Failed attempts use monotonic
cooldowns of 15, 30, 60 and up to 120 seconds; a successful actual probe resets
the failure streak. An explicit disconnect invalidates pending probe results and
the failover generation so an old preflight cannot reconnect the user.

Neither preserved TCP/voice sessions nor recovery with an exhausted candidate
queue is promised. Switching may require reconnecting Discord voice. ProxiFyre's
Discord-only process list, sing-box routing, DNS, Firewall rules, profile format,
source catalog and updater implementation are unchanged.

## Verification

`npm run check` covers the monitor state machine, failure threshold/backoff,
old-result invalidation, personal-mode exclusion, bounded history, stale UI,
manual checks and non-overlapping polling after disconnection. The existing
route-generation and encrypted-profile regression tests remain part of the gate.

The optional `isolated_engine_authentication_and_cleanup` integration check
exercises real sing-box with local Shadowsocks and HTTP marker endpoints. It is
not a live Discord HTTPS or voice test. See `TESTING.md` for release-specific
results and remaining manual checks.
