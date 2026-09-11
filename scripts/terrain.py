#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["pillow"]
# ///

from json import dumps
from math import cos, pi, sin, sqrt
from pathlib import Path
from random import Random

from PIL import Image, ImageColor, ImageDraw, ImageFilter

SIZE = 512
CENTER = (256, 278)
RADIUS = 224
DEPTH = 18
PALETTES = {
    "forest": ("#71916d", "#9aaf78", "#456d59"),
    "hills": ("#bf9676", "#ddba8a", "#ad6f54"),
    "pasture": ("#a1b881", "#c5cd93", "#779d70"),
    "fields": ("#c4ae66", "#e2cc88", "#a5965c"),
    "mountains": ("#96a49a", "#bcc1af", "#6e8783"),
    "desert": ("#d3bb86", "#ebd39c", "#bca070"),
}


def shade(color: str, change: int) -> tuple[int, int, int, int]:
    return (
        *[max(0, min(255, value + change)) for value in ImageColor.getrgb(color)],
        255,
    )


def line(
    draw: ImageDraw.ImageDraw,
    points: list[tuple[float, float]],
    color: str,
    width: int = 2,
) -> None:
    draw.line(points, fill=color, width=width, joint="curve")


def tree(
    draw: ImageDraw.ImageDraw, x: float, y: float, size: float, rng: Random
) -> None:
    draw.ellipse(
        (x - size * 0.35, y - size * 0.12, x + size * 0.65, y + size * 0.17),
        fill="#526e52",
    )
    line(draw, [(x, y), (x - 2, y - size * 0.6)], "#6d5943", max(2, round(size * 0.08)))
    for layer in range(3):
        by = y - size * (0.18 + layer * 0.21)
        w = size * (0.44 - layer * 0.09)
        top = by - size * 0.49
        polygon = [
            (x - w, by),
            (x - w * 0.7, by - size * 0.15),
            (x - w * 0.56, by - size * 0.12),
            (x, top),
            (x + w * 0.58, by - size * 0.12),
            (x + w, by + size * 0.03),
            (x + w * 0.21, by + size * 0.1),
        ]
        draw.polygon(polygon, fill=rng.choice(["#3f6754", "#49735a", "#587b5c"]))
        draw.polygon(
            [(x - w, by), (x, top), (x - 2, by + size * 0.05)],
            fill=rng.choice(["#81a06e", "#718e66", "#91a873"]),
        )
        line(draw, [(x - w * 0.8, by), (x - 4, top + size * 0.15)], "#aab685", 1)


def mountain(
    draw: ImageDraw.ImageDraw, x: float, y: float, size: float, rng: Random
) -> None:
    peak = (x - size * 0.16, y - size * 1.24)
    left, right, front = (
        (x - size, y - size * 0.1),
        (x + size, y),
        (x + size * 0.2, y + size * 0.35),
    )
    draw.polygon([left, peak, front], fill="#bec3b4")
    draw.polygon([front, peak, right], fill="#6f8987")
    draw.polygon([peak, (x + size * 0.15, y - size * 0.67), right], fill="#8d9f9a")
    draw.polygon([left, (x - size * 0.4, y - size * 0.34), front], fill="#9fae9c")
    draw.polygon(
        [
            peak,
            (peak[0] + size * 0.35, peak[1] + size * 0.48),
            (peak[0] + size * 0.14, peak[1] + size * 0.33),
            (peak[0] - size * 0.12, peak[1] + size * 0.44),
            (peak[0] - size * 0.31, peak[1] + size * 0.42),
        ],
        fill="#ecebd4",
    )
    for _ in range(10):
        t = rng.uniform(0.5, 0.9)
        sx = peak[0] * (1 - t) + front[0] * t
        sy = peak[1] * (1 - t) + front[1] * t
        line(draw, [(sx, sy), (sx + size * 0.18, sy + size * 0.12)], "#a5b0a6", 1)


def sheep(draw: ImageDraw.ImageDraw, x: float, y: float, size: float) -> None:
    draw.ellipse((x - size, y, x + size * 1.3, y + size * 0.4), fill="#819966")
    for dx in [-0.5, 0.45]:
        line(
            draw,
            [(x + dx * size, y - size * 0.15), (x + dx * size, y + size * 0.22)],
            "#6d6850",
            3,
        )
    draw.ellipse(
        (x - size, y - size * 0.8, x + size * 0.8, y + size * 0.08), fill="#dddabb"
    )
    for dx, dy, r in [
        (-0.55, -0.6, 0.45),
        (0, -0.72, 0.43),
        (0.4, -0.58, 0.4),
        (-0.2, -0.28, 0.52),
    ]:
        draw.ellipse(
            (
                x + (dx - r) * size,
                y + (dy - r) * size,
                x + (dx + r) * size,
                y + (dy + r) * size,
            ),
            fill="#f2eccc",
        )
    draw.ellipse(
        (x + size * 0.48, y - size * 0.68, x + size * 1.1, y - size * 0.08),
        fill="#615f4a",
    )
    draw.ellipse(
        (x + size * 0.65, y - size * 0.51, x + size * 0.77, y - size * 0.39),
        fill="#ede8cf",
    )


def terrain(kind: str, variation: int) -> Image.Image:
    rng = Random(f"{kind}:{variation}")
    base, light, dark = PALETTES[kind]
    image = Image.new("RGBA", (SIZE, SIZE))
    draw = ImageDraw.Draw(image)
    cx, cy = CENTER
    vertices = [
        (cx + RADIUS * sin(i * pi / 3), cy + RADIUS * 0.825 * cos(i * pi / 3))
        for i in range(6)
    ]
    shadow = Image.new("RGBA", image.size)
    sd = ImageDraw.Draw(shadow)
    sd.polygon([(x + 4, y + DEPTH + 5) for x, y in vertices], fill=(38, 57, 44, 70))
    image.alpha_composite(shadow.filter(ImageFilter.GaussianBlur(6)))
    draw = ImageDraw.Draw(image)
    for i in range(6):
        a, b = vertices[i], vertices[(i + 1) % 6]
        draw.polygon(
            [a, b, (b[0], b[1] + DEPTH), (a[0], a[1] + DEPTH)],
            fill="#928568" if i < 3 else "#aaa17a",
        )
        line(draw, [(a[0], a[1] + 5), (b[0], b[1] + 5)], "#c2b391", 2)
    mask = Image.new("L", image.size)
    ImageDraw.Draw(mask).polygon(vertices, fill=255)
    surface = Image.new("RGBA", image.size, base)
    d = ImageDraw.Draw(surface)
    for _ in range(170):
        x, y = rng.uniform(40, 460), rng.uniform(80, 460)
        radius = rng.uniform(6, 55)
        color = light if rng.random() < 0.48 else base
        d.ellipse(
            (x - radius, y - radius * 0.45, x + radius, y + radius * 0.45),
            fill=shade(color, rng.randrange(-9, 10)),
        )
    surface = surface.filter(ImageFilter.GaussianBlur(5))
    d = ImageDraw.Draw(surface)
    for _ in range(2300):
        x, y = rng.randrange(SIZE), rng.randrange(SIZE)
        radius = rng.choice([0.5, 0.7, 1, 1.4])
        d.ellipse(
            (x, y, x + radius, y + radius * 0.6),
            fill=shade(rng.choice([base, light, dark]), rng.randrange(-5, 6)),
        )
    if kind == "fields":
        for patch, (px, py, w, h) in enumerate(
            [
                (125, 193, 120, 70),
                (350, 239, 130, 80),
                (207, 388, 155, 60),
                (363, 356, 100, 50),
            ]
        ):
            polygon = [
                (px - w / 2, py - h / 2),
                (px + w / 2, py - h / 2 + 16),
                (px + w / 2 - 24, py + h / 2),
                (px - w / 2 - 20, py + h / 2 - 8),
            ]
            d.polygon(polygon, fill=["#b09454", "#c0a25d", "#c5ac6e", "#b39e64"][patch])
            for row in range(7):
                yy = py - h / 2 + row * h / 7
                for column in range(14):
                    xx = px - w / 2 + column * w / 14 - row * 2
                    height = rng.randrange(7, 14)
                    line(d, [(xx, yy + 3), (xx + 2, yy - height)], "#8e854e", 1)
                    line(
                        d,
                        [
                            (xx - 2, yy - height + 5),
                            (xx + 2, yy - height),
                            (xx + 5, yy - height + 3),
                        ],
                        "#ebd794",
                        2,
                    )
            line(d, [polygon[0], polygon[3], polygon[2]], "#d6c88f", 4)
    elif kind == "desert":
        for x, y in [(170, 209), (335, 234), (243, 371), (133, 331), (377, 324)]:
            ridge = [
                (x - 61, y + 28),
                (x - 17, y - 19),
                (x + 2, y - 27),
                (x + 63, y + 10),
                (x + 89, y + 28),
            ]
            d.polygon(ridge, fill="#b9a071")
            d.polygon([ridge[0], ridge[1], ridge[2], (x + 25, y + 6)], fill="#eed7a2")
            line(d, ridge[:3], "#f5e0b0", 3)
        for _ in range(70):
            x, y = rng.uniform(90, 420), rng.uniform(140, 420)
            line(d, [(x, y), (x + rng.uniform(3, 18), y - 1)], "#d6be88", 1)
    elif kind == "hills":
        for x, y, radius in [
            (215, 166, 65),
            (350, 258, 75),
            (170, 370, 80),
            (345, 395, 51),
        ]:
            for layer in range(4):
                r = radius * (1 - layer * 0.18)
                top = y - layer * 13
                points = [
                    (x + r * sin(i * pi / 5), top + r * 0.34 * cos(i * pi / 5))
                    for i in range(10)
                ]
                d.polygon(
                    [(px, py + 13) for px, py in points], fill=shade(dark, layer * 5)
                )
                d.polygon(points, fill=shade(light, -layer * 7))
                line(d, points[4:9], "#e6c49c", 2)
        d.rounded_rectangle((344, 277, 363, 295), 7, fill="#74583f")
        line(d, [(353, 295), (323, 321)], "#d3b692", 7)
    image.paste(surface, (0, 0), mask)
    draw = ImageDraw.Draw(image)
    line(draw, vertices + [vertices[0]], "#d4c6a0", 3)
    objects = []
    if kind == "forest":
        for _ in range(43):
            x, y = rng.uniform(90, 425), rng.uniform(165, 417)
            dx, dy = abs(x - cx) / RADIUS, abs(y - cy) / (RADIUS * 0.825)
            if (
                dx < 0.72
                and dy + dx / sqrt(3) < 0.9
                and ((x - cx) / 74) ** 2 + ((y - cy) / 58) ** 2 > 1
            ):
                objects.append((x, y, rng.uniform(42, 78)))
        for x, y, size in sorted(objects, key=lambda value: value[1]):
            tree(draw, x, y, size, rng)
    elif kind == "mountains":
        for x, y, size in [
            (230, 202, 80),
            (306, 210, 57),
            (144, 300, 50),
            (363, 352, 68),
            (204, 426, 57),
        ]:
            mountain(draw, x + rng.uniform(-12, 12), y + rng.uniform(-6, 6), size, rng)
        for x, y in [(112, 323), (305, 388), (265, 440)]:
            tree(draw, x, y, 28, rng)
    elif kind == "pasture":
        for x, y in [
            (146, 225),
            (349, 229),
            (311, 374),
            (184, 356),
            (339, 327),
            (140, 281),
        ]:
            sheep(
                draw,
                x + rng.uniform(-14, 14),
                y + rng.uniform(-12, 12),
                rng.uniform(10, 15),
            )
        for row in [0, 1]:
            line(
                draw,
                [(109, 335 - row * 9), (164, 351 - row * 9), (199, 365 - row * 9)],
                "#d2bd88",
                3,
            )
        for x, y in [(109, 335), (137, 343), (164, 351), (199, 365)]:
            line(draw, [(x, y + 4), (x, y - 17)], "#9a8659", 4)
        tree(draw, 239, 190, 48, rng)
        for _ in range(45):
            x, y = rng.uniform(125, 365), rng.uniform(330, 410)
            draw.ellipse(
                (x, y, x + 2, y + 2), fill=rng.choice(["#f3e3b2", "#ede9cf", "#b48885"])
            )
    glaze = Image.new("RGBA", image.size)
    gd = ImageDraw.Draw(glaze)
    for _ in range(5000):
        x, y = rng.randrange(SIZE), rng.randrange(SIZE)
        if image.getpixel((x, y))[3]:
            gd.point((x, y), fill=(255, 245, 205, rng.randrange(8, 27)))
    image.alpha_composite(glaze)
    return image


output = Path(__file__).resolve().parents[1] / "public/art"
output.mkdir(parents=True, exist_ok=True)
atlas = Image.new("RGBA", (SIZE * 6, SIZE * 3))
frames = {}
for variant in range(3):
    for index, kind in enumerate(PALETTES):
        x, y = index * SIZE, variant * SIZE
        atlas.paste(terrain(kind, variant), (x, y))
        frames[f"{kind}-{variant}"] = {
            "frame": {"x": x, "y": y, "w": SIZE, "h": SIZE},
            "rotated": False,
            "trimmed": False,
            "spriteSourceSize": {"x": 0, "y": 0, "w": SIZE, "h": SIZE},
            "sourceSize": {"w": SIZE, "h": SIZE},
        }
atlas.save(output / "terrain.webp", quality=91, method=6)
(output / "terrain.json").write_text(
    dumps(
        {
            "frames": frames,
            "meta": {
                "image": "terrain.webp",
                "format": "RGBA8888",
                "size": {"w": atlas.width, "h": atlas.height},
                "scale": "1",
            },
        },
        separators=(",", ":"),
    )
    + "\n",
    encoding="utf-8",
)
