#!/usr/bin/env python3
"""neon-vector's own sources for the shape family (README.md): the art brief
asks for near-black faces and lines in 2 to 4 glowing hues, so each base
model's materials are rewritten to black with an emissive hue, and the crate
and ground glbs take a near-black texture with a glowing grid.

    neon.py <shape-3d example dir>
"""
import os, re, subprocess, sys
sys.path.insert(0, os.path.dirname(__file__))
from make3d import crate

HUES = {"coat": "0.1 0.9 1", "skin": "1 0.3 0.8", "boot": "0.1 0.9 1", "iron": "0.1 0.9 1", "flame": "1 0.3 0.8",
        "bark": "1 0.3 0.8", "leaves": "0.2 1 0.5", "glass": "1 0.3 0.8", "hill": "0.4 0.3 1", "far-hill": "0.3 0.2 0.8",
        "wall": "0.1 0.9 1", "roof": "1 0.3 0.8"}
ex = sys.argv[1]
out = os.path.join(ex, "styles/neon-vector/models")
os.makedirs(out, exist_ok=True)
for name in ["hero", "tree", "lantern", "hills"]:
    text = open(os.path.join(ex, "assets/models", f"{name}.ogt")).read()
    def mat(m):
        hue = HUES.get(m.group(1), "0.1 0.9 1")
        return f'material "{m.group(1)}" {{\n  color 0.02 0.02 0.04\n  roughness 1\n  emissive {hue}\n  emissiveIntensity 0.6\n}}'
    text = re.sub(r'material "([\w-]+)" \{[^}]*\}', mat, text)
    open(os.path.join(out, f"{name}.ogt"), "w").write(text.replace("---\n", "---\n# neon-vector's version (tools/art-samples/neon.py): black faces, glowing hues.\n", 1))
grid = os.path.join(out, "grid.png")
subprocess.run(["magick", "-size", "64x64", "xc:#05050a", "-fill", "none", "-stroke", "#20e0ff", "-strokewidth", "2",
                "-draw", "rectangle 1,1 62,62", grid], check=True)
crate(os.path.join(out, "crate.glb"), open(grid, "rb").read())
field = open(os.path.join(ex, "assets/models/field.ogt")).read().replace("color 0.27 0.42 0.2", "color 0.02 0.02 0.04")
open(os.path.join(out, "field.ogt"), "w").write(field)
os.remove(grid)
