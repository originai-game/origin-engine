"""Prints, for each art image, its face's light and dark extremes: the 90th and 10th luminance percentile of the opaque pixels inside its slice insets."""
import json, sys
import numpy as np
from PIL import Image
images = json.load(open(sys.argv[2]))
out = {}
for name, im in images.items():
    a = np.asarray(Image.open(f"{sys.argv[1]}/{im['src']}").convert("RGBA")).astype(float) / 255
    t, r, b, l = [int(v) for v in im["slice"]]
    c = a[t:a.shape[0] - b or None, l:a.shape[1] - r or None].reshape(-1, 4)
    c = c[c[:, 3] > 0.85][:, :3]
    if len(c) == 0:
        continue
    lin = np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
    y = lin @ [0.2126, 0.7152, 0.0722]
    pick = lambda q: "#" + "".join(f"{round(v * 255):02x}" for v in c[np.argsort(y)[int(q * (len(y) - 1))]])
    out[name] = [pick(0.9), pick(0.1)]
print(json.dumps(out))
