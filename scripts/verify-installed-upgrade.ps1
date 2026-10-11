param([Parameter(Mandatory)][string]$Installer)
$ErrorActionPreference = 'Stop'
# Installers modify shortcuts and the uninstall registry. Never run on a user's PC.
if ($env:GITHUB_ACTIONS -ne 'true' -or !$env:RUNNER_TEMP) {
    throw 'This installation test is restricted to disposable GitHub Actions Windows runners.'
}
$project = (Resolve-Path (Split-Path $PSScriptRoot -Parent)).Path
$Installer = (Resolve-Path -LiteralPath $Installer).Path
$stage = Join-Path $env:RUNNER_TEMP ('disroute-upgrade-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $stage | Out-Null
$destination = Join-Path $stage 'installed app'
$oldInstaller = Join-Path $stage 'previous.exe'
Invoke-WebRequest 'https://github.com/AlirezaZexter/disroute/releases/download/v0.5.9/DisRoute-0.5.9-windows-x64-setup.exe' -OutFile $oldInstaller -TimeoutSec 120
if ((Get-FileHash $oldInstaller -Algorithm SHA256).Hash -ne '67A00ED8C49975AC6F35DA96C1B5A69E161DF869D8F790D8816BE6D6714671B9') {
    throw 'Previous public installer failed pinned checksum verification.'
}
function Install-Package([string]$Path) {
    # NSIS /D must be last and unquoted, including when it contains spaces.
    $process = Start-Process -FilePath $Path -ArgumentList "/S /D=$destination" -WindowStyle Hidden -PassThru
    if (!$process.WaitForExit(180000)) { throw 'Installer timed out; discard this runner.' }
    if ($process.ExitCode -ne 0) { throw "Installer failed with code $($process.ExitCode)." }
    if (!(Test-Path -LiteralPath (Join-Path $destination 'DisRoute.exe'))) { throw 'Installed executable missing.' }
}
Install-Package $oldInstaller
$data = Join-Path $env:LOCALAPPDATA 'app.disroute.desktop'
if (Test-Path -LiteralPath $data) { throw 'Refusing to overwrite existing test account data.' }
New-Item -ItemType Directory -Path $data | Out-Null
Add-Type -AssemblyName System.Security
$plain = [Text.Encoding]::UTF8.GetBytes('{"name":"synthetic upgrade test","configLink":"trojan://synthetic@example.com:443"}')
$encrypted = [Security.Cryptography.ProtectedData]::Protect($plain, $null, [Security.Cryptography.DataProtectionScope]::CurrentUser)
$profile = Join-Path $data 'profile.dpapi'
[IO.File]::WriteAllBytes($profile, $encrypted)
$before = (Get-FileHash $profile -Algorithm SHA256).Hash
Install-Package $Installer
if ((Get-FileHash $profile -Algorithm SHA256).Hash -ne $before) { throw 'Upgrade changed encrypted profile data.' }
$restored = [Security.Cryptography.ProtectedData]::Unprotect([IO.File]::ReadAllBytes($profile), $null, [Security.Cryptography.DataProtectionScope]::CurrentUser)
if ([Text.Encoding]::UTF8.GetString($restored) -ne [Text.Encoding]::UTF8.GetString($plain)) { throw 'Profile no longer decrypts.' }
[Array]::Clear($plain, 0, $plain.Length)
[Array]::Clear($restored, 0, $restored.Length)
foreach ($name in 'ProxiFyre.exe','sing-box.exe','socksify.dll') {
    $installed = Join-Path $destination "resources/engine/$name"
    $expected = Join-Path $project "src-tauri/resources/engine/$name"
    if (!(Test-Path $installed) -or (Get-FileHash $installed).Hash -ne (Get-FileHash $expected).Hash) { throw "Engine payload mismatch: $name" }
}
$bootstrapper = 'resources/prerequisites/ProxiFyre-2.6.1-win-x64-setup.exe'
if ((Get-FileHash (Join-Path $destination $bootstrapper) -Algorithm SHA256).Hash -ne 'C08CBB5C15ACD04D77D7C330712AE366D2A9A8E2A290586E8FE9AA73C78A1908') { throw 'Prerequisite bootstrapper missing or changed.' }
Write-Output 'PASS: real NSIS install/reinstall, packaged engines/bootstrapper, encrypted profile preservation and decryption.'
Write-Output 'NOT TESTED: clean OS, driver installation, UAC interaction, native updater relaunch, Discord sessions.'
