"""Bulk Picture Manipulator — transformation pipeline applied over a batch.

Pipeline is data: Op union + list of ops. Pure fold. Dry-run predicts
dims without touching pixels. Batch isolates per-image errors.
"""

from __future__ import annotations
from dataclasses import dataclass
from enum import Enum
from typing import Union

from graphics.grayscale import Algorithm, Image, Rgb, pixel_to_gray


class OpKind(Enum):
    RESIZE = "resize"
    GRAYSCALE = "grayscale"
    ROTATE_90 = "rotate_90"
    ROTATE_180 = "rotate_180"
    ROTATE_270 = "rotate_270"
    FLIP_H = "flip_h"
    FLIP_V = "flip_v"
    CROP = "crop"
    BRIGHTNESS = "brightness"
    CONTRAST = "contrast"


@dataclass(frozen=True)
class Op:
    kind: OpKind
    # payload fields — only the relevant ones are populated per kind
    width: int = 0
    height: int = 0
    x: int = 0
    y: int = 0
    algorithm: Algorithm = Algorithm.LUMINOSITY
    brightness_delta: int = 0
    contrast_factor: float = 1.0


def op_resize(w: int, h: int) -> Op:
    return Op(OpKind.RESIZE, width=w, height=h)

def op_grayscale(algo: Algorithm) -> Op:
    return Op(OpKind.GRAYSCALE, algorithm=algo)

def op_rotate_90() -> Op: return Op(OpKind.ROTATE_90)
def op_rotate_180() -> Op: return Op(OpKind.ROTATE_180)
def op_rotate_270() -> Op: return Op(OpKind.ROTATE_270)
def op_flip_h() -> Op: return Op(OpKind.FLIP_H)
def op_flip_v() -> Op: return Op(OpKind.FLIP_V)

def op_crop(x: int, y: int, w: int, h: int) -> Op:
    return Op(OpKind.CROP, x=x, y=y, width=w, height=h)

def op_brightness(delta: int) -> Op:
    return Op(OpKind.BRIGHTNESS, brightness_delta=delta)

def op_contrast(factor: float) -> Op:
    return Op(OpKind.CONTRAST, contrast_factor=factor)


class OpError(Exception):
    pass


class CropOutOfBounds(OpError):
    pass


class ZeroDimension(OpError):
    pass


Pipeline = list[Op]


def apply_op(img: Image, op: Op) -> Image:
    k = op.kind
    if k == OpKind.RESIZE:
        if op.width == 0 or op.height == 0: raise ZeroDimension()
        return _resize_nearest(img, op.width, op.height)
    if k == OpKind.GRAYSCALE:
        return _grayscale(img, op.algorithm)
    if k == OpKind.ROTATE_90:
        return _rotate_90(img)
    if k == OpKind.ROTATE_180:
        return _rotate_180(img)
    if k == OpKind.ROTATE_270:
        return _rotate_270(img)
    if k == OpKind.FLIP_H:
        return _flip_h(img)
    if k == OpKind.FLIP_V:
        return _flip_v(img)
    if k == OpKind.CROP:
        if op.width == 0 or op.height == 0: raise ZeroDimension()
        if op.x + op.width > img.width or op.y + op.height > img.height:
            raise CropOutOfBounds()
        return _crop(img, op.x, op.y, op.width, op.height)
    if k == OpKind.BRIGHTNESS:
        return _map_pixels(img, lambda p: _adj_brightness(p, op.brightness_delta))
    if k == OpKind.CONTRAST:
        return _map_pixels(img, lambda p: _adj_contrast(p, op.contrast_factor))
    raise AssertionError(f"unreachable: {k}")


def apply_pipeline(img: Image, pipeline: Pipeline) -> Image:
    cur = img
    for op in pipeline:
        cur = apply_op(cur, op)
    return cur


@dataclass
class BatchOk:
    index: int
    output: Image

@dataclass
class BatchErr:
    index: int
    error: str

BatchResult = Union[BatchOk, BatchErr]


def run_batch(images: list[Image], pipeline: Pipeline) -> list[BatchResult]:
    results: list[BatchResult] = []
    for i, img in enumerate(images):
        try:
            out = apply_pipeline(img, pipeline)
            results.append(BatchOk(i, out))
        except OpError as e:
            results.append(BatchErr(i, type(e).__name__))
    return results


def dry_run_dims(start: tuple[int, int], pipeline: Pipeline) -> list[tuple[int, int]]:
    out = [start]
    cur = start
    for op in pipeline:
        k = op.kind
        if k == OpKind.RESIZE:
            if op.width == 0 or op.height == 0: raise ZeroDimension()
            cur = (op.width, op.height)
        elif k in (OpKind.GRAYSCALE, OpKind.FLIP_H, OpKind.FLIP_V,
                   OpKind.ROTATE_180, OpKind.BRIGHTNESS, OpKind.CONTRAST):
            pass  # cur unchanged
        elif k in (OpKind.ROTATE_90, OpKind.ROTATE_270):
            cur = (cur[1], cur[0])
        elif k == OpKind.CROP:
            if op.width == 0 or op.height == 0: raise ZeroDimension()
            if op.x + op.width > cur[0] or op.y + op.height > cur[1]:
                raise CropOutOfBounds()
            cur = (op.width, op.height)
        out.append(cur)
    return out


# ---------- primitives ----------

def _map_pixels(img: Image, f) -> Image:
    return Image(img.width, img.height, [f(p) for p in img.pixels])


def _adj_brightness(p: Rgb, delta: int) -> Rgb:
    clamp = lambda v: max(0, min(255, v + delta))
    return Rgb(clamp(p.r), clamp(p.g), clamp(p.b))


def _adj_contrast(p: Rgb, factor: float) -> Rgb:
    def adj(v: int) -> int:
        return max(0, min(255, round((v - 128) * factor + 128)))
    return Rgb(adj(p.r), adj(p.g), adj(p.b))


def _grayscale(img: Image, algo: Algorithm) -> Image:
    def to_gray_px(p: Rgb) -> Rgb:
        v = pixel_to_gray(p, algo)
        return Rgb(v, v, v)
    return _map_pixels(img, to_gray_px)


def _get(img: Image, x: int, y: int) -> Rgb:
    return img.pixels[y * img.width + x]


def _resize_nearest(img: Image, new_w: int, new_h: int) -> Image:
    pixels: list[Rgb] = []
    for y in range(new_h):
        for x in range(new_w):
            sx = min(img.width - 1, (x * img.width) // new_w)
            sy = min(img.height - 1, (y * img.height) // new_h)
            pixels.append(_get(img, sx, sy))
    return Image(new_w, new_h, pixels)


def _rotate_90(img: Image) -> Image:
    new = [Rgb(0, 0, 0)] * (img.width * img.height)
    for y in range(img.height):
        for x in range(img.width):
            new_x = img.height - 1 - y
            new_y = x
            new[new_y * img.height + new_x] = _get(img, x, y)
    return Image(img.height, img.width, new)


def _rotate_180(img: Image) -> Image:
    return Image(img.width, img.height, list(reversed(img.pixels)))


def _rotate_270(img: Image) -> Image:
    new = [Rgb(0, 0, 0)] * (img.width * img.height)
    for y in range(img.height):
        for x in range(img.width):
            new_x = y
            new_y = img.width - 1 - x
            new[new_y * img.height + new_x] = _get(img, x, y)
    return Image(img.height, img.width, new)


def _flip_h(img: Image) -> Image:
    new = [Rgb(0, 0, 0)] * (img.width * img.height)
    for y in range(img.height):
        for x in range(img.width):
            new[y * img.width + (img.width - 1 - x)] = _get(img, x, y)
    return Image(img.width, img.height, new)


def _flip_v(img: Image) -> Image:
    new = [Rgb(0, 0, 0)] * (img.width * img.height)
    for y in range(img.height):
        for x in range(img.width):
            new[(img.height - 1 - y) * img.width + x] = _get(img, x, y)
    return Image(img.width, img.height, new)


def _crop(img: Image, x0: int, y0: int, w: int, h: int) -> Image:
    pixels: list[Rgb] = []
    for y in range(h):
        for x in range(w):
            pixels.append(_get(img, x0 + x, y0 + y))
    return Image(w, h, pixels)


def demo() -> None:
    # 4x4 checker
    pixels = []
    for y in range(4):
        for x in range(4):
            if (x + y) % 2 == 0:
                pixels.append(Rgb(200, 100, 50))
            else:
                pixels.append(Rgb(50, 100, 200))
    img = Image(4, 4, pixels)

    pipeline: Pipeline = [
        op_rotate_90(),
        op_brightness(10),
        op_grayscale(Algorithm.LUMINOSITY),
    ]

    out = apply_pipeline(img, pipeline)
    dims = dry_run_dims((4, 4), pipeline)
    print(f"pipeline steps: {len(pipeline)}")
    print(f"dry-run dims: {dims}")
    print(f"output: {out.width}x{out.height} pixels[0]=({out.pixels[0].r},{out.pixels[0].g},{out.pixels[0].b})")

    # batch with a mix of valid and invalid
    batch = [Image(6, 6, [Rgb(128, 128, 128)] * 36),
             Image(2, 2, [Rgb(128, 128, 128)] * 4)]
    crop_pipeline = [op_crop(0, 0, 3, 3)]
    results = run_batch(batch, crop_pipeline)
    for r in results:
        if isinstance(r, BatchOk):
            print(f"  [{r.index}] ok: {r.output.width}x{r.output.height}")
        else:
            print(f"  [{r.index}] err: {r.error}")


if __name__ == "__main__":
    demo()
