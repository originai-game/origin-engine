"""Subsets Noto Sans SC to the characters Hollowbell shows.

    python3 tools/subset-font.py

Reads every string of assets/strings/*.ogt (and EXTRA: what the playtests
type into the name field), subsets the engine's project-template font
(templates/game/assets/fonts/NotoSansSC-Regular-subset.ttf, OFL) to those
characters (CJK only: the style's pixel face draws the Latin, so a label
mixing scripts takes each run from its own face, og-ui §7), and writes
assets/fonts/NotoSansSC-hollowbell.ttf with its import settings (`chars`
naming every CJK character, so the cook's atlas holds them all). Needs
fontTools (pip install fonttools). Copied from examples/lantern-vale's,
as a game author copies a tool that works.
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
EXTRA = "小铃"


def strings():
    chars = set(EXTRA)
    folder = os.path.join(ASSETS, "strings")
    for name in sorted(os.listdir(folder)):
        if not name.endswith(".ogt"):
            continue
        with open(os.path.join(folder, name), encoding="utf-8") as f:
            for line in f:
                if line.lstrip().startswith("#"):
                    continue
                for s in re.findall(r'"((?:[^"\\]|\\.)*)"', line):
                    chars.update(s)
    return chars


def main():
    chars = strings()
    cjk = sorted(c for c in chars if ord(c) > 0x2E7F)
    text = "".join(cjk) + " 、。，？！：「」…"
    out = os.path.join(ASSETS, "fonts", "NotoSansSC-hollowbell.ttf")
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
        f.write("# CJK punctuation and every CJK character of assets/strings/*.ogt and the playtests' typed name (tools/subset-font.py).\n")
        f.write("font {\n  charset cjk-punct\n")
        f.write('  chars "' + "".join(cjk) + '"\n')
        f.write("  em_px 32\n  px_range 4\n}\n")
    print(f"{out}: {len(cjk)} CJK characters, {os.path.getsize(out)} bytes")


if __name__ == "__main__":
    sys.exit(main())
