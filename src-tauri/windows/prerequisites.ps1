$ErrorActionPreference = 'Stop'
# Read-only native x64 checks. Never use Win32_Product (it triggers MSI repairs).
function File-Version([string]$Path) {
    if (!(Test-Path -LiteralPath $Path)) { return $null }
    $info = [Diagnostics.FileVersionInfo]::GetVersionInfo($Path)
    return [version]::new($info.FileMajorPart, $info.FileMinorPart, $info.FileBuildPart, $info.FilePrivatePart)
}
$system = Join-Path $env:SystemRoot 'System32'
$driver = File-Version (Join-Path $system 'drivers\ndisrd.sys')
$registered = Test-Path -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Services\NDISRD'
$vc = $true
foreach ($file in 'msvcp140.dll','msvcp140_atomic_wait.dll','vcruntime140.dll','vcruntime140_1.dll') {
    $version = File-Version (Join-Path $system $file)
    if (!$version -or $version -lt [version]'14.44.35211.0') { $vc = $false }
}
$ucrt = File-Version (Join-Path $system 'ucrtbase.dll')
if (!$ucrt -or $ucrt -lt [version]'10.0.10240.0') { $vc = $false }
$framework = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full' -ErrorAction SilentlyContinue
$webviewPaths = @(
    'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
    'HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
)
$webview = $false
foreach ($path in $webviewPaths) {
    $item = Get-ItemProperty -LiteralPath $path -ErrorAction SilentlyContinue
    if ($item.pv -and $item.pv -ne '0.0.0.0') { $webview = $true }
}
[ordered]@{
    packetFilter = [bool]($registered -and $driver -and $driver -ge [version]'3.6.1.0' -and $driver -lt [version]'4.0.0.0')
    visualCpp = $vc
    dotNet = [bool]($framework.Release -ge 461808)
    webview = $webview
    pendingReboot = [bool]((Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending') -or (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired'))
} | ConvertTo-Json -Compress
