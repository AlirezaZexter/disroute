# Security policy

## Supported versions

DisRoute is preview software. Security fixes are made only on the latest release and the `main` branch.

## Reporting a vulnerability

Do not open a public issue for credential exposure, privilege escalation, update compromise, traffic leaks, or unsafe routing behavior. Use GitHub's private vulnerability reporting form when it is available in the repository Security tab.

If no private reporting form is available, do not publish exploit details or credentials. Open a minimal public issue asking the maintainer to provide a private contact channel, without including technical reproduction details.

Never attach:

- a complete VLESS URI, UUID, Reality key, short ID, or server address;
- `profile.dpapi` or `sing-box.json`;
- unredacted ProxiFyre/sing-box logs;
- packet captures containing user traffic;
- account, machine, or public-IP identifiers.

## Scope notes

High-value reports include process-rule escapes, unexpected non-Discord routing, local secret exposure, unsafe update or packaging behavior, certificate-validation bypass, and privilege-boundary mistakes. General server availability and third-party VLESS provider performance are not security vulnerabilities.
