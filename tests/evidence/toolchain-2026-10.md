# Toolchain 2026-10

The toolchain move of 2026-10-19 (c1290aa..HEAD on main), measured on the
Apple M4 Pro (24 GB) while another agent ran platform validation on the
same machine, so wall times are indicative. Versions now in force:
docs/ci.md §Toolchain.

## Versions

| Tool | Before | After |
|---|---|---|
| Rust | 1.97.0, edition 2021, rust-version 1.97 | 1.99.1, edition 2024, rust-version 1.99 |
| wgpu / naga | 30.0.1 | 30.0.1 (31 is not published) |
| Cargo.lock within semver | | nothing to update |
| wasm-bindgen / js-sys, web-sys / wasm-bindgen-futures | 0.2.127 / 0.3.104 / 0.4.77 | 0.2.129 / 0.3.106 / 0.4.79 |
| binaryen | 123 (Homebrew on macOS, unpinned there) | 133 (release archive everywhere, sha256 checked) |
| wasm-opt flags | `-Oz --enable-bulk-memory --enable-nontrapping-float-to-int` | the same plus `--low-memory-unused` |
| TypeScript | 5.9.3 | 7.0.2 (native `tsc`) |
| bun | 1.4.2 | 1.4.2 (1.4.3 exists; not moved, see below) |
| cargo-nextest | 0.9.143 | 0.9.148 |
| cargo-zigbuild / zig | 0.22.1 / 0.15.2 | 0.23.4 / 0.17.0 |

## Shipping wasm (`og_host_web-publish`)

| State | raw | after wasm-bindgen | after wasm-opt | brotli -q 11 |
|---|---|---|---|---|
| Rust 1.97.0, binaryen 123 (c1290aa) | 13,970,595 | 12,366,907 | 9,438,293 | 2,543,558 |
| Rust 1.99.1, edition 2024, binaryen 123 | 13,917,422 | 12,314,999 | 9,444,434 | 2,555,208 |
| binaryen 133 | 13,917,422 | 12,314,999 | 9,383,787 | 2,542,432 |
| plus `--low-memory-unused` | 13,917,422 | 12,314,999 | 9,283,648 | 2,518,738 |
| wasm-bindgen 0.2.129 (HEAD) | 13,954,272 | 12,317,295 | 9,283,994 | 2,518,068 |

The new compiler grows the optimised module by 6 KB (11.7 KB brotli);
binaryen 133 takes back 61 KB (12.8 KB brotli) and `--low-memory-unused`
another 100 KB (23.7 KB brotli). Net: -154 KB, -25.5 KB brotli (-1.0 %),
60 % of the 4 MiB budget. Other wasm-opt flags tried on binaryen 133 and
not kept: `--converge` (+887 B, +1,745 brotli), a second `-Oz` (the same),
`--strip-debug --strip-producers --strip-target-features` (-265 B but
+201 brotli), `--zero-filled-memory` and the explicit sign-ext,
mutable-globals, reference-types and multivalue features (no change).
`--low-memory-unused` is sound here because rustc links wasm32 stack-first
(`.rodata` begins at 1,048,576; the stack is [0, 1 MiB)), so only an
overflowing stack reaches the low 1 KiB.

## Type checking

`bun run typecheck` in packages/engine (both configs): 1.20 s on
TypeScript 5.9.3 (1.03 s for tsconfig.runtime.json, 131 files, 11,621
lines; 0.95 s total time of which 0.66 s checking), 0.25–0.30 s on 7.0.2,
no option changes needed. A deliberate type error in a file under the
runtime config is reported by 7.0.2 (TS2322). The templates, checked
against packages/engine's types through a `paths` mapping, report the same
errors at the same places under both versions (template tests without
`@types/node`, and fx-showcase passing numbers where handles are typed);
they did before this move and are not in any gate.

## Determinism

`tools/wasm-determinism` on 1.99.1: math_digest 6117185439544271997,
bouncing_boxes_hash 15551159556835976588 on host debug, host release,
wasm32 scalar and simd128; Rust 1.97.0 gives the same two numbers.

## Frames

Every committed reference frame other than fx-showcase's badge passes
without re-recording on the new toolchain: og-tests' gfx references on
metal in the gate's test step (348 tests); `og test` on templates/game,
ui-kit (17 suites, the six skin packs included), runner, match3, topdown
and fx-showcase on metal; `make web-reference` (fx-showcase) and the
topdown web runner on webgpu and webgl2 at RMSE 0 against their own
references; runner, match3 and topdown on vulkan-cpu (lavapipe in a
linux/amd64 Docker container, `og` and the addon from cargo-zigbuild) at
RMSE 0. og-tests' vulkan-cpu references were not run here (CI's gate runs
them on lavapipe).

## Gate

Full lane at the end (warm, 3 min 13 s): fmt 1 s, clippy 0, wasm-build 7,
test 110 (348 tests under nextest), wasm-opt 32, size-gate 17, drift 1,
bun-test 23, determinism 1. The first full lane after the compiler move
(cold for the new rustc): wasm-build 114 s, test 112, wasm-opt 27,
size-gate 19, bun-test 105, then a determinism failure fixed in c285937.
Before the move (875a166, the last gate of the E4 round): wasm-build 77,
test 107, wasm-opt 28, size-gate 18, bun-test 71, determinism 20.

## Left as they are

- wgpu and naga 31: not on crates.io yet.
- bun 1.4.3: installing it replaces `~/.local/bin/bun`, which every
  process on this machine (the platform agent's validation included)
  resolves first; left for a separate move.
- tools/bench `three` 0.180.0 (0.186.1 exists): the spike's comparison
  scenes are measured on 0.180; a 0.x minor is a breaking range.
- tools/bench `e2b` 2.52.1 (2.56.0 exists): only `e2b-run.ts` uses it,
  billable and not run here.
- Crate majors: bincode 3, cpal 0.18, meshopt 0.7, miniz_oxide 0.9,
  nalgebra 0.35, syn 3.
