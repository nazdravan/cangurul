# Instalatorul pentru Windows 10/11:
#   dist/release/cangurul-<versiune>-windows-<x64|x86>.msi
# Se ruleaza dupa package.ps1 (foloseste dosarul pregatit de acesta). Cere WiX
# Toolset v5: dotnet tool install --global wix --version 5.0.2
# Extensia UI (ferestrele instalatorului) se adauga singura, o data pe calculator.
param(
    [Parameter(Mandatory)]
    [ValidateSet('x64', 'x86')]
    [string]$Arch
)
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..\..')

$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
# MSI accepta doar versiuni numerice: 3.1.0-rc.1 -> 3.1.0.
$msiVersion = $version -replace '-.*$', ''

$stage = "dist/stage/cangurul-$version-windows-$Arch-portable"
if (-not (Test-Path "$stage/Cangurul.exe")) {
    throw "Lipseste ${stage}: rulati intai packaging/windows/package.ps1 -Arch $Arch"
}
$out = "dist/release/cangurul-$version-windows-$Arch.msi"
wix extension add --global WixToolset.UI.wixext/5.0.2
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
wix build packaging/windows/cangurul.wxs -arch $Arch -pdbtype none `
    -ext WixToolset.UI.wixext -culture ro-RO -loc packaging/windows/cangurul.ro-RO.wxl `
    -d "Version=$msiVersion" -d "Stage=$((Resolve-Path $stage).Path)" `
    -d "Icon=$((Resolve-Path 'assets/cangurul.ico').Path)" -o $out
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "Gata: $out"
