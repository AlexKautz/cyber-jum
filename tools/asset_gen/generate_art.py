"""Generate all PNG art assets for Cyber Jum.

Every sprite is described as ASCII pixel art (one character per pixel) so the
art is reviewable and editable as plain text.  Tilesets and the title screen
are drawn procedurally.  Output goes to ../../assets/ with 4x upscaled copies
in ../../assets/preview/ for human review.

GBA constraint: sprites and backgrounds are 4 bits per pixel, i.e. at most
15 colors + transparency per image.  Each emitted PNG is checked against that.

Run:  uv run python generate_art.py
"""

from __future__ import annotations

import random
from pathlib import Path

from PIL import Image

ASSETS = Path(__file__).resolve().parents[2] / "assets"
SPRITES = ASSETS / "sprites"
BACKGROUNDS = ASSETS / "backgrounds"
PREVIEW = ASSETS / "preview"

# ---------------------------------------------------------------------------
# Palette (PICO-8 inspired).  '.' is transparent.
# ---------------------------------------------------------------------------
PALETTE: dict[str, tuple[int, int, int] | None] = {
    ".": None,                 # transparent
    "K": (26, 28, 44),         # outline / near-black
    "N": (51, 60, 87),         # navy
    "a": (90, 95, 120),        # dark gray
    "A": (164, 168, 180),      # gray
    "W": (244, 244, 244),      # white
    "S": (255, 205, 117),      # skin
    "s": (239, 125, 87),       # skin shade / mouth
    "H": (94, 54, 67),         # hair brown
    "T": (133, 76, 48),        # trunk / coat brown
    "B": (65, 166, 246),       # hero blue
    "D": (59, 93, 201),        # hero dark blue
    "C": (115, 239, 232),      # cyan accent
    "R": (177, 62, 83),        # deep red
    "r": (255, 0, 77),         # bright red
    "G": (56, 183, 100),       # green
    "g": (37, 113, 121),       # dark green / teal
    "Y": (255, 236, 39),       # yellow
    "O": (255, 163, 0),        # orange
    "P": (181, 80, 136),       # purple
    "p": (126, 37, 83),        # dark purple
    "M": (255, 119, 168),      # pink / magenta
}


def sprite_from_ascii(rows: list[str]) -> Image.Image:
    """Convert ASCII pixel art rows into an RGBA image."""
    width = len(rows[0])
    for i, row in enumerate(rows):
        assert len(row) == width, f"row {i} has width {len(row)}, expected {width}"
        assert all(ch in PALETTE for ch in row), f"unknown color char in row {i}: {row}"
    img = Image.new("RGBA", (width, len(rows)), (0, 0, 0, 0))
    px = img.load()
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            color = PALETTE[ch]
            if color is not None:
                px[x, y] = (*color, 255)
    return img


def hstrip(frames: list[Image.Image]) -> Image.Image:
    """Lay frames out left-to-right in one horizontal strip."""
    w, h = frames[0].size
    strip = Image.new("RGBA", (w * len(frames), h), (0, 0, 0, 0))
    for i, frame in enumerate(frames):
        assert frame.size == (w, h)
        strip.paste(frame, (i * w, 0))
    return strip


def count_colors(img: Image.Image) -> int:
    colors = {p[:3] for p in img.getdata() if p[3] != 0}
    return len(colors)


def save(img: Image.Image, rel: str) -> None:
    """Save a PNG plus a 4x preview, enforcing the 15-color GBA limit."""
    n = count_colors(img)
    assert n <= 15, f"{rel} uses {n} colors; GBA 4bpp allows at most 15 + transparent"
    out = ASSETS / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    img.save(out)
    big = img.resize((img.width * 4, img.height * 4), Image.NEAREST)
    preview = PREVIEW / rel.replace("/", "_")
    preview.parent.mkdir(parents=True, exist_ok=True)
    # Previews get a dark backdrop so white/light pixels are visible.
    backdrop = Image.new("RGBA", big.size, (40, 44, 60, 255))
    backdrop.alpha_composite(big)
    backdrop.save(preview)
    print(f"  {rel:42s} {img.width}x{img.height}  {n} colors")


# ---------------------------------------------------------------------------
# Hero, top-down (16x16).  Frames are head+torso templates joined with leg
# variants so walking animations stay consistent.  Only right-facing side
# frames are drawn; the game flips them horizontally for left.
# ---------------------------------------------------------------------------
HEAD_DOWN = [
    "................",
    ".....KKKKKK.....",
    "....KHHHHHHK....",
    "...KHHHHHHHHK...",
    "...KHSHHHHSHK...",
    "...KSSSSSSSSK...",
    "...KSKSSSSKSK...",
    "...KSSSSSSSSK...",
    "....KSSssSSK....",
    ".....KKKKKK.....",
]
TORSO_DOWN = [
    "....KBBBBBBK....",
    "...KSBBCCBBSK...",
    "...KKBBCCBBKK...",
]
HEAD_UP = [
    "................",
    ".....KKKKKK.....",
    "....KHHHHHHK....",
    "...KHHHHHHHHK...",
    "...KHHHHHHHHK...",
    "...KHHHHHHHHK...",
    "...KHHHHHHHHK...",
    "...KHHHHHHHHK...",
    "....KHHHHHHK....",
    ".....KKKKKK.....",
]
TORSO_UP = [
    "....KBBBBBBK....",
    "...KSBBBBBBSK...",
    "...KKBBBBBBKK...",
]
HEAD_RIGHT = [
    "................",
    ".....KKKKKK.....",
    "....KHHHHHHK....",
    "...KHHHHHHHHK...",
    "...KHHSSSSSSK...",
    "...KHHSSKSSSK...",
    "...KHHSSSSSSK...",
    "...KHHSSSssSK...",
    "....KHSSSSSK....",
    ".....KKKKKK.....",
]
TORSO_RIGHT = [
    "....KBBBBBBK....",
    "....KBBBBBSK....",
    "....KKBBBBKK....",
]

LEGS_FRONT_IDLE = [
    "....KDDKKDDK....",
    "....KDD..DDK....",
    "....KKK..KKK....",
]
LEGS_FRONT_STEP_L = [
    "....KDDKKDDK....",
    "....KDD..KKK....",
    "....KKK.........",
]
LEGS_FRONT_STEP_R = [
    "....KDDKKDDK....",
    "....KKK..DDK....",
    ".........KKK....",
]
LEGS_SIDE_IDLE = [
    ".....KDDDDK.....",
    ".....KDD.DK.....",
    ".....KKK.KK.....",
]
LEGS_SIDE_STEP_A = [
    ".....KDDDDK.....",
    "....KDD.KDDK....",
    "....KKK..KKK....",
]
LEGS_SIDE_STEP_B = [
    ".....KDDDDK.....",
    "......KDDK......",
    "......KKKK......",
]


def hero_topdown() -> Image.Image:
    """9-frame strip: down/up/right, each idle + two walking steps."""
    frames = []
    for head, torso, legs in [
        (HEAD_DOWN, TORSO_DOWN, LEGS_FRONT_IDLE),
        (HEAD_DOWN, TORSO_DOWN, LEGS_FRONT_STEP_L),
        (HEAD_DOWN, TORSO_DOWN, LEGS_FRONT_STEP_R),
        (HEAD_UP, TORSO_UP, LEGS_FRONT_IDLE),
        (HEAD_UP, TORSO_UP, LEGS_FRONT_STEP_L),
        (HEAD_UP, TORSO_UP, LEGS_FRONT_STEP_R),
        (HEAD_RIGHT, TORSO_RIGHT, LEGS_SIDE_IDLE),
        (HEAD_RIGHT, TORSO_RIGHT, LEGS_SIDE_STEP_A),
        (HEAD_RIGHT, TORSO_RIGHT, LEGS_SIDE_STEP_B),
    ]:
        frames.append(sprite_from_ascii(head + torso + legs))
    return hstrip(frames)


# ---------------------------------------------------------------------------
# Hero, platformer mode (16x16): same hero wearing a cyber visor helmet.
# Frames: idle, run1, run2, jump, hurt.
# ---------------------------------------------------------------------------
HELMET_HEAD = [
    "................",
    ".....KKKKKK.....",
    "....KAAAAAAK....",
    "...KAAAAAAAAK...",
    "...KACCCCCCAK...",
    "...KACCCCCCAK...",
    "...KAAAAAAAAK...",
    "....KSSssSSK....",
    ".....KKKKKK.....",
]
HELMET_HEAD_HURT = HELMET_HEAD[:4] + [
    "...KAMMMMMMAK...",
    "...KAMMMMMMAK...",
] + HELMET_HEAD[6:]

PLAT_TORSO = [
    "....KBBBBBBK....",
    "...KSBBCCBBSK...",
    "...KKBBCCBBKK...",
]
PLAT_LEGS_IDLE = [
    "....KDDKKDDK....",
    "....KDD..DDK....",
    "....KDD..DDK....",
    "....KKK..KKK....",
]
PLAT_LEGS_RUN1 = [
    "....KDDKKDDK....",
    "...KDDK..KDDK...",
    "..KDDK....KDDK..",
    "..KKK......KKK..",
]
PLAT_LEGS_RUN2 = [
    "....KDDKKDDK....",
    "....KDDKKDDK....",
    ".....KDDDDK.....",
    ".....KKKKKK.....",
]
PLAT_LEGS_JUMP = [
    "....KDDKKDDK....",
    "...KDDK..KDDK...",
    "...KKK....KKK...",
    "................",
]


def hero_platform() -> Image.Image:
    frames = [
        sprite_from_ascii(HELMET_HEAD + PLAT_TORSO + PLAT_LEGS_IDLE),
        sprite_from_ascii(HELMET_HEAD + PLAT_TORSO + PLAT_LEGS_RUN1),
        sprite_from_ascii(HELMET_HEAD + PLAT_TORSO + PLAT_LEGS_RUN2),
        sprite_from_ascii(HELMET_HEAD + PLAT_TORSO + PLAT_LEGS_JUMP),
        sprite_from_ascii(HELMET_HEAD_HURT + PLAT_TORSO + PLAT_LEGS_JUMP),
    ]
    return hstrip(frames)


# ---------------------------------------------------------------------------
# The five challengers (16x16, facing down, one frame each).
# 1 kid with red cap, 2 old man, 3 purple-haired woman, 4 hooded ranger,
# 5 robot (the final, most "cyber" challenger).
# ---------------------------------------------------------------------------
NPC_KID = [
    "................",
    ".....KKKKKK.....",
    "....KRRRRRRK....",
    "...KRRRRRRRRK...",
    "...KKKKKKKKKK...",
    "...KSSSSSSSSK...",
    "...KSKSSSSKSK...",
    "...KSSSSSSSSK...",
    "....KSSssSSK....",
    ".....KKKKKK.....",
    "....KrrrrrrK....",
    "...KSrrWWrrSK...",
    "...KKrrWWrrKK...",
    "....KNNKKNNK....",
    "....KNN..NNK....",
    "....KKK..KKK....",
]
NPC_OLDMAN = [
    "................",
    ".....KKKKKK.....",
    "....KSSSSSSK....",
    "...KSSSSSSSSK...",
    "...KASSSSSSAK...",
    "...KSSSSSSSSK...",
    "...KSKSSSSKSK...",
    "...KSAAAAAASK...",
    "....KAAAAAAK....",
    ".....KAAAAK.....",
    "....KTTTTTTK....",
    "...KSTTTTTTSK...",
    "...KKTTTTTTKK...",
    "....KTTKKTTK....",
    "....KTT..TTK....",
    "....KKK..KKK....",
]
NPC_WOMAN = [
    "................",
    ".....KKKKKK.....",
    "....KPPPPPPK....",
    "...KPPPPPPPPK...",
    "...KPSPPPPSPK...",
    "...KPSSSSSSPK...",
    "...KPKSSSSKPK...",
    "...KPSSSSSSPK...",
    "...KPSSssSSPK...",
    "...KPPKKKKPPK...",
    "...KPGGGGGGPK...",
    "...KSGGGGGGSK...",
    "....KGGGGGGK....",
    "....KGGGGGGK....",
    "....KSS..SSK....",
    "....KKK..KKK....",
]
NPC_RANGER = [
    "................",
    ".....KKKKKK.....",
    "....KggggggK....",
    "...KggggggggK...",
    "...KgKKKKKKgK...",
    "...KgKSSSSKgK...",
    "...KgKsSSsKgK...",
    "...KgKSSSSKgK...",
    "....KgKKKKgK....",
    ".....KggggK.....",
    "....KGGGGGGK....",
    "...KSGGGGGGSK...",
    "...KKGGGGGGKK...",
    "....KggKKggK....",
    "....Kgg..ggK....",
    "....KKK..KKK....",
]
NPC_ROBOT = [
    "......KK........",
    "......KYK.......",
    ".....KKKKKK.....",
    "....KAAAAAAK....",
    "...KAAAAAAAAK...",
    "...KAKYKKYKAK...",
    "...KAAAAAAAAK...",
    "...KAKWWWWKAK...",
    "....KAAAAAAK....",
    ".....KKKKKK.....",
    "....KaaaaaaK....",
    "...KAaaYYaaAK...",
    "...KKaaYYaaKK...",
    "....KaaKKaaK....",
    "....Kaa..aaK....",
    "....KKK..KKK....",
]


def npcs() -> Image.Image:
    return hstrip([sprite_from_ascii(n)
                   for n in [NPC_KID, NPC_OLDMAN, NPC_WOMAN, NPC_RANGER, NPC_ROBOT]])


# ---------------------------------------------------------------------------
# Enemy: "glitch bug" (16x16).  Frames: walk1, walk2, squashed.
# ---------------------------------------------------------------------------
BUG_BODY = [
    "................",
    "................",
    "....KK....KK....",
    "...KppK..KppK...",
    "...KpPPKKPPpK...",
    "..KPPPPPPPPPPK..",
    ".KPPWWPPPPWWPPK.",
    ".KPPWKPPPPWKPPK.",
    ".KPPPPPPPPPPPPK.",
    ".KPMMPPPPPPMMPK.",
    ".KPPPPPPPPPPPPK.",
    "..KPPPPPPPPPPK..",
]
BUG_LEGS_A = [
    "..KpKpKppKpKpK..",
    "..Kp.Kp..pK.pK..",
    "................",
    "................",
]
BUG_LEGS_B = [
    "..KpKpKppKpKpK..",
    "..pK.pK..Kp.Kp..",
    "................",
    "................",
]
BUG_SQUASHED = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "....KK....KK....",
    "...KppKKKKppK...",
    "..KPPPPPPPPPPK..",
    ".KPWKPPPPPPWKPK.",
    ".KPPPPMMMMPPPPK.",
    ".KpKpKpppKpKpKp.",
    "................",
]


def enemy() -> Image.Image:
    return hstrip([
        sprite_from_ascii(BUG_BODY + BUG_LEGS_A),
        sprite_from_ascii(BUG_BODY + BUG_LEGS_B),
        sprite_from_ascii(BUG_SQUASHED),
    ])


# ---------------------------------------------------------------------------
# Goal flag (16x32, two waving frames).
# ---------------------------------------------------------------------------
FLAG_A = [
    "................",
    ".KK.............",
    ".KAKKKKKKKKKKK..",
    ".KAKWWKKWWKKWK..",
    ".KAKWWKKWWKKWK..",
    ".KAKKKWWKKWWKK..",
    ".KAKKKWWKKWWKK..",
    ".KAKWWKKWWKKWK..",
    ".KAKWWKKWWKKWK..",
    ".KAKKKWWKKWWKK..",
    ".KAKKKKKKKKKKK..",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    ".KAK............",
    "KKAKK...........",
    "KKKKK...........",
]
FLAG_B = [row.replace("WWKKWWKKW", "WKKWWKKWW") if 2 < i < 11 else row
          for i, row in enumerate(FLAG_A)]


def flag() -> Image.Image:
    return hstrip([sprite_from_ascii(FLAG_A), sprite_from_ascii(FLAG_B)])


# ---------------------------------------------------------------------------
# UI sprites (8x8): full heart, empty heart, alert "!".
# ---------------------------------------------------------------------------
HEART_FULL = [
    "........",
    ".KK..KK.",
    "KrrKKrrK",
    "KrWrrrrK",
    "KrrrrrrK",
    ".KrrrrK.",
    "..KrrK..",
    "...KK...",
]
HEART_EMPTY = [
    "........",
    ".KK..KK.",
    "KNNKKNNK",
    "KNNNNNNK",
    "KNNNNNNK",
    ".KNNNNK.",
    "..KNNK..",
    "...KK...",
]
ALERT = [
    "..KKKK..",
    ".KYYYYK.",
    ".KYYYYK.",
    "..KYYK..",
    "..KYYK..",
    "...KK...",
    "..KYYK..",
    "..KKKK..",
]


def ui() -> Image.Image:
    return hstrip([sprite_from_ascii(s) for s in [HEART_FULL, HEART_EMPTY, ALERT]])


# ---------------------------------------------------------------------------
# Top-down tileset.  8x8 tiles in a strip; the game references them by index.
#  0 grass        1 grass tufts   2 grass flower  3 dirt path
#  4 path edge L  5 path edge R   6 tree NW       7 tree NE
#  8 tree SW      9 tree SE      10 bush         11 tall grass
# ---------------------------------------------------------------------------
def grass_tile(rng: random.Random, tufts: int, flower: bool = False) -> Image.Image:
    img = Image.new("RGBA", (8, 8), (*PALETTE["G"], 255))
    px = img.load()
    for _ in range(tufts):
        x, y = rng.randrange(8), rng.randrange(8)
        px[x, y] = (*PALETTE["g"], 255)
    if flower:
        px[3, 3] = (*PALETTE["Y"], 255)
        px[5, 5] = (*PALETTE["M"], 255)
    return img


def dirt_tile(rng: random.Random, grass_left: bool = False,
              grass_right: bool = False) -> Image.Image:
    img = Image.new("RGBA", (8, 8), (*PALETTE["S"], 255))
    px = img.load()
    for _ in range(5):
        x, y = rng.randrange(8), rng.randrange(8)
        px[x, y] = (*PALETTE["T"], 255)
    if grass_left:
        for y in range(8):
            px[0, y] = (*PALETTE["G"], 255)
            if y % 2 == 0:
                px[1, y] = (*PALETTE["g"], 255)
    if grass_right:
        for y in range(8):
            px[7, y] = (*PALETTE["G"], 255)
            if y % 2 == 1:
                px[6, y] = (*PALETTE["g"], 255)
    return img


TREE = [  # 16x16, sliced into four 8x8 tiles
    "....KKKKKKK.....",
    "..KKGGGGGGGKK...",
    ".KGGGgGGGGGGGK..",
    ".KGGGGGGGgGGGK..",
    "KGGgGGGGGGGGGGK.",
    "KGGGGGGgGGGGgGK.",
    "KGgGGGGGGGGGGGK.",
    ".KGGGgGGGGgGGK..",
    ".KGGGGGGGGGGGK..",
    "..KKGGgGGGKKK...",
    "....KKKKKKK.....",
    "......KTTK......",
    "......KTTK......",
    ".....KTTTTK.....",
    "....KTTTTTTK....",
    "....KKKKKKKK....",
]
BUSH = [
    "........",
    "..KKKK..",
    ".KGGGGK.",
    "KGgGGgGK",
    "KGGGGGGK",
    "KgGGgGGK",
    ".KKKKKK.",
    "........",
]
TALL_GRASS = [
    "........",
    "g.g..g.g",
    "gg.gg.gg",
    ".ggGgg.g",
    "gGggGggG",
    "gggGgggg",
    "GgGggGgG",
    "gGggGggG",
]


def topdown_tiles() -> Image.Image:
    rng = random.Random(7)
    tree = sprite_from_ascii(TREE)
    tiles = [
        grass_tile(rng, 4),
        grass_tile(rng, 8),
        grass_tile(rng, 4, flower=True),
        dirt_tile(rng),
        dirt_tile(rng, grass_left=True),
        dirt_tile(rng, grass_right=True),
        tree.crop((0, 0, 8, 8)),
        tree.crop((8, 0, 16, 8)),
        tree.crop((0, 8, 8, 16)),
        tree.crop((8, 8, 16, 16)),
        sprite_from_ascii(BUSH),
        sprite_from_ascii(TALL_GRASS),
    ]
    return hstrip(tiles)


# ---------------------------------------------------------------------------
# Platformer tileset (cyber/synthwave look).  8x8 tiles:
#  0 block top     1 block fill    2 platform L    3 platform M
#  4 platform R    5 sky           6 star sky      7 grid hill
#  8 cloud L       9 cloud R      10 horizon glow
# ---------------------------------------------------------------------------
def solid(color: str) -> Image.Image:
    return Image.new("RGBA", (8, 8), (*PALETTE[color], 255))


def block_top() -> Image.Image:
    img = solid("N")
    px = img.load()
    for x in range(8):
        px[x, 0] = (*PALETTE["C"], 255)
        px[x, 1] = (*PALETTE["B"], 255)
    px[0, 3] = px[7, 5] = px[3, 6] = (*PALETTE["D"], 255)
    return img


def block_fill(rng: random.Random) -> Image.Image:
    img = solid("N")
    px = img.load()
    for _ in range(4):
        px[rng.randrange(8), rng.randrange(8)] = (*PALETTE["D"], 255)
    return img


def platform_tile(kind: str) -> Image.Image:
    img = Image.new("RGBA", (8, 8), (0, 0, 0, 0))
    px = img.load()
    for x in range(8):
        px[x, 2] = (*PALETTE["C"], 255)
        px[x, 3] = (*PALETTE["A"], 255)
        px[x, 4] = (*PALETTE["a"], 255)
        px[x, 5] = (*PALETTE["K"], 255)
    if kind == "left":
        px[0, 2] = px[0, 3] = px[0, 4] = px[0, 5] = (*PALETTE["K"], 255)
    if kind == "right":
        px[7, 2] = px[7, 3] = px[7, 4] = px[7, 5] = (*PALETTE["K"], 255)
    return img


def sky_tile(stars: bool, rng: random.Random) -> Image.Image:
    img = solid("p") if stars else solid("p")
    px = img.load()
    if stars:
        for _ in range(2):
            px[rng.randrange(8), rng.randrange(8)] = (*PALETTE["W"], 255)
    return img


def grid_hill() -> Image.Image:
    img = solid("p")
    px = img.load()
    for x in range(8):
        px[x, 0] = (*PALETTE["M"], 255)
        if x % 4 == 0:
            for y in range(8):
                px[x, y] = (*PALETTE["M"], 255)
    for x in range(8):
        px[x, 4] = (*PALETTE["P"], 255)
    return img


def cloud_tile(side: str) -> Image.Image:
    img = solid("p")
    px = img.load()
    shape = ["..PPPP..", ".PPPPPP.", "PPPPPPPP"] if side == "l" else \
            ["..PPP...", ".PPPPP..", "PPPPPPP."]
    for y, row in enumerate(shape):
        for x, ch in enumerate(row):
            if ch == "P":
                px[x, y + 3] = (*PALETTE["M"], 255)
    return img


def horizon_glow() -> Image.Image:
    img = solid("p")
    px = img.load()
    for x in range(8):
        px[x, 6] = (*PALETTE["O"], 255)
        px[x, 7] = (*PALETTE["Y"], 255)
    return img


def platform_tiles() -> Image.Image:
    rng = random.Random(11)
    tiles = [
        block_top(),
        block_fill(rng),
        platform_tile("left"),
        platform_tile("mid"),
        platform_tile("right"),
        sky_tile(False, rng),
        sky_tile(True, rng),
        grid_hill(),
        cloud_tile("l"),
        cloud_tile("r"),
        horizon_glow(),
    ]
    return hstrip(tiles)


# ---------------------------------------------------------------------------
# UI panel tiles (8x8) for dialogue boxes and menus:
#  0 fill   1 top border   2 bottom border
# ---------------------------------------------------------------------------
def panel_tile(border: str | None) -> Image.Image:
    img = solid("N")
    px = img.load()
    if border == "top":
        for x in range(8):
            px[x, 0] = (*PALETTE["C"], 255)
            px[x, 1] = (*PALETTE["K"], 255)
    if border == "bottom":
        for x in range(8):
            px[x, 7] = (*PALETTE["C"], 255)
            px[x, 6] = (*PALETTE["K"], 255)
    return img


def ui_panel_tiles() -> Image.Image:
    return hstrip([panel_tile(None), panel_tile("top"), panel_tile("bottom")])


# ---------------------------------------------------------------------------
# Title screen (240x160): synthwave sun + grid + "CYBER JUM!" pixel logo.
# ---------------------------------------------------------------------------
FONT_5X7 = {
    "C": ["01110", "10001", "10000", "10000", "10000", "10001", "01110"],
    "Y": ["10001", "10001", "01010", "00100", "00100", "00100", "00100"],
    "B": ["11110", "10001", "10001", "11110", "10001", "10001", "11110"],
    "E": ["11111", "10000", "10000", "11110", "10000", "10000", "11111"],
    "R": ["11110", "10001", "10001", "11110", "10100", "10010", "10001"],
    "J": ["00111", "00010", "00010", "00010", "00010", "10010", "01100"],
    "U": ["10001", "10001", "10001", "10001", "10001", "10001", "01110"],
    "M": ["10001", "11011", "10101", "10101", "10001", "10001", "10001"],
    "!": ["00100", "00100", "00100", "00100", "00100", "00000", "00100"],
    " ": ["00000", "00000", "00000", "00000", "00000", "00000", "00000"],
}


def draw_text(px, text: str, ox: int, oy: int, scale: int,
              color: tuple, shadow: tuple | None = None) -> int:
    """Draw pixel text; returns total width in pixels."""
    x = ox
    for ch in text:
        glyph = FONT_5X7[ch]
        for gy, row in enumerate(glyph):
            for gx, bit in enumerate(row):
                if bit == "1":
                    for sy in range(scale):
                        for sx in range(scale):
                            tx = x + gx * scale + sx
                            ty = oy + gy * scale + sy
                            if shadow:
                                px[tx + scale, ty + scale] = (*shadow, 255)
                            px[tx, ty] = (*color, 255)
        x += 6 * scale
    return x - ox


def text_width(text: str, scale: int) -> int:
    return len(text) * 6 * scale - scale


def title_screen() -> Image.Image:
    rng = random.Random(3)
    img = Image.new("RGBA", (240, 160), (0, 0, 0, 255))
    px = img.load()
    # Night sky bands.
    for y in range(160):
        if y < 50:
            c = PALETTE["K"]
        elif y < 80:
            c = PALETTE["N"]
        elif y < 104:
            c = PALETTE["p"]
        else:
            c = PALETTE["N"]
        for x in range(240):
            px[x, y] = (*c, 255)
    # Stars.
    for _ in range(70):
        x, y = rng.randrange(240), rng.randrange(95)
        px[x, y] = (*PALETTE["W"], 255)
    # Synthwave sun with horizontal cuts.
    cx, cy, radius = 120, 100, 34
    for y in range(cy - radius, cy + radius):
        for x in range(cx - radius, cx + radius):
            if (x - cx) ** 2 + (y - cy) ** 2 <= radius * radius and y <= 104:
                if y % 6 != 5:
                    px[x, y] = (*(PALETTE["Y"] if y < cy - 8 else PALETTE["O"]), 255)
    # Horizon line and perspective grid.
    for x in range(240):
        px[x, 104] = (*PALETTE["M"], 255)
    for row, y in enumerate([110, 118, 130, 146]):
        for x in range(240):
            px[x, y] = (*PALETTE["M"], 255)
    for k in range(-6, 7):
        for y in range(105, 160):
            x = 120 + k * (y - 104)
            if 0 <= x < 240:
                px[x, y] = (*PALETTE["M"], 255)
    # Logo.
    for text, scale, y in [("CYBER", 4, 18), ("JUM!", 5, 52)]:
        w = text_width(text, scale)
        draw_text(px, text, (240 - w) // 2, y, scale, PALETTE["C"], PALETTE["D"])
    return img


def main() -> None:
    print("Generating art assets...")
    save(hero_topdown(), "sprites/hero_topdown.png")
    save(hero_platform(), "sprites/hero_platform.png")
    save(npcs(), "sprites/npcs.png")
    save(enemy(), "sprites/enemy.png")
    save(flag(), "sprites/flag.png")
    save(ui(), "sprites/ui.png")
    save(topdown_tiles(), "backgrounds/topdown_tiles.png")
    save(platform_tiles(), "backgrounds/platform_tiles.png")
    save(ui_panel_tiles(), "backgrounds/ui_panel.png")
    save(title_screen(), "backgrounds/title.png")
    print("Done. Review the 4x upscaled copies in assets/preview/")


if __name__ == "__main__":
    main()
