# M2 evidence

One line per file: what it is, how it was produced, at which revision.

- `hello-world-hash-web.json` — cd packages/engine && sh scripts/build.sh && bun tests/web/run.ts [--webgl | --no-isolation] (headless Chrome for Testing 150.0.7871.115, M4 Pro, ANGLE Metal); three runs merged (rev 924ca0d+dirty, 2026-10-17, sha256 108dcdba13ec7b88)
- `hello-world-hash-node.json` — og run --headless packages/engine/examples/hello.ts --ticks 600 --setup spawn_wave:10000 --json (bun 1.4.2); node packages/engine/src/node/run.ts (same args, node 26.3.0); bun tests/support/hello-hash.ts --host node|web; macOS arm64 addon (rev 924ca0d+dirty, 2026-10-17, sha256 2f7bd15cfe8d708c)
- `view-benchmark.json` — cd packages/engine && bun bench/views-best.ts --runs 7; Chrome numbers from bun tests/web/run.ts; bun tools/bench/views.ts 10000; M4 Pro (rev 924ca0d+dirty, 2026-10-17, sha256 0bbb2530d4ad2ac0)
- `wasm-size.json` — CARGO_TARGET_DIR=target/agent-tools scripts/check.sh wasm-build wasm-opt size-gate (wasm-bindgen 0.2.127 --target web, wasm-opt -Oz, brotli -q 11) (rev 924ca0d+dirty, 2026-10-17, sha256 afbd936384c591f0)
- `hello-world-two-hosts.md` — summary of the files above, written from their numbers (rev 924ca0d+dirty, 2026-10-17, sha256 8367623e7de33be4)
