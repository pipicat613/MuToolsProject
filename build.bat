@echo off
cd /d "%~dp0\MuToolsCode"
npm run tauri build
echo done.
exit /b %ERRORLEVEL%