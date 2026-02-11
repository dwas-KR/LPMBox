import time
import subprocess
from pathlib import Path
from .constants import READBACK_DIR, IMAGE_DIR, SPFT_EXE
from .utils import log, log_text
from .i18n import get_string
COUNTRIES: list[tuple[str, str]] = [('Argentina', 'AR'), ('Armenia', 'AM'), ('Australia', 'AU'), ('Austria', 'AT'), ('Azerbaijan', 'AZ'), ('Bahrain', 'BH'), ('Belgium', 'BE'), ('Brazil', 'BR'), ('Bulgaria', 'BG'), ('Canada', 'CA'), ('Chile', 'CL'), ('China', 'CN'), ('Colombia', 'CO'), ('Costa Rica', 'CR'), ('Croatia', 'HR'), ('Cyprus', 'CY'), ('Czech Republic', 'CZ'), ('Denmark', 'DK'), ('Ecuador', 'EC'), ('Egypt', 'EG'), ('El Salvador', 'SV'), ('Estonia', 'EE'), ('Finland', 'FI'), ('France', 'FR'), ('Georgia', 'GE'), ('Germany', 'DE'), ('Ghana', 'GH'), ('Greece', 'GR'), ('Guatemala', 'GT'), ('Hong Kong', 'HK'), ('Hungary', 'HU'), ('Iceland', 'IS'), ('India', 'IN'), ('Indonesia', 'ID'), ('Israel', 'IL'), ('Italy', 'IT'), ('Japan', 'JP'), ('Jordan', 'JO'), ('Kazakhstan', 'KZ'), ('Kenya', 'KE'), ('Korea', 'KR'), ('Kuwait', 'KW'), ('Kyrgyzstan', 'KG'), ('Latvia', 'LV'), ('Lebanon', 'LB'), ('Lithuania', 'LT'), ('Malaysia', 'MY'), ('Mexico', 'MX'), ('Moldova', 'MD'), ('Morocco', 'MA'), ('Mozambique', 'MZ'), ('Netherlands', 'NL'), ('New Zealand', 'NZ'), ('Nigeria', 'NG'), ('Norway', 'NO'), ('Oman', 'OM'), ('Pakistan', 'PK'), ('Panama', 'PA'), ('Peru', 'PE'), ('Philippines', 'PH'), ('Poland', 'PL'), ('Portugal', 'PT'), ('Qatar', 'QA'), ('Romania', 'RO'), ('Russia', 'RU'), ('Saudi Arabia', 'SA'), ('Serbia', 'RS'), ('Singapore', 'SG'), ('Slovakia', 'SK'), ('Slovenia', 'SI'), ('South Africa', 'ZA'), ('Spain', 'ES'), ('Sweden', 'SE'), ('Switzerland', 'CH'), ('Taiwan', 'TW'), ('Tajikistan', 'TJ'), ('Tanzania', 'TZ'), ('Thailand', 'TH'), ('Tunisia', 'TN'), ('Turkey', 'TR'), ('Uganda', 'UG'), ('Ukraine', 'UA'), ('United Arab Emirates', 'AE'), ('United Kingdom', 'GB'), ('United States of America', 'US'), ('Uruguay', 'UY'), ('Uzbekistan', 'UZ'), ('Venezuela', 'VE'), ('Vietnam', 'VN')]

def _detect_current_code(data: bytes) -> str:
    for _, code in COUNTRIES:
        token = (code + 'XX').encode('ascii')
        idx = data.find(token)
        if idx != -1:
            return code + 'XX'
    idx = data.find(b'CNXX')
    if idx != -1:
        return 'CNXX'
    return ''

def _patch_country(data: bytes, new_code: str) -> bytes:
    token_new = (new_code + 'XX').encode('ascii')
    for _, code in COUNTRIES:
        token_old = (code + 'XX').encode('ascii')
        idx = data.find(token_old)
        if idx != -1:
            if len(token_old) != len(token_new):
                return data
            return data[:idx] + token_new + data[idx + len(token_old):]
    idx = data.find(b'CNXX')
    if idx != -1 and len(token_new) == 4:
        return data[:idx] + token_new + data[idx + 4:]
    return data

def _print_country_menu() -> None:
    log_text(get_string('country.menu_line'))
    per_row = 3
    rows = []
    row = []
    for idx, (name, code) in enumerate(COUNTRIES, start=1):
        item = f'{idx:3d}. {name} ({code})'
        row.append(item.ljust(35))
        if len(row) == per_row:
            rows.append(' '.join(row))
            row = []
    if row:
        rows.append(' '.join(row))
    for line in rows:
        log_text(line)
    log_text(get_string('country.menu_line'))

def _select_country() -> str:
    _print_country_menu()
    log_text(get_string('country.notice'))
    while True:
        try:
            raw = input(get_string('country.number_prompt'))
        except EOFError:
            return ''
        choice = raw.strip()
        if not choice:
            return ''
        if not choice.isdigit():
            log('country.invalid')
            continue
        num = int(choice)
        if num < 1 or num > len(COUNTRIES):
            log('country.invalid')
            continue
        name, code = COUNTRIES[num - 1]
        log('country.selected', name=name, code=code)
        return code

def wait_and_patch_proinfo(platform: str) -> None:
    READBACK_DIR.mkdir(parents=True, exist_ok=True)
    log('flow.wait_proinfo')
    proinfo_path: Path | None = None
    while proinfo_path is None:
        files = list(READBACK_DIR.glob('proinfo*'))
        if files:
            proinfo_path = max(files, key=lambda p: p.stat().st_mtime)
            break
        time.sleep(3)
    time.sleep(7)
    data: bytes | None = None
    while True:
        try:
            data = proinfo_path.read_bytes()
            break
        except PermissionError:
            time.sleep(1)
        except Exception:
            data = None
            break
    if data is None:
        log('country.no_file')
        return
    try:
        exe_name = SPFT_EXE.name
    except Exception:
        exe_name = 'SPFlashToolV6.exe'
    try:
        subprocess.run(['taskkill', '/f', '/im', exe_name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
    except Exception:
        pass
    current = _detect_current_code(data)
    if not current:
        log('country.not_detected')
    new_code = _select_country()
    if not new_code:
        log('country.no_change')
        patched = data
    else:
        log('country.patching', code=new_code)
        patched = _patch_country(data, new_code)
    dst = IMAGE_DIR / 'proinfo'
    dst.write_bytes(patched)
    log('flow.proinfo_copied')
