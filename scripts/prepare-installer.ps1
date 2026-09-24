param([string]$DownloadCache = '')
$ErrorActionPreference = 'Stop'

$project = (Resolve-Path (Split-Path $PSScriptRoot -Parent)).Path
$resourceRoot = Join-Path $project 'src-tauri\resources'
$resourceParent = (Resolve-Path (Join-Path $project 'src-tauri')).Path
if (![IO.Path]::GetFullPath($resourceRoot).StartsWith($resourceParent + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Refusing to prepare resources outside src-tauri.'
}
if (!$DownloadCache) { $DownloadCache = Join-Path $project 'work\downloads' }
New-Item -ItemType Directory -Force -Path $DownloadCache | Out-Null

function Get-VerifiedAsset($Name, $Url, $Hash) {
    $path = Join-Path $DownloadCache $Name
    if (!(Test-Path -LiteralPath $path)) { Invoke-WebRequest -Uri $Url -OutFile $path }
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $Hash) {
        throw "Checksum mismatch: $Name"
    }
    return $path
}

$proxy = Get-VerifiedAsset 'ProxiFyre-v2.6.1-x64.zip' 'https://github.com/wiresock/proxifyre/releases/download/v2.6.1/ProxiFyre-v2.6.1-x64.zip' '86B81504D49194E002ACAE5C4DE7E2E84A79B087CBE1669255BB510E4072ABEF'
$sing = Get-VerifiedAsset 'sing-box-1.14.0-windows-amd64.zip' 'https://github.com/SagerNet/sing-box/releases/download/v1.14.0/sing-box-1.14.0-windows-amd64.zip' '3FFB56267DA14E287BE48BD10CF7E6505260125BAD940B75101FBB4D5D58E5D6'

if (Test-Path -LiteralPath $resourceRoot) { Remove-Item -LiteralPath $resourceRoot -Recurse -Force }
$engine = New-Item -ItemType Directory -Force -Path (Join-Path $resourceRoot 'engine')
$licenses = New-Item -ItemType Directory -Force -Path (Join-Path $resourceRoot 'licenses')
$stage = Join-Path ([IO.Path]::GetTempPath()) ('disroute-installer-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $stage | Out-Null

Expand-Archive -LiteralPath $proxy -DestinationPath (Join-Path $stage 'proxy')
Expand-Archive -LiteralPath $sing -DestinationPath (Join-Path $stage 'sing')
Get-ChildItem (Join-Path $stage 'proxy') -File |
    Where-Object { $_.Extension -in '.exe', '.dll', '.config' } |
    Copy-Item -Destination $engine.FullName
Copy-Item (Join-Path $stage 'sing\sing-box-1.14.0-windows-amd64\sing-box.exe') $engine.FullName
Copy-Item (Join-Path $stage 'sing\sing-box-1.14.0-windows-amd64\LICENSE') (Join-Path $licenses.FullName 'sing-box-GPL-3.0.txt')
Copy-Item (Join-Path $project 'LICENSE') (Join-Path $licenses.FullName 'DisRoute-AGPL-3.0.txt')
Copy-Item (Join-Path $project 'LICENSE') (Join-Path $licenses.FullName 'ProxiFyre-AGPL-3.0.txt')
Copy-Item (Join-Path $project 'THIRD_PARTY_NOTICES.md') $licenses.FullName

Write-Output "Prepared verified installer resources in $resourceRoot"
