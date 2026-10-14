#!/usr/bin/env python3
"""Draw the Clean Your Mac mark into a macOS .iconset directory."""

import math
import struct
import sys
import zlib
from pathlib import Path

GREEN = (0x1E, 0xD7, 0x60, 255)
CLEAR = (0, 0, 0, 0)
BLACK = (0, 0, 0, 255)


def png(size: int, pixel) -> bytes:
    raw = bytearray()
    for y in range(size):
        raw.append(0)
        for x in range(size):
            raw.extend(pixel(x, y))

    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def mark(size: int):
    center = (size - 1) / 2
    radius = size * 0.46
    bar_w = max(1.0, radius * 0.28)
    gap = radius * 0.22
    heights = (0.42, 0.72, 0.54)
    total = bar_w * 3 + gap * 2
    left = center - total / 2

    def inside_bar(x: float, y: float) -> bool:
        for index, height in enumerate(heights):
            h = radius * 1.15 * height
            bx = left + index * (bar_w + gap)
            top = center + radius * 0.42 - h
            bottom = top + h
            if bx <= x <= bx + bar_w and top <= y <= bottom:
                return True
        return False

    def at(x: int, y: int):
        dx = x - center
        dy = y - center
        if math.hypot(dx, dy) > radius:
            return CLEAR
        if inside_bar(x, y):
            return BLACK
        return GREEN

    return at


def main() -> None:
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    names = {
        16: ["icon_16x16.png"],
        32: ["icon_16x16@2x.png", "icon_32x32.png"],
        64: ["icon_32x32@2x.png"],
        128: ["icon_128x128.png"],
        256: ["icon_128x128@2x.png", "icon_256x256.png"],
        512: ["icon_256x256@2x.png", "icon_512x512.png"],
        1024: ["icon_512x512@2x.png"],
    }
    for size, files in names.items():
        data = png(size, mark(size))
        for name in files:
            (out / name).write_bytes(data)


if __name__ == "__main__":
    main()
