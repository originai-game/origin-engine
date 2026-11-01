"""Adds the marks the UI draws (− × · … ← →) to bundled font subsets that
lack them, copied from the family's full font (Google Fonts, OFL 1.1) at the
instance whose shared glyphs fit the subset best, scaled to its em. Existing
glyphs are untouched; a mark the family itself lacks is reported, and og-ui
draws it from the next font of the text's chain.

    pip install fonttools
    OG_FONT_SOURCES=<dir of the families' full .ttf> python add-marks.py <font>...

The import settings (`<font>.ogt`) must name a mark outside `charset latin
punct` in `chars` for the cook to put it in the atlas."""
import sys, os, itertools
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.boundsPen import BoundsPen

MARKS = [0x2212, 0xD7, 0xB7, 0x2026, 0x2190, 0x2192]
SRC = os.environ.get("OG_FONT_SOURCES", ".").rstrip("/") + "/"
FAMILY_SRC = {
    "Noto Sans SC": "NotoSansSC[wght].ttf", "Noto Serif SC": "NotoSerifSC[wght].ttf",
    "DotGothic16": "DotGothic16-Regular.ttf", "Newsreader": "Newsreader[opsz,wght].ttf",
    "Cinzel": "Cinzel[wght].ttf", "Orbitron": "Orbitron[wght].ttf", "Rajdhani": "Rajdhani-Bold.ttf",
    "Fredoka": "Fredoka[wdth,wght].ttf", "Lilita One": "LilitaOne-Regular.ttf",
    "ZCOOL KuaiLe": "ZCOOLKuaiLe-Regular.ttf", "Inter": "Inter[opsz,wght].ttf", "Nunito": "Nunito[wght].ttf",
    "Nunito ExtraLight": "Nunito[wght].ttf",
}
_src_cache = {}

def source(path):
    if path not in _src_cache:
        _src_cache[path] = TTFont(SRC + path)
    return _src_cache[path]

def candidates(src, weight):
    if "fvar" not in src:
        yield None, src
        return
    axes = {a.axisTag: a for a in src["fvar"].axes}
    grids = []
    for tag, a in axes.items():
        if tag == "wght":
            vals = sorted({min(max(weight, a.minValue), a.maxValue), a.defaultValue})
            vals = sorted(set(vals) | {min(max(w, a.minValue), a.maxValue) for w in range(int(a.minValue), int(a.maxValue) + 1, 50)})
        else:
            vals = sorted({a.minValue, a.defaultValue, a.maxValue} | (set(range(int(a.minValue), int(a.maxValue) + 1)) if tag == "opsz" else set()))
        grids.append([(tag, v) for v in vals])
    combos = [dict(c) for c in itertools.product(*grids)]
    combos.sort(key=lambda c: abs(c.get("wght", weight) - weight))
    for c in combos:
        yield c, None

def score(tgt, inst):
    """Mean difference of advances and outline bounds over the glyphs both map."""
    tc, ic = tgt.getBestCmap(), inst.getBestCmap()
    tg, ig = tgt.getGlyphSet(), inst.getGlyphSet()
    k = tgt["head"].unitsPerEm / inst["head"].unitsPerEm
    errs = []
    shared = [u for u in tc if u in ic][:400]
    for u in shared:
        a, b = BoundsPen(tg), BoundsPen(ig)
        tg[tc[u]].draw(a)
        ig[ic[u]].draw(b)
        e = abs(tg[tc[u]].width - ig[ic[u]].width * k)
        if a.bounds and b.bounds:
            e += sum(abs(x - y * k) for x, y in zip(a.bounds, b.bounds)) / 4
        errs.append(e)
    return (sum(errs) / len(errs) if errs else 1e9), len(errs)

def small_source(path, tgt):
    from fontTools import subset
    src = TTFont(SRC + path)
    cm = sorted(tgt.getBestCmap())
    want = set(cm[:400]) | set(MARKS)
    opts = subset.Options()
    opts.layout_features = []
    opts.notdef_outline = True
    opts.name_IDs = ["*"]
    sub = subset.Subsetter(opts)
    sub.populate(unicodes=sorted(want))
    sub.subset(src)
    return src

def best_instance(fam, tgt):
    path = FAMILY_SRC[fam]
    src = small_source(path, tgt)
    weight = tgt["OS/2"].usWeightClass
    best = None
    for loc, static in candidates(src, weight):
        inst = static if static is not None else instancer.instantiateVariableFont(src, loc, inplace=False)
        sc, n = score(tgt, inst)
        if best is None or sc < best[0]:
            best = (sc, loc, inst, n)
        if sc < 0.5:
            break
    return best

def add(tgt_path, dry=False):
    tgt = TTFont(tgt_path)
    n = tgt["name"]
    fam = n.getDebugName(16) or n.getDebugName(1)
    if fam not in FAMILY_SRC:
        return f"skip (no source for {fam})"
    cmap = tgt.getBestCmap()
    missing = [u for u in MARKS if u not in cmap]
    if not missing:
        return "complete"
    sc, loc, inst, nshared = best_instance(fam, tgt)
    icmap = inst.getBestCmap()
    have = [u for u in missing if u in icmap]
    if not have:
        return f"family lacks {''.join(chr(u) for u in missing)}"
    k = tgt["head"].unitsPerEm / inst["head"].unitsPerEm
    gs = inst.getGlyphSet()
    order = tgt.getGlyphOrder()
    glyf, hmtx = tgt["glyf"], tgt["hmtx"]
    added = []
    for u in have:
        sname = icmap[u]
        name = f"uni{u:04X}"
        while name in glyf.glyphs:
            name += ".u6b"
        from fontTools.pens.recordingPen import DecomposingRecordingPen
        rec = DecomposingRecordingPen(gs)
        gs[sname].draw(rec)
        pen = TTGlyphPen(None)
        rec.replay(TransformPen(pen, (k, 0, 0, k, 0, 0)))
        g = pen.glyph()
        glyf[name] = g

        g.recalcBounds(glyf)
        adv = round(gs[sname].width * k)
        lsb = g.xMin if hasattr(g, "xMin") else 0
        hmtx.metrics[name] = (adv, lsb)
        if "vmtx" in tgt:
            ref = next(iter(tgt["vmtx"].metrics.values()))
            tgt["vmtx"].metrics[name] = (ref[0], 0)
        for t in tgt["cmap"].tables:
            if t.isUnicode():
                t.cmap[u] = name
        added.append(chr(u))
    tgt.setGlyphOrder(list(glyf.glyphOrder))
    if "post" in tgt and tgt["post"].formatType == 2.0:
        tgt["post"].extraNames = []
        tgt["post"].mapping = {}
    if not dry:
        tgt.save(tgt_path)
    rest = [chr(u) for u in missing if u not in icmap]
    return f"added {''.join(added)} from {FAMILY_SRC[fam]} {loc or ''} (fit {sc:.2f} over {nshared})" + (f"; family lacks {''.join(rest)}" if rest else "")

if __name__ == "__main__":
    for p in sys.argv[1:]:
        print(p, "->", add(p))
