#!/usr/bin/env python3
"""Migrate engine-owned behaviour examples to the public BT.CPP vocabulary.

Scoped to behaviour sources, never generated wiki or unrelated UI selectors.
Idempotent; retained as the recipe for embedded and standalone assets.
"""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
NAMES = {
    "selector": "ReactiveFallback",
    "sequence": "ReactiveSequence",
    "steps": "Sequence",
    "invert": "Inverter",
    "succeed": "ForceSuccess",
    "enter": "entry",
}
paths = list((ROOT / "crates/og-ai").rglob("*.rs")) + [
    ROOT / "crates/og-ai/README.md",
    ROOT / "crates/og-cook/tests/behaviour.rs",
    ROOT / "crates/og-api/api/ai.ogt",
]
paths += [p for p in ROOT.glob("tests/fixtures/**/*.ogt")
          if p.read_text().startswith("type behaviour\n")]
for path in paths:
    text = path.read_text()
    old = text
    for before, after in NAMES.items():
        # Asset node at a real/escaped line start, including Rust doc examples.
        text = re.sub(r"(^|\\n)([ \t]*(?://! )?[ \t]*)" + before + r"\b(?=\s|\{)",
                      lambda m: m[1] + m[2] + after, text, flags=re.M)
        # Quoted parser keywords and backtick-documented names / display paths.
        text = text.replace('"' + before + '"', '"' + after + '"')
        text = re.sub(r"(`| > |\")" + before + r"\b", r"\g<1>" + after, text)
    text = text.replace("selector, sequence, steps, parallel, utility, invert, succeed,",
                        "ReactiveFallback, ReactiveSequence, Sequence, parallel, utility, Inverter, ForceSuccess,")
    text = text.replace("its enter actions", "its entry actions")
    if text != old:
        path.write_text(text)
        print(path.relative_to(ROOT))
