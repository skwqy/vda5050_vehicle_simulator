# Build release binary and copy runtime files into dist/vda5050_vehicle_simulator/
$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

Write-Host "cargo build --release"
cargo build --release

$exeName = "vda5050_vehicle_simulator.exe"
$exe = Join-Path $root "target\release\$exeName"
if (-not (Test-Path $exe)) {
    throw "Expected output not found: $exe"
}

$out = Join-Path $root "dist\vda5050_vehicle_simulator"
New-Item -ItemType Directory -Force -Path $out | Out-Null

Copy-Item $exe $out -Force
Copy-Item (Join-Path $root "config.toml") $out -Force

$mapsSrc = Join-Path $root "maps"
if (Test-Path $mapsSrc) {
    Copy-Item $mapsSrc (Join-Path $out "maps") -Recurse -Force
    Write-Host "Copied maps\ -> $out\maps\"
} else {
    Write-Warning "maps folder not found at $mapsSrc - skipped (add maps for [map].xml_path)"
}

Write-Host ""
Write-Host "Done. Copy this folder elsewhere and run $exeName"
Write-Host $out
