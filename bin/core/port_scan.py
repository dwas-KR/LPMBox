from __future__ import annotations
import subprocess
import time
from .utils import log
POWERSHELL_CMD = [
    "powershell",
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-Command",
    "(Get-PnpDevice | Where-Object { $_.FriendlyName -like '*MediaTek*' -and ($_.FriendlyName -like '*PreLoader*' -or $_.FriendlyName -like '*USB Port*' -or $_.FriendlyName -like '*VCOM*') } | Select-Object -First 1).FriendlyName"
]
_preloader_wait_logged: bool = False
 
def wait_for_preloader(timeout: int | None=None) -> bool:
    global _preloader_wait_logged
    if not _preloader_wait_logged:
        log('preloader.waiting')
        _preloader_wait_logged = True
    start = time.time()
    while True:
        try:
            result = subprocess.run(POWERSHELL_CMD, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, encoding='utf-8', errors='ignore')
            name = (result.stdout or '').strip()
            if name:
                log('preloader.detected', name=name)
                return True
        except Exception:
            pass
        if timeout is not None and time.time() - start >= timeout:
            log('adb.timeout')
            return False
