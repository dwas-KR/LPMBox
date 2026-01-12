@echo off
chcp 65001 >nul
setlocal ENABLEDELAYEDEXPANSION
title LPMBox
mode con: cols=145 lines=38
cls

set "ROOT=%~dp0"
if "%ROOT:~-1%"=="\" set "ROOT=%ROOT:~0,-1%"

if not exist "%ROOT%\logs" mkdir "%ROOT%\logs"

if not exist "%ROOT%\bin" mkdir "%ROOT%\bin"
if not exist "%ROOT%\bin\python" mkdir "%ROOT%\bin\python"

set "LOG_TS="
for /f "tokens=1-6 delims=/.:- " %%a in ("%date% %time%") do (
    if not defined LOG_TS (
        set "LOG_TS=%%a.%%b.%%c_%%d.%%e.%%f"
    )
)
if not defined LOG_TS set "LOG_TS=0000.00.00_00.00.00"

set "LOG_FILE=%ROOT%\logs\log_%LOG_TS%.log"
set "MTK_LOG_FILE=%LOG_FILE%"

set "PYTHONEXE="

if exist "%ROOT%\bin\python\python.exe" (
    set "PYTHONEXE=%ROOT%\bin\python\python.exe"
)

if "%PYTHONEXE%"=="" (
    if exist "%ROOT%\bin\py\python.exe" (
        set "PYTHONEXE=%ROOT%\bin\py\python.exe"
    )
)

if "%PYTHONEXE%"=="" (
    where /q py
    if not errorlevel 1 (
        set "PYTHONEXE=py"
    )
)

if "%PYTHONEXE%"=="" (
    where /q python
    if not errorlevel 1 (
        set "PYTHONEXE=python"
    )
)

if "%PYTHONEXE%"=="" (
    echo No Python interpreter found. >> "%LOG_FILE%"
    echo No Python interpreter found.
    goto wait_exit
)

set "PYTHONUTF8=1"
set "PYTHONIOENCODING=utf-8"
set "PYTHONPATH=%ROOT%\bin"

pushd "%ROOT%"
"%PYTHONEXE%" -m core.bootstrap
popd

:wait_exit
echo.
echo Press ENTER to exit.
pause >nul
taskkill /IM adb.exe /F >nul 2>&1
endlocal
