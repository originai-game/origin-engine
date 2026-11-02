"""The fox: a four-legged animal facing right, one SVG group per part of
the quadruped template (docs/art.md §5), drawn where it sits in the setup
pose and overlapping its parent at each joint: `-l` parts are the far
side, drawn behind the body. make.sh renders each group to a canvas-sized
PNG and stacks them into fox.psd. Made here for the engine; CC0."""

W, H = 768, 512
INK = "#2a1a12"

def grad(id, a, b, x2=1, y2=0):
    return (f'<linearGradient id="{id}" x1="0" y1="0" x2="{x2}" y2="{y2}">'
            f'<stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient>')

DEFS = "".join([
    grad("fur", "#f0893a", "#c5531c", 0, 1),
    grad("fur-far", "#c86a2a", "#8f3c14", 0, 1),
    grad("belly", "#fff4e6", "#f1d9bf", 0, 1),
    grad("sock", "#3a2a24", "#1f1512", 0, 1),
    grad("tail", "#f39a48", "#c5531c", 1, 0),
])

def leg(end, side):
    far = side == "l"
    fur = "url(#fur-far)" if far else "url(#fur)"
    x = (476 if end == "front" else 286) + (-16 if far else 0)
    top = 286 if end == "front" else 262
    upper_w = 38 if end == "front" else 52
    return {
        f"leg-{end}-upper-{side}": (f'<rect x="{x - upper_w / 2}" y="{top}" width="{upper_w}" height="{386 - top}" rx="{upper_w / 2}" '
                                    f'fill="{fur}" stroke="{INK}" stroke-width="4"/>'),
        f"leg-{end}-lower-{side}": (f'<rect x="{x - 14}" y="366" width="28" height="94" rx="14" fill="url(#sock)" stroke="{INK}" stroke-width="4"/>'),
        f"paw-{end}-{side}": (f'<path d="M{x - 16} 446 L{x + 14} 446 Q{x + 40} 452 {x + 40} 476 L{x - 18} 476 Z" '
                              f'fill="url(#sock)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'),
    }

PARTS = {
    **leg("back", "l"),
    **leg("front", "l"),
    "ear-l": (f'<path d="M560 118 L572 50 L604 104 Z" fill="url(#fur-far)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'),
    "tail": (f'<path d="M262 236 Q200 222 140 262 Q80 300 52 360 Q100 352 132 330 Q120 352 128 372 Q186 330 220 300 Q252 282 270 268 Z" '
             f'fill="url(#tail)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
             f'<path d="M52 360 Q74 330 104 316 Q100 336 112 348 Q84 356 52 360 Z" fill="#fff4e6" stroke="{INK}" stroke-width="3" stroke-linejoin="round"/>'),
    "body": (f'<path d="M248 226 Q360 196 470 214 Q530 228 524 284 Q516 336 470 340 Q360 352 262 330 Q220 316 222 276 Q224 236 248 226 Z" '
             f'fill="url(#fur)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
             f'<path d="M300 328 Q380 344 470 330 Q440 306 380 312 Q330 314 300 328 Z" fill="url(#belly)"/>'),
    "neck": (f'<path d="M456 230 Q480 170 540 150 L580 186 Q560 236 520 290 Q484 284 456 230 Z" fill="url(#fur)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
             f'<path d="M520 288 Q548 240 568 196 Q580 230 556 268 Z" fill="url(#belly)"/>'),
    "head": (f'<path d="M532 120 Q574 88 616 110 Q640 124 690 150 Q700 160 690 168 Q650 182 606 186 Q556 190 534 170 Q516 146 532 120 Z" '
             f'fill="url(#fur)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
             f'<path d="M606 186 Q650 182 690 168 Q660 160 628 160 Q600 164 586 176 Z" fill="url(#belly)"/>'
             f'<circle cx="692" cy="158" r="7" fill="{INK}"/>'
             f'<path d="M598 132 Q608 124 618 132" fill="none" stroke="{INK}" stroke-width="5" stroke-linecap="round"/>'),
    "jaw": (f'<path d="M604 180 Q640 178 676 172 Q672 190 640 194 Q614 196 600 188 Z" fill="url(#belly)" stroke="{INK}" stroke-width="3" stroke-linejoin="round"/>'),
    "ear-r": (f'<path d="M548 122 L548 44 L592 106 Z" fill="url(#fur)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
              f'<path d="M554 108 L554 66 L578 100 Z" fill="#3a2a24"/>'),
    **leg("back", "r"),
    **leg("front", "r"),
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
