param([string]$Executable = '', [string]$DownloadCache = '')
$ErrorActionPreference = 'Stop'
$project = Split-Path $PSScriptRoot -Parent
$version = (Get-Content (Join-Path $project 'package.json') -Raw | ConvertFrom-Json).version
if (!$Executable) { $Executable = Join-Path $project 'src-tauri/target/release/disroute.exe' }
if (!(Test-Path -LiteralPath $Executable)) { throw 'Build the portable executable first.' }
if (!$DownloadCache) { $DownloadCache = Join-Path $project 'work/downloads' }
New-Item -ItemType Directory -Force -Path $DownloadCache | Out-Null
$stage = Join-Path ([IO.Path]::GetTempPath()) ('disroute-package-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $stage | Out-Null
$bundle = Join-Path $stage "DisRoute-$version-windows-x64"
$engine = Join-Path $bundle 'engine'
$licenses = Join-Path $bundle 'licenses'
New-Item -ItemType Directory -Path $engine,$licenses | Out-Null
function Get-VerifiedAsset($Name, $Url, $Hash) {
    $path = Join-Path $DownloadCache $Name
    if (!(Test-Path -LiteralPath $path)) { Invoke-WebRequest -Uri $Url -OutFile $path }
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $Hash) { throw "Checksum mismatch: $Name" }
    return $path
}
$proxy = Get-VerifiedAsset 'ProxiFyre-v2.6.1-x64.zip' 'https://github.com/wiresock/proxifyre/releases/download/v2.6.1/ProxiFyre-v2.6.1-x64.zip' '86B81504D49194E002ACAE5C4DE7E2E84A79B087CBE1669255BB510E4072ABEF'
$sing = Get-VerifiedAsset 'sing-box-1.14.0-windows-amd64.zip' 'https://github.com/SagerNet/sing-box/releases/download/v1.14.0/sing-box-1.14.0-windows-amd64.zip' '3FFB56267DA14E287BE48BD10CF7E6505260125BAD940B75101FBB4D5D58E5D6'
$setup = Get-VerifiedAsset 'ProxiFyre-2.6.1-win-x64-setup.exe' 'https://github.com/wiresock/proxifyre/releases/download/v2.6.1/ProxiFyre-2.6.1-win-x64-setup.exe' 'C08CBB5C15ACD04D77D7C330712AE366D2A9A8E2A290586E8FE9AA73C78A1908'
Expand-Archive -LiteralPath $proxy -DestinationPath (Join-Path $stage 'proxy')
Expand-Archive -LiteralPath $sing -DestinationPath (Join-Path $stage 'sing')
# Only pristine verified engine archives are used; never copy a running engine directory.
Get-ChildItem (Join-Path $stage 'proxy') -File | Where-Object { $_.Extension -in '.exe','.dll','.config' } | Copy-Item -Destination $engine
Copy-Item (Join-Path $stage 'sing/sing-box-1.14.0-windows-amd64/sing-box.exe') $engine
Copy-Item (Join-Path $stage 'sing/sing-box-1.14.0-windows-amd64/LICENSE') (Join-Path $licenses 'sing-box-GPL-3.0.txt')
Copy-Item (Join-Path $project 'LICENSE') (Join-Path $licenses 'DisRoute-AGPL-3.0.txt')
Copy-Item (Join-Path $project 'LICENSE') (Join-Path $licenses 'ProxiFyre-AGPL-3.0.txt')
Copy-Item (Join-Path $project 'THIRD_PARTY_NOTICES.md') $licenses
Copy-Item -LiteralPath $Executable -Destination (Join-Path $bundle 'DisRoute.exe')
Copy-Item -LiteralPath $setup -Destination $bundle
Copy-Item (Join-Path $project 'docs/START-HERE-FA.txt') $bundle
$outputs = Join-Path $project 'outputs'
New-Item -ItemType Directory -Force -Path $outputs | Out-Null
$zip = Join-Path $outputs "DisRoute-$version-windows-x64.zip"
if (Test-Path -LiteralPath $zip) { throw 'Release archive already exists; choose a new version or move the previous archive.' }
Compress-Archive -LiteralPath $bundle -DestinationPath $zip
$hash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$zip.sha256", "$hash  $([IO.Path]::GetFileName($zip))`n")
Write-Output "Created $zip"
Write-Output "SHA256 $hash"
# Leave unique staging directory available for inspection; no recursive deletion.
