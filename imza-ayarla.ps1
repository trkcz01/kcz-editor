# Guncelleme imzasinin acik anahtarini src-tauri\tauri.conf.json'a yazar.
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
$p = (Get-Content 'imza\kcz-imza.key.pub' -Raw).Trim()
$f = 'src-tauri\tauri.conf.json'
$t = Get-Content $f -Raw -Encoding UTF8
$t = [regex]::Replace($t, '"pubkey"\s*:\s*"[^"]*"', ('"pubkey": "' + $p + '"'))
[IO.File]::WriteAllText((Resolve-Path $f), $t, (New-Object Text.UTF8Encoding $false))
Write-Host '       Guncelleme imzasi hazir.'
