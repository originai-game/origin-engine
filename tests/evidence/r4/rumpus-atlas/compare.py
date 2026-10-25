"""Compare current Rumpus references to the contact-corrected, stale-atlas set."""
import io
import json
import subprocess
from pathlib import Path

import numpy as np
from PIL import Image, ImageChops, ImageEnhance, ImageDraw

base = "52d7553d9050820d6e5ce65ffbdddf5c3f8c0cd3"
out = Path("tests/evidence/r4/rumpus-atlas")
rows = []
for path in sorted(Path("examples/rumpus-isle/references/metal").glob("*.png")):
    before = Image.open(io.BytesIO(subprocess.check_output(["git", "show", f"{base}:{path}"]))).convert("RGB")
    after = Image.open(path).convert("RGB")
    delta = np.abs(np.array(before).astype(float) - np.array(after).astype(float))
    changed = np.any(delta > 0, axis=2)
    rows.append({"frame": path.name, "pixels": int(changed.sum()), "outlierPercent": float(np.any(delta > 32, axis=2).mean() * 100), "rmse": float(np.sqrt(np.mean((delta / 255) ** 2)))})
    diff = ImageEnhance.Brightness(ImageChops.difference(before, after)).enhance(8)
    diff.save(out / f"{path.stem}-diff.png")
    panel = Image.new("RGB", (1920, 390), "#222222")
    draw = ImageDraw.Draw(panel)
    for index, (label, image) in enumerate([("Before", before), ("After", after), ("Difference x8", diff)]):
        draw.text((index * 640 + 8, 8), f"{path.name}: {label}", fill="white")
        panel.paste(image.resize((640, 360)), (index * 640, 30))
    panel.save(out / f"{path.stem}-comparison.png")
(out / "frame-differences.json").write_text(json.dumps(rows, indent=2) + "\n")
print(json.dumps(rows, indent=2))
