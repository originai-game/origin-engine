# M4 evidence

One line per file: what it is, how it was produced, at which revision.

- `cook-report-template.md` — CARGO_TARGET_DIR=target/agent-assets cargo test --release -p og-cook --test template on M4 Pro (macOS, Metal): cooking_twice_gives_identical_bytes writes the report of the shared cook of templates/game/assets (all cores, basisu 1.16) (rev 4648b60+dirty, 2026-10-17, sha256 30c4c5dd0ad97639)
- `cook-determinism.json` — CARGO_TARGET_DIR=target/agent-assets cargo test --release -p og-cook --test template on M4 Pro (macOS, Metal): cooking_twice_gives_identical_bytes, FNV-1a over every cooked/ file of two fresh cooks (all cores vs one thread) (rev 4648b60+dirty, 2026-10-17, sha256 06ed0257427d8d0f)
- `cooked-gltf-reference.png` — CARGO_TARGET_DIR=target/agent-assets cargo test --release -p og-cook --test template on M4 Pro (macOS, Metal): cooked_models_render_the_gltf_reference_scene, og-tests gltf-reference with the models from cooked/ (RMSE 0.00429 vs references/metal/gltf-reference.png) (rev 4648b60+dirty, 2026-10-17, sha256 0368545aa6203236)
- `cooked-gltf-reference-compare.json` — CARGO_TARGET_DIR=target/agent-assets cargo test --release -p og-cook --test template on M4 Pro (macOS, Metal): cooked_models_render_the_gltf_reference_scene, RMSE and frame stats (rev 4648b60+dirty, 2026-10-17, sha256 ded71785b50b1d1b)
- `texture-psnr.md` — CARGO_TARGET_DIR=target/agent-assets cargo test --release -p og-cook --test template on M4 Pro (macOS, Metal): textures_decode_close_to_their_sources, transcoded level 0 vs source pixels (rev 4648b60+dirty, 2026-10-17, sha256 243bf0599f1b4182)
