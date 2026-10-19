#!/usr/bin/env python3
"""Re-record reference frames with their evidence.

    python3 tests/evidence/P2/ui/rerecord.py <tag> <new.png> <reference.png> [...pairs]

For each pair: writes tests/evidence/P2/ui/rerecord/<tag>-<reference name>
as the old reference (left) beside the new frame (right), each scaled to at
most 960 px wide, then copies the new frame over the reference. Prints the
outlier share and RMSE between them under og-tests' rule (32/255 per
channel), the numbers the commit message quotes."""
import os, shutil, sys
from PIL import Image
import numpy as np

here = os.path.dirname(os.path.abspath(__file__))
tag, pairs = sys.argv[1], sys.argv[2:]
for new, ref in zip(pairs[::2], pairs[1::2]):
    a = Image.open(ref).convert("RGB")
    b = Image.open(new).convert("RGB")
    if a.size == b.size:
        d = np.abs(np.asarray(a, np.int16) - np.asarray(b, np.int16))
        out = (d.max(axis=2) > 32).mean() * 100
        rmse = float(np.sqrt(((d / 255.0) ** 2).mean()))
        print(f"{os.path.basename(ref)}: {out:.3f} % outliers, RMSE {rmse:.5f}")
    else:
        print(f"{os.path.basename(ref)}: size {a.size} -> {b.size}")
    k = min(1.0, 960 / max(a.width, b.width))
    a2 = a.resize((max(1, int(a.width * k)), max(1, int(a.height * k))), Image.LANCZOS)
    b2 = b.resize((max(1, int(b.width * k)), max(1, int(b.height * k))), Image.LANCZOS)
    pair = Image.new("RGB", (a2.width + b2.width + 8, max(a2.height, b2.height)), (255, 0, 255))
    pair.paste(a2, (0, 0))
    pair.paste(b2, (a2.width + 8, 0))
    pair.save(os.path.join(here, "rerecord", f"{tag}-{os.path.basename(ref)}"), optimize=True)
    shutil.copyfile(new, ref)
