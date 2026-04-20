"""Grayscale Converter — pixel buffers + named conversion algorithms + histogram.

Pure pixel model: Image is row-major Vec<Rgb>. Algorithms: Luminosity
(Rec.709 weights), Average, Lightness (max+min)/2, single-channel R/G/B.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


@dataclass
class Rgb:
    r: int
    g: int
    b: int

    @classmethod
    def black(cls) -> Rgb: return cls(0, 0, 0)
    @classmethod
    def white(cls) -> Rgb: return cls(255, 255, 255)


@dataclass
class Image:
    width: int
    height: int
    pixels: list[Rgb] = field(default_factory=list)

    @classmethod
    def new(cls, width: int, height: int) -> Image:
        return cls(width, height, [Rgb.black() for _ in range(width * height)])

    @classmethod
    def filled(cls, width: int, height: int, color: Rgb) -> Image:
        return cls(width, height, [Rgb(color.r, color.g, color.b) for _ in range(width * height)])

    @classmethod
    def from_pixels(cls, width: int, height: int, pixels: list[Rgb]) -> Image | None:
        if len(pixels) != width * height:
            return None
        return cls(width, height, pixels)

    def get(self, x: int, y: int) -> Rgb | None:
        if x < 0 or x >= self.width or y < 0 or y >= self.height:
            return None
        return self.pixels[y * self.width + x]

    def set(self, x: int, y: int, pixel: Rgb) -> bool:
        if x < 0 or x >= self.width or y < 0 or y >= self.height:
            return False
        self.pixels[y * self.width + x] = pixel
        return True


@dataclass
class GrayImage:
    width: int
    height: int
    pixels: list[int] = field(default_factory=list)

    @classmethod
    def new(cls, width: int, height: int) -> GrayImage:
        return cls(width, height, [0] * (width * height))

    def get(self, x: int, y: int) -> int | None:
        if x < 0 or x >= self.width or y < 0 or y >= self.height:
            return None
        return self.pixels[y * self.width + x]


class Algorithm(Enum):
    LUMINOSITY = "luminosity"
    AVERAGE = "average"
    LIGHTNESS = "lightness"
    RED = "red"
    GREEN = "green"
    BLUE = "blue"


def pixel_to_gray(p: Rgb, algo: Algorithm) -> int:
    if algo == Algorithm.LUMINOSITY:
        v = 0.2126 * p.r + 0.7152 * p.g + 0.0722 * p.b
        return max(0, min(255, round(v)))
    if algo == Algorithm.AVERAGE:
        return (p.r + p.g + p.b) // 3
    if algo == Algorithm.LIGHTNESS:
        mx = max(p.r, p.g, p.b)
        mn = min(p.r, p.g, p.b)
        return (mx + mn) // 2
    if algo == Algorithm.RED: return p.r
    if algo == Algorithm.GREEN: return p.g
    if algo == Algorithm.BLUE: return p.b
    return 0


def convert(image: Image, algo: Algorithm) -> GrayImage:
    pixels = [pixel_to_gray(p, algo) for p in image.pixels]
    return GrayImage(image.width, image.height, pixels)


def histogram(gray: GrayImage) -> list[int]:
    bins = [0] * 256
    for v in gray.pixels:
        bins[v] += 1
    return bins


def mean_brightness(gray: GrayImage) -> float:
    if not gray.pixels:
        return 0.0
    return sum(gray.pixels) / len(gray.pixels)


def contrast_stddev(gray: GrayImage) -> float:
    if not gray.pixels:
        return 0.0
    mean = mean_brightness(gray)
    var = sum((v - mean) ** 2 for v in gray.pixels) / len(gray.pixels)
    return var ** 0.5


def dynamic_range(gray: GrayImage) -> tuple[int, int] | None:
    if not gray.pixels:
        return None
    return (min(gray.pixels), max(gray.pixels))


def demo() -> None:
    img = Image.from_pixels(2, 2, [
        Rgb.black(),
        Rgb(255, 0, 0),
        Rgb(0, 255, 0),
        Rgb(0, 0, 255),
    ])
    for algo in [Algorithm.LUMINOSITY, Algorithm.AVERAGE, Algorithm.LIGHTNESS]:
        g = convert(img, algo)
        print(f"{algo.value:10}: pixels={g.pixels} "
              f"mean={mean_brightness(g):.2f} "
              f"stddev={contrast_stddev(g):.2f} "
              f"range={dynamic_range(g)}")


if __name__ == "__main__":
    demo()
