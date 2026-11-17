# KÇZ Launcher (Tauri) - yeni sürümü derleyip GitHub'a yükler.
# Launcher'ı kuran herkesin sol üstünde "Güncelle" butonu çıkar.
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
$Depo = 'trkcz01/kcz-launcher'
$Utf8 = New-Object Text.UTF8Encoding $false

function Yaz($m) { Write-Host "  $m" }

# ---------- GitHub anahtarı ----------
if (-not (Test-Path 'gh-token.txt')) {
  Yaz 'Bir kerelik: GitHub anahtarı gerekiyor. Açılan sayfada en alttaki yeşil "Generate token" butonuna bas,'
  Yaz '"ghp_..." ile başlayan yazıyı kopyala.'
  Start-Process 'https://github.com/settings/tokens/new?scopes=public_repo&description=KCZ%20Launcher%20yayin'
  $t = Read-Host '  Anahtarı buraya yapıştır ve Enter''a bas'
  if (-not $t) { throw 'Anahtar girilmedi.' }
  [IO.File]::WriteAllText((Join-Path $PSScriptRoot 'gh-token.txt'), $t.Trim(), $Utf8)
}
$Token = (Get-Content 'gh-token.txt' -Raw).Trim()
$Basliklar = @{ Authorization = "Bearer $Token"; Accept = 'application/vnd.github+json'; 'User-Agent' = 'KCZ-Launcher' }

# ---------- Sürüm numarasını bir artır (0.0.1 -> 0.0.2) ----------
$conf = 'src-tauri\tauri.conf.json'
$j = Get-Content $conf -Raw -Encoding UTF8
if ($j -notmatch '"version":\s*"(\d+)\.(\d+)\.(\d+)"') { throw 'Sürüm numarası bulunamadı.' }
$Yeni = "$($Matches[1]).$($Matches[2]).$([int]$Matches[3] + 1)"
$Eski = "$($Matches[1]).$($Matches[2]).$($Matches[3])"
[IO.File]::WriteAllText((Resolve-Path $conf), ($j -replace "`"version`":\s*`"$Eski`"", "`"version`": `"$Yeni`""), $Utf8)
$cargo = 'src-tauri\Cargo.toml'
$c = Get-Content $cargo -Raw -Encoding UTF8
[IO.File]::WriteAllText((Resolve-Path $cargo), ($c -replace "(?m)^version = `"$Eski`"", "version = `"$Yeni`""), $Utf8)
Yaz "Sürüm: $Eski -> $Yeni"

# ---------- Derleme (imzalı) ----------
$env:TAURI_SIGNING_PRIVATE_KEY = Join-Path $PSScriptRoot 'imza\kcz-imza.key'
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = 'kcz-launcher-imza'
if (-not (Test-Path $env:TAURI_SIGNING_PRIVATE_KEY)) { throw 'Once TAURI-KUR.bat calistirilmali (imza anahtari yok).' }
$nsis = 'src-tauri\target\release\bundle\nsis'
if (Test-Path $nsis) { Remove-Item $nsis -Recurse -Force }
# Oyun ici KCZ menusu (KCZ Client) her surum icin derlenip launcher'a gomulur
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'istemci-mod\DERLE.ps1')
& npx.cmd tauri build
if ($LASTEXITCODE -ne 0) { throw 'Derleme başarısız.' }

$exe = Get-ChildItem $nsis -Filter '*-setup.exe' | Select-Object -First 1
$sig = Get-ChildItem $nsis -Filter '*-setup.exe.sig' | Select-Object -First 1
if (-not $exe -or -not $sig) { throw 'Kurulum dosyası ya da imzası bulunamadı.' }
$DosyaAdi = "KCZ-Launcher_${Yeni}_x64-setup.exe"

# ---------- latest.json ----------
$latest = [ordered]@{
  version  = $Yeni
  notes    = "KÇZ Launcher $Yeni"
  pub_date = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
  platforms = @{
    'windows-x86_64' = @{
      signature = (Get-Content $sig.FullName -Raw).Trim()
      url       = "https://github.com/$Depo/releases/download/v$Yeni/$DosyaAdi"
    }
  }
}
$latestYol = Join-Path $PSScriptRoot 'latest.json'
[IO.File]::WriteAllText($latestYol, ($latest | ConvertTo-Json -Depth 5), $Utf8)

# ---------- GitHub'a yükle ----------
Yaz 'GitHub''a yükleniyor...'
$govde = @{ tag_name = "v$Yeni"; name = "KÇZ Launcher $Yeni"; body = "KÇZ Launcher $Yeni"; draft = $false; prerelease = $false } | ConvertTo-Json
$rel = Invoke-RestMethod -Method Post -Uri "https://api.github.com/repos/$Depo/releases" -Headers $Basliklar -Body ([Text.Encoding]::UTF8.GetBytes($govde)) -ContentType 'application/json'
$yukle = $rel.upload_url -replace '\{.*\}', ''
Invoke-RestMethod -Method Post -Uri "${yukle}?name=$DosyaAdi" -Headers $Basliklar -InFile $exe.FullName -ContentType 'application/octet-stream' | Out-Null
Invoke-RestMethod -Method Post -Uri "${yukle}?name=latest.json" -Headers $Basliklar -InFile $latestYol -ContentType 'application/json' | Out-Null

Write-Host ''
Yaz "TAMAM! $Yeni yayınlandı. Açık launcher'larda birkaç dakika içinde sol üstte 'Güncelle' çıkar."
