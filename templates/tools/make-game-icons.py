#!/usr/bin/env python3
"""Regenerates the engine's game icon set, templates/styles/shared/icons/:
game.ogt (the set, cooked like a font by og-cook's cook::icons) and its
SVGs under game/, plus third_party/game-icons/ICONS.md (each borrowed
icon's author, for the CC BY 3.0 attribution).

    python3 templates/tools/make-game-icons.py [repo]

Two kinds of icon, both on a 24-unit grid:

- The hero icons (currencies, gems, stars, hearts, shields, energy, ammo,
  lock, rewards, rarity and badges) are our own, drawn here as paths in
  tones: each path's paint is a theme colour, var(--color-icon-*), so a
  pack's theme recolours them, and the engine draws them as stickers
  (outline and drop shadow from the set's `sticker` line). Their shapes
  were redrawn by hand from a generated concept sheet; no raster output
  is shipped. CC0, like the engine's other self-made art.
- The rest are game-icons.net silhouettes (CC BY 3.0, authors in
  ICONS.md), fetched at GAME_ICONS_REV, their 512-unit paths scaled to the
  24-unit grid and drawn in the icon's ink (one tone).

Standard library only; the output is the same on every run.
"""
import math, os, re, sys, urllib.request

REPO = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), "..", "..")
OUT = os.path.join(REPO, "templates", "styles", "shared", "icons")
GAME_ICONS_REV = "82d948812bfe3f269ef8f731dcdb07b08160edc4"

# The authors' names as game-icons.net's licence lists them, by folder.
AUTHORS = {
    "lorc": "Lorc", "delapouite": "Delapouite", "carl-olsen": "Carl Olsen", "sbed": "Sbed",
    "skoll": "Skoll", "lord-berandas": "Lord Berandas", "faithtoken": "Faithtoken",
}

# The borrowed icons by what games show them for: name -> game-icons.net path.
BORROWED = [
    ("Combat and stats", [
        ("attack", "lorc/crossed-swords"), ("sword", "lorc/broadsword"), ("axe", "lorc/battle-axe"),
        ("bow", "lorc/pocket-bow"), ("staff", "lorc/wizard-staff"), ("magic", "lorc/magic-swirl"),
        ("armor", "lorc/breastplate"), ("speed", "lorc/sprint"), ("crit", "lorc/on-target"),
        ("accuracy", "delapouite/crosshair"), ("strength", "delapouite/biceps"),
        ("intellect", "lorc/brain"), ("luck", "lorc/clover"), ("levelUp", "delapouite/upgrade"),
        ("bomb", "lorc/unlit-bomb"), ("skull", "lorc/skull-crossed-bones"),
        ("flame", "carl-olsen/flame"), ("barrier", "lorc/bolt-shield"),
    ]),
    ("Navigation and menus", [
        ("home", "delapouite/house"), ("shop", "delapouite/shop"), ("bag", "lorc/knapsack"),
        ("hero", "delapouite/person"), ("team", "delapouite/meeple-group"),
        ("friends", "delapouite/three-friends"), ("quest", "lorc/scroll-unfurled"),
        ("map", "lorc/treasure-map"), ("mail", "lorc/envelope"), ("chat", "delapouite/chat-bubble"),
        ("leaderboard", "delapouite/podium"), ("calendar", "delapouite/calendar"),
        ("event", "delapouite/party-popper"), ("castle", "delapouite/castle"),
        ("banner", "lorc/flying-flag"), ("compass", "lorc/compass"), ("book", "lorc/book-cover"),
        ("search", "lorc/magnifying-glass"), ("cards", "faithtoken/card-pick"),
        ("dice", "delapouite/rolling-dices"), ("rocket", "lorc/rocket"),
        ("inventory", "delapouite/backpack"), ("skillTree", "lorc/branch-arrow"),
        ("rank", "skoll/rank-3"),
    ]),
    ("Settings and system", [
        ("sound", "delapouite/speaker"), ("soundOff", "delapouite/speaker-off"),
        ("music", "delapouite/musical-notes"), ("bell", "lorc/ringing-bell"),
        ("vibrate", "delapouite/vibrating-smartphone"), ("language", "lorc/globe"),
        ("gamepad", "delapouite/gamepad"), ("save", "delapouite/save"),
        ("exit", "delapouite/exit-door"), ("power", "lord-berandas/power-button"),
        ("info", "delapouite/info"), ("account", "delapouite/id-card"),
        ("phone", "delapouite/smartphone"), ("unlock", "delapouite/padlock-open"),
        ("settings", "lorc/cog"), ("refresh", "delapouite/clockwise-rotation"),
    ]),
    ("Social", [
        ("like", "delapouite/thumb-up"), ("dislike", "delapouite/thumb-down"),
        ("handshake", "delapouite/shaking-hands"), ("share", "delapouite/share"),
        ("megaphone", "delapouite/megaphone"), ("trade", "lorc/trade"),
    ]),
    ("Shop", [
        ("cart", "delapouite/shopping-cart"), ("sale", "delapouite/price-tag"),
        ("wallet", "delapouite/wallet"), ("crate", "delapouite/wooden-crate"),
        ("piggyBank", "delapouite/piggy-bank"), ("coins", "delapouite/coins-pile"),
        ("cash", "delapouite/money-stack"), ("bank", "delapouite/bank"),
        ("video", "delapouite/video-camera"), ("vip", "lorc/crowned-heart"),
        ("pass", "delapouite/vertical-banner"), ("ad", "delapouite/tv"),
    ]),
    ("Rewards", [
        ("chestOpen", "skoll/open-treasure-chest"), ("medal", "lorc/medal"),
        ("laurels", "lorc/laurels"), ("ribbon", "delapouite/ribbon-medal"),
        ("lootBag", "lorc/swap-bag"), ("missions", "lorc/checkbox-tree"),
        ("target", "lorc/archery-target"), ("sparkles", "delapouite/sparkles"),
        ("claimAll", "delapouite/basket"), ("stamina", "delapouite/lungs"),
    ]),
    ("Status effects", [
        ("burn", "lorc/flame-claws"), ("freeze", "lorc/snowflake-2"),
        ("poison", "lorc/poison-bottle"), ("bleed", "lorc/bleeding-wound"),
        ("stun", "delapouite/knocked-out-stars"), ("sleep", "delapouite/night-sleep"),
        ("silence", "delapouite/silenced"), ("slow", "lorc/snail"),
        ("haste", "delapouite/running-shoe"), ("regen", "delapouite/healing"),
        ("curse", "lorc/cursed-star"), ("blind", "delapouite/blindfold"),
        ("shock", "sbed/electric"), ("ghost", "lorc/ghost"), ("root", "lorc/root-tip"),
        ("weak", "lorc/broken-heart"), ("charm", "lorc/charm"),
    ]),
]


def f(v):
    """A coordinate: two decimals, no trailing zeros."""
    s = f"{v:.2f}".rstrip("0").rstrip(".")
    return "0" if s in ("-0", "") else s


def poly(pts):
    return "M" + " L".join(f"{f(x)} {f(y)}" for x, y in pts) + " Z"


def circle(cx, cy, r, cw=True):
    """A circle as two arcs; `cw` false winds it the other way (a hole)."""
    s = 1 if cw else 0
    return (f"M{f(cx - r)} {f(cy)} A{f(r)} {f(r)} 0 1 {s} {f(cx + r)} {f(cy)} "
            f"A{f(r)} {f(r)} 0 1 {s} {f(cx - r)} {f(cy)} Z")


def rect(x0, y0, x1, y1, r=0.0):
    if r <= 0:
        return poly([(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
    return (f"M{f(x0 + r)} {f(y0)} H{f(x1 - r)} A{f(r)} {f(r)} 0 0 1 {f(x1)} {f(y0 + r)} "
            f"V{f(y1 - r)} A{f(r)} {f(r)} 0 0 1 {f(x1 - r)} {f(y1)} H{f(x0 + r)} "
            f"A{f(r)} {f(r)} 0 0 1 {f(x0)} {f(y1 - r)} V{f(y0 + r)} A{f(r)} {f(r)} 0 0 1 {f(x0 + r)} {f(y0)} Z")


def arc(cx, cy, r, a0, a1):
    """An open arc from angle a0 to a1 (degrees, counter-clockwise, y up)."""
    p = lambda a: (cx + r * math.cos(math.radians(a)), cy - r * math.sin(math.radians(a)))
    (x0, y0), (x1, y1) = p(a0), p(a1)
    large = 1 if abs(a1 - a0) > 180 else 0
    return f"M{f(x0)} {f(y0)} A{f(r)} {f(r)} 0 {large} 0 {f(x1)} {f(y1)}"


def crescent(cx, cy, r, inner):
    """The lower crescent of a disc: between its bottom and an ellipse `inner` tall."""
    return (f"M{f(cx - r)} {f(cy)} A{f(r)} {f(r)} 0 0 0 {f(cx + r)} {f(cy)} "
            f"A{f(r)} {f(inner)} 0 0 1 {f(cx - r)} {f(cy)} Z")


def star_pts(cx, cy, R, r, n=5, rot=90):
    out = []
    for k in range(2 * n):
        a = math.radians(rot + k * 180 / n)
        rad = R if k % 2 == 0 else r
        out.append((cx + rad * math.cos(a), cy - rad * math.sin(a)))
    return out


def fill(tone, d, rounded=0.0):
    """A filled path in `tone` (an icon-* role, or "ink"), its corners
    rounded by a stroke `rounded` wide in the same tone."""
    paint = "currentColor" if tone == "ink" else f"var(--color-{tone})"
    if rounded:
        return (f'<path d="{d}" fill="{paint}" stroke="{paint}" stroke-width="{f(rounded)}" '
                f'stroke-linejoin="round"/>')
    return f'<path d="{d}" fill="{paint}"/>'


def line(tone, d, w):
    paint = "currentColor" if tone == "ink" else f"var(--color-{tone})"
    return (f'<path d="{d}" fill="none" stroke="{paint}" stroke-width="{f(w)}" '
            f'stroke-linecap="round" stroke-linejoin="round"/>')


SHADE, GLOSS, MARK, OUTLINE = "icon-shade", "icon-gloss", "icon-mark", "icon-outline"


def disc_icon(tone, mark=()):
    """A round body: its shade, a gloss arc and a light glint, then marks."""
    return [fill(tone, circle(12, 11.5, 9)), fill(SHADE, crescent(12, 11.5, 9, 7.2)), *mark,
            line(GLOSS, arc(12, 11.5, 6.6, 112, 162), 1.6), fill(GLOSS, circle(8.9, 6.5, 0.9))]


def sparkle(cx, cy, R):
    return poly(star_pts(cx, cy, R, R * 0.32, 4))


HERO = {}

HERO["coin"] = [
    fill("icon-gold", circle(12, 11.5, 9)), fill(SHADE, crescent(12, 11.5, 9, 7.2)),
    fill(SHADE, circle(12, 11.5, 6.2)), fill("icon-gold", circle(12, 11.6, 5.2)),
    fill(SHADE, poly([(12, 8.2), (14.4, 11.6), (12, 15), (9.6, 11.6)])),
    line(GLOSS, arc(12, 11.5, 7.6, 112, 165), 1.4), fill(GLOSS, circle(8.6, 6.3, 0.8)),
]
HERO["gem"] = [
    fill("icon-purple", poly([(6.6, 3.5), (17.4, 3.5), (21.2, 8.8), (12, 20.6), (2.8, 8.8)]), 1),
    fill(GLOSS, poly([(6.6, 3.5), (11.6, 3.5), (8.9, 8.8), (2.8, 8.8)])),
    fill(SHADE, poly([(15.1, 8.8), (21.2, 8.8), (12, 20.6)])),
    fill(SHADE, poly([(12.4, 3.5), (17.4, 3.5), (21.2, 8.8), (15.1, 8.8)])),
    fill(GLOSS, poly([(8.9, 8.8), (11.2, 8.8), (12, 17.6)])),
]
_star = star_pts(12, 12.2, 9.2, 4.2)
HERO["star"] = [
    fill("icon-gold", poly(_star), 2),
    fill(SHADE, poly([(12, 12.2), _star[2], _star[3], _star[4], _star[5], _star[6]])),
    line(GLOSS, f"M{f(11.3)} {f(6.2)} L{f(10.1)} {f(9.0)}", 1.4),
    fill(GLOSS, circle(6.9, 9.6, 0.8)),
]
_heart = ("M12 20.2 C5 15.6 2.8 12.1 2.8 8.7 C2.8 5.7 5.1 3.6 7.6 3.6 C9.4 3.6 11 4.6 12 6.2 "
          "C13 4.6 14.6 3.6 16.4 3.6 C18.9 3.6 21.2 5.7 21.2 8.7 C21.2 12.1 19 15.6 12 20.2 Z")
HERO["heart"] = [
    fill("icon-red", _heart),
    fill(SHADE, "M21.2 8.7 C21.2 12.1 19 15.6 12 20.2 C8.4 17.8 6 15.6 4.5 13.4 "
         "C6.6 15.4 9 16.8 12 17.8 C17.2 14.6 20.6 11.8 21.2 8.7 Z"),
    line(GLOSS, "M5.3 9.4 C5.1 7.5 6.2 6.1 7.8 6", 1.6),
]
_shield = "M12 2.6 L20 5.4 L20 11.2 C20 15.8 16.8 19 12 21 C7.2 19 4 15.8 4 11.2 L4 5.4 Z"
HERO["shield"] = [
    fill("icon-blue", _shield, 0.6),
    fill(SHADE, "M12 2.6 L20 5.4 L20 11.2 C20 15.8 16.8 19 12 21 Z"),
    fill(SHADE, "M12 6 L16.8 7.7 L16.8 11.3 C16.8 14.3 14.9 16.4 12 17.7 C9.1 16.4 7.2 14.3 7.2 11.3 L7.2 7.7 Z"),
    fill("icon-blue", "M12 7.4 L15.5 8.6 L15.5 11.3 C15.5 13.6 14.1 15.2 12 16.2 C9.9 15.2 8.5 13.6 8.5 11.3 L8.5 8.6 Z"),
    line(GLOSS, "M5.9 7 L5.9 11.2 C5.9 13.4 6.9 15.3 8.5 16.6", 1.3),
]
_bolt = [(13.8, 2.4), (4.6, 13.4), (11.1, 13.4), (9.4, 21.6), (19.4, 9.4), (13, 9.4), (16.4, 2.4)]
HERO["energy"] = [
    fill("icon-gold", poly(_bolt), 1.2),
    fill(SHADE, poly([(12.5, 13.4), (9.4, 21.6), (19.4, 9.4), (17.2, 9.4)])),
    line(GLOSS, "M13.3 4.3 L7.9 11", 1.3),
]
HERO["ammo"] = [
    fill("icon-gold", rect(8.6, 9, 15.4, 18.2)),
    fill("icon-gold", rect(7.6, 17.6, 16.4, 21, 0.8)),
    fill("icon-orange", "M8.6 9.4 C8.6 5.8 10 3.2 12 2.6 C14 3.2 15.4 5.8 15.4 9.4 Z"),
    fill(SHADE, rect(13.4, 9.4, 15.4, 17.6)),
    fill(SHADE, "M13.4 9.4 L15.4 9.4 C15.4 6.6 14.6 4.4 13.4 3.3 Z"),
    fill(SHADE, rect(7.6, 19.6, 16.4, 21, 0.6)),
    line(GLOSS, "M10.4 10.6 L10.4 16.4", 1.2),
    line(GLOSS, "M10.3 7.6 C10.4 6.4 10.8 5.4 11.4 4.7", 1.1),
]
HERO["lock"] = [
    line("icon-steel", "M7.6 10.8 V7.8 A4.4 4.4 0 0 1 16.4 7.8 V10.8", 2.6),
    fill("icon-gold", rect(4.4, 10, 19.6, 21, 2)),
    fill(SHADE, rect(4.4, 17.6, 19.6, 21, 2)),
    fill(OUTLINE, circle(12, 14.2, 1.7)),
    fill(OUTLINE, poly([(11.2, 14.6), (12.8, 14.6), (13.3, 18.2), (10.7, 18.2)])),
    line(GLOSS, "M6.6 12.3 L6.6 15.6", 1.2),
]
HERO["key"] = [
    fill("icon-gold", circle(7.4, 12, 4.8) + " " + circle(7.4, 12, 1.9, False)),
    fill("icon-gold", rect(11, 10.6, 21.2, 13.4, 0.5)),
    fill("icon-gold", rect(15.2, 13, 17.2, 16.4, 0.4)),
    fill("icon-gold", rect(18.8, 13, 20.8, 17.6, 0.4)),
    fill(SHADE, rect(11.6, 12.4, 21.2, 13.4, 0.4)),
    line(GLOSS, arc(7.4, 12, 3.4, 105, 170), 1.2),
]
_lid = "M3.4 10.2 V8.2 C3.4 5.4 5.8 3.6 8.6 3.6 H15.4 C18.2 3.6 20.6 5.4 20.6 8.2 V10.2 Z"
HERO["chest"] = [
    fill("icon-wood", rect(3.4, 9.6, 20.6, 20.8, 1.2)),
    fill("icon-wood", _lid),
    fill(SHADE, rect(3.4, 9.4, 20.6, 10.6)),
    fill(SHADE, rect(3.4, 18, 20.6, 20.8, 1.2)),
    fill("icon-gold", rect(6, 4.6, 8.2, 20.8)), fill("icon-gold", rect(15.8, 4.6, 18, 20.8)),
    fill("icon-gold", rect(9.9, 8.4, 14.1, 13.6, 0.7)),
    fill(OUTLINE, circle(12, 10.5, 0.9)), fill(OUTLINE, rect(11.55, 10.6, 12.45, 12.4)),
    line(GLOSS, "M9.6 5.4 H14.4", 1.1),
]
HERO["gift"] = [
    fill("icon-red", rect(4.2, 10.2, 19.8, 20.8, 0.8)),
    fill("icon-red", rect(3, 7.2, 21, 10.8, 0.8)),
    fill(SHADE, rect(4.2, 10.8, 19.8, 12)),
    fill(SHADE, rect(16.8, 12, 19.8, 20.8, 0.8)),
    fill("icon-gold", rect(10.6, 7.2, 13.4, 20.8)),
    fill("icon-gold", "M12 7.4 C9.4 3.2 5.6 3.8 6.4 6.2 C6.9 7.6 9.6 7.6 12 7.4 Z"),
    fill("icon-gold", "M12 7.4 C14.6 3.2 18.4 3.8 17.6 6.2 C17.1 7.6 14.4 7.6 12 7.4 Z"),
    line(GLOSS, "M5 8.6 H8.6", 1.1),
]
HERO["crown"] = [
    fill("icon-gold", "M3.6 8.6 L7.6 12.6 L12 5.6 L16.4 12.6 L20.4 8.6 L18.8 17.6 H5.2 Z", 1.2),
    fill("icon-gold", circle(3.6, 7.6, 1.5)), fill("icon-gold", circle(12, 4.4, 1.5)),
    fill("icon-gold", circle(20.4, 7.6, 1.5)),
    fill("icon-gold", rect(5, 17, 19, 20.6, 0.8)),
    fill(SHADE, rect(5, 17, 19, 18.2)),
    fill("icon-red", poly([(12, 11.8), (13.8, 14), (12, 16.2), (10.2, 14)])),
    line(GLOSS, "M7 18.8 H10.6", 1),
]
HERO["potion"] = [
    fill("icon-glass", rect(9.6, 4.4, 14.4, 9.6)),
    fill("icon-glass", circle(12, 14.4, 6.6)),
    fill("icon-red", f"M{f(12 - math.sqrt(5.4**2 - 0.8**2))} 13.6 "
         f"A5.4 5.4 0 0 0 {f(12 + math.sqrt(5.4**2 - 0.8**2))} 13.6 Z"),
    fill(SHADE, f"M{f(12 - math.sqrt(5.4**2 - 2.4**2))} 16.8 A5.4 5.4 0 0 0 {f(12 + math.sqrt(5.4**2 - 2.4**2))} 16.8 Z"),
    fill("icon-wood", rect(9.2, 2.4, 14.8, 5.2, 0.6)),
    line(GLOSS, arc(12, 14.4, 4.7, 115, 170), 1.3),
]
HERO["ticket"] = [
    fill("icon-orange", "M2.6 6.4 H21.4 V10 A2 2 0 0 0 21.4 14 V17.6 H2.6 V14 A2 2 0 0 0 2.6 10 Z", 0.8),
    fill(SHADE, rect(2.6, 15.6, 21.4, 17.6)),
    line(SHADE, rect(6.2, 8.8, 17.8, 15.2), 1),
    line(GLOSS, "M4.4 8.2 H9", 1.1),
]
HERO["xp"] = disc_icon("icon-green", [fill(MARK, sparkle(12, 11.5, 4.6))])
HERO["alert"] = disc_icon("icon-red", [
    line(MARK, "M12 6.6 V12.4", 2.6), fill(MARK, circle(12, 16.1, 1.5))])
_burst = star_pts(12, 12, 10, 7.6, 12, 90)
HERO["new"] = [
    fill("icon-red", poly(_burst), 0.8),
    fill(SHADE, poly([(12, 12)] + _burst[16:24] + [_burst[0]])),
    fill(MARK, sparkle(12, 12, 5)),
]
HERO["rarity"] = [
    fill("icon-blue", poly([(12, 2.6), (21, 12), (12, 21.4), (3, 12)]), 1),
    fill(GLOSS, poly([(12, 2.6), (12, 7.6), (7.6, 12), (3, 12)])),
    fill(SHADE, poly([(21, 12), (12, 21.4), (12, 16.4), (16.4, 12)])),
]
HERO["trophy"] = [
    line("icon-gold", "M6.4 5.8 H4 C3.6 9.4 5 11 7.2 11.2", 1.7),
    line("icon-gold", "M17.6 5.8 H20 C20.4 9.4 19 11 16.8 11.2", 1.7),
    fill("icon-gold", "M6 3.4 H18 V8.4 C18 12.2 15.4 14.6 12 14.6 C8.6 14.6 6 12.2 6 8.4 Z"),
    fill("icon-gold", rect(10.8, 14, 13.2, 17.6)),
    fill("icon-wood", rect(7, 17.4, 17, 20.8, 0.9)),
    fill(SHADE, "M14.6 3.4 H18 V8.4 C18 11.6 16.2 13.8 13.6 14.4 C15.2 12.6 15.6 10.4 14.6 3.4 Z"),
    line(GLOSS, "M8.4 5.4 V8.6", 1.2),
]
HERO["hourglass"] = [
    fill("icon-glass", "M6.6 4.6 H17.4 C17.4 9 13.2 10.6 13.2 12 C13.2 13.4 17.4 15 17.4 19.4 H6.6 "
         "C6.6 15 10.8 13.4 10.8 12 C10.8 10.6 6.6 9 6.6 4.6 Z"),
    fill("icon-gold", "M8.4 7 H15.6 C15 8.8 12.8 10 12 11 C11.2 10 9 8.8 8.4 7 Z"),
    fill("icon-gold", "M12 14.6 C13.2 15.6 15.8 17 16 19.4 H8 C8.2 17 10.8 15.6 12 14.6 Z"),
    fill("icon-wood", rect(4.8, 2.4, 19.2, 5, 0.8)),
    fill("icon-wood", rect(4.8, 19, 19.2, 21.6, 0.8)),
    line(GLOSS, "M8.4 5.8 C8.6 7.6 9.8 8.8 10.8 9.6", 1),
]


def svg(body):
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">\n  '
            + "\n  ".join(body) + "\n</svg>\n")


def tokens(d):
    """A path's data as commands and numbers; an arc's flags are single
    digits however they are packed (`a1 1 0 0110 10`)."""
    out, i, cmd, argn = [], 0, None, 0
    ARGS = {"m": 2, "l": 2, "h": 1, "v": 1, "c": 6, "s": 4, "q": 4, "t": 2, "a": 7, "z": 0}
    num = re.compile(r"[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?")
    while i < len(d):
        c = d[i]
        if c.isalpha():
            cmd, argn = c, 0
            out.append(c)
            i += 1
        elif c in " ,\t\n\r":
            i += 1
        else:
            if cmd and cmd.lower() == "a" and argn % 7 in (3, 4):
                out.append(float(c))
                i += 1
            else:
                m = num.match(d, i)
                if not m:
                    raise ValueError(f"bad path data at {d[i:i + 20]!r}")
                out.append(float(m.group()))
                i = m.end()
            argn += 1
    return out


def scaled(d, k):
    """Path data `d` scaled by `k` (every coordinate and radius; an arc's
    rotation and flags kept)."""
    out, cmd, n = [], None, 0
    for t in tokens(d):
        if isinstance(t, str):
            cmd, n = t, 0
            out.append(t)
            continue
        if cmd.lower() == "a" and n % 7 in (2, 3, 4):
            out.append(str(int(t)) if n % 7 != 2 else f(t))
        else:
            out.append(f(t * k))
        n += 1
    s = ""
    for t in out:
        if t[0].isalpha():
            s += t
        else:
            s += ("" if s == "" or s[-1].isalpha() else " ") + t
    return s


def borrowed(path):
    url = f"https://raw.githubusercontent.com/game-icons/icons/{GAME_ICONS_REV}/{path}.svg"
    text = urllib.request.urlopen(url, timeout=60).read().decode()
    paths = re.findall(r'<path([^>]*)/>', text)
    body = []
    for attrs in paths:
        d = re.search(r'\bd="([^"]+)"', attrs).group(1)
        if d.strip() in ("M0 0h512v512H0z", "M0 0h512v512H0z "):
            continue  # the site's black square behind the icon
        body.append(f'<path d="{scaled(d, 24 / 512)}" fill="currentColor"/>')
    if not body:
        raise ValueError(f"{path}: no paths")
    return svg(body)


def main():
    os.makedirs(os.path.join(OUT, "game"), exist_ok=True)
    names = []
    for name, body in HERO.items():
        open(os.path.join(OUT, "game", f"{name}.svg"), "w").write(svg(body))
        names.append(name)
    credits = []
    for name, path in (item for _, items in BORROWED for item in items):
        assert name not in HERO, name
        open(os.path.join(OUT, "game", f"{name}.svg"), "w").write(borrowed(path))
        names.append(name)
        credits.append((name, path))
    lines = [
        "type icons", "version 1", "---",
        "# The engine's game icon set (templates/tools/make-game-icons.py writes",
        "# this file and game/): our own hero icons in tones first, drawn as",
        "# stickers (an icon-outline rim and an icon-shadow drop, in 24ths of",
        "# the em), then game-icons.net silhouettes (CC BY 3.0,",
        "# third_party/game-icons/ICONS.md) in the icon's ink.",
        'source "game"', "size 48", "range 8", "join round", "sticker 1.3 0 1.2",
    ]
    lines += [f'icon "{n}"' for n in names]
    open(os.path.join(OUT, "game.ogt"), "w").write("\n".join(lines) + "\n")
    tp = os.path.join(REPO, "third_party", "game-icons")
    os.makedirs(tp, exist_ok=True)
    with open(os.path.join(tp, "ICONS.md"), "w") as out:
        out.write("# game-icons.net icons in the engine's game icon set\n\n"
                  f"From https://github.com/game-icons/icons at `{GAME_ICONS_REV}`, CC BY 3.0\n"
                  "(`LICENSE`), their 512-unit paths scaled to the 24-unit grid and the\n"
                  "site's background square dropped by `templates/tools/make-game-icons.py`.\n"
                  "The engine's credits (`packages/engine/src/credits.ts`) name each author.\n\n"
                  "| Engine name | game-icons.net icon | Author |\n|---|---|---|\n")
        for name, path in credits:
            out.write(f"| `{name}` | `{path}` | {AUTHORS[path.split('/')[0]]} |\n")
    readme(names)
    print(f"{len(HERO)} hero icons, {len(credits)} borrowed, {len(names)} in game.ogt")


def readme(names):
    """The set's README: every name, the hero icons with their body colours."""
    sticker = {SHADE, GLOSS, MARK, OUTLINE}
    out = [
        "# The engine's game icon set", "",
        "`game.ogt` is the icon set `<Icon name>` draws from when the game has no icon of",
        "that name (the default pack's `iconSet`, docs/ui.md §3.3); names it lacks fall",
        "back to the engine's Lucide line icons (`back`, `close`, `chevronLeft`, `gear` …).",
        "Every icon sits on a 24-unit grid and is cooked as a distance field at 48 px",
        "an em, crisp from 16 to 64 px. `templates/tools/make-game-icons.py` writes this",
        "directory; edit the tool, not the files.", "",
        "## Hero icons (tones)", "",
        "Our own designs (CC0), drawn as stickers: an `icon-outline` rim and an",
        "`icon-shadow` drop under the body, which is painted in theme colours, then",
        "`icon-shade` and `icon-gloss` shapes and `icon-mark` marks over it. A pack",
        "restyles all of them through those colours (CONVENTIONS.md §The default",
        "pack's names); a pack whose own `iconSet` names an icon replaces it.", "",
        "| Name | Body colours |", "|---|---|",
    ]
    for name, body in HERO.items():
        roles = sorted(set(re.findall(r"var\(--color-([a-z-]+)\)", "".join(body))) - sticker)
        out.append(f"| `{name}` | " + (", ".join(f"`{r}`" for r in roles) or "the ink (`fill-*` colours it)") + " |")
    out += ["", "## Ink icons", "",
            "Filled game-icons.net silhouettes (CC BY 3.0; authors in",
            "`third_party/game-icons/ICONS.md`, credited on the Credits screen), drawn in the",
            "icon's ink (`text-*`, or `fill-*`).", ""]
    for kind, items in BORROWED:
        out.append(f"- **{kind}**: " + ", ".join(f"`{n}`" for n, _ in items))
    open(os.path.join(OUT, "README.md"), "w").write("\n".join(out) + "\n")


if __name__ == "__main__":
    main()
