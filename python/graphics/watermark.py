"""Watermarking Application — LSB steganography + visible overlay + position ADT.

LSB: hide arbitrary bytes in pixel low-bits with a 32-bit length-prefix header.
Capacity: pure function of image size × bits_per_channel.
Position ADT: TopLeft/TopRight/BottomLeft/BottomRight/Center/Tiled.
Alpha-blend compositing for visible watermarks.
"""

from __future__ import annotations
from dataclasses import dataclass
from enum import Enum
from typing import Union

from graphics.grayscale import Image, Rgb


class Position(Enum):
    TOP_LEFT = "top_left"
    TOP_RIGHT = "top_right"
    BOTTOM_LEFT = "bottom_left"
    BOTTOM_RIGHT = "bottom_right"
    CENTER = "center"
    TILED = "tiled"


class WatermarkError(Exception):
    pass


class PayloadTooLarge(WatermarkError):
    def __init__(self, needed: int, capacity: int):
        super().__init__(f"payload {needed} > capacity {capacity}")
        self.needed = needed
        self.capacity = capacity


class InvalidBitsPerChannel(WatermarkError):
    pass


class TruncatedData(WatermarkError):
    pass


def lsb_capacity(image: Image, bits_per_channel: int) -> int:
    if bits_per_channel <= 0 or bits_per_channel > 4:
        return 0
    total_bits = len(image.pixels) * 3 * bits_per_channel
    total_bytes = total_bits // 8
    return max(0, total_bytes - 4)


def embed_lsb(image: Image, payload: bytes, bits_per_channel: int) -> Image:
    if bits_per_channel <= 0 or bits_per_channel > 4:
        raise InvalidBitsPerChannel()
    cap = lsb_capacity(image, bits_per_channel)
    if len(payload) > cap:
        raise PayloadTooLarge(len(payload), cap)

    bits: list[int] = []
    len_bytes = len(payload).to_bytes(4, byteorder="big", signed=False)
    for byte in list(len_bytes) + list(payload):
        for i in range(8):
            bits.append((byte >> (7 - i)) & 1)

    mask = (~((1 << bits_per_channel) - 1)) & 0xFF
    new_pixels = []
    bit_idx = 0
    total_bits = len(bits)

    def consume_chunk() -> int:
        nonlocal bit_idx
        chunk = 0
        for _ in range(bits_per_channel):
            if bit_idx >= total_bits:
                break
            chunk = (chunk << 1) | bits[bit_idx]
            bit_idx += 1
        return chunk

    for pixel in image.pixels:
        if bit_idx >= total_bits:
            new_pixels.append(Rgb(pixel.r, pixel.g, pixel.b))
            continue
        r = (pixel.r & mask) | consume_chunk()
        if bit_idx < total_bits:
            g = (pixel.g & mask) | consume_chunk()
        else:
            g = pixel.g
        if bit_idx < total_bits:
            b = (pixel.b & mask) | consume_chunk()
        else:
            b = pixel.b
        new_pixels.append(Rgb(r, g, b))

    return Image(image.width, image.height, new_pixels)


def extract_lsb(image: Image, bits_per_channel: int) -> bytes:
    if bits_per_channel <= 0 or bits_per_channel > 4:
        raise InvalidBitsPerChannel()
    mask = (1 << bits_per_channel) - 1
    bits: list[int] = []
    for pixel in image.pixels:
        for channel in (pixel.r, pixel.g, pixel.b):
            chunk = channel & mask
            for i in range(bits_per_channel):
                bits.append((chunk >> (bits_per_channel - 1 - i)) & 1)

    if len(bits) < 32:
        raise TruncatedData()
    len_bytes = bytearray(4)
    for i in range(32):
        len_bytes[i // 8] = (len_bytes[i // 8] << 1) | bits[i]
    payload_len = int.from_bytes(bytes(len_bytes), byteorder="big")
    total_bits_needed = 32 + payload_len * 8
    if len(bits) < total_bits_needed:
        raise TruncatedData()

    payload = bytearray(payload_len)
    for byte_i in range(payload_len):
        b = 0
        for bit_j in range(8):
            b = (b << 1) | bits[32 + byte_i * 8 + bit_j]
        payload[byte_i] = b
    return bytes(payload)


def resolve_position(base: Image, wm: Image, pos: Position) -> tuple[int, int]:
    bw, bh = base.width, base.height
    ww, wh = wm.width, wm.height
    if pos == Position.TOP_LEFT: return (0, 0)
    if pos == Position.TOP_RIGHT: return (max(0, bw - ww), 0)
    if pos == Position.BOTTOM_LEFT: return (0, max(0, bh - wh))
    if pos == Position.BOTTOM_RIGHT: return (max(0, bw - ww), max(0, bh - wh))
    if pos == Position.CENTER: return ((bw - ww) // 2, (bh - wh) // 2)
    if pos == Position.TILED: return (0, 0)
    raise AssertionError("unreachable")


def _blend_channel(a: int, b: int, alpha: int) -> int:
    return (a * (255 - alpha) + b * alpha) // 255


def _blend_region(out: Image, x0: int, y0: int, wm: Image, opacity: int) -> None:
    for wy in range(wm.height):
        for wx in range(wm.width):
            bx, by = x0 + wx, y0 + wy
            if bx < 0 or by < 0: continue
            if bx >= out.width or by >= out.height: continue
            base_idx = by * out.width + bx
            wm_idx = wy * wm.width + wx
            bp = out.pixels[base_idx]
            wp = wm.pixels[wm_idx]
            out.pixels[base_idx] = Rgb(
                _blend_channel(bp.r, wp.r, opacity),
                _blend_channel(bp.g, wp.g, opacity),
                _blend_channel(bp.b, wp.b, opacity),
            )


def apply_visible_watermark(base: Image, watermark: Image,
                              position: Position, opacity: int) -> Image:
    out = Image(base.width, base.height, list(base.pixels))
    if position == Position.TILED:
        y = 0
        while y < base.height:
            x = 0
            while x < base.width:
                _blend_region(out, x, y, watermark, opacity)
                x += watermark.width
            y += watermark.height
    else:
        x, y = resolve_position(base, watermark, position)
        _blend_region(out, x, y, watermark, opacity)
    return out


def demo() -> None:
    base = Image(32, 32, [Rgb(128, 128, 128) for _ in range(32 * 32)])
    print(f"capacity @ 1 bit/chan: {lsb_capacity(base, 1)} bytes")
    print(f"capacity @ 2 bits/chan: {lsb_capacity(base, 2)} bytes")
    print(f"capacity @ 4 bits/chan: {lsb_capacity(base, 4)} bytes")

    payload = b"Hello, watermark!"
    embedded = embed_lsb(base, payload, 1)
    extracted = extract_lsb(embedded, 1)
    print(f"roundtrip ok: {extracted == payload}")
    print(f"extracted: {extracted.decode()}")

    # Visible watermark
    wm = Image(8, 8, [Rgb(255, 0, 0) for _ in range(64)])
    tl = apply_visible_watermark(base, wm, Position.TOP_LEFT, 255)
    br = apply_visible_watermark(base, wm, Position.BOTTOM_RIGHT, 255)
    tr = apply_visible_watermark(base, wm, Position.TOP_RIGHT, 128)
    print(f"top-left (0,0): {tl.pixels[0]}")
    print(f"bottom-right (31,31): {br.pixels[31*32+31]}")
    print(f"top-right (31,0) blended: {tr.pixels[31]}")

    # Tiled — all pixels become watermark color at full opacity
    tiled = apply_visible_watermark(base, wm, Position.TILED, 255)
    all_red = all(p == Rgb(255, 0, 0) for p in tiled.pixels)
    print(f"tiled all red: {all_red}")


if __name__ == "__main__":
    demo()
