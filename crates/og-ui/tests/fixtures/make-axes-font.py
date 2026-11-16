#!/usr/bin/env python3
"""Writes axes-test.ttf beside this script: a variable font for the font
variation tests (CC0 1.0, self-made by the Origin Engine authors).

Run with fontTools 4.60 or later: python3 make-axes-font.py

Four axes, each moving the advance of one glyph only, so a test reads the
location a text was shaped at from its advances (font units, 1000 per em):

    A  wght 100..400..900   advance 500 / 600 / 1000
    B  wdth  50..100..200   advance 300 / 600 / 900
    C  opsz   8..14..72     advance 700 / 600 / 400
    D  GRAD -100..0..100    advance 500 / 600 / 800
    E  (no axis)            advance 500

Each glyph is a rectangle 50 units in from either side of its advance; the
advances vary through gvar's phantom points and HVAR alike. Deterministic:
the same fontTools writes the same bytes (head.created and head.modified
are fixed).
"""

import os
import tempfile

from fontTools.designspaceLib import AxisDescriptor, DesignSpaceDocument, SourceDescriptor
from fontTools.fontBuilder import FontBuilder
from fontTools.misc.timeTools import timestampFromString
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.varLib import build as build_variable

UPM = 1000
# head.created and head.modified: fixed, so the bytes do not depend on when.
STAMP = timestampFromString("Thu Jan  1 00:00:00 2026")
HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "axes-test.ttf")

AXES = [
    # tag, name, minimum, default, maximum
    ("wght", "Weight", 100, 400, 900),
    ("wdth", "Width", 50, 100, 200),
    ("opsz", "Optical size", 8, 14, 72),
    ("GRAD", "Grade", -100, 0, 100),
]
DEFAULT = {"A": 600, "B": 600, "C": 600, "D": 600, "E": 500}
# The masters away from the default: (axis, value) and the glyph it moves.
MASTERS = [
    ("wght", 100, "A", 500),
    ("wght", 900, "A", 1000),
    ("wdth", 50, "B", 300),
    ("wdth", 200, "B", 900),
    ("opsz", 8, "C", 700),
    ("opsz", 72, "C", 400),
    ("GRAD", -100, "D", 500),
    ("GRAD", 100, "D", 800),
]
ORDER = [".notdef", "space", "A", "B", "C", "D", "E"]


def rectangle(advance):
    pen = TTGlyphPen(None)
    pen.moveTo((50, 0))
    pen.lineTo((50, 700))
    pen.lineTo((advance - 50, 700))
    pen.lineTo((advance - 50, 0))
    pen.closePath()
    return pen.glyph()


def master(advances, path):
    fb = FontBuilder(UPM, isTTF=True)
    fb.setupGlyphOrder(ORDER)
    fb.setupCharacterMap({0x20: "space", **{ord(c): c for c in "ABCDE"}})
    glyphs = {".notdef": rectangle(500), "space": TTGlyphPen(None).glyph()}
    glyphs.update({g: rectangle(a) for g, a in advances.items()})
    fb.setupGlyf(glyphs)
    metrics = {".notdef": (500, 50), "space": (250, 0)}
    metrics.update({g: (a, 50) for g, a in advances.items()})
    fb.setupHorizontalMetrics(metrics)
    fb.setupHorizontalHeader(ascent=800, descent=-200)
    fb.setupNameTable({"familyName": "OG Axes Test", "styleName": "Regular"})
    fb.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200,
                sxHeight=500, sCapHeight=700)
    fb.setupPost()
    fb.updateHead(created=STAMP, modified=STAMP)
    fb.save(path)


def main():
    with tempfile.TemporaryDirectory() as tmp:
        doc = DesignSpaceDocument()
        for tag, name, lo, default, hi in AXES:
            a = AxisDescriptor()
            a.tag, a.name, a.minimum, a.default, a.maximum = tag, name, lo, default, hi
            doc.addAxis(a)
        defaults = {name: default for _, name, _, default, _ in AXES}
        names = {tag: name for tag, name, *_ in AXES}

        def source(path, location):
            s = SourceDescriptor()
            s.path = path
            s.location = {**defaults, **location}
            doc.addSource(s)

        base = os.path.join(tmp, "default.ttf")
        master(DEFAULT, base)
        source(base, {})
        for i, (tag, value, glyph, advance) in enumerate(MASTERS):
            path = os.path.join(tmp, f"m{i}.ttf")
            master({**DEFAULT, glyph: advance}, path)
            source(path, {names[tag]: value})
        font, _, _ = build_variable(doc)
        font["head"].created = font["head"].modified = STAMP
        font.save(OUT)


if __name__ == "__main__":
    main()
