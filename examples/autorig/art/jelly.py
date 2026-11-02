"""The jelly: a simple creature, one SVG group per part of the creature
template (docs/art.md §5): a domed body, two eyes on it and five tentacles
hanging from it, the back two behind the body. make.sh renders each
group to a canvas-sized PNG and stacks them into jelly.psd. Made here for
the engine; CC0."""

W, H = 512, 512
INK = "#1a2440"

def grad(id, a, b, x2=1, y2=0):
    return (f'<linearGradient id="{id}" x1="0" y1="0" x2="{x2}" y2="{y2}">'
            f'<stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient>')

DEFS = "".join([
    grad("dome", "#9ff0e0", "#3fb8b0", 0, 1),
    grad("arm", "#5fd0c4", "#2a8c96", 0, 1),
    grad("arm-back", "#3f9fa4", "#1f6670", 0, 1),
])

def tentacle(x, back):
    fill = "url(#arm-back)" if back else "url(#arm)"
    return (f'<path d="M{x - 15} 300 Q{x - 20} 380 {x - 6} 440 Q{x + 2} 474 {x + 10} 484 Q{x + 14} 470 {x + 8} 440 '
            f'Q{x + 2} 380 {x + 15} 300 Z" fill="{fill}" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>')

def eye(x):
    return (f'<ellipse cx="{x}" cy="224" rx="24" ry="28" fill="#ffffff" stroke="{INK}" stroke-width="4"/>'
            f'<ellipse cx="{x + 4}" cy="230" rx="12" ry="15" fill="{INK}"/>'
            f'<circle cx="{x + 9}" cy="222" r="5" fill="#ffffff"/>')

PARTS = {
    "limb-2": tentacle(211, True),
    "limb-4": tentacle(301, True),
    "body": (f'<path d="M132 300 Q120 130 256 112 Q392 130 380 300 Q350 322 320 304 Q288 326 256 306 Q224 326 192 304 Q162 322 132 300 Z" '
             f'fill="url(#dome)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
             f'<path d="M176 168 Q210 136 256 134" fill="none" stroke="#ffffff" stroke-width="10" stroke-linecap="round" opacity="0.7"/>'
             f'<path d="M232 270 Q256 286 280 270" fill="none" stroke="{INK}" stroke-width="5" stroke-linecap="round"/>'),
    "eye-l": eye(212),
    "eye-r": eye(300),
    "limb-1": tentacle(166, False),
    "limb-3": tentacle(256, False),
    "limb-5": tentacle(346, False),
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
