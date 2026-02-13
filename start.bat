@echo off
setlocal
 
chcp 65001 >nul 2>&1
title LPMBox

set "ROOT=%~dp0"
if "%ROOT:~-1%"=="\" set "ROOT=%ROOT:~0,-1%"

if not exist "%ROOT%\logs" mkdir "%ROOT%\logs"
if not exist "%ROOT%\bin" mkdir "%ROOT%\bin"
if not exist "%ROOT%\bin\python" mkdir "%ROOT%\bin\python"

for /f %%t in ('powershell -NoProfile -Command "(Get-Date).ToString(\"yyyy.MM.dd_HH.mm.ss\")"') do set "LOG_TS=%%t"
if not defined LOG_TS set "LOG_TS=0000.00.00_00.00.00"
set "LOG_FILE=%ROOT%\logs\log_%LOG_TS%.log"
set "MTK_LOG_FILE=%LOG_FILE%"
set "LPMBOX_LOG_FILE=%LOG_FILE%"

call "%ROOT%\bin\core\install_python.bat" "%ROOT%"
if errorlevel 1 goto wait_exit

set "PYTHONEXE=%ROOT%\bin\python\python.exe"
set "PYTHONUTF8=1"
set "PYTHONIOENCODING=utf-8"
set "PYTHONPATH=%ROOT%\bin"

pushd "%ROOT%\bin"
"%PYTHONEXE%" -m core.bootstrap
set "EXITCODE=%ERRORLEVEL%"
popd

if "%EXITCODE%"=="" set "EXITCODE=0"
if "%EXITCODE%"=="0" goto end

echo [!] Python exited with code %EXITCODE%.>>"%LOG_FILE%"
echo [!] Python exited with code %EXITCODE%.
echo.
echo Press ENTER to exit.
pause >nul
goto end

:wait_exit
echo.
echo Press ENTER to exit.
pause >nul

:end
endlocal
