"""Subsets Noto Sans SC to the characters Lantern Vale shows.

    python3 tools/subset-font.py

Reads every string of assets/strings/*.ogt (and EXTRA: what the playtests
type into the name field), subsets the engine's project-template font
(templates/game/assets/fonts/NotoSansSC-Regular-subset.ttf, OFL) to those
characters plus Latin-1, and writes assets/fonts/NotoSansSC-subset.ttf with
its import settings (`chars` naming every CJK character, so the cook's
atlas holds them all). Needs fontTools (pip install fonttools): bun has no
font subsetter, which is why this one tool is Python (FINDINGS.md).
"""
import os
import re
import sys

from fontTools import subset

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
ASSETS = os.path.join(ROOT, "assets")
SOURCE = os.path.normpath(os.path.join(ROOT, "..", "..", "templates", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf"))
# Typed by tests/settings.og.ts into the name field (an input method's commit).
EXTRA = "小灯美"


def strings():
    chars = set(EXTRA)
    folder = os.path.join(ASSETS, "strings")
    for name in sorted(os.listdir(folder)):
        if not name.endswith(".ogt"):
            continue
        with open(os.path.join(folder, name), encoding="utf-8") as f:
            for line in f:
                for s in re.findall(r'"((?:[^"\\]|\\.)*)"', line):
                    chars.update(s)
    return chars


def main():
    chars = strings()
    cjk = sorted(c for c in chars if ord(c) > 0x2E7F)
    text = "".join(sorted(chars)) + "".join(chr(c) for c in range(0x20, 0x100))
    out = os.path.join(ASSETS, "fonts", "NotoSansSC-subset.ttf")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    opts = subset.Options()
    opts.layout_features = ["*"]
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    font = subset.load_font(SOURCE, opts)
    s = subset.Subsetter(opts)
    s.populate(text=text)
    s.subset(font)
    subset.save_font(font, out, opts)
    with open(out + ".ogt", "w", encoding="utf-8") as f:
        f.write("type import\nversion 1\n---\n")
        f.write("# Latin-1, CJK punctuation and every CJK character of assets/strings/*.ogt and the playtests' typed name (tools/subset-font.py).\n")
        f.write("font {\n  charset latin cjk-punct\n")
        f.write('  chars "' + "".join(cjk) + '"\n')
        f.write("  em_px 32\n  px_range 4\n}\n")
    print(f"{out}: {len(cjk)} CJK characters, {os.path.getsize(out)} bytes")


if __name__ == "__main__":
    sys.exit(main())
