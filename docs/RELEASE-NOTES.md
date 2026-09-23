# DisRoute 0.2.6 — Windows preview

- Replaced the ambiguous one-way route markers with a clear bidirectional Discord ↔ Proxy ↔ Internet path.
- Added VLESS share-link support for XUDP, `packetaddr`, and explicitly disabled packet encoding instead of forcing XUDP for every server.
- Reduced background ProxiFyre logging and documented the real upload, UDP, and TCP head-of-line limits that affect Discord streaming.
- Rebuilt the GitHub landing page and contributor documentation with current architecture, security boundaries, troubleshooting, changelog, and issue templates.
- Retained the safer connection probe from 0.2.5: a valid SOCKS/VLESS tunnel is no longer stopped by redirects, WAF responses, or an inconclusive optional HTTPS check.

Download the Windows x64 ZIP and read START-HERE-FA.txt. Each person supplies their own VLESS server. This unsigned preview is not a privacy or leak-prevention guarantee. Stream quality depends on the physical upload path, server capacity, packet loss, and working UDP support.
