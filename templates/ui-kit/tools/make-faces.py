#!/usr/bin/env python3
"""The UI kit's display and rounded faces and its sample-script fonts.

    python3 tools/make-faces.py <inter-latin-opsz-normal.woff2> \\
        <nunito-latin-wght-normal.woff2> <NotoSansHebrew-Regular.woff2> \\
        <NotoSansDevanagari-Regular.woff2>
    python3 tools/make-faces.py --garamond <EBGaramond[wght].ttf>

Sources (SIL OFL 1.1): Inter Variable and Nunito Variable as the platform
self-hosts them (@fontsource-variable/inter and /nunito 5.x: the design
language's `font-display` and `font-rounded`), Noto Sans Hebrew and Noto
Sans Devanagari (notofonts; Godot 4.7's thirdparty copies). Writes into
assets/fonts/:

- Inter-Display.ttf: Inter instanced at its display optical size (opsz
  32) and semibold (wght 600), the display steps' face;
- Nunito-Rounded.ttf: Nunito at bold (wght 700), for levels and scores;
- NotoSansHebrew-subset.ttf and NotoSansDevanagari-subset.ttf: the
  characters of the kit's text screen (SAMPLES) and printable ASCII;
- EBGaramond-Medium.ttf (`--garamond`, google/fonts ofl/ebgaramond, SIL
  OFL 1.1): EB Garamond at wght 500, Latin-1, with those of the OpenType
  features og-ui applies (FEATURES) it has: small caps (its default
  figures are already tabular lining). The kit's serif with small caps
  and the fixture of og-ui's feature tests.

Each keeps its layout features (kerning; Devanagari's conjuncts and mark
positions, which og-ui shapes with rustybuzz), and gets import settings
(.ogt) beside it.
"""
import sys
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

KIT = Path(__file__).resolve().parent.parent
FONTS = KIT / "assets" / "fonts"
ASCII = "".join(chr(c) for c in range(0x20, 0x7F))
LATIN1 = "".join(chr(c) for c in range(0xA0, 0x100))
PUNCT = "–—‘’‚“”„•…′″‹›€™−×"
# The text screen's strings in scripts the kit's other fonts lack (src/main.ts).
SAMPLES = {
    "hebrew": "שלום עולם גרסה חדשה שלב מתוך בדיקה ממשק",
    "devanagari": "नमस्ते दुनिया हिन्दी भाषा स्तर खेल शुरू करें",
}


# The OpenType features og-ui's type steps can turn on, and kerning.
FEATURES = ["kern", "smcp", "c2sc", "tnum", "lnum", "case"]


def write_subset(font: TTFont, text: str, out: Path, features: list[str] | None = None) -> None:
    opts = subset.Options()
    opts.layout_features = features or ["*"]
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    opts.glyph_names = False
    opts.hinting = False
    sub = subset.Subsetter(opts)
    sub.populate(text=text)
    sub.subset(font)
    font.flavor = None
    font.save(out)


def settings(out: Path, charset: str, chars: str, em_px: int, features: str = "") -> None:
    out.with_name(out.name + ".ogt").write_text(
        "type import\n"
        "version 1\n"
        "---\n"
        "# Made by tools/make-faces.py.\n"
        "font {\n"
        f"  charset {charset}\n"
        f'  chars "{chars}"\n'
        f"  em_px {em_px}\n"
        f"  px_range {-(-em_px // 3)}\n"  # a third of the em: a 2 px outline at 16 px
        + (f"  features {features}\n" if features else "")
        + "}\n",
        encoding="utf-8",
    )


def main() -> None:
    if len(sys.argv) == 3 and sys.argv[1] == "--garamond":
        f = instancer.instantiateVariableFont(TTFont(sys.argv[2]), {"wght": 500})
        out = FONTS / "EBGaramond-Medium.ttf"
        write_subset(f, ASCII + LATIN1 + PUNCT, out, FEATURES)
        kept = {r.FeatureTag for r in TTFont(out)["GSUB"].table.FeatureList.FeatureRecord}
        settings(out, "latin", PUNCT, 48, " ".join(t for t in FEATURES if t in kept and t != "kern"))
        return
    if len(sys.argv) != 5:
        sys.exit(__doc__)
    inter, nunito, hebrew, devanagari = (Path(a) for a in sys.argv[1:])
    latin = ASCII + LATIN1 + PUNCT

    f = instancer.instantiateVariableFont(TTFont(inter), {"opsz": 32, "wght": 600})
    write_subset(f, latin, FONTS / "Inter-Display.ttf")
    settings(FONTS / "Inter-Display.ttf", "latin", PUNCT, 48)

    f = instancer.instantiateVariableFont(TTFont(nunito), {"wght": 700})
    write_subset(f, latin, FONTS / "Nunito-Rounded.ttf")
    settings(FONTS / "Nunito-Rounded.ttf", "latin", PUNCT, 48)

    write_subset(TTFont(hebrew), ASCII + SAMPLES["hebrew"], FONTS / "NotoSansHebrew-subset.ttf")
    settings(FONTS / "NotoSansHebrew-subset.ttf", "ascii", SAMPLES["hebrew"].replace(" ", ""), 32)

    write_subset(TTFont(devanagari), ASCII + SAMPLES["devanagari"], FONTS / "NotoSansDevanagari-subset.ttf")
    settings(FONTS / "NotoSansDevanagari-subset.ttf", "ascii", SAMPLES["devanagari"].replace(" ", ""), 32)


if __name__ == "__main__":
    main()
