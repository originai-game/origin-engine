#!/bin/bash
# The generated models (raw/models/, never committed: each asset's last
# gateway output repacked with 2048 JPEG maps, README §Assets) -> the
# game-ready files in assets/models/ that assets.json pins: maps capped and
# stored as JPEG (tools/pack-model.py),
# meshes simplified by screen size, quantized and meshopt-compressed
# (gltfpack; the cook reads EXT_meshopt_compression and KHR_mesh_quantization).
#
#     tools/shrink-models.sh            # every model
#     tools/shrink-models.sh ped_a      # one
#
# name  map size  triangle ratio  glow
# The courier and the bike are the frame's heroes: 2048 maps, every triangle.
# The rest are sized for 1080p at the cover camera.
set -euo pipefail
cd "$(dirname "$0")/.."
TABLE="
courier  2048 1    0
bike     2048 1    0
ped_a    1024 0.26 0
ped_b    1024 0.26 0
ped_c    1024 0.26 0
stall    1024 0.3  0
block-a  1024 0.35 0
block-b  1024 0.35 0
gate     1024 0.5  0
lantern  1024 0.3  0
"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
mkdir -p assets/models
echo "$TABLE" | while read -r name size ratio glow; do
  [ -z "$name" ] && continue
  [ $# -gt 0 ] && [ "$1" != "$name" ] && continue
  python3 tools/pack-model.py "raw/models/$name.glb" "$tmp/$name.glb" "$size" "$glow"
  simplify=(); [ "$ratio" != 1 ] && simplify=(-si "$ratio" -se 0.02)
  npx --yes gltfpack@1.3 -i "$tmp/$name.glb" -o "assets/models/$name.glb" -cc -kn -vp 16 -af 30 "${simplify[@]}" >/dev/null
  printf '%-8s %9d bytes\n' "$name" "$(stat -c %s "assets/models/$name.glb")"
done
