@echo off
setlocal

set "ROOT=%~1"
if "%ROOT%"=="" set "ROOT=%~dp0..\.."
if "%ROOT:~-1%"=="\" set "ROOT=%ROOT:~0,-1%"

set "PYTHON_DIR=%ROOT%\bin\python"
set "PYTHON_VERSION=3.14.2"

rem Detect Windows OS architecture (x86 / x64 / ARM64)
set "ARCH="
if /I "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
    set "ARCH=amd64"
) else if /I "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
    set "ARCH=arm64"
) else if /I "%PROCESSOR_ARCHITECTURE%"=="x86" (
    if defined PROCESSOR_ARCHITEW6432 (
        rem 32-bit cmd on 64-bit OS
        set "ARCH=amd64"
    ) else (
        set "ARCH=win32"
    )
)

if "%ARCH%"=="" (
    rem Fallback: assume 64-bit
    set "ARCH=amd64"
)

set "PYTHON_ZIP_URL=https://www.python.org/ftp/python/%PYTHON_VERSION%/python-%PYTHON_VERSION%-embed-%ARCH%.zip"
set "PYTHON_ZIP_PATH=%PYTHON_DIR%\python_embed.zip"
set "PYTHON_PTH_FILE=%PYTHON_DIR%\python314._pth"
set "GETPIP_URL=https://bootstrap.pypa.io/get-pip.py"
set "GETPIP_PATH=%PYTHON_DIR%\get-pip.py"

if not exist "%PYTHON_DIR%" mkdir "%PYTHON_DIR%"

if not exist "%PYTHON_DIR%\python.exe" (
    echo [*] Python not found. Detected architecture: %ARCH%. Downloading embedded Python...
    curl --ssl-no-revoke -L "%PYTHON_ZIP_URL%" -o "%PYTHON_ZIP_PATH%" || (
        endlocal & exit /b 1
    )
    echo [*] Extracting embedded Python...
    tar -xf "%PYTHON_ZIP_PATH%" -C "%PYTHON_DIR%" || (
        endlocal & exit /b 1
    )
    del "%PYTHON_ZIP_PATH%"
    > "%PYTHON_PTH_FILE%" (
        echo python314.zip
        echo .
        echo ..\
        echo .\Lib\site-packages
        echo import site
    )
)

if not exist "%PYTHON_DIR%\python.exe" (
    echo [!] Embedded Python executable not found.
    endlocal & exit /b 1
)

if not exist "%PYTHON_DIR%\Scripts\pip.exe" (
    echo [*] pip not found. Installing...
    curl --ssl-no-revoke -L "%GETPIP_URL%" -o "%GETPIP_PATH%" || (
        endlocal & exit /b 1
    )
    "%PYTHON_DIR%\python.exe" "%GETPIP_PATH%" || (
        endlocal & exit /b 1
    )
    del "%GETPIP_PATH%"
)

endlocal & exit /b 0
