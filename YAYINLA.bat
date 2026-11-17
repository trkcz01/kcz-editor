@echo off
chcp 65001 >nul
title KCZ Launcher - Guncelleme yayinla
cd /d "%~dp0"
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
if not exist imza\kcz.key (
  echo  [!] Once TAURI-KUR.bat dosyasini bir kere calistir.
  pause
  exit /b 1
)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0yayinla.ps1"
if errorlevel 1 (
  echo.
  echo  [X] Bir hata oldu. Bu penceredeki yaziyi Claude'a gonder.
  echo  ^(Anahtar yanlissa gh-token.txt dosyasini silip tekrar dene.^)
)
echo.
pause
