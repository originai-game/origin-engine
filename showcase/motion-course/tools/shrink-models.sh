#!/bin/bash
# The generated characters (raw/models/, never committed: each one's Meshy
# animate output, README §Assets) -> the game-ready files in assets/models/
# that assets.json pins: maps capped at 1024 and stored as JPEG
# (tools/pack-model.py), meshes simplified to half, quantized and
# meshopt-compressed (gltfpack; the cook reads EXT_meshopt_compression and
# KHR_mesh_quantization). Each stands about a fifth of the frame high.
#
#     tools/shrink-models.sh            # both
#     tools/shrink-models.sh runner     # one
set -euo pipefail
cd "$(dirname "$0")/.."
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
mkdir -p assets/models
for name in runner porter; do
  [ $# -gt 0 ] && [ "$1" != "$name" ] && continue
  python3 tools/pack-model.py "raw/models/$name.glb" "$tmp/$name.glb" 1024
  npx --yes gltfpack@1.3 -i "$tmp/$name.glb" -o "assets/models/$name.glb" -cc -kn -vp 16 -af 30 -si 0.5 -se 0.02 >/dev/null
  printf '%-8s %9d bytes\n' "$name" "$(stat -c %s "assets/models/$name.glb")"
done
