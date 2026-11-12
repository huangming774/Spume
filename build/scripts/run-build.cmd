@echo off
cd /d "%~dp0.."
echo === python build_content.py ===
python build_content.py
if errorlevel 1 exit /b 1
echo === npm install ===
call npm install
if errorlevel 1 exit /b 1
echo === npm run build ===
call npm run build
if errorlevel 1 exit /b 1
echo === done ===
