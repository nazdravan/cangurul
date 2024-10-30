# Arhiva portabila pentru Windows 10/11:
#   dist/release/cangurul-<versiune>-windows-<x64|x86>-portable.zip
# Ruleaza pe Windows cu Rust (MSVC). Pentru Windows 7 vezi packaging/windows7.
param(
    [Parameter(Mandatory)]
    [ValidateSet('x64', 'x86')]
    [string]$Arch
)
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..\..')

$target = @{ x64 = 'x86_64-pc-windows-msvc'; x86 = 'i686-pc-windows-msvc' }[$Arch]
# Singurul loc al versiunii: [package] version din Cargo.toml (primul `version =`).
$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value

cargo build --release --locked --target $target
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$name = "cangurul-$version-windows-$Arch-portable"
$stage = "dist/stage/$name"
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force "$stage/Exemple", 'dist/release' | Out-Null
Copy-Item "target/$target/release/cangurul.exe" "$stage/Cangurul.exe"
Copy-Item examples/*.cng "$stage/Exemple/"
Get-Content assets/license-header.txt, assets/fonts/LICENSE.txt | Set-Content -Encoding utf8 "$stage/LICENTE.txt"
Copy-Item LICENSE "$stage/LICENSE.txt"
Compress-Archive -Path $stage -DestinationPath "dist/release/$name.zip" -Force
Write-Host "Gata: dist/release/$name.zip"
