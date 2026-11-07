#!/usr/bin/env python3
"""Migrate engine-owned tween/timeline source names; never generated output."""
from pathlib import Path
import re

root = Path(__file__).resolve().parent.parent
names = {"linear": "none"}
for family, gsap in [("quad", "power1"), ("cubic", "power2"), ("quart", "power3"),
                     ("expo", "expo"), ("back", "back"), ("elastic", "elastic"),
                     ("bounce", "bounce")]:
    for suffix in ["In", "Out", "InOut"]:
        names[family + suffix] = gsap + "." + suffix[0].lower() + suffix[1:]

paths = [
    "crates/og-api/api/tween.ogt", "crates/og-api/api/camera.ogt",
    "crates/og-tween/src/ease.rs", "crates/og-tween/src/lib.rs",
    "crates/og-tween/README.md", "crates/og-timeline/src/model.rs",
    "crates/og-timeline/src/model/text.rs", "crates/og-timeline/src/eval.rs",
    "crates/og-cook/tests/timeline.rs", "crates/og-runtime/tests/timeline.rs",
    "crates/og-runtime/tests/timeline_tracks.rs",
    "crates/og-runtime/tests/time_groups.rs",
    "tests/fixtures/cook-assets/assets/timelines/intro.ogt",
    "packages/engine/tests/support/feel-hash.ts",
    "packages/engine/tests/feel.test.ts",
]
for name in paths:
    path = root / name
    old = text = path.read_text()
    for before, after in names.items():
        if before != "linear":
            text = re.sub(r"\b" + before + r"\b", after, text)
        else:
            # linear also names path interpolation and resamplers, not only Ease.
            text = text.replace('ease: "linear"', 'ease: "none"')
            text = text.replace('"ease": "linear"', '"ease": "none"')
            text = re.sub(r"(ease\s+)linear\b", r"\g<1>none", text)
            if name in ["crates/og-api/api/tween.ogt", "crates/og-tween/src/ease.rs"]:
                text = text.replace('"linear"', '"none"').replace('variant linear ', 'variant none ')
    if name in ["crates/og-api/api/tween.ogt", "packages/engine/tests/support/feel-hash.ts"]:
        text = re.sub(r"\bloops\b", "repeat", text).replace("Loops,", "Repeat,")
    if text != old:
        path.write_text(text)
        print(name)
