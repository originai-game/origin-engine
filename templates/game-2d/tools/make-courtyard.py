#!/usr/bin/env python3
"""Draws the 2D starter's courtyard (its top-down-2d view), byte for byte:

    python3 tools/make-courtyard.py         # from templates/game-2d

- assets/sprites/courier-top.png + .ogt: the courier seen from above (a
  three-quarter view, as the old top-down adventures drew theirs): idle,
  walk and dash clips facing down, up and to the side (flipX faces left);
  courier-top-p2 is the second seat's, in the rooftops' P2 palette,
- assets/tiles/courtyard.png + .ogt: the courtyard tileset (paving, grass,
  hedges, the canal's water and banks, the buildings' teal roofs and lit
  brick walls, the gate, flowerbeds, crates and lamp posts),
- assets/maps/courtyard.ogt: the courtyard itself, a tilemap over it with
  its objects (the spawn, the coins, the canal, the gate, the keeper's
  place and the lamps),
- assets/audio/dash.wav and splash.wav.

The rooftops' assets are the runner archetype's
(templates/archetypes/runner/tools/make-assets.py): this draws in the same
pixel style with the same code, importing that file's pixel grid, palette,
side-on courier and coin, so it runs in the engine repository. Standard
library only; deterministic.
"""
import importlib.util, math, os, sys

here = os.path.dirname(os.path.abspath(__file__))
root = os.path.dirname(here)
runner_tools = os.path.normpath(os.path.join(root, "..", "archetypes", "runner", "tools"))
sys.path.insert(0, runner_tools)
spec = importlib.util.spec_from_file_location("runner_assets", os.path.join(runner_tools, "make-assets.py"))
R = importlib.util.module_from_spec(spec)
spec.loader.exec_module(R)
from oglite import write_wav, tone, noise, mix, concat  # noqa: E402

assets = os.path.join(root, "assets")
Pix, sheet, courier, swapped, INK = R.Pix, R.sheet, R.courier, R.swapped, R.INK
FW, FH = R.FW, R.FH
SKIN, JACKET, DARK, SCARF, HELMET, SHOE, BAG, STRAP, PANTS, PANTS_DARK, SLEEVE = (
    R.SKIN, R.JACKET, R.DARK, R.SCARF, R.HELMET, R.SHOE, R.BAG, R.STRAP, R.PANTS, R.PANTS_DARK, R.SLEEVE)
VISOR = "#c8d4e0"
BAG_LIP = "#e0a85a"
HAIR = "#4a3428"


# ── the courier from above: 32 x 40 frames, feet on the bottom rows ────────
#
# The rooftops' courier, turned: the same helmet, scarf, jacket, strap and
# mail bag, drawn front-on (facing down the screen) and from behind (facing
# up). The side frames are the rooftops' own drawing at a walking gait.

def legs_front(p, step, back_view):
    """Two legs side by side under the hips (y 25); `step` -1, 0, 1: which foot is forward."""
    for side, x in ((-1, 12), (1, 17)):
        forward = step == side
        lifted = step == -side
        # Seen from the front a forward foot comes down the screen, from behind it goes up it.
        foot = 38 + (1 if forward and not back_view else 0) - (2 if lifted else 0) - (1 if forward and back_view else 0)
        col = PANTS if side < 0 else PANTS_DARK
        p.rect(x, 25, 3, foot - 25, col)
        p.rect(x - (1 if side < 0 else 0), foot, 4, 2, SHOE)


def courier_front(step=0, bob=0, swing=0, dash=False):
    """Facing down the screen. swing: the left hand forward (+) or back (-)."""
    p = Pix(FW, FH)
    legs_front(p, step, False)
    sh = 15 + bob
    # the mail bag on the right hip, under the arm
    p.rect(21, sh + 5, 5, 6, BAG)
    p.rect(21, sh + 5, 5, 1, BAG_LIP)
    # torso, shaded on its right
    p.rect(11, sh - 1, 10, 11, JACKET)
    p.rect(19, sh - 1, 2, 11, DARK)
    p.line(12, sh, 20, sh + 7, STRAP)
    p.rect(11, sh + 9, 10, 2, PANTS)
    # arms at the sides: a hand forward comes down a texel, one back goes up
    for x, s, col in ((9, swing, SLEEVE), (21, -swing, DARK)):
        if dash:
            p.line(x + (0.5 if x < 16 else 1.5), sh, x + (-2 if x < 16 else 4), sh + 6, col, 2)
            p.rect(x + (-3 if x < 16 else 4), sh + 6, 2, 2, SKIN)
        else:
            p.rect(x, sh, 2, 8 + s, col)
            p.rect(x, sh + 8 + s, 2, 2, SKIN)
    # head: face, helmet with its visor, eyes, the scarf knotted at the neck with its end in front
    hx, hy = 16, sh - 6
    p.disc(hx, hy, 4.5, SKIN)
    p.rect(hx - 5, hy - 5, 10, 3, HELMET)
    p.rect(hx - 4, hy - 6, 8, 1, HELMET)
    p.rect(hx - 4, hy - 2, 8, 1, VISOR)
    p.rect(hx - 2, hy, 1, 1, SHOE)
    p.rect(hx + 1, hy, 1, 1, SHOE)
    p.rect(11, sh - 2, 10, 2, SCARF)
    p.rect(12, sh, 2, 4 + (1 if step else 0), SCARF)
    p.outline(INK)
    return p


def courier_back(step=0, bob=0, swing=0, dash=False):
    """Facing up the screen: the helmet's back, the scarf's tail down the back, the bag on the left hip."""
    p = Pix(FW, FH)
    legs_front(p, step, True)
    sh = 15 + bob
    p.rect(11, sh - 1, 10, 11, JACKET)
    p.rect(19, sh - 1, 2, 11, DARK)
    p.line(20, sh, 12, sh + 7, STRAP)
    p.rect(11, sh + 9, 10, 2, PANTS)
    p.rect(6, sh + 5, 5, 6, BAG)
    p.rect(6, sh + 5, 5, 1, BAG_LIP)
    for x, s, col in ((9, -swing, SLEEVE), (21, swing, DARK)):
        if dash:
            p.line(x + (0.5 if x < 16 else 1.5), sh, x + (-2 if x < 16 else 4), sh + 6, col, 2)
            p.rect(x + (-3 if x < 16 else 4), sh + 6, 2, 2, SKIN)
        else:
            p.rect(x, sh, 2, 8 + s, col)
            p.rect(x, sh + 8 + s, 2, 2, SKIN)
    hx, hy = 16, sh - 6
    # the back of the head: hair under the helmet's shell, the neck below it
    p.disc(hx, hy, 4.5, HAIR)
    p.rect(hx - 2, hy + 3, 4, 2, SKIN)
    p.rect(hx - 5, hy - 5, 10, 5, HELMET)
    p.rect(hx - 4, hy - 6, 8, 1, HELMET)
    p.rect(hx - 4, hy - 1, 8, 1, VISOR)
    p.rect(11, sh - 2, 10, 2, SCARF)
    # the tail trails down the back, swinging with the stride (and flying out in a dash)
    tail = 3 if dash else 0
    p.rect(14 + step, sh, 2, 6 - tail, SCARF)
    p.rect(13 + step * 2, sh + 6 - tail, 2, 2 + (0 if dash else 1), SCARF)
    p.outline(INK)
    return p


def walk_side(k):
    """Frame k of 8 of the rooftops' courier at a walk: smaller swings than its run, upright."""
    ph = 2 * math.pi * k / 8

    def leg(off):
        s = math.sin(ph + off)
        return (0.5 * s, 0.5 * s - (0.1 + 0.7 * max(0.0, -math.cos(ph + off))))

    def arm(off):
        s = math.sin(ph + off)
        return (-0.5 * s, -0.5 * s - 0.5)

    return courier((leg(math.pi), leg(0)), (arm(0), arm(math.pi)), bob=0 if k % 4 == 2 else -1)


IDLE = [courier_front(bob=b) for b in (0, 1)] + [courier_back(bob=b) for b in (0, 1)] + R.FRAMES[14:16]
WALK_DOWN = [courier_front(step=s, bob=b, swing=w) for s, b, w in ((-1, 0, 1), (0, -1, 0), (1, 0, -1), (0, -1, 0))]
WALK_UP = [courier_back(step=s, bob=b, swing=w) for s, b, w in ((-1, 0, 1), (0, -1, 0), (1, 0, -1), (0, -1, 0))]
WALK_SIDE = [walk_side(k) for k in range(8)]
DASH = [courier_front(step=1, bob=1, dash=True), courier_back(step=1, bob=1, dash=True),
        courier(((-0.9, -1.7), (1.1, 0.5)), ((-1.3, -1.7), (-0.9, -1.3)), bob=1, lean=2)]
FRAMES = IDLE + WALK_DOWN + WALK_UP + WALK_SIDE + DASH

COURIER_TOP_OGT = """type flipbook
version 1
---
# The courier from above (tools/make-courtyard.py): 32 x 40 frames standing
# on their feet (the pivot: the entity's position is the feet). Each action
# faces down, up and to the side (the side frames face right; flipX faces
# left). Clips are numbered in file order, the facing's offset added to the
# action's first clip: idle 0-2, walk 3-5, dash 6-8 (src/courtyard.ts's CLIP
# checks them).
texture "courier-top.png"
pivot 0.5 0
grid 32 40 8 4
spacing 2
clip idle-down {
  frames 0 1
  fps 2
}
clip idle-up {
  frames 2 3
  fps 2
}
clip idle-side {
  frames 4 5
  fps 2
}
clip walk-down {
  frames 6 7 8 9
  fps 8
  event 0 "step"
  event 2 "step"
}
clip walk-up {
  frames 10 11 12 13
  fps 8
  event 0 "step"
  event 2 "step"
}
clip walk-side {
  frames 14 15 16 17 18 19 20 21
  fps 12
  event 2 "step"
  event 6 "step"
}
clip dash-down {
  frames 22
  fps 1
}
clip dash-up {
  frames 23
  fps 1
}
clip dash-side {
  frames 24
  fps 1
}
"""


# ── the courtyard tileset: 16 x 16 tiles, 4 x 6 ───────────────────────────

ROOF, ROOF_LIP, ROOF_HI = R.ROOF, R.ROOF_LIP, "#8fe0d4"
BRICK, MORTAR, LIT, LIT2, DARKWIN = R.BRICK, R.MORTAR, R.LIT, R.LIT2, R.DARKWIN
STONE, STONE_HI, STONE_LO = "#5e5a78", "#77739a", "#3b3656"
MOSS = "#3f7a5e"
GRASS, GRASS_HI, GRASS_LO = "#2f6b5a", "#4a9a78", "#245446"
HEDGE, HEDGE_HI, HEDGE_LO = "#22544a", "#3f9a78", "#163a34"
WATER, WATER_HI, WATER_LO, FOAM = "#1d3b6b", "#2f5d9a", "#142a52", "#7fd1e8"
LIP, LIP_HI, LIP_LO = "#6b6f80", "#8a8fa3", "#4a4d5c"
WOOD, WOOD_HI, WOOD_LO = "#7a4a2a", "#b8743f", "#5a341c"
IRON = "#2a2433"


def cobbles(p, base, hi, lo, seed):
    """Flagstones: two courses of 8 texels, staggered, each stone lit along its top and left edges."""
    p.rect(0, 0, 16, 16, lo)
    for row in range(2):
        y = row * 8
        off = (4 + seed * 3) % 8 if row else (seed * 5) % 8
        for x in range(-off, 16, 8):
            w = 8 if (x + seed) % 3 else 7
            p.rect(x + 1, y + 1, w - 1, 7, base)
            p.rect(x + 1, y + 1, w - 1, 1, hi)
            p.rect(x + 1, y + 1, 1, 7, hi)
    # a worn speck or two
    for x, y in ((3 + seed, 4), (11, 12 - seed % 3)):
        p.set(x % 16, y, lo)


def tile(kind):
    p = Pix(16, 16)
    if kind in ("paving", "paving_moss", "paving_shade"):         # 0, 1, 2
        cobbles(p, STONE, STONE_HI, STONE_LO, 0 if kind == "paving" else 3)
        if kind == "paving_moss":
            for x, y in ((3, 4), (4, 4), (10, 12), (11, 13), (12, 12), (6, 9)):
                p.set(x, y, MOSS)
        if kind == "paving_shade":
            # the north wall's shadow on the paving below it
            for y in range(6):
                for x in range(16):
                    if (x + y) % 2 == 0 or y < 3:
                        p.set(x, y, STONE_LO if p.get(x, y) != STONE_LO else "#241f3a")
    elif kind in ("grass", "grass_flowers"):                      # 3, 4
        p.rect(0, 0, 16, 16, GRASS)
        for i, (x, y) in enumerate(((2, 3), (7, 1), (12, 4), (4, 9), (10, 8), (14, 12), (1, 13), (7, 14))):
            p.rect(x, y, 1, 2, GRASS_HI)
            p.set(x + 1, y + 1, GRASS_LO)
        if kind == "grass_flowers":
            for x, y, col in ((4, 5, SCARF), (11, 11, "#ffd23f"), (9, 3, "#ff8fd0"), (3, 12, "#ffd23f")):
                p.rect(x, y, 2, 2, col)
                p.set(x, y + 2, GRASS_LO)
    elif kind == "hedge":                                         # 5: the hedge's top, seen from above
        p.rect(0, 0, 16, 16, HEDGE)
        for x, y in ((1, 2), (6, 1), (11, 3), (3, 7), (9, 6), (14, 8), (2, 12), (7, 11), (12, 13)):
            p.rect(x, y, 3, 2, HEDGE_HI)
            p.set(x + 1, y + 2, HEDGE_LO)
    elif kind == "hedge_face":                                    # 6: its front, darkening to the ground
        p.rect(0, 0, 16, 16, HEDGE)
        for x, y in ((2, 1), (8, 2), (13, 0), (5, 6), (11, 7)):
            p.rect(x, y, 2, 2, HEDGE_HI)
        p.rect(0, 10, 16, 6, HEDGE_LO)
        p.rect(0, 15, 16, 1, INK)
    elif kind == "roof":                                          # 7: the buildings' teal roofs
        p.rect(0, 0, 16, 16, ROOF_LIP)
        for row in range(4):
            off = 4 if row % 2 else 0
            for x in range(-4 + off, 16, 8):
                p.rect(x, row * 4, 7, 3, ROOF)
                p.rect(x, row * 4, 7, 1, ROOF_HI)
    elif kind == "roof_eave":                                     # 8: the roof's edge over the wall below
        p.rect(0, 0, 16, 16, ROOF_LIP)
        for row in range(2):
            off = 4 if row % 2 else 0
            for x in range(-4 + off, 16, 8):
                p.rect(x, row * 4, 7, 3, ROOF)
                p.rect(x, row * 4, 7, 1, ROOF_HI)
        p.rect(0, 8, 16, 3, ROOF_LIP)
        p.rect(0, 11, 16, 2, "#2a6e67")
        p.rect(0, 13, 16, 3, BRICK)
    elif kind in ("wall", "wall_lit", "wall_dark"):               # 9, 10, 11
        p.rect(0, 0, 16, 16, BRICK)
        R.bricks(p, ((1, 0), (5, 4), (9, 0), (13, 4)))
        if kind != "wall":
            p.rect(4, 2, 8, 10, INK)
            p.rect(5, 3, 6, 8, LIT if kind == "wall_lit" else DARKWIN)
            p.rect(8, 3, 1, 8, INK)
            p.rect(5, 7, 6, 1, INK)
            if kind == "wall_lit":
                p.rect(5, 3, 2, 1, LIT2)
            p.rect(3, 12, 10, 2, LIP)
        p.rect(0, 15, 16, 1, INK)
    elif kind in ("water", "water_b"):                            # 12, 13: the canal, its ripples moving
        p.rect(0, 0, 16, 16, WATER)
        p.rect(0, 0, 16, 2, WATER_LO)
        shift = 0 if kind == "water" else 4
        for x0, y in ((1, 5), (9, 8), (3, 12)):
            x = (x0 + shift) % 16
            p.rect(x, y, 4, 1, WATER_HI)
            p.rect((x + 1) % 16, y - 1, 2, 1, FOAM if y == 8 else WATER_HI)
    elif kind == "bank_n":                                        # 14: paving ending in the canal's stone lip (water below)
        cobbles(p, STONE, STONE_HI, STONE_LO, 1)
        p.rect(0, 11, 16, 3, LIP)
        p.rect(0, 11, 16, 1, LIP_HI)
        p.rect(0, 14, 16, 2, LIP_LO)
        for x in range(0, 16, 5):
            p.rect(x, 11, 1, 3, LIP_LO)
    elif kind == "bank_s":                                        # 15: the far lip (water above)
        cobbles(p, STONE, STONE_HI, STONE_LO, 2)
        p.rect(0, 0, 16, 3, LIP)
        p.rect(0, 0, 16, 1, LIP_HI)
        p.rect(0, 3, 16, 1, LIP_LO)
        for x in range(2, 16, 5):
            p.rect(x, 0, 1, 3, LIP_LO)
    elif kind in ("gate_tl", "gate_tr", "gate_bl", "gate_br"):    # 16-19: the gate, 2 x 2, lit within
        p.rect(0, 0, 16, 16, BRICK)
        R.bricks(p, ((1, 0), (5, 4), (9, 0), (13, 4)))
        # the arch over two doors, the right one swung open on a lit hall; each tile is its quarter
        g = Pix(32, 32)
        for yy in range(32):
            for xx in range(32):
                dx = (xx + 0.5 - 16) / 13
                dy = (yy + 0.5 - 14) / 12
                inside = (yy >= 14 and 3 <= xx < 29) or (yy < 14 and dx * dx + dy * dy <= 1)
                rim = (yy >= 14 and (2 <= xx < 4 or 28 <= xx < 30)) or (yy < 14 and 1 < dx * dx + dy * dy <= 1.25)
                if rim:
                    g.set(xx, yy, LIP if yy % 4 else LIP_HI)
                elif inside:
                    g.set(xx, yy, LIT2 if yy < 12 else LIT)
        # the doors: the left one shut, the right one swung open
        g.rect(4, 12, 12, 20, WOOD)
        for x in (6, 10, 14):
            g.rect(x, 13, 1, 19, WOOD_LO)
        g.rect(4, 12, 12, 1, WOOD_HI)
        g.rect(13, 21, 2, 2, IRON)
        g.rect(25, 12, 3, 20, WOOD)
        g.rect(25, 12, 1, 20, WOOD_HI)
        g.rect(16, 28, 9, 4, "#c98b3f")
        g.rect(0, 31, 32, 1, INK)
        p.blit(g, 0 if kind.endswith("l") else -16, 0 if "_t" in kind else -16)
    elif kind == "flowerbed":                                     # 20: a stone planter, solid
        p.rect(0, 2, 16, 14, LIP)
        p.rect(0, 2, 16, 1, LIP_HI)
        p.rect(1, 3, 14, 9, "#3a2a24")
        for x, y, col in ((2, 4, SCARF), (6, 6, "#ffd23f"), (10, 4, "#ff8fd0"), (12, 8, SCARF), (4, 9, "#ffd23f"), (8, 9, GRASS_HI)):
            p.rect(x, y, 2, 2, col)
            p.set(x, y + 2, GRASS)
        p.rect(0, 12, 16, 4, LIP_LO)
        p.rect(0, 15, 16, 1, INK)
    elif kind == "crate":                                         # 21: the rooftops' crate
        p.blit(R.crate(), 0, 0)
    elif kind == "lamp":                                          # 22: an iron lamp post's foot (the ground shows round it), its lantern above
        p.disc(8, 13, 4, IRON, ry=2)
        p.rect(7, 0, 2, 13, IRON)
        p.rect(4, 0, 8, 1, IRON)
    elif kind == "lamp_top":                                      # 23: the lantern on the post (drawn on the walls layer)
        p.rect(7, 12, 2, 4, IRON)
        p.rect(4, 3, 8, 9, IRON)
        p.rect(5, 4, 6, 7, LIT)
        p.rect(6, 5, 2, 3, "#fff3c4")
        p.rect(3, 2, 10, 1, IRON)
        p.rect(7, 0, 2, 2, IRON)
    return p


TILES = ["paving", "paving_moss", "paving_shade", "grass",
         "grass_flowers", "hedge", "hedge_face", "roof",
         "roof_eave", "wall", "wall_lit", "wall_dark",
         "water", "water_b", "bank_n", "bank_s",
         "gate_tl", "gate_tr", "gate_bl", "gate_br",
         "flowerbed", "crate", "lamp", "lamp_top"]

COURTYARD_OGT = """type tileset
version 1
---
# The courtyard (tools/make-courtyard.py), seen from above: 0-2 paving
# (plain, mossy, in the north wall's shade), 3-4 grass, 5-6 a hedge's top
# and front, 7-8 the buildings' teal roofs and an eave, 9-11 brick walls
# (bare, a lit window, a dark one), 12-13 the canal's water, 14-15 its
# banks, 16-19 the gate (2 x 2), 20 a flowerbed, 21 a crate, 22-23 a lamp
# post. What stands collides as whole cells on a layer marked
# `collision true`; the water does not: the courier walks into it.
texture "tiles/courtyard.png"
grid 16 16 4 6
spacing 2
filter nearest
solid 5 6 7 8 9 10 11 16 17 18 19 20 21 22
tile 12 {
  frames 12 13
  durations 0.6 0.6
}
tile 10 {
  frames 10 11 10
  durations 3.1 0.3 1.4
}
"""


def tileset_sheet():
    """4 x 6 tiles, 2 texels apart, each edge extruded into the gap (a tile never samples its neighbour)."""
    step = 16 + R.SPACING
    s = Pix(4 * step - R.SPACING, 6 * step - R.SPACING)
    for i, k in enumerate(TILES):
        t = tile(k)
        ox, oy = (i % 4) * step, (i // 4) * step
        for y in range(-1, 17):
            for x in range(-1, 17):
                c = t.get(min(15, max(0, x)), min(15, max(0, y)))
                if c is not None:
                    s.set(ox + x, oy + y, c)
    return s


# ── the courtyard: 34 x 24 cells, one world unit each ─────────────────────
#
# North: the buildings' roofs and their brick face, the gate in the middle
# (the goal), the lamp keeper waiting beside it. A canal runs the courtyard's
# width below the north bank: walking in costs a life, a dash crosses it.
# South: the yard the courier starts in, its coins among the hedges, the
# flowerbeds and the crates. Roofs close the east, west and south.

GROUND = [
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
    "RR..............................RR",
    "RR..............................RR",
    "RRssssssssssssssssssssssssssssssRR",
    "RR...,.......,.........,....,...RR",
    "RR..,...........,.........,.....RR",
    "RRnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnRR",
    "RR~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~RR",
    "RRuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuRR",
    "RR.......,..............,.......RR",
    "RR.vvvv*..........,.......vvvv*.RR",
    "RR.v*vvv..................vvv*v.RR",
    "RR.vvv*v..,...............v*vvv.RR",
    "RR..............................RR",
    "RR..............................RR",
    "RR.......,..........,...........RR",
    "RR..,..................,........RR",
    "RR..........vv*vvv*vvv....,.....RR",
    "RR...,......v*vvvvvv*v..........RR",
    "RR..........vvvv*vvvvv..........RR",
    "RR...........,..........,.......RR",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
]

WALLS = [
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
    "RReeeeeeeeeeeeeeeeeeeeeeeeeeeeeeRR",
    "RRwLwwDwwLwwwwwwGHwwwwLwwDwwwLwwRR",
    "RRwwwwwwwwwwwwwwghwwwwwwwwwwwwwwRR",
    "RR..............................RR",
    "RR..............................RR",
    "RR..............................RR",
    "RR..............................RR",
    "RR..............................RR",
    "RR..............................RR",
    "RR..............................RR",
    "RR..............................RR",
    "RR..............................RR",
    "RR..............................RR",
    "RR...YYYYYYY..........YYYYYYY...RR",
    "RR...yyyyyyy..........yyyyyyy...RR",
    "RR..............................RR",
    "RR..cc....................bbb...RR",
    "RR..c...........................RR",
    "RR..............................RR",
    "RR.........................c....RR",
    "RR..............................RR",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
]

# The lamp posts: the post's foot `p` on the walls layer, its lantern a cell above (`t`) without collision.
POSTS = [(10, 7), (22, 7), (10, 11), (22, 11)]

GROUND_LEGEND = [(".", 0), (",", 1), ("s", 2), ("v", 3), ("*", 4), ("R", 7), ("~", 12), ("n", 14), ("u", 15)]
WALL_LEGEND = [("R", 7), ("e", 8), ("w", 9), ("L", 10), ("D", 11), ("G", 16), ("H", 17), ("g", 18), ("h", 19),
               ("Y", 5), ("y", 6), ("b", 20), ("c", 21), ("p", 22)]

H = len(GROUND)
W = len(GROUND[0])


def cell_y(row):
    """A cell row's bottom edge in world units (y up, the map's bottom edge at 0)."""
    return H - 1 - row


def walls_rows():
    rows = [list(r) for r in WALLS]
    tops = [["."] * W for _ in range(H)]
    for x, row in POSTS:
        rows[row][x] = "p"
        rows[row - 1][x] = "."
        tops[row - 1][x] = "t"
    return ["".join(r) for r in rows], ["".join(r) for r in tops]


# Coins, the spawn and the keeper in world units (y up from the map's bottom edge); objects are written y down from its top.
COINS = [(6.5, 11.5), (27.5, 11.5), (17.0, 9.5), (8.5, 5.5), (25.5, 5.5), (17.0, 13.0), (5.5, 18.5), (28.5, 18.5)]
SPAWN = (17.0, 2.5)


def map_ogt():
    for r in GROUND + WALLS:
        assert len(r) == W, r
    walls, tops = walls_rows()
    out = ["type tilemap", "version 1", "---",
           "# The courtyard (tools/make-courtyard.py), seen from above: 34 x 24 cells",
           "# of 16 px, one world unit each, over the courtyard tileset. The walls",
           "# layer collides (og.Tilemap.colliderLayers 1): roofs, the brick face, the",
           "# gate, hedges, flowerbeds, crates and lamp posts. The canal's water is",
           "# ground the courier can walk into: the `canal` object is where that costs",
           "# a life, and a dash carries the courier over it.",
           f"size {W} {H}", "tile 16 16", "unit 1", 'tileset "tiles/courtyard.ogt"']

    def layer(name, rows, legend, collide):
        out.append(f"layer {name} {{")
        out.append("  z 0")
        if collide:
            out.append("  collision true")
        for ch, t in legend:
            out.append(f'  legend "{ch}" {t}')
        for r in rows:
            out.append(f'  row "{r}"')
        out.append("}")

    layer("ground", GROUND, GROUND_LEGEND, False)
    layer("walls", walls, WALL_LEGEND, True)
    layer("lamps", tops, [("t", 23)], False)
    objects = [("spawn", "spawn", f"at {SPAWN[0]} {H - SPAWN[1]}")]
    for i, (x, y) in enumerate(COINS):
        objects.append((f"coin-{i}", "coin", f"at {x} {H - y}"))
    canal = GROUND.index(next(r for r in GROUND if "~" in r))
    x0 = GROUND[canal].index("~")
    x1 = GROUND[canal].rindex("~") + 1
    objects.append(("canal", "hazard", f"rect {x0} {canal} {x1 - x0} 1"))
    gate = WALLS[2].index("G")
    objects.append(("goal", "goal", f"rect {gate} 4 2 1"))
    objects.append(("keeper", "keeper", f"at {gate - 3.5} {H - cell_y(4) - 0.05:.2f}"))
    for i, (x, row) in enumerate(POSTS):
        objects.append((f"lamp-{i}", "lamp", f"at {x + 0.5} {H - cell_y(row - 1) - 0.6:.1f}"))
    for name, kind, place in objects:
        out += [f"object {name} {{", f'  kind "{kind}"', f"  {place}", "}"]
    return "\n".join(out) + "\n"


def main():
    def write(rel, data):
        path = os.path.join(assets, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb" if isinstance(data, bytes) else "w") as f:
            f.write(data)

    def png(rel, pix):
        write(rel, pix.png())
        write(rel + ".ogt", R.PIXEL_IMPORT.replace("tools/make-assets.py", "tools/make-courtyard.py"))

    top = sheet(FRAMES, FW, FH, 8)
    png("sprites/courier-top.png", top)
    png("sprites/courier-top-p2.png", swapped(top, R.P2_SWAP))
    write("sprites/courier-top.ogt", COURIER_TOP_OGT)
    write("sprites/courier-top-p2.ogt", COURIER_TOP_OGT.replace('texture "courier-top.png"', 'texture "courier-top-p2.png"').replace(
        "# The courier from above (tools/make-courtyard.py)", "# The second seat's courier from above (tools/make-courtyard.py: the runner's P2_SWAP palette)"))
    png("tiles/courtyard.png", tileset_sheet())
    write("tiles/courtyard.ogt", COURTYARD_OGT)
    write("maps/courtyard.ogt", map_ogt())
    a = os.path.join(assets, "audio")
    write_wav(os.path.join(a, "dash.wav"), mix(noise(0.22, decay=0.07, gain=0.45, seed=11),
                                                tone(0.22, 260, 900, shape="tri", decay=0.08, gain=0.25)))
    write_wav(os.path.join(a, "splash.wav"), concat(mix(noise(0.5, decay=0.14, gain=0.6, seed=5),
                                                        tone(0.5, 220, 70, shape="sine", decay=0.12, gain=0.45))))


if __name__ == "__main__":
    main()
