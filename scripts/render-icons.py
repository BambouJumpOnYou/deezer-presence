"""Renders assets/logo.png (README) and assets/AppIcon.png (macOS app icon). Pure Python, no dependencies."""
import math
import struct
import sys
import zlib
from pathlib import Path

ASSETS = Path(__file__).resolve().parent.parent / "assets"
HEIGHTS = [0.25, 0.5, 0.7, 0.4, 0.2]
COLORS = [(0xA2, 0x38, 0xFF), (0x8F, 0x3D, 0xFF), (0x7C, 0x42, 0xFF), (0x69, 0x47, 0xFF), (0x56, 0x4C, 0xFF)]
BACKGROUND_TOP = (0x24, 0x22, 0x2E)
BACKGROUND_BOTTOM = (0x0E, 0x0D, 0x13)


def blend(dst, color, alpha):
    a = dst[3] / 255
    out_a = alpha + a * (1 - alpha)
    if out_a == 0:
        return (0, 0, 0, 0)
    rgb = tuple(round((color[i] * alpha + dst[i] * a * (1 - alpha)) / out_a) for i in range(3))
    return (*rgb, round(out_a * 255))


def coverage(distance):
    return min(max(0.5 - distance, 0.0), 1.0)


def draw_bars(px, size, x0, y0, extent):
    bar_w = extent * 0.11
    gap = extent * 0.08
    r = bar_w / 2
    left = x0 + (extent - (5 * bar_w + 4 * gap)) / 2
    cy = y0 + extent / 2
    for i, h in enumerate(HEIGHTS):
        cx = left + i * (bar_w + gap) + r
        half = max(h * extent / 2 - r, 0)
        for y in range(max(int(cy - half - r) - 1, 0), min(int(cy + half + r) + 2, size)):
            for x in range(max(int(cx - r) - 1, 0), min(int(cx + r) + 2, size)):
                d = math.hypot(x + 0.5 - cx, max(abs(y + 0.5 - cy) - half, 0)) - r
                a = coverage(d)
                if a > 0:
                    px[y][x] = blend(px[y][x], COLORS[i], a)


def draw_squircle(px, size, inset, radius):
    lo, hi = inset, size - inset
    for y in range(size):
        t = (y - lo) / (hi - lo)
        color = tuple(round(BACKGROUND_TOP[i] * (1 - t) + BACKGROUND_BOTTOM[i] * t) for i in range(3))
        for x in range(size):
            qx = max(abs(x + 0.5 - size / 2) - (size / 2 - inset - radius), 0)
            qy = max(abs(y + 0.5 - size / 2) - (size / 2 - inset - radius), 0)
            a = coverage(math.hypot(qx, qy) - radius)
            if a > 0:
                px[y][x] = blend(px[y][x], color, a)


def write_png(path, px):
    size = len(px)
    raw = b"".join(b"\0" + bytes(c for p in row for c in p) for row in px)

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def logo():
    size = 256
    px = [[(0, 0, 0, 0)] * size for _ in range(size)]
    draw_bars(px, size, 0, 0, size)
    write_png(ASSETS / "logo.png", px)


def app_icon():
    # Apple's macOS icon grid: 824px body inside a 1024px canvas.
    size, inset = 1024, 100
    px = [[(0, 0, 0, 0)] * size for _ in range(size)]
    draw_squircle(px, size, inset, 185)
    draw_bars(px, size, inset + 60, inset + 60, size - 2 * inset - 120)
    write_png(ASSETS / "AppIcon.png", px)


if __name__ == "__main__":
    ASSETS.mkdir(exist_ok=True)
    targets = sys.argv[1:] or ["logo", "app_icon"]
    for name in targets:
        globals()[name]()
