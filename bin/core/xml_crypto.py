import hashlib
import struct
from pathlib import Path

def _pbkdf1(password: str, salt: bytes, out_len: int, iterations: int=1000) -> bytes:
    data = password.encode('utf-8') + salt
    digest = hashlib.sha256(data).digest()
    for _ in range(iterations - 1):
        digest = hashlib.sha256(digest).digest()
    return digest[:out_len]
 
def decrypt_scatter_x(path: Path | str) -> bytes:
    from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
    p = Path(path)
    data = p.read_bytes()
    if len(data) < 64:
        raise ValueError('invalid scatter.x')
    iv = data[:16]
    salt = data[16:32]
    body = data[32:]
    key = _pbkdf1('OSD', salt, 32, 1000)
    cipher = Cipher(algorithms.AES(key), modes.CBC(iv))
    decryptor = cipher.decryptor()
    plain = decryptor.update(body) + decryptor.finalize()
    if len(plain) < 16 + 32:
        raise ValueError('invalid decrypted data')
    size = struct.unpack('<q', plain[:8])[0]
    signature = plain[8:16]
    expected = b'\xcf\x06\x05\x04\x03\x02\x01\xfc'
    if signature != expected:
        raise ValueError('invalid signature')
    payload = plain[16:16 + size]
    digest = plain[16 + size:16 + size + 32]
    if hashlib.sha256(payload).digest() != digest:
        raise ValueError('hash mismatch')
    return payload
