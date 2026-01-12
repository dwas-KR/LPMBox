from pathlib import Path

BASE_DIR = Path(__file__).resolve().parents[2]
BIN_DIR = BASE_DIR / "bin"
CORE_DIR = BIN_DIR / "core"
TOOLS_DIR = BASE_DIR / "tools"
IMAGE_DIR = BASE_DIR / "image"
PYTHON_DIR = BIN_DIR / "python"
PLATFORM_TOOLS_DIR = TOOLS_DIR / "platform-tools"
TOOLS_DOWNLOAD_DIR = TOOLS_DIR / "download files"
SPFT_EXE = TOOLS_DIR / "SPFlashToolV6.exe"
PRC_DIR = TOOLS_DIR / "PRC"
READBACK_DIR = TOOLS_DIR / "Readback"
DOWNLOAD_AGENT_IMAGE_DIR = IMAGE_DIR / "download_agent"
FLASH_XML_DLAGENT = DOWNLOAD_AGENT_IMAGE_DIR / "flash.xml"
FLASH_XML_ROOT = IMAGE_DIR / "flash.xml"
DA_AUTH_DLAGENT = DOWNLOAD_AGENT_IMAGE_DIR / "da.auth"
DA_AUTH_ROOT = IMAGE_DIR / "da.auth"
LOGS_DIR = BASE_DIR / "logs"
LOG_ENV_VAR = "MTK_LOG_FILE"

PLATFORM_TOOLS_URLS = [
    "https://dl.google.com/android/repository/platform-tools-latest-windows.zip?hl"
]

SPFT_ZIP_URLS = [
    "https://spflashtools.com/wp-content/uploads/SP_Flash_Tool_V6.2404_Win.zip",
    "https://raw.githubusercontent.com/dwas-KR/LPMBox/refs/heads/Downloads/SPFlashToolV6.zip"
]

PRC_ZIP_URLS = [
    "https://raw.githubusercontent.com/dwas-KR/LPMBox/refs/heads/Downloads/PRC.zip",
    "https://github.com/dwas-KR/LPMBox/raw/Downloads/PRC.zip"
]

LK_DTBO_DIR = TOOLS_DIR / "lk, dtbo"
LK_DTBO_ZIP_URLS = [
    "https://raw.githubusercontent.com/dwas-KR/LPMBox/ee1fad3913cfe4dfc0214dbd95df3a82c0517b54/lk%2C%20dtbo.zip",
    "https://github.com/dwas-KR/LPMBox/raw/ee1fad3913cfe4dfc0214dbd95df3a82c0517b54/lk%2C%20dtbo.zip"
]

PYTHON_VERSION = "3.14.2"
PYTHON_EMBED_URL_TEMPLATE = "https://www.python.org/ftp/python/{version}/python-{version}-embed-{arch}.zip"
PYTHON_PTH_FILENAME = "python314._pth"
GET_PIP_URL = "https://bootstrap.pypa.io/get-pip.py"
REQUIRED_PYTHON_PACKAGES = ["cffi", "pycparser", "cryptography"]
