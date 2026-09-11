#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["pillow"]
# ///

from json import dumps
from math import cos, pi, sin
from pathlib import Path
from random import Random

from PIL import Image, ImageDraw

WIDTH, HEIGHT, SCALE = 256, 224, 3
INK = "#54665d"
CREAM = "#f6e7bd"
GOLD = "#ceac60"


class Painting:
    def __init__(self) -> None:
        self.image = Image.new("RGBA", (WIDTH * SCALE, HEIGHT * SCALE))
        self.draw = ImageDraw.Draw(self.image)

    def polygon(self, points: list[tuple[float, float]], fill: str) -> None:
        self.draw.polygon([(x * SCALE, y * SCALE) for x, y in points], fill=fill)

    def ellipse(
        self,
        box: tuple[float, float, float, float],
        fill: str,
        outline: str | None = None,
        width: int = 1,
    ) -> None:
        self.draw.ellipse(
            tuple(value * SCALE for value in box),
            fill=fill,
            outline=outline,
            width=width * SCALE,
        )

    def rect(
        self, box: tuple[float, float, float, float], fill: str, radius: int = 0
    ) -> None:
        self.draw.rounded_rectangle(
            tuple(value * SCALE for value in box), radius * SCALE, fill=fill
        )

    def line(
        self, points: list[tuple[float, float]], fill: str, width: int = 2
    ) -> None:
        self.draw.line(
            [(x * SCALE, y * SCALE) for x, y in points],
            fill=fill,
            width=width * SCALE,
            joint="curve",
        )

    def arc(
        self,
        box: tuple[float, float, float, float],
        start: int,
        end: int,
        fill: str,
        width: int = 2,
    ) -> None:
        self.draw.arc(
            tuple(value * SCALE for value in box),
            start,
            end,
            fill=fill,
            width=width * SCALE,
        )

    def finish(self) -> Image.Image:
        return self.image.resize((WIDTH, HEIGHT), Image.Resampling.LANCZOS)


def resource(kind: str) -> Image.Image:
    p = Painting()
    p.ellipse((47, 171, 219, 197), "#53624828")
    if kind == "wood":
        for x, y in [(75, 103), (115, 138), (64, 145)]:
            p.polygon(
                [(x, y - 24), (x + 87, y - 58), (x + 113, y - 38), (x + 29, y + 20)],
                "#93653f",
            )
            p.line([(x + 11, y - 18), (x + 89, y - 49)], "#c79f67", 5)
            p.line([(x + 19, y - 7), (x + 103, y - 40)], "#6d5137", 3)
            p.ellipse((x - 9, y - 23, x + 37, y + 24), "#dec391", "#795c3d", 3)
            for radius in [7, 14]:
                p.ellipse(
                    (x + 14 - radius, y - radius, x + 14 + radius, y + radius),
                    "#dec391",
                    "#b7925c",
                    1,
                )
    elif kind == "brick":
        for x, y in [(80, 112), (138, 142), (81, 157)]:
            p.polygon(
                [(x - 37, y - 19), (x + 15, y - 44), (x + 81, y - 17), (x + 22, y + 9)],
                "#d19a76",
            )
            p.polygon(
                [(x - 37, y - 19), (x + 22, y + 9), (x + 22, y + 34), (x - 37, y + 7)],
                "#b97957",
            )
            p.polygon(
                [(x + 22, y + 9), (x + 81, y - 17), (x + 81, y + 8), (x + 22, y + 34)],
                "#91634b",
            )
            p.line([(x - 32, y - 16), (x + 21, y + 7), (x + 76, y - 17)], "#e4b58e", 2)
    elif kind == "wool":
        p.ellipse((58, 56, 196, 181), "#c3bc98")
        for x, y, r in [
            (90, 87, 32),
            (130, 75, 34),
            (166, 98, 32),
            (88, 136, 30),
            (132, 139, 34),
            (163, 145, 27),
        ]:
            p.ellipse((x - r, y - r, x + r, y + r), "#eae3c2")
            for i in range(3):
                p.arc(
                    (x - r + i * 5, y - r + i * 5, x + r - i * 5, y + r - i * 5),
                    195,
                    345,
                    "#fbf5d8",
                    3,
                )
        p.line(
            [(187, 145), (206, 158), (203, 177), (179, 184), (211, 191)], "#d4c79f", 4
        )
    elif kind == "grain":
        for x, y, bend in [(101, 172, -15), (127, 186, 0), (147, 174, 17)]:
            p.line([(128, 186), (x, y - 75), (x + bend, y - 130)], "#8e965a", 4)
            for n in range(6):
                yy = y - 74 - n * 10
                xx = x + bend * n / 6
                p.ellipse((xx - 16, yy - 11, xx + 1, yy + 2), GOLD)
                p.ellipse((xx - 1, yy - 17, xx + 15, yy - 4), "#ead092")
                p.line([(xx - 13, yy - 9), (xx - 19, yy - 20)], "#bc9d5e", 1)
                p.line([(xx + 11, yy - 15), (xx + 18, yy - 27)], "#cbb77a", 1)
        p.line([(110, 161), (149, 155)], "#bd8562", 8)
    elif kind == "ore":
        for x, y, size in [(96, 154, 48), (147, 153, 68), (186, 170, 31)]:
            p.polygon(
                [
                    (x - size, y),
                    (x - size * 0.4, y - size * 1.2),
                    (x + size * 0.4, y - size * 0.98),
                    (x + size * 0.7, y + size * 0.35),
                    (x - size * 0.35, y + size * 0.4),
                ],
                "#849591",
            )
            p.polygon(
                [
                    (x - size, y),
                    (x - size * 0.4, y - size * 1.2),
                    (x, y - size * 0.35),
                    (x - size * 0.35, y + size * 0.4),
                ],
                "#bbbfad",
            )
            p.polygon(
                [
                    (x, y - size * 0.35),
                    (x + size * 0.4, y - size * 0.98),
                    (x + size * 0.7, y + size * 0.35),
                ],
                "#5f7775",
            )
            p.line(
                [
                    (x - size * 0.3, y),
                    (x - size * 0.05, y - size * 0.27),
                    (x + size * 0.2, y - size * 0.22),
                ],
                "#d6bd7b",
                2,
            )
    elif kind == "cloth":
        p.polygon([(54, 95), (168, 68), (204, 166), (94, 189)], "#c38b87")
        p.polygon([(93, 128), (172, 104), (203, 166), (94, 189)], "#a87174")
        p.line([(57, 95), (83, 154), (94, 183), (198, 162)], "#e1bbb0", 4)
        p.ellipse((47, 60, 112, 122), "#dab4a1")
        p.ellipse((61, 70, 101, 113), "#b88b84")
        p.arc((69, 78, 94, 106), 20, 345, "#efceba", 4)
        for n in range(8):
            p.line(
                [(108 + n * 10, 177 - n * 2), (110 + n * 10, 188 - n * 2)], "#e6c7b0", 2
            )
    elif kind == "coin":
        for x, y, layers in [(105, 166, 4), (157, 171, 2), (151, 123, 4)]:
            for n in range(layers):
                yy = y - n * 10
                p.ellipse((x - 35, yy - 8, x + 35, yy + 13), "#a58042")
                p.ellipse((x - 35, yy - 14, x + 35, yy + 6), "#dcc185", "#eedba8", 2)
                p.ellipse((x - 24, yy - 10, x + 24, yy + 1), "#d4b56d", "#b4924c", 1)
            p.polygon(
                [(x, yy - 10), (x + 8, yy - 5), (x, yy), (x - 8, yy - 5)], "#f0dca4"
            )
    else:
        p.polygon([(66, 63), (190, 80), (180, 180), (61, 162)], "#f0e2b9")
        p.polygon([(63, 152), (184, 168), (180, 180), (61, 164)], "#d6cba6")
        p.ellipse((51, 48, 86, 168), "#dbcba0")
        p.ellipse((55, 46, 86, 68), "#f9edc8", "#beaa7b", 2)
        p.ellipse((64, 51, 78, 63), "#bba878")
        for y in range(92, 150, 10):
            p.line([(99, y), (161, y + 8)], "#bcb894", 2)
        p.ellipse((151, 135, 181, 165), "#749681")
        p.line([(159, 144), (173, 156)], "#bfd1ac", 2)
    return p.finish()


def ship(p: Painting) -> None:
    p.ellipse((45, 168, 217, 189), "#89aba2")
    for y in [181, 191]:
        p.line([(52, y), (90, y + 2), (130, y - 1), (186, y + 2)], "#cee0cc", 2)
    p.polygon([(48, 146), (218, 137), (183, 173), (81, 176)], "#9a704e")
    p.polygon([(59, 151), (201, 144), (183, 155), (75, 166)], "#ceaa76")
    p.line([(128, 153), (128, 39)], INK, 4)
    p.polygon([(133, 47), (195, 127), (133, 130)], CREAM)
    p.polygon([(120, 61), (66, 135), (120, 134)], "#bfd1b1")
    p.line([(135, 57), (176, 118)], "#e4d4a7", 2)
    p.polygon([(128, 40), (160, 48), (128, 57)], "#b67560")


def shield(p: Painting, broken: bool = False) -> None:
    p.line([(84, 175), (183, 45)], "#788881", 8)
    p.line([(78, 159), (102, 178)], GOLD, 7)
    p.polygon([(77, 69), (129, 52), (188, 76), (179, 143), (132, 181), (86, 143)], GOLD)
    p.polygon(
        [(85, 76), (129, 60), (179, 81), (171, 139), (132, 170), (94, 139)], "#66868a"
    )
    p.polygon([(129, 60), (129, 168), (171, 139), (179, 81)], "#496f79")
    p.line([(114, 93), (141, 88), (132, 146)], "#eee4be", 7)
    if broken:
        p.line([(146, 58), (124, 104), (141, 117), (113, 164)], "#ece2c6", 7)


def book(p: Painting) -> None:
    p.polygon(
        [
            (55, 72),
            (117, 64),
            (134, 78),
            (196, 68),
            (201, 158),
            (139, 180),
            (120, 174),
            (63, 180),
        ],
        "#65827c",
    )
    p.polygon(
        [(62, 63), (116, 60), (130, 73), (134, 167), (117, 157), (65, 165)], "#eee1b9"
    )
    p.polygon([(132, 73), (190, 61), (196, 150), (137, 168)], "#f8edca")
    for y in range(84, 144, 10):
        p.line([(75, y), (112, y - 2)], "#c5b996", 2)
        p.line([(145, y + 3), (181, y - 4)], "#c5b996", 2)
    p.line([(128, 78), (133, 166)], "#b0a380", 3)
    p.polygon([(166, 65), (181, 63), (186, 148), (179, 142), (171, 154)], "#bd806a")


def bottle(p: Painting, x: int, y: int, color: str) -> None:
    p.rect((x - 13, y - 105, x + 13, y - 73), "#bad0be", 4)
    p.ellipse((x - 38, y - 80, x + 38, y), "#bad0be")
    p.ellipse((x - 32, y - 54, x + 32, y - 6), color)
    p.rect((x - 15, y - 111, x + 15, y - 101), "#af8e61", 3)
    p.arc((x - 30, y - 75, x + 29, y - 9), 120, 244, "#eef2d4", 4)
    p.ellipse((x - 10, y - 33, x - 2, y - 25), "#e9e9bf")


def card(kind: str, items: dict[str, Image.Image]) -> Image.Image:
    p = Painting()
    p.ellipse((35, 21, 224, 209), "#ebdfbb")
    p.ellipse((48, 34, 211, 197), "#f5edcf")
    p.line([(48, 183), (216, 183)], "#c9bf9c", 2)
    if kind in {"commercialHarbor", "merchantFleet"}:
        ship(p)
        if kind == "commercialHarbor":
            p.rect((38, 158, 69, 173), "#b69462")
            p.rect((40, 138, 64, 158), "#d1b783")
    elif kind in {"knight", "encouragement", "treason", "intrigue", "sabotage"}:
        shield(p, kind in {"treason", "sabotage"})
        if kind == "encouragement":
            p.line([(60, 157), (60, 35)], INK, 4)
            p.polygon([(62, 38), (112, 50), (100, 67), (112, 78), (62, 69)], "#bb785f")
        if kind == "intrigue":
            p.polygon([(139, 160), (173, 75), (181, 78), (157, 168)], "#eee2b6")
    elif kind in {"constitution", "printing", "espionage", "diplomacy", "invention"}:
        book(p)
        if kind == "espionage":
            p.polygon([(74, 125), (148, 114), (159, 171), (85, 180)], "#e0cca1")
            p.line([(74, 125), (119, 148), (148, 114)], "#b3996e", 3)
            p.ellipse((109, 143, 125, 159), "#a46759")
        elif kind == "diplomacy":
            p.polygon([(147, 173), (168, 54), (190, 32), (191, 84)], "#b9cab5")
            p.line([(147, 178), (187, 42)], "#6a8e7e", 3)
        elif kind == "printing":
            p.line([(74, 40), (74, 173), (191, 173), (191, 40), (74, 40)], "#856c4a", 9)
            p.line([(132, 37), (132, 115)], "#b7a572", 7)
            p.line([(107, 104), (157, 104)], "#9d8460", 8)
        elif kind == "invention":
            for angle in range(12):
                a = angle * pi / 6
                p.line(
                    [
                        (182 + cos(a) * 25, 137 + sin(a) * 25),
                        (182 + cos(a) * 34, 137 + sin(a) * 34),
                    ],
                    GOLD,
                    9,
                )
            p.ellipse((156, 111, 208, 163), GOLD)
            p.ellipse((165, 120, 199, 154), "#f4e7c2")
    elif kind in {"alchemy", "medicine"}:
        bottle(p, 97, 183, "#83a18a")
        bottle(p, 166, 162, "#bb988a")
        if kind == "medicine":
            p.line([(72, 190), (166, 72)], "#67886a", 4)
            for x, y in [(106, 145), (128, 119), (149, 92)]:
                p.ellipse((x - 26, y - 13, x, y + 2), "#88a377")
                p.ellipse((x + 3, y - 22, x + 24, y - 4), "#a8b585")
    elif kind in {"engineering", "crane", "irrigation", "roadBuilding"}:
        for y, row in [(154, 0), (131, 1), (108, 0)]:
            for x in range(66 + row * 16, 193, 33):
                p.rect((x, y, x + 29, y + 20), "#a2aa95", 2)
                p.line([(x + 2, y + 2), (x + 27, y + 2)], "#d7d6b9", 2)
        if kind == "crane":
            p.line([(104, 174), (104, 39), (208, 54)], "#93764d", 8)
            p.line([(111, 104), (183, 52)], "#aa8d5a", 5)
            p.line([(186, 54), (186, 108)], INK, 2)
            p.rect((172, 106, 206, 128), "#bfb294", 3)
        elif kind == "irrigation":
            p.rect((56, 134, 196, 175), "#8baaa1", 10)
            p.line([(56, 148), (199, 148)], "#d7e4cc", 3)
            p.arc((63, 86, 133, 169), 182, 360, "#b8bd9e", 9)
        elif kind == "roadBuilding":
            p.polygon([(78, 198), (145, 202), (198, 75), (167, 61)], "#d2b98a")
            for y in range(87, 187, 17):
                p.line(
                    [(176 - (y - 80) * 0.6, y), (192 - (y - 80) * 0.35, y + 4)],
                    "#a89470",
                    3,
                )
    elif kind in {"smithing", "mining"}:
        p.polygon(
            [
                (70, 132),
                (198, 132),
                (168, 153),
                (160, 177),
                (174, 182),
                (96, 182),
                (111, 174),
                (103, 151),
            ],
            "#82928b",
        )
        p.polygon([(58, 120), (92, 107), (185, 116), (198, 132), (72, 137)], "#b7beaa")
        p.line([(111, 153), (161, 49)], "#a28051", 10)
        p.polygon([(136, 39), (182, 55), (175, 78), (128, 62)], "#748880")
        p.line([(140, 43), (178, 57)], "#d0d0b7", 3)
    elif kind == "wedding":
        for x in [108, 152]:
            p.ellipse((x - 35, 96, x + 35, 169), "#f5edcf", GOLD, 12)
            p.arc((x - 32, 100, x + 31, 163), 200, 290, "#f1dca0", 4)
        p.line(
            [(87, 176), (113, 151), (132, 188), (153, 159), (190, 177)], "#bf8980", 7
        )
    elif kind == "victoryPoint":
        p.polygon(
            [
                (65, 107),
                (80, 169),
                (184, 169),
                (199, 106),
                (168, 124),
                (131, 70),
                (101, 123),
            ],
            GOLD,
        )
        p.line([(83, 158), (181, 158)], "#ead49c", 6)
        for x, y in [(65, 105), (131, 70), (199, 104)]:
            p.ellipse((x - 6, y - 6, x + 6, y + 6), "#eee0ad")
    else:
        selected = {
            "guildDues": ["coin", "paper"],
            "merchant": ["cloth", "coin"],
            "resourceMonopoly": ["grain", "wood"],
            "commodityMonopoly": ["coin", "cloth"],
            "irrigation": ["grain"],
            "plenty": ["grain", "wool"],
            "monopoly": ["wood", "brick"],
            "taxation": ["coin", "ore"],
        }.get(kind, ["grain", "ore"])
        for index, item in enumerate(selected):
            illustration = items[item].resize(
                (165 * SCALE, 145 * SCALE), Image.Resampling.LANCZOS
            )
            p.image.alpha_composite(
                illustration,
                (round((25 + index * 50) * SCALE), round((55 + index * 31) * SCALE)),
            )
        if kind == "merchant":
            p.line([(53, 175), (53, 60), (199, 60), (199, 177)], "#9a8256", 4)
            p.polygon([(44, 62), (66, 39), (186, 39), (208, 62)], "#be826d")
    rng = Random(kind)
    for _ in range(350):
        x, y = rng.randrange(50, 215), rng.randrange(38, 188)
        p.ellipse((x, y, x + 0.4, y + 0.4), "#fff6d629")
    return p.finish()


output = Path(__file__).resolve().parents[1] / "public/art"
output.mkdir(parents=True, exist_ok=True)
items = {
    kind: resource(kind)
    for kind in ["wood", "brick", "wool", "grain", "ore", "cloth", "coin", "paper"]
}
illustrations = dict(items)
for kind in [
    "knight",
    "victoryPoint",
    "roadBuilding",
    "plenty",
    "monopoly",
    "commercialHarbor",
    "guildDues",
    "merchant",
    "merchantFleet",
    "resourceMonopoly",
    "commodityMonopoly",
    "diplomacy",
    "encouragement",
    "espionage",
    "intrigue",
    "sabotage",
    "taxation",
    "treason",
    "constitution",
    "wedding",
    "alchemy",
    "crane",
    "engineering",
    "invention",
    "irrigation",
    "medicine",
    "mining",
    "smithing",
    "printing",
]:
    illustrations[kind] = card(kind, items)
rows = (len(illustrations) + 7) // 8
atlas = Image.new("RGBA", (WIDTH * 8, HEIGHT * rows))
frames = {}
for index, (name, image) in enumerate(illustrations.items()):
    x, y = index % 8 * WIDTH, index // 8 * HEIGHT
    atlas.paste(image, (x, y))
    frames[name] = {"x": x, "y": y, "w": WIDTH, "h": HEIGHT}
atlas.save(output / "illustrations.webp", quality=93, method=6)
(output.parent.parent / "src/art.json").write_text(
    dumps(
        {"frames": frames, "width": atlas.width, "height": atlas.height},
        separators=(",", ":"),
    )
    + "\n",
    encoding="utf-8",
)
