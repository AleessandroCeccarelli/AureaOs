#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Convert the restricted RGBA8 PNG output of the AureaOS asset pipeline to raw RGBA."""

from __future__ import annotations

import argparse
import struct
import zlib
from pathlib import Path

PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
MAX_DIMENSION = 256
MAX_OUTPUT_BYTES = MAX_DIMENSION * MAX_DIMENSION * 4


def paeth(left: int, above: int, upper_left: int) -> int:
    estimate = left + above - upper_left
    distances = (
        abs(estimate - left),
        abs(estimate - above),
        abs(estimate - upper_left),
    )
    return (left, above, upper_left)[distances.index(min(distances))]


def decode_rgba8(data: bytes) -> tuple[int, int, bytes]:
    if not data.startswith(PNG_SIGNATURE):
        raise ValueError("input is not a PNG file")

    offset = len(PNG_SIGNATURE)
    width = height = bit_depth = color_type = interlace = None
    compressed = bytearray()
    saw_end = False

    while offset + 12 <= len(data):
        length = struct.unpack_from(">I", data, offset)[0]
        chunk_type = data[offset + 4:offset + 8]
        chunk_end = offset + 12 + length
        if chunk_end > len(data):
            raise ValueError("truncated PNG chunk")
        chunk = data[offset + 8:offset + 8 + length]
        expected_crc = struct.unpack_from(">I", data, offset + 8 + length)[0]
        actual_crc = zlib.crc32(chunk_type + chunk) & 0xFFFFFFFF
        if actual_crc != expected_crc:
            raise ValueError("PNG chunk CRC mismatch")

        if chunk_type == b"IHDR":
            if width is not None or length != 13:
                raise ValueError("invalid PNG header")
            width, height, bit_depth, color_type, compression, filtering, interlace = (
                struct.unpack(">IIBBBBB", chunk)
            )
            if compression != 0 or filtering != 0:
                raise ValueError("unsupported PNG compression or filter method")
        elif chunk_type == b"IDAT":
            compressed.extend(chunk)
            if len(compressed) > MAX_OUTPUT_BYTES + 65536:
                raise ValueError("compressed PNG exceeds the asset budget")
        elif chunk_type == b"IEND":
            saw_end = True
            break
        elif chunk_type[0] & 0x20 == 0 and chunk_type not in (b"IHDR",):
            raise ValueError(f"unsupported critical PNG chunk: {chunk_type!r}")

        offset = chunk_end

    if not saw_end or width is None or height is None:
        raise ValueError("incomplete PNG file")
    if not (1 <= width <= MAX_DIMENSION and 1 <= height <= MAX_DIMENSION):
        raise ValueError("PNG dimensions exceed the asset limit")
    if bit_depth != 8 or color_type != 6 or interlace != 0:
        raise ValueError("only non-interlaced 8-bit RGBA PNG is supported")

    stride = width * 4
    expected = height * (stride + 1)
    decompressor = zlib.decompressobj()
    raw = decompressor.decompress(bytes(compressed), expected + 1)
    if len(raw) > expected or decompressor.unconsumed_tail:
        raise ValueError("PNG decompressed data exceeds the expected image size")
    raw += decompressor.flush()
    if len(raw) != expected or not decompressor.eof or decompressor.unused_data:
        raise ValueError("PNG decompressed data length is invalid")

    pixels = bytearray(height * stride)
    source_offset = 0
    for y in range(height):
        filter_type = raw[source_offset]
        source_offset += 1
        row = bytearray(raw[source_offset:source_offset + stride])
        source_offset += stride
        previous = pixels[(y - 1) * stride:y * stride] if y else None

        for x in range(stride):
            left = row[x - 4] if x >= 4 else 0
            above = previous[x] if previous is not None else 0
            upper_left = previous[x - 4] if previous is not None and x >= 4 else 0
            if filter_type == 1:
                row[x] = (row[x] + left) & 0xFF
            elif filter_type == 2:
                row[x] = (row[x] + above) & 0xFF
            elif filter_type == 3:
                row[x] = (row[x] + ((left + above) // 2)) & 0xFF
            elif filter_type == 4:
                row[x] = (row[x] + paeth(left, above, upper_left)) & 0xFF
            elif filter_type != 0:
                raise ValueError(f"unsupported PNG filter: {filter_type}")
        pixels[y * stride:(y + 1) * stride] = row

    return width, height, bytes(pixels)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()

    width, height, rgba = decode_rgba8(args.source.read_bytes())
    args.destination.parent.mkdir(parents=True, exist_ok=True)
    args.destination.write_bytes(rgba)
    print(f"{width}x{height} RGBA8 ({len(rgba)} bytes): {args.destination}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
