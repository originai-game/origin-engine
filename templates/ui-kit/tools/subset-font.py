#!/usr/bin/env python3
"""Subsets a font to the characters a game's string tables use, and writes
its import settings: the one tool a localised game needs for a CJK, Korean,
Arabic or other large face (og-i18n §3).

    python3 tools/subset-font.py <source font> <out font> [options]

    <source font>   a .ttf, .otf, .ttc or .otc (Noto Sans JP, KR, SC, TC,
                    Arabic, ... from notofonts, SIL OFL 1.1)
    <out font>      where to write the subset, e.g. assets/fonts/NotoSansJP-game.otf;
                    its import settings go beside it as <out font>.ogt
    --strings DIR   the string tables to read (default assets/strings); every
                    `string "key" "text"` line of every .ogt there
    --locale TAG    read only DIR/<TAG>.ogt (repeatable; default every table)
    --extra TEXT    more characters: what a playtest types, a name list
    --face N        the face of a .ttc/.otc collection (Noto Sans CJK's
                    Regular TTC: 0 Japanese, 1 Korean, 2 Simplified Chinese,
                    3 Traditional Chinese, 4 Hong Kong)
    --cjk-only      keep only CJK characters and punctuation, so a label that
                    mixes scripts draws its Latin in the skin's face, run by
                    run (og-ui §7); without it printable ASCII is kept and the
                    whole label stays in this face
    --charset SETS  the import settings' charset (default "cjk-punct" with
                    --cjk-only, else "ascii cjk-punct")

It prints every character the source font lacks and exits 1 when there are
any: a Japanese table subset from a Chinese face (Noto Sans SC) loses the
Japanese forms of kanji such as 鐘 鈴 険 続 設 語 楽 効, and the frame would
show boxes. Run it again after a string changes, then `og cook`. Needs
fontTools (pip install fonttools). Promoted from examples/lantern-vale's and
examples/hollowbell's tools/subset-font.py, which subset Noto Sans SC.
"""
import argparse
import os
import re
import sys

from fontTools import subset
from fontTools.ttLib import TTFont

ASCII = "".join(chr(c) for c in range(0x20, 0x7F))
CJK_PUNCT = " 、。，．？！：；「」『』（）〈〉《》【】…・ー～"
STRING = re.compile(r'^\s*string\s+"(?:[^"\\]|\\.)*"\s+"((?:[^"\\]|\\.)*)"', re.M)
ESCAPE = re.compile(r'\\(u\{[0-9A-Fa-f]+\}|.)')


def is_cjk(c):
    o = ord(c)
    return o > 0x2E7F or c in CJK_PUNCT


def unescape(s):
    """An .ogt string's text, its escapes resolved (backslash, quote, n, r, t and u{hex})."""
    simple = {"n": "\n", "r": "\r", "t": "\t"}
    return ESCAPE.sub(lambda m: chr(int(m.group(1)[2:-1], 16)) if m.group(1).startswith("u{") else simple.get(m.group(1), m.group(1)), s)


def table_chars(folder, locales):
    chars = set()
    names = [f"{t}.ogt" for t in locales] if locales else sorted(n for n in os.listdir(folder) if n.endswith(".ogt"))
    for name in names:
        with open(os.path.join(folder, name), encoding="utf-8") as f:
            for m in STRING.finditer(f.read()):
                chars.update(unescape(m.group(1)))
    return chars


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("source")
    ap.add_argument("out")
    ap.add_argument("--strings", default=os.path.join("assets", "strings"))
    ap.add_argument("--locale", action="append", default=[])
    ap.add_argument("--extra", default="")
    ap.add_argument("--face", type=int, default=0)
    ap.add_argument("--cjk-only", action="store_true")
    ap.add_argument("--charset")
    a = ap.parse_args()

    chars = table_chars(a.strings, a.locale) | set(a.extra)
    chars -= set("\n\r\t")
    if a.cjk_only:
        keep = sorted(c for c in chars if is_cjk(c)) + sorted(set(CJK_PUNCT) - chars)
    else:
        keep = sorted(chars | set(ASCII))
    text = "".join(keep)

    opts = subset.Options()
    opts.layout_features = ["*"]  # joining forms, ligatures and vertical forms the shaper uses
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    opts.font_number = a.face
    font = subset.load_font(a.source, opts)
    cmap = font.getBestCmap()
    missing = [c for c in sorted(chars) if ord(c) not in cmap and not c.isspace()]
    sub = subset.Subsetter(opts)
    sub.populate(text=text)
    sub.subset(font)
    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
    subset.save_font(font, a.out, opts)

    charset = a.charset or ("cjk-punct" if a.cjk_only else "ascii cjk-punct")
    listed = "".join(c for c in keep if ord(c) > 0x7E and c not in '"\\' and ord(c) in cmap)
    with open(a.out + ".ogt", "w", encoding="utf-8") as f:
        f.write("type import\nversion 1\n---\n")
        f.write(f"# Every character of the string tables (tools/subset-font.py {os.path.basename(a.source)}).\n")
        f.write(f'font {{\n  charset {charset}\n  chars "{listed}"\n  em_px 32\n  px_range 4\n}}\n')
    print(f"{a.out}: {len(keep)} characters, {os.path.getsize(a.out)} bytes; {a.out}.ogt")
    if missing:
        print(f"the source font lacks {len(missing)}: {''.join(missing)}", file=sys.stderr)
        print("use a face for this script (Noto Sans JP for Japanese, KR for Korean, TC for Traditional Chinese)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
