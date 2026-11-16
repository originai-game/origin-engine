#!/usr/bin/env python3
"""Writes emoji-text-test.ttf beside this script: a text font with text forms
of emoji characters, for the font-variant-emoji tests (CC0 1.0, self-made by
the Origin Engine authors).

Run with fontTools 4.60 or later: python3 make-emoji-text-font.py

1000 units per em, every advance 1000, each glyph an outline square: a, b,
the watch U+231A (Emoji_Presentation: drawn as emoji by default) and the
copyright sign U+00A9 (Emoji but not Emoji_Presentation: drawn as text by
default). No glyph is a colour glyph.

Deterministic: the same fontTools writes the same bytes (head.created and
head.modified are fixed).
"""

import os

from fontTools.fontBuilder import FontBuilder
from fontTools.misc.timeTools import timestampFromString
from fontTools.pens.ttGlyphPen import TTGlyphPen

UPEM = 1000
TOP = 800


def rect(x0, y0, x1, y1):
    pen = TTGlyphPen(None)
    pen.moveTo((x0, y0))
    pen.lineTo((x0, y1))
    pen.lineTo((x1, y1))
    pen.lineTo((x1, y0))
    pen.closePath()
    return pen.glyph()


def main():
    chars = {"a": "a", "b": "b", "watch": "\u231a", "copyright": "\u00a9"}
    order = [".notdef", *chars]
    glyphs = {".notdef": rect(100, 0, 900, TOP)}
    glyphs.update({g: rect(0, 0, UPEM, TOP) for g in chars})
    fb = FontBuilder(UPEM, isTTF=True)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap({ord(c): g for g, c in chars.items()})
    fb.setupGlyf(glyphs)
    lsb = {".notdef": 100}
    fb.setupHorizontalMetrics({g: (UPEM, lsb.get(g, 0)) for g in order})
    fb.setupHorizontalHeader(ascent=TOP, descent=-200)
    fb.setupNameTable({"familyName": "OG Emoji Text Test", "styleName": "Regular"})
    fb.setupOS2(sTypoAscender=TOP, sTypoDescender=-200, usWinAscent=TOP, usWinDescent=200)
    fb.setupPost()
    stamp = timestampFromString("Mon Jan  1 00:00:00 2024")
    fb.font["head"].created = stamp
    fb.font["head"].modified = stamp
    fb.save(os.path.join(os.path.dirname(os.path.abspath(__file__)), "emoji-text-test.ttf"))


if __name__ == "__main__":
    main()
