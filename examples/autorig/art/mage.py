"""The mage: a front-view humanoid facing the viewer, one SVG group per
part of the humanoid-front template (docs/art.md §5), drawn where it sits
in the setup pose and overlapping its parent at each joint: `-l` parts on
the image's left. make.sh renders each group to a canvas-sized PNG and
stacks them into mage.psd. Made here for the engine; CC0."""

W, H = 512, 768
INK = "#1d1830"

def grad(id, a, b, x2=1, y2=0):
    return (f'<linearGradient id="{id}" x1="0" y1="0" x2="{x2}" y2="{y2}">'
            f'<stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient>')

DEFS = "".join([
    grad("robe", "#5a7bd8", "#2c3f8f", 0, 1),
    grad("robe-l", "#3f5bb5", "#24357a", 1, 0),
    grad("robe-r", "#6f8fe6", "#3a51a6", 1, 0),
    grad("cape", "#7a2a5c", "#3b1030", 0, 1),
    grad("skin", "#f6d2b0", "#e2ac86", 0, 1),
    grad("hair", "#f2c45a", "#c98a22", 0, 1),
    grad("boot", "#6b4428", "#3c2414", 0, 1),
    grad("legs", "#38304f", "#211b33", 1, 0),
    grad("wood", "#a8743e", "#6a4520", 1, 0),
    grad("gem", "#bff6ff", "#3fb6d8", 1, 1),
])

def capsule(x, y1, y2, w, fill, extra=""):
    r = w / 2
    return (f'<rect x="{x - r}" y="{y1}" width="{w}" height="{y2 - y1}" rx="{r}" '
            f'fill="{fill}" stroke="{INK}" stroke-width="4"/>{extra}')

def arm(side):
    s = -1 if side == "l" else 1
    x = 256 + s * 74
    robe = "url(#robe-l)" if side == "l" else "url(#robe-r)"
    cuff = f'<rect x="{x - 19}" y="430" width="38" height="14" rx="6" fill="#e8c35a" stroke="{INK}" stroke-width="3"/>'
    return {
        f"arm-upper-{side}": capsule(x, 244, 364, 42, robe),
        f"arm-lower-{side}": capsule(x, 342, 446, 38, robe, cuff),
        f"hand-{side}": (f'<rect x="{x - 18}" y="432" width="36" height="46" rx="16" fill="url(#skin)" stroke="{INK}" stroke-width="4"/>'
                         f'<path d="M{x - 10} 470 L{x + 10} 470" stroke="{INK}" stroke-width="2" stroke-linecap="round" opacity="0.5"/>'),
    }

def leg(side):
    s = -1 if side == "l" else 1
    x = 256 + s * 26
    return {
        f"leg-upper-{side}": capsule(x, 470, 606, 44, "url(#legs)"),
        f"leg-lower-{side}": capsule(x, 584, 700, 40, "url(#legs)"),
        f"foot-{side}": (f'<ellipse cx="{x + s * 16}" cy="716" rx="34" ry="14" fill="url(#boot)" stroke="{INK}" stroke-width="4"/>'
                         f'<rect x="{x - 22}" y="672" width="44" height="46" rx="10" fill="url(#boot)" stroke="{INK}" stroke-width="4"/>'),
    }

PARTS = {
    "cape": (f'<path d="M190 250 Q256 228 322 250 Q360 420 372 610 Q256 640 140 610 Q152 420 190 250 Z" '
             f'fill="url(#cape)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'),
    "hair": (f'<path d="M168 150 Q168 70 256 66 Q344 70 344 150 L352 300 Q330 318 306 300 L300 210 L212 210 L206 300 Q182 318 160 300 Z" '
             f'fill="url(#hair)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'),
    **leg("l"),
    **leg("r"),
    "pelvis": (f'<path d="M206 392 L306 392 Q330 450 348 512 Q256 534 164 512 Q182 450 206 392 Z" '
               f'fill="url(#robe)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
               f'<path d="M172 500 Q256 520 340 500" fill="none" stroke="#e8c35a" stroke-width="6"/>'),
    "torso": (f'<path d="M206 240 Q256 226 306 240 Q330 250 330 280 L310 404 Q256 414 202 404 L182 280 Q182 250 206 240 Z" '
              f'fill="url(#robe)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
              f'<path d="M256 244 L256 404" stroke="#e8c35a" stroke-width="5"/>'
              f'<path d="M210 380 Q256 392 302 380" fill="none" stroke="#e8c35a" stroke-width="8"/>'),
    "neck": capsule(256, 196, 252, 36, "url(#skin)"),
    "head": (f'<path d="M196 140 Q196 84 256 82 Q316 84 316 140 Q316 210 256 220 Q196 210 196 140 Z" fill="url(#skin)" stroke="{INK}" stroke-width="4"/>'
             f'<path d="M196 132 Q210 84 256 84 Q302 84 316 132 Q290 112 270 118 Q256 98 236 116 Q214 108 196 132 Z" fill="url(#hair)" stroke="{INK}" stroke-width="3" stroke-linejoin="round"/>'
             f'<ellipse cx="234" cy="154" rx="9" ry="12" fill="{INK}"/><ellipse cx="278" cy="154" rx="9" ry="12" fill="{INK}"/>'
             f'<circle cx="237" cy="150" r="3" fill="#fff"/><circle cx="281" cy="150" r="3" fill="#fff"/>'
             f'<path d="M246 188 Q256 196 266 188" fill="none" stroke="{INK}" stroke-width="3" stroke-linecap="round"/>'
             f'<ellipse cx="220" cy="176" rx="9" ry="5" fill="#f19a9a" opacity="0.6"/><ellipse cx="292" cy="176" rx="9" ry="5" fill="#f19a9a" opacity="0.6"/>'),
    **arm("l"),
    "arm-upper-r": arm("r")["arm-upper-r"],
    "arm-lower-r": arm("r")["arm-lower-r"],
    "weapon": (f'<rect x="323" y="236" width="14" height="404" rx="6" fill="url(#wood)" stroke="{INK}" stroke-width="4"/>'
               f'<path d="M330 172 L352 206 L330 242 L308 206 Z" fill="url(#gem)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
               f'<path d="M314 240 Q330 226 346 240" fill="none" stroke="#e8c35a" stroke-width="6"/>'),
    "hand-r": arm("r")["hand-r"],
}

def svg(only=None):
    hide = ' display="none"'
    body = "".join(f'<g id="{n}"{"" if only in (None, n) else hide}>{g}</g>' for n, g in PARTS.items())
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}"><defs>{DEFS}</defs>{body}</svg>'

if __name__ == "__main__":
    import os, sys
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    open(os.path.join(out, "all.svg"), "w").write(svg())
    for i, n in enumerate(PARTS):
        open(os.path.join(out, f"{i + 1:02d}-{n}.svg"), "w").write(svg(n))
