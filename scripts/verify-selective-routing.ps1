param([string]$RouterConfig = '', [string]$TunnelConfig = '')
$ErrorActionPreference = 'Stop'
# Read-only diagnostic. Output contains no endpoints, credentials or user paths.
if (!$RouterConfig) {
    $process = Get-CimInstance Win32_Process -Filter "Name='ProxiFyre.exe'" | Where-Object ExecutablePath | Select-Object -First 1
    if (!$process) { throw 'Connect DisRoute first, then run this read-only check as Administrator.' }
    $RouterConfig = Join-Path (Split-Path $process.ExecutablePath -Parent) 'app-config.json'
}
if (!$TunnelConfig) { $TunnelConfig = Join-Path $env:LOCALAPPDATA 'app.disroute.desktop/runtime/sing-box.json' }
$router = Get-Content -LiteralPath $RouterConfig -Raw | ConvertFrom-Json
$tunnel = Get-Content -LiteralPath $TunnelConfig -Raw | ConvertFrom-Json
$expected = @('Discord.exe', 'DiscordCanary.exe', 'DiscordPTB.exe', '\Discord\Update.exe')
$actual = @($router.proxies[0].appNames)
if (@($router.proxies).Count -ne 1 -or (Compare-Object $expected $actual)) { throw 'Unexpected application routing rule.' }
if ($router.proxies[0].socks5ProxyEndpoint -ne '127.0.0.1:2080') { throw 'Unexpected router endpoint.' }
if (@($tunnel.inbounds).Count -ne 1 -or $tunnel.inbounds[0].type -ne 'mixed' -or $tunnel.inbounds[0].listen -ne '127.0.0.1' -or $tunnel.inbounds[0].set_system_proxy -ne $false -or $tunnel.dns) { throw 'Unexpected system proxy, DNS or TUN configuration.' }
$internet = Get-ItemProperty 'HKCU:/Software/Microsoft/Windows/CurrentVersion/Internet Settings' -ErrorAction SilentlyContinue
[ordered]@{
    discordOnlyApplicationList = $true
    loopbackOnly = $true
    systemProxyRequestedByTunnel = $false
    tunInbound = $false
    dnsOverride = $false
    windowsUserProxyCurrentlyEnabled = [bool]$internet.ProxyEnable
    limitation = 'Configuration inspection only; not a packet capture or a before/after OS route comparison.'
} | ConvertTo-Json
