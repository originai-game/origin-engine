#!/usr/bin/env bash
# Use the unchanged comparison runner, cooling between modes to keep the load cap.
set -eu
index=0
failed=0
while IFS='|' read -r scene mode; do
  index=$((index + 1))
  out="tests/evidence/r4/measure/threejs-separated/$index"
  echo "$scene / $mode"
  if ! bash tests/evidence/r4/measure/idle-check.sh "$out/run" \
    env OG_ENGINE_PACKAGE="$PWD/packages/engine" bun tools/bench/threejs-compare.ts \
    --frames 600 --warm 60 --rounds 2 --only "$scene" --mode "$mode" --out "$out/results"; then
    failed=1
  fi
done <<'MODES'
reference|engine WebGPU
reference|engine WebGL2
reference|three WebGL2, InstancedMesh
reference|three WebGL2, one Mesh each
spheres|engine WebGPU
spheres|engine WebGL2
spheres|three WebGL2, InstancedMesh
spheres|three WebGL2, one Mesh each
spheres|three WebGPU, InstancedMesh
MODES
exit "$failed"
