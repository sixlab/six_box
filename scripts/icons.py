"""生成 M1 几何占位图标；不代表最终品牌设计。仅使用 Python 标准库。"""
import pathlib
import struct
import zlib

root = pathlib.Path(__file__).resolve().parents[1] / 'apps/desktop/src-tauri/icons'
root.mkdir(parents=True, exist_ok=True)

def png(size, tray=False):
    pixels = bytearray()
    for y in range(size):
        pixels.append(0)
        for x in range(size):
            u, v = x / size, y / size
            on = any(.20 + col * .22 < u < .36 + col * .22 and .31 + row * .22 < v < .47 + row * .22 for col in range(3) for row in range(2))
            pixels.extend((0, 0, 0, 255) if tray and on else (0, 0, 0, 0) if tray else (226, 244, 216, 255) if on else (40, 92, 71, 255))
    def chunk(kind, data):
        return struct.pack('!I', len(data)) + kind + data + struct.pack('!I', zlib.crc32(kind + data))
    return b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('!2I5B', size, size, 8, 6, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(pixels)) + chunk(b'IEND', b'')

(root / 'icon.png').write_bytes(png(256))
(root / 'tray.png').write_bytes(png(32, True))
