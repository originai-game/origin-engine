# M0 evidence

One line per file: what it is, how it was produced, at which revision.

- `spike-frame-reproduced.png` — og-render before the sky pass and mips: CARGO_TARGET_DIR=target/agent-render cargo build --release -p og-tests && target/agent-render/release/og-gfx gltf-spike-parity --frames 60 --out frame.png on Apple M4 Pro, Metal; byte-identical to .spike-out/native.png (cmp; magick compare RMSE 0, AE 0), 77 draws as in the spike (rev 8e5252d+dirty, 2026-10-17, sha256 a4fa2db2fdaf40d9)
