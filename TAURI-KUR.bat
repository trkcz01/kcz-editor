@echo off
chcp 65001 >nul
title KCZ Launcher (Tauri) - Kurulum
cd /d "%~dp0"
echo.
echo  ==========================================
echo        KCZ LAUNCHER (TAURI) KURULUMU
echo  ==========================================
echo.
echo  Ilk seferde gerekli araclar kurulur (Rust ve Visual Studio C++ araclari).
echo  Bu bir kere olur, 10-30 dakika surebilir. Sonraki seferlerde cok daha hizli.
echo.

REM ---------- Node.js ----------
where node >nul 2>nul
if errorlevel 1 (
  echo  [!] Node.js bulunamadi. nodejs.org adresinden LTS surumunu kurup tekrar calistir.
  pause
  exit /b 1
)

REM ---------- Rust ----------
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>nul
if errorlevel 1 (
  echo  [1/5] Rust kuruluyor...
  winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements
  set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)
where cargo >nul 2>nul
if errorlevel 1 (
  echo  [!] Rust kurulamadi. https://rustup.rs adresinden kurup bu dosyayi tekrar calistir.
  pause
  exit /b 1
)
call rustup default stable-msvc >nul 2>nul
echo  [1/5] Rust hazir.

REM ---------- Visual Studio C++ araclari ----------
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "VC_VAR="
if exist "%VSWHERE%" (
  for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VC_VAR=%%i"
)
if not defined VC_VAR (
  echo  [2/5] Visual Studio C++ araclari kuruluyor ^(birkac GB, biraz surer^)...
  winget install --id Microsoft.VisualStudio.2022.BuildTools -e --accept-source-agreements --accept-package-agreements --override "--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
)
echo  [2/5] C++ araclari hazir.

REM ---------- Tauri araci ----------
echo  [3/5] Tauri araci hazirlaniyor...
call npm install
if errorlevel 1 goto hata

REM ---------- Guncelleme imzasi (bir kere olusturulur) ----------
REM Sifreli anahtar: Windows bos sifreyi aktaramadigi icin sabit bir sifre kullanilir.
set "TAURI_SIGNING_PRIVATE_KEY_PASSWORD=kcz-launcher-imza"
if not exist imza\kcz-imza.key (
  echo        Guncelleme imza anahtari olusturuluyor ^(bir kere^)...
  if not exist imza mkdir imza
  call npx.cmd tauri signer generate -w imza\kcz-imza.key -p "%TAURI_SIGNING_PRIVATE_KEY_PASSWORD%" --ci -f
  if errorlevel 1 goto hata
)
REM Acik anahtari tauri.conf.json'a yaz (her seferinde, anahtar degistiyse de dogru olsun)
powershell -NoProfile -ExecutionPolicy Bypass -File "imza-ayarla.ps1"
if errorlevel 1 goto hata

REM ---------- KCZ Client (oyun ici menu) ----------
echo  [4/5] KCZ Client ^(oyun ici menu^) derleniyor ^(ilk seferde 5-10 dakika^)...
powershell -NoProfile -ExecutionPolicy Bypass -File "istemci-mod\DERLE.ps1"

REM ---------- Derleme ----------
echo  [5/5] Launcher derleniyor ^(ilk seferde 5-15 dakika^)...
set "TAURI_SIGNING_PRIVATE_KEY=%~dp0imza\kcz-imza.key"
if exist src-tauri\target\release\bundle\nsis rmdir /s /q src-tauri\target\release\bundle\nsis
powershell -NoProfile -Command "& npx.cmd tauri build 2>&1 | Tee-Object -FilePath derleme-kaydi.txt; exit $LASTEXITCODE"
if errorlevel 1 goto hata

for %%f in ("src-tauri\target\release\bundle\nsis\*-setup.exe") do set "KURULUM=%%f"
if not defined KURULUM goto hata
echo.
echo  ==========================================
echo   TAMAM! Kurulum aciliyor...
echo  ==========================================
start "" "%KURULUM%"
echo.
pause
exit /b 0

:hata
echo.
echo  [X] Bir hata oldu.
echo  "derleme-kaydi.txt" dosyasi bu klasorde. Icindeki yaziyi Claude'a gonder.
echo  (Dosya yoksa bu penceredeki yaziyi gonder.)
echo.
pause
exit /b 1
