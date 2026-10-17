# M0 evidence

One line per file: what it is, how it was produced, at which revision.

- `spike-frame-reproduced.png` — the spike's reference scene drawn by og-render before the sky pass and mips (scene `gltf-spike-parity`: flat background (0.5, 0.65, 0.9), `Mips::None`, the spike's camera at t = 59/60 s), `target/agent-render/release/og-gfx gltf-spike-parity --frames 60 --out frame.png` on Apple M4 Pro, Metal; byte-identical to `.spike-out/native.png` (cmp; magick compare RMSE 0, AE 0), 77 draws as in the spike. The scene was retired when the IBL cube-face fix changed ambient lighting; `gltf-reference` is its successor (rev 8e5252d+dirty, 2026-10-17, sha256 a4fa2db2fdaf40d9)
