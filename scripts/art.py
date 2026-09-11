#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["pillow"]
# ///

from math import cos, pi, sin, sqrt
from pathlib import Path
from random import Random

from PIL import Image, ImageDraw, ImageFilter

SIZE = 1024
SCALE = 2
image = Image.new("RGBA", (SIZE * SCALE, SIZE * SCALE))
draw = ImageDraw.Draw(image)


def project(x: float, y: float, z: float = 0) -> tuple[float, float]:
    return ((512 + x * 1.65) * SCALE, (534 + y * 1.2 - z * 1.5) * SCALE)


def polygon(points: list[tuple[float, float, float]], color: str) -> None:
    draw.polygon([project(*point) for point in points], fill=color)


def line(points: list[tuple[float, float, float]], color: str, width: float) -> None:
    draw.line([project(*point) for point in points], fill=color, width=round(width * SCALE), joint="curve")


def ellipse(x: float, y: float, z: float, rx: float, ry: float, color: str) -> None:
    px, py = project(x, y, z)
    draw.ellipse((px - rx * SCALE, py - ry * SCALE, px + rx * SCALE, py + ry * SCALE), fill=color)


def tile(x: float, y: float, radius: float, color: str, seed: int) -> None:
    rng = Random(seed)
    top = [(x + radius * sin(i * pi / 3), y + radius * cos(i * pi / 3), 0) for i in range(6)]
    for i in range(6):
        a, b = top[i], top[(i + 1) % 6]
        polygon([a, b, (b[0], b[1], -24), (a[0], a[1], -24)], "#a38d62" if i < 3 else "#796f54")
    polygon(top, color)
    line(top + [top[0]], "#e7d8a5", 4)
    for _ in range(110):
        px, py = x + rng.uniform(-radius * .68, radius * .68), y + rng.uniform(-radius * .6, radius * .6)
        ellipse(px, py, .2, rng.uniform(.8, 2), rng.uniform(.4, 1), rng.choice(["#b8ba78", "#a6af6e", "#d6cd91"]))


def pine(x: float, y: float, height: float, seed: int) -> None:
    rng = Random(seed)
    ellipse(x + 8, y + 5, 0, height * .54, 9, "#51795c")
    line([(x, y, 0), (x, y, height * .65)], "#76614a", 6)
    for layer in range(3):
        z = height * (.18 + layer * .24)
        r = height * (.31 - layer * .065)
        peak = (x, y, z + height * .45)
        left, front, right, back = (x - r, y, z), (x, y + r * .65, z), (x + r, y, z), (x, y - r * .65, z)
        polygon([left, back, peak], "#83a985")
        polygon([back, right, peak], "#527f70")
        polygon([left, front, peak], rng.choice(["#477862", "#54896b"]))
        polygon([front, right, peak], "#2f5e52")


def rock(x: float, y: float, radius: float, height: float) -> None:
    peak = (x - radius * .18, y - radius * .14, height)
    a, b, c, d = (x - radius, y, 0), (x, y + radius * .7, 0), (x + radius, y, 0), (x, y - radius * .7, 0)
    polygon([a, d, peak], "#adc0ba")
    polygon([d, c, peak], "#879f9b")
    polygon([a, b, peak], "#9fb2ad")
    polygon([b, c, peak], "#658781")
    snow = [peak, (x + radius * .28, y - radius * .04, height * .62), (x, y + radius * .10, height * .69), (x - radius * .50, y, height * .63)]
    polygon(snow, "#ecedda")


def house(x: float, y: float, size: float, roof: str) -> None:
    h, r = size, size * .68
    a, b, c, d = (x - r, y - r * .6), (x + r, y - r * .6), (x + r, y + r * .6), (x - r, y + r * .6)
    ellipse(x + 8, y + 9, 0, size * 1.55, size * .5, "#637f58")
    polygon([(d[0], d[1], 0), (c[0], c[1], 0), (c[0], c[1], h), (d[0], d[1], h)], "#f6e8bd")
    polygon([(b[0], b[1], 0), (c[0], c[1], 0), (c[0], c[1], h), (b[0], b[1], h)], "#c7b98e")
    ridge_a, ridge_b = (x, a[1] - 3, h * 1.72), (x, d[1] + 3, h * 1.72)
    polygon([(a[0] - 4, a[1] - 3, h), (d[0] - 4, d[1] + 3, h), ridge_b, ridge_a], roof)
    polygon([ridge_a, ridge_b, (c[0] + 4, c[1] + 3, h), (b[0] + 4, b[1] - 3, h)], "#a34f40")
    polygon([(d[0], d[1], h), (c[0], c[1], h), ridge_b], "#f9e9bd")
    polygon([(x - r * .22, d[1] + .2, 0), (x + r * .22, d[1] + .2, 0), (x + r * .22, d[1] + .2, h * .58), (x - r * .22, d[1] + .2, h * .58)], "#496d64")
    ellipse(x, d[1] + .3, h * 1.23, 4, 5, "#597568")
    for side in (-1, 1):
        xx = x + side * r * .63
        polygon([(xx - 3, d[1] + .3, h * .45), (xx + 3, d[1] + .3, h * .45), (xx + 3, d[1] + .3, h * .71), (xx - 3, d[1] + .3, h * .71)], "#58877d")


shadow = Image.new("RGBA", image.size)
shadow_draw = ImageDraw.Draw(shadow)
shadow_draw.ellipse((130 * SCALE, 680 * SCALE, 915 * SCALE, 850 * SCALE), fill=(21, 58, 61, 60))
image.alpha_composite(shadow.filter(ImageFilter.GaussianBlur(27 * SCALE)))
draw = ImageDraw.Draw(image)

centers = [(0, -156, "#b7c28e"), (-135, -78, "#8dba92"), (135, -78, "#c3b38b"), (0, 0, "#bbcb87"), (-135, 78, "#cdad68"), (135, 78, "#b7cb8a"), (0, 156, "#91b4aa")]
for i, (x, y, color) in enumerate(centers):
    tile(x, y, 89, color, i + 41)

rock(-21, -160, 42, 93)
rock(24, -133, 30, 57)
for x, y, height in [(-155, -104, 64), (-115, -87, 75), (-160, -62, 73), (-120, -46, 61)]:
    pine(x, y, height, round(x + y))
for x, y in [(111, -99), (147, -64), (108, -48)]:
    rock(x, y, 22, 25)
for offset in range(-35, 40, 12):
    for step in range(-28, 35, 9):
        xx, yy = -135 + offset, 78 + step
        line([(xx, yy, 0), (xx, yy, 12)], "#a58943", 2)
        line([(xx - 3, yy, 7), (xx, yy, 13), (xx + 3, yy, 9)], "#f6d67d", 3)
line([(-65, 8, .5), (0, 39, .5), (72, 17, .5)], "#dfcf95", 14)
line([(0, 39, .5), (0, 104, .5), (39, 135, .5)], "#dfcf95", 14)
house(-23, -9, 32, "#db8056")
house(35, 14, 23, "#e09462")
house(-7, 59, 25, "#dc8759")
for x, y in [(115, 59), (157, 86), (112, 105)]:
    ellipse(x + 3, y + 3, 0, 14, 5, "#8aaa70")
    line([(x - 4, y, 0), (x - 4, y, 7)], "#716654", 3)
    line([(x + 4, y, 0), (x + 4, y, 7)], "#716654", 3)
    ellipse(x, y, 11, 13, 9, "#fff2d4")
    ellipse(x + 7, y + 1, 12, 5, 5, "#635c51")
line([(-60, 133, 0), (-19, 143, 1), (9, 175, 1), (53, 166, 1)], "#c0d4bc", 5)
line([(-49, 161, 0), (-18, 169, 0)], "#c9dcbe", 4)
line([(3, 204, 0), (31, 210, 0)], "#c9dcbe", 4)
line([(25, 121, 0), (25, 148, 0)], "#b08d57", 11)
line([(39, 127, 0), (39, 154, 0)], "#b08d57", 11)
line([(22, 121, 3), (44, 132, 3)], "#f0d099", 5)
line([(22, 136, 3), (44, 147, 3)], "#f0d099", 5)

output = Path(__file__).resolve().parents[1] / "src-tauri/icons"
output.mkdir(parents=True, exist_ok=True)
image.resize((SIZE, SIZE), Image.Resampling.LANCZOS).save(output / "icon.png")
