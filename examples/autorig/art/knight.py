"""The knight: a side-view humanoid facing right, one SVG group per part
of the humanoid-side template (docs/art.md §2.8), drawn where it sits in the
setup pose and overlapping its parent at each joint. make.sh renders each
group to a canvas-sized PNG and stacks them into knight.psd. Made here for
the engine; CC0."""

W, H = 512, 768
INK = "#1b1f2a"

def grad(id, a, b, x2=1, y2=0):
    return (f'<linearGradient id="{id}" x1="0" y1="0" x2="{x2}" y2="{y2}">'
            f'<stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient>')

DEFS = "".join([
    grad("steel", "#e6ebf2", "#7d8796"),
    grad("steel-far", "#a9b0bc", "#59616f"),
    grad("steel-v", "#eef2f7", "#8a94a3", 0, 1),
    grad("cape", "#b3283a", "#5e1020", 0, 1),
    grad("tabard", "#c93545", "#7d1626", 0, 1),
    grad("plume", "#e2404f", "#8f1727", 1, 1),
    grad("blade", "#ffffff", "#9aa4b4", 1, 0),
    grad("leather", "#7a4f30", "#4a2e1b", 0, 1),
])

def capsule(x, y1, y2, w, fill, extra=""):
    """A limb from y1 to y2, w wide, rounded ends, centred on x."""
    r = w / 2
    return (f'<rect x="{x - r}" y="{y1}" width="{w}" height="{y2 - y1}" rx="{r}" '
            f'fill="{fill}" stroke="{INK}" stroke-width="4"/>{extra}')

def band(x, y, w, fill="#c9a43a"):
    return f'<rect x="{x - w / 2 + 3}" y="{y - 4}" width="{w - 6}" height="8" rx="3" fill="{fill}" stroke="{INK}" stroke-width="2"/>'

def arm(side):
    far = side == "l"
    dx = -8 if far else 6
    steel = "url(#steel-far)" if far else "url(#steel)"
    x = 256 + dx
    return {
        f"arm-upper-{side}": capsule(x, 232, 352, 46, steel,
            f'<path d="M{x - 30} 236 q30 -26 60 0 l-4 40 q-26 -10 -52 0 z" fill="{steel}" stroke="{INK}" stroke-width="4"/>'),
        f"arm-lower-{side}": capsule(x, 330, 448, 40, steel, band(x, 432, 40)),
        f"hand-{side}": (f'<rect x="{x - 21}" y="428" width="42" height="50" rx="16" fill="url(#leather)" stroke="{INK}" stroke-width="4"/>'
                         f'<rect x="{x - 21}" y="428" width="42" height="14" rx="5" fill="#5b3a22" stroke="{INK}" stroke-width="3"/>'),
    }

def leg(side):
    far = side == "l"
    x = 248 if far else 266
    steel = "url(#steel-far)" if far else "url(#steel)"
    return {
        f"leg-upper-{side}": capsule(x, 384, 552, 54, steel),
        f"leg-lower-{side}": capsule(x, 524, 690, 48, steel,
            f'<circle cx="{x + 4}" cy="540" r="17" fill="{steel}" stroke="{INK}" stroke-width="4"/>'),
        f"foot-{side}": (f'<path d="M{x - 26} 668 L{x + 18} 668 Q{x + 64} 680 {x + 70} 716 L{x - 28} 716 Z" '
                         f'fill="{steel}" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'),
    }

PARTS = {
    "cape": (f'<path d="M214 236 Q260 222 286 242 L272 470 Q240 560 150 586 Q170 470 176 360 Q182 280 214 236 Z" '
             f'fill="url(#cape)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
             f'<path d="M206 300 Q196 420 170 540" fill="none" stroke="#4a0c18" stroke-width="5" stroke-linecap="round"/>'),
    **arm("l"),
    **leg("l"),
    "pelvis": (f'<path d="M206 380 L310 380 L318 470 Q258 490 198 470 Z" fill="url(#tabard)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
               f'<rect x="202" y="372" width="112" height="22" rx="6" fill="url(#leather)" stroke="{INK}" stroke-width="4"/>'
               f'<rect x="248" y="374" width="20" height="18" rx="3" fill="#d9b64a" stroke="{INK}" stroke-width="3"/>'),
    "torso": (f'<path d="M206 236 Q256 214 310 236 Q330 320 314 398 L202 398 Q186 320 206 236 Z" fill="url(#steel-v)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
              f'<path d="M232 260 Q286 250 300 290" fill="none" stroke="#ffffff" stroke-width="6" stroke-linecap="round" opacity="0.7"/>'
              f'<path d="M210 340 Q258 352 312 338" fill="none" stroke="{INK}" stroke-width="3"/>'),
    "neck": capsule(258, 196, 250, 38, "url(#steel-far)"),
    "head": (f'<path d="M206 160 Q206 92 262 90 Q322 92 322 160 L318 214 Q262 230 208 214 Z" fill="url(#steel)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
             f'<path d="M262 142 L326 142 L326 178 L270 178 Z" fill="#2a3040" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
             f'<path d="M276 160 L322 160" stroke="#8fd0ff" stroke-width="4" stroke-linecap="round"/>'
             f'<path d="M226 112 Q240 100 262 100" fill="none" stroke="#ffffff" stroke-width="6" stroke-linecap="round" opacity="0.8"/>'),
    "hair": (f'<path d="M248 96 Q236 52 270 40 Q246 64 214 74 Q186 92 168 142 Q178 104 212 96 Q232 92 248 106 Z" '
             f'fill="url(#plume)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'),
    **leg("r"),
    "arm-upper-r": arm("r")["arm-upper-r"],
    "arm-lower-r": arm("r")["arm-lower-r"],
    "weapon": (f'<path d="M270 448 L420 300 L432 312 L282 460 Z" fill="url(#blade)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
               f'<path d="M236 438 L300 470 L294 482 L230 450 Z" fill="#d9b64a" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
               f'<rect x="250" y="446" width="16" height="44" rx="5" transform="rotate(-45 258 468)" fill="url(#leather)" stroke="{INK}" stroke-width="3"/>'
               f'<circle cx="240" cy="486" r="9" fill="#d9b64a" stroke="{INK}" stroke-width="3"/>'),
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
