#!/usr/bin/env python3
"""Writes palette-test.ttf beside this script: a colour font for the COLR and
font-palette tests (CC0 1.0, self-made by the Origin Engine authors).

Run with fontTools 4.60 or later: python3 make-palette-font.py

1000 units per em, every advance 1000. Three colour glyphs, each filling
the em square from 0 to 1000 across and from 0 to 800 up:

    A  COLRv0: a square in palette entry 0, its right half covered by a
       layer in entry 1
    B  COLRv1: a linear gradient from entry 0 at its left edge to entry 1
       at its right edge
    C  COLRv0: one layer in the text's colour (entry 0xFFFF)
    D  no colour glyph: an outline square

Three CPAL palettes, entries 0 and 1:

    0  red   #FF0000, green  #00FF00   (no type)
    1  blue  #0000FF, yellow #FFFF00   (usable on a light background)
    2  white #FFFFFF, black  #000000   (usable on a dark background)

Deterministic: the same fontTools writes the same bytes (head.created and
head.modified are fixed).
"""

import os

from fontTools.colorLib.builder import buildCOLR, buildCPAL
from fontTools.fontBuilder import FontBuilder
from fontTools.misc.timeTools import timestampFromString
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib.tables import otTables as ot

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
    order = [".notdef", "A", "B", "C", "D", "full", "right"]
    glyphs = {
        ".notdef": rect(100, 0, 900, TOP),
        "A": rect(0, 0, UPEM, TOP),
        "B": rect(0, 0, UPEM, TOP),
        "C": rect(0, 0, UPEM, TOP),
        "D": rect(0, 0, UPEM, TOP),
        "full": rect(0, 0, UPEM, TOP),
        "right": rect(UPEM // 2, 0, UPEM, TOP),
    }
    fb = FontBuilder(UPEM, isTTF=True)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap({ord(c): c for c in "ABCD"})
    fb.setupGlyf(glyphs)
    # Each left side bearing is its outline's xMin, as TrueType places it.
    lsb = {".notdef": 100, "right": UPEM // 2}
    fb.setupHorizontalMetrics({g: (UPEM, lsb.get(g, 0)) for g in order})
    fb.setupHorizontalHeader(ascent=TOP, descent=-200)
    fb.setupNameTable({"familyName": "OG Palette Test", "styleName": "Regular"})
    fb.setupOS2(sTypoAscender=TOP, sTypoDescender=-200, usWinAscent=TOP, usWinDescent=200)
    fb.setupPost()
    gradient = {
        "Format": ot.PaintFormat.PaintGlyph,
        "Glyph": "full",
        "Paint": {
            "Format": ot.PaintFormat.PaintLinearGradient,
            "ColorLine": {
                "Extend": "pad",
                "ColorStop": [
                    {"StopOffset": 0.0, "PaletteIndex": 0, "Alpha": 1.0},
                    {"StopOffset": 1.0, "PaletteIndex": 1, "Alpha": 1.0},
                ],
            },
            "x0": 0,
            "y0": 0,
            "x1": UPEM,
            "y1": 0,
            "x2": 0,
            "y2": UPEM,
        },
    }
    fb.font["COLR"] = buildCOLR(
        {
            "A": [("full", 0), ("right", 1)],
            "B": gradient,
            "C": [("full", 0xFFFF)],
        },
        glyphMap=fb.font.getReverseGlyphMap(),
    )
    red, green = (1.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.0, 1.0)
    blue, yellow = (0.0, 0.0, 1.0, 1.0), (1.0, 1.0, 0.0, 1.0)
    white, black = (1.0, 1.0, 1.0, 1.0), (0.0, 0.0, 0.0, 1.0)
    fb.font["CPAL"] = buildCPAL(
        [[red, green], [blue, yellow], [white, black]],
        paletteTypes=[0, 1, 2],
    )
    stamp = timestampFromString("Mon Jan  1 00:00:00 2024")
    fb.font["head"].created = stamp
    fb.font["head"].modified = stamp
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "palette-test.ttf")
    fb.save(out)


if __name__ == "__main__":
    main()
