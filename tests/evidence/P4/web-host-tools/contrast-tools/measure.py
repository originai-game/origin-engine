# Polish's text contrast (packages/engine/src/node/polish.ts textBoxes) on og run shots.
import json, sys
from PIL import Image
def lin(v):
    s = v / 255
    return s / 12.92 if s <= 0.04045 else ((s + 0.055) / 1.055) ** 2.4
LUT = [lin(v) for v in range(256)]
def lum(p): return 0.2126 * LUT[p[0]] + 0.7152 * LUT[p[1]] + 0.0722 * LUT[p[2]]
def measure(png, ui, want):
    img = Image.open(png).convert("RGB"); W, H = img.size; px = img.load()
    els = json.load(open(ui))
    els = els.get("elements", els.get("value", {}).get("elements", [])) if isinstance(els, dict) else els
    out = []
    for e in els:
        if e.get("id") not in want and not any(e.get("id","").endswith("/"+w) or e.get("id","")==w for w in want): continue
        if not e.get("text"): continue
        ix = min(e["height"] / 2, 12) if e["width"] > 3 * e["height"] else 0; iy = e["height"] * 0.12
        x0, y0 = max(0, int(e["x"] + ix)), max(0, int(e["y"] + iy))
        x1, y1 = min(W, -int(-(e["x"] + e["width"] - ix))), min(H, -int(-(e["y"] + e["height"] - iy)))
        L = [lum(px[x, y]) for y in range(y0, y1) for x in range(x0, x1)]
        s = sorted(L); g = s[len(s)//2]
        dev = sorted(abs(v - g) for v in L); far = dev[min(len(dev)-1, int(0.99 * len(dev)))]
        up = [v for v in L if abs(v-g) >= far and v > g]; dn = [v for v in L if abs(v-g) >= far and v <= g]
        ink = sum(up)/len(up) if len(up) >= len(dn) else sum(dn)/len(dn)
        c = (max(ink, g) + 0.05) / (min(ink, g) + 0.05)
        out.append((e["id"], e["text"][:24], round(c, 2), round(g, 4), round(ink, 4)))
    return out
if __name__ == "__main__":
    for r in measure(sys.argv[1], sys.argv[2], sys.argv[3].split(",")): print(*r, sep="\t")
