from pathlib import Path
from PIL import Image
import hashlib
import json

root = Path(__file__).parent
rows = []
for game in ["game", "meridian"]:
    hashes = []
    for mode in ["default", "fast-up", "fast-none", "store-none"]:
        path = root / mode / game / "capture.png"
        image = Image.open(path).convert("RGBA")
        assert image.size == (1280, 720)
        digest = hashlib.sha256(image.tobytes()).hexdigest()
        hashes.append(digest)
        rows.append(dict(game=game, mode=mode, rgba_sha256=digest, png_bytes=path.stat().st_size))
    assert len(set(hashes)) == 1, f"encoding settings changed pixels: {game}"
(root / "pixels.json").write_text(json.dumps(rows, indent=2) + "\n")
print("Both scenes: four encodings have identical decoded final-frame pixels")
