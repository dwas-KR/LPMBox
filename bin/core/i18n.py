import json
import threading
from pathlib import Path
_lang_lock = threading.Lock()
_current_lang = 'en'
_fallback_lang = 'en'
_lang_data = {}
_lang_dir = Path(__file__).resolve().parent / 'lang'
 
def _load_lang(code: str) -> None:
    global _lang_data
    data = {}
    fallback_path = _lang_dir / f'{_fallback_lang}.json'
    if fallback_path.is_file():
        try:
            with fallback_path.open('r', encoding='utf-8') as f:
                data.update(json.load(f))
        except Exception:
            data = {}
    if code != _fallback_lang:
        lang_path = _lang_dir / f'{code}.json'
        if lang_path.is_file():
            try:
                with lang_path.open('r', encoding='utf-8') as f:
                    data.update(json.load(f))
            except Exception:
                pass
    _lang_data = data

def set_language(code: str) -> None:
    global _current_lang
    if not code:
        code = _fallback_lang
    code = code.lower()
    if code not in ('ko', 'en', 'ru', 'jp'):
        code = _fallback_lang
    with _lang_lock:
        _current_lang = code
        _load_lang(code)

def get_language() -> str:
    return _current_lang

def get_string(key: str) -> str:
    with _lang_lock:
        data = _lang_data
        lang = _current_lang
    if not data:
        _load_lang(lang)
        data = _lang_data
    value = data.get(key)
    if value is None:
        return key
    return value
